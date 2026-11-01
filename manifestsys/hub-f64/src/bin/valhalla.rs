//! Valhalla 统一程式入口（Rust，2026-11-01 整合）
//!
//! 取代 PowerShell 运行器成为唯一程式化入口：构建、库测、检查包、
//! 证明包、全流程、可视化监控，全部由这一个二进制调度。
//! 子程式仍是既有各 bin（库零改动）；本程式只做编排：
//! 定位仓库根、注入 CUDA 构建环境、逐件落日志、按退出码报 PASS/FAIL。
//!
//! 用法：
//!   valhalla                 默认全套（L0+L1+L2电池+L3，约 3 分钟）
//!   valhalla run [--rounds N] [--seed S] [--incubation] [--viz] [--skip-build]
//!   valhalla check           L0 库测 + L1 检查包
//!   valhalla battery         L2 行业电池
//!   valhalla incubation      L2 孵化证明（15 seeds，约 8 分钟）
//!   valhalla flow [--rounds N] [--seed S] [--cycles N]
//!   valhalla viz  [--rounds N] [--seed S] [--open]   生成并（可选）打开监控页
//!   valhalla build           只构建全套所需 bin
//!
//! 判读纪律见 docs/valhalla/RUN_MODE.md。

use std::env;
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::process::{exit, Command, Stdio};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

const CHECK_BINS: [&str; 6] = [
    "valhalla_fate_storage",
    "valhalla_storage_test",
    "valhalla_data_flow",
    "valhalla_monitor_check",
    "valhalla_light_train_check",
    "valhalla_fate_decision_check",
];

const SUITE_BINS: [&str; 10] = [
    "valhalla_fate_storage",
    "valhalla_storage_test",
    "valhalla_data_flow",
    "valhalla_monitor_check",
    "valhalla_light_train_check",
    "valhalla_fate_decision_check",
    "valhalla_industry_battery",
    "valhalla_incubation_proof",
    "valhalla_full_flow",
    "valhalla_viz",
];

struct Opts {
    rounds: u32,
    seed: u32,
    cycles: Option<u32>,
    incubation: bool,
    viz: bool,
    open: bool,
    skip_build: bool,
}

impl Default for Opts {
    fn default() -> Self {
        Opts { rounds: 20, seed: 7, cycles: None, incubation: false, viz: false, open: false, skip_build: false }
    }
}

fn parse_opts(args: &[String]) -> Result<Opts, String> {
    let mut o = Opts::default();
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--rounds" => { i += 1; o.rounds = args.get(i).ok_or("--rounds 需要数值")?.parse().map_err(|_| "rounds 非整数")?; }
            "--seed" => { i += 1; o.seed = args.get(i).ok_or("--seed 需要数值")?.parse().map_err(|_| "seed 非整数")?; }
            "--cycles" => { i += 1; o.cycles = Some(args.get(i).ok_or("--cycles 需要数值")?.parse().map_err(|_| "cycles 非整数")?); }
            "--incubation" => o.incubation = true,
            "--viz" => o.viz = true,
            "--open" => o.open = true,
            "--skip-build" => o.skip_build = true,
            other => return Err(format!("未知参数: {other}")),
        }
        i += 1;
    }
    Ok(o)
}

/// 仓库根 = 同时有根 Cargo.toml 与 manifestsys/hub-f64 的目录。
/// 从当前目录与可执行文件位置双向向上找（exe 在 target/release 下）。
fn repo_root() -> PathBuf {
    let mut starts: Vec<PathBuf> = Vec::new();
    if let Ok(cwd) = env::current_dir() { starts.push(cwd); }
    if let Ok(exe) = env::current_exe() { starts.push(exe); }
    for start in &starts {
        let mut dir: &Path = start.as_path();
        loop {
            if dir.join("Cargo.toml").is_file() && dir.join("manifestsys/hub-f64/Cargo.toml").is_file() {
                return dir.to_path_buf();
            }
            match dir.parent() { Some(p) => dir = p, None => break }
        }
    }
    eprintln!("找不到 Valhalla 仓库根（需含 Cargo.toml 与 manifestsys/hub-f64）");
    exit(2);
}

/// CUDA 构建环境：目录存在才注入（与 RUN_MODE.md 既定环境一致）。
fn apply_cuda_env(cmd: &mut Command) {
    let lib = Path::new(r"C:\Ai\cuda-libx64");
    if lib.is_dir() {
        cmd.env("RUSTFLAGS", r"-L C:\Ai\cuda-libx64");
        let cuda_bin = r"C:\Program Files\NVIDIA GPU Computing Toolkit\CUDA\v13.4\bin";
        let old = env::var("PATH").unwrap_or_default();
        let mut parts = vec![lib.to_string_lossy().to_string()];
        if Path::new(cuda_bin).is_dir() { parts.push(cuda_bin.to_string()); }
        parts.push(old);
        cmd.env("PATH", parts.join(";"));
    }
}

struct Runner {
    root: PathBuf,
    log_dir: PathBuf,
    results: Vec<(String, String, bool, f64)>,
}

impl Runner {
    fn new(root: PathBuf) -> Runner {
        let stamp = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
        let log_dir = root.join("logs").join(format!("suite_{stamp}"));
        let _ = fs::create_dir_all(&log_dir);
        Runner { root, log_dir, results: Vec::new() }
    }

    fn exe(&self, bin: &str) -> PathBuf {
        self.root.join("target").join("release").join(format!("{bin}.exe"))
    }

    /// 跑一件：输出全量落日志，控制台只报一行 PASS/FAIL。
    fn item(&mut self, layer: &str, name: &str, mut cmd: Command) -> bool {
        let log_path = self.log_dir.join(format!("{name}.log"));
        let t = Instant::now();
        let status = (|| -> std::io::Result<std::process::ExitStatus> {
            let out = File::create(&log_path)?;
            let err = out.try_clone()?;
            cmd.current_dir(&self.root)
                .stdout(Stdio::from(out))
                .stderr(Stdio::from(err))
                .status()
        })();
        let secs = t.elapsed().as_secs_f64();
        let (ok, code) = match &status {
            Ok(s) => (s.success(), s.code().map(|c| c.to_string()).unwrap_or_else(|| "?".into())),
            Err(e) => {
                let _ = fs::write(&log_path, format!("spawn failed: {e}\n"));
                (false, "spawn".to_string())
            }
        };
        println!("{} [{}] {}  ({:.1}s, exit {})", if ok { "PASS" } else { "FAIL" }, layer, name, secs, code);
        self.results.push((layer.to_string(), name.to_string(), ok, secs));
        ok
    }

    fn cargo(&mut self, layer: &str, name: &str, args: &[&str]) -> bool {
        let mut cmd = Command::new("cargo");
        cmd.args(args);
        apply_cuda_env(&mut cmd);
        self.item(layer, name, cmd)
    }

    fn bin(&mut self, layer: &str, name: &str, envs: &[(&str, String)]) -> bool {
        let exe = self.exe(name);
        if !exe.is_file() {
            println!("FAIL [{}] {}  (未构建: {})", layer, name, exe.display());
            self.results.push((layer.into(), name.into(), false, 0.0));
            return false;
        }
        let mut cmd = Command::new(exe);
        for (k, v) in envs { cmd.env(k, v); }
        self.item(layer, name, cmd)
    }

    fn build(&mut self, bins: &[&str]) -> bool {
        let mut args: Vec<&str> = vec!["build", "--release", "-p", "hub-f64"];
        for b in bins { args.push("--bin"); args.push(b); }
        println!("== 构建（{} 件） ==", bins.len());
        self.cargo("BUILD", "build", &args)
    }

    fn summary(&self) -> bool {
        println!("\n== 汇总（日志: {}）==", self.log_dir.display());
        for (layer, name, ok, secs) in &self.results {
            println!("  {} [{}] {:32} {:.1}s", if *ok { "PASS" } else { "FAIL" }, layer, name, secs);
        }
        let failed: Vec<&str> = self.results.iter().filter(|r| !r.2).map(|r| r.1.as_str()).collect();
        if failed.is_empty() { println!("ALL PASS"); true } else { println!("FAILED: {}", failed.join(", ")); false }
    }
}

fn flow_envs(o: &Opts) -> Vec<(&'static str, String)> {
    let mut v = vec![("VH_ROUNDS", o.rounds.to_string()), ("VH_SEED", o.seed.to_string())];
    if let Some(c) = o.cycles { v.push(("VH_CYCLES", c.to_string())); }
    v
}

fn layer_l0(r: &mut Runner) {
    println!("== L0 库测 ==");
    r.cargo("L0", "lib_tests", &["test", "--release", "--lib", "-p", "fate-f64", "-p", "stem-cell", "-p", "valhalla-tile-quattro"]);
}

fn layer_l1(r: &mut Runner) {
    println!("== L1 检查包 ==");
    for b in CHECK_BINS { r.bin("L1", b, &[]); }
}

fn main() {
    let argv: Vec<String> = env::args().skip(1).collect();
    let cmd = argv.first().map(|s| s.as_str()).unwrap_or("run");
    let rest: &[String] = if argv.is_empty() { &[] } else { &argv[1..] };
    let opts = match parse_opts(rest) {
        Ok(o) => o,
        Err(e) => { eprintln!("{e}\n用法见 valhalla --help（或任意未知子命令）"); exit(2); }
    };
    let root = repo_root();
    let mut r = Runner::new(root.clone());

    let ok = match cmd {
        "build" => {
            r.build(&SUITE_BINS);
            r.summary()
        }
        "check" => {
            if !opts.skip_build { r.build(&CHECK_BINS); }
            layer_l0(&mut r);
            layer_l1(&mut r);
            r.summary()
        }
        "battery" => {
            if !opts.skip_build { r.build(&["valhalla_industry_battery"]); }
            r.bin("L2", "valhalla_industry_battery", &[]);
            r.summary()
        }
        "incubation" => {
            if !opts.skip_build { r.build(&["valhalla_incubation_proof"]); }
            r.bin("L2", "valhalla_incubation_proof", &[]);
            r.summary()
        }
        "flow" => {
            if !opts.skip_build { r.build(&["valhalla_full_flow"]); }
            println!("== L3 全流程（{} 轮，seed {}） ==", opts.rounds, opts.seed);
            r.bin("L3", "valhalla_full_flow", &flow_envs(&opts));
            r.summary()
        }
        "viz" | "monitor" => {
            if !opts.skip_build { r.build(&["valhalla_viz"]); }
            println!("== 展示（valhalla_viz，{} 轮，seed {}） ==", opts.rounds, opts.seed);
            let passed = r.bin("VIZ", "valhalla_viz", &flow_envs(&opts));
            let page = root.join("viz_out").join("index.html");
            if passed && page.is_file() {
                println!("监控页: {}", page.display());
                if opts.open {
                    #[cfg(windows)]
                    { let _ = Command::new("cmd").args(["/C", "start", "", &page.to_string_lossy()]).status(); }
                }
            }
            r.summary()
        }
        "run" => {
            if !opts.skip_build { r.build(&SUITE_BINS); }
            layer_l0(&mut r);
            layer_l1(&mut r);
            println!("== L2 证明包 ==");
            r.bin("L2", "valhalla_industry_battery", &[]);
            if opts.incubation {
                r.bin("L2", "valhalla_incubation_proof", &[]);
            } else {
                println!("SKIP [L2] valhalla_incubation_proof（加 --incubation 才跑，约 8 分钟）");
            }
            println!("== L3 全流程（{} 轮，seed {}） ==", opts.rounds, opts.seed);
            r.bin("L3", "valhalla_full_flow", &flow_envs(&opts));
            if opts.viz {
                println!("== 展示（valhalla_viz） ==");
                r.bin("VIZ", "valhalla_viz", &flow_envs(&opts));
                let page = root.join("viz_out").join("index.html");
                if page.is_file() {
                    println!("监控页: {}", page.display());
                    if opts.open {
                        #[cfg(windows)]
                        { let _ = Command::new("cmd").args(["/C", "start", "", &page.to_string_lossy()]).status(); }
                    }
                }
            }
            r.summary()
        }
        "--help" | "-h" | "help" => {
            println!("Valhalla 统一程式入口\n");
            println!("  valhalla                 默认全套 L0+L1+L2电池+L3（约 3 分钟）");
            println!("  valhalla run [--rounds N] [--seed S] [--incubation] [--viz [--open]] [--skip-build]");
            println!("  valhalla check           L0 库测 + L1 检查包");
            println!("  valhalla battery         L2 行业电池");
            println!("  valhalla incubation      L2 孵化证明（约 8 分钟）");
            println!("  valhalla flow [--rounds N] [--seed S] [--cycles N]");
            println!("  valhalla viz  [--rounds N] [--seed S] [--open]   Tile/Stem 可视化监控页");
            println!("  valhalla build           只构建全套所需 bin");
            println!("\n判读纪律: docs/valhalla/RUN_MODE.md");
            true
        }
        other => {
            eprintln!("未知子命令: {other}\n");
            println!("  valhalla [run|check|battery|incubation|flow|viz|build] [--rounds N] [--seed S] [--incubation] [--viz] [--open] [--skip-build]");
            false
        }
    };
    exit(if ok { 0 } else { 1 });
}

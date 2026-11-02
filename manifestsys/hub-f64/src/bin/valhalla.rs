//! Valhalla unified programme entry (Rust, consolidated 2026-11-01)
//!
//! Replaces the PowerShell runner as the single programmatic entry:
//! build, library tests, check packages, proof packages, full flow
//! and the visual monitor are all dispatched by this one binary.
//! Sub-programmes remain the existing bins (library untouched); this
//! programme only orchestrates: locate the repo root, inject the CUDA
//! build env, log each item, report PASS/FAIL by exit code.
//!
//! Usage:
//!   valhalla                 default full suite (L0+L1+L2 battery+L3, ~3 min)
//!   valhalla run [--rounds N] [--seed S] [--incubation] [--viz] [--skip-build]
//!   valhalla check           L0 library tests + L1 check packages
//!   valhalla battery         L2 industry battery
//!   valhalla incubation      L2 incubation proof (15 seeds, ~8 min)
//!   valhalla flow [--rounds N] [--seed S] [--cycles N]
//!   valhalla viz  [--rounds N] [--seed S] [--open]   generate and (optionally) open the dashboard
//!   valhalla build           build only the bins the suite needs
//!
//! Reading discipline: docs/valhalla/RUN_MODE.md.

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
            "--rounds" => { i += 1; o.rounds = args.get(i).ok_or("--rounds needs a value")?.parse().map_err(|_| "rounds is not an integer")?; }
            "--seed" => { i += 1; o.seed = args.get(i).ok_or("--seed needs a value")?.parse().map_err(|_| "seed is not an integer")?; }
            "--cycles" => { i += 1; o.cycles = Some(args.get(i).ok_or("--cycles needs a value")?.parse().map_err(|_| "cycles is not an integer")?); }
            "--incubation" => o.incubation = true,
            "--viz" => o.viz = true,
            "--open" => o.open = true,
            "--skip-build" => o.skip_build = true,
            other => return Err(format!("unknown argument: {other}")),
        }
        i += 1;
    }
    Ok(o)
}

/// Repo root = a directory holding both the root Cargo.toml and
/// manifestsys/hub-f64. Searched upward from cwd and from the exe
/// location (the exe sits under target/release).
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
    eprintln!("Cannot find the Valhalla repo root (needs Cargo.toml and manifestsys/hub-f64)");
    exit(2);
}

/// CUDA build env: injected only when the directories exist (matches RUN_MODE.md).
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

    /// Run one item: full output to its log; the console reports one PASS/FAIL line.
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
            println!("FAIL [{}] {}  (not built: {})", layer, name, exe.display());
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
        println!("== Build ({} bins) ==", bins.len());
        self.cargo("BUILD", "build", &args)
    }

    fn summary(&self) -> bool {
        println!("\n== Summary (logs: {}) ==", self.log_dir.display());
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
    println!("== L0 library tests ==");
    r.cargo("L0", "lib_tests", &["test", "--release", "--lib", "-p", "fate-f64", "-p", "stem-cell", "-p", "valhalla-tile-quattro"]);
}

fn layer_l1(r: &mut Runner) {
    println!("== L1 check packages ==");
    for b in CHECK_BINS { r.bin("L1", b, &[]); }
}

fn main() {
    let argv: Vec<String> = env::args().skip(1).collect();
    let cmd = argv.first().map(|s| s.as_str()).unwrap_or("run");
    let rest: &[String] = if argv.is_empty() { &[] } else { &argv[1..] };
    let opts = match parse_opts(rest) {
        Ok(o) => o,
        Err(e) => { eprintln!("{e}\nsee valhalla --help (or any unknown subcommand)"); exit(2); }
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
            println!("== L3 full flow ({} rounds, seed {}) ==", opts.rounds, opts.seed);
            r.bin("L3", "valhalla_full_flow", &flow_envs(&opts));
            r.summary()
        }
        "viz" | "monitor" => {
            if !opts.skip_build { r.build(&["valhalla_viz"]); }
            println!("== Display (valhalla_viz, {} rounds, seed {}) ==", opts.rounds, opts.seed);
            let passed = r.bin("VIZ", "valhalla_viz", &flow_envs(&opts));
            let page = root.join("viz_out").join("index.html");
            if passed && page.is_file() {
                println!("Dashboard: {}", page.display());
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
            println!("== L2 proof packages ==");
            r.bin("L2", "valhalla_industry_battery", &[]);
            if opts.incubation {
                r.bin("L2", "valhalla_incubation_proof", &[]);
            } else {
                println!("SKIP [L2] valhalla_incubation_proof (runs only with --incubation, ~8 min)");
            }
            println!("== L3 full flow ({} rounds, seed {}) ==", opts.rounds, opts.seed);
            r.bin("L3", "valhalla_full_flow", &flow_envs(&opts));
            if opts.viz {
                println!("== Display (valhalla_viz) ==");
                r.bin("VIZ", "valhalla_viz", &flow_envs(&opts));
                let page = root.join("viz_out").join("index.html");
                if page.is_file() {
                    println!("Dashboard: {}", page.display());
                    if opts.open {
                        #[cfg(windows)]
                        { let _ = Command::new("cmd").args(["/C", "start", "", &page.to_string_lossy()]).status(); }
                    }
                }
            }
            r.summary()
        }
        "--help" | "-h" | "help" => {
            println!("Valhalla unified programme entry\n");
            println!("  valhalla                 default full suite L0+L1+L2 battery+L3 (~3 min)");
            println!("  valhalla run [--rounds N] [--seed S] [--incubation] [--viz [--open]] [--skip-build]");
            println!("  valhalla check           L0 library tests + L1 check packages");
            println!("  valhalla battery         L2 industry battery");
            println!("  valhalla incubation      L2 incubation proof (~8 min)");
            println!("  valhalla flow [--rounds N] [--seed S] [--cycles N]");
            println!("  valhalla viz  [--rounds N] [--seed S] [--open]   Tile/Stem visual dashboard");
            println!("  valhalla build           build only the bins the suite needs");
            println!("\nReading discipline: docs/valhalla/RUN_MODE.md");
            true
        }
        other => {
            eprintln!("unknown subcommand: {other}\n");
            println!("  valhalla [run|check|battery|incubation|flow|viz|build] [--rounds N] [--seed S] [--incubation] [--viz] [--open] [--skip-build]");
            false
        }
    };
    exit(if ok { 0 } else { 1 });
}

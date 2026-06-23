//! Hybrid Valhalla product — full Rust Tier A + Tier B + merge.

use std::env;
use std::path::PathBuf;
use std::time::SystemTime;

use hub_f64::fair_suite::{load_suite, select_corpus, select_prompts};
use hub_f64::hybrid::{
    load_patch_rows, run_hybrid_eval, run_hybrid_eval_full, HybridConfig, MergeMode,
    HYBRID_PRODUCT_VERSION,
};
use hub_f64::patch_lm::{resolve_model_dir, PatchLmConfig, DEFAULT_MAX_NEW, DEFAULT_STRENGTH};

fn usage() -> ! {
    eprintln!(
        "Usage: valhalla_hybrid --phase smoke|val|test [options]\n\
         全 Rust 模式（默认）: --run-patch [--model PATH|HF_ID] [--patch-body triad] [--strength 0.08]\n\
         复用 patch JSON: --patch-json PATH\n\
         通用: [--suite PATH] [--merge-mode deploy|oracle] [--out PATH] [--cpu]"
    );
    std::process::exit(2);
}

struct Cli {
    phase: String,
    suite: PathBuf,
    patch_json: Option<PathBuf>,
    run_patch: bool,
    model: String,
    patch_body: String,
    strength: f64,
    max_new: usize,
    merge_mode: MergeMode,
    out: Option<PathBuf>,
    use_cpu: bool,
}

fn parse_args() -> Cli {
    let mut phase = "test".to_string();
    let mut suite = PathBuf::from("tools/fair_benchmark/fair_benchmark_suite.json");
    let mut patch_json: Option<PathBuf> = None;
    let mut run_patch = true;
    let mut model = "Qwen/Qwen2.5-0.5B-Instruct".to_string();
    let mut patch_body = "triad".to_string();
    let mut strength = DEFAULT_STRENGTH;
    let mut max_new = DEFAULT_MAX_NEW;
    let mut merge_mode = MergeMode::Deploy;
    let mut out: Option<PathBuf> = None;
    let mut use_cpu = false;

    let args: Vec<String> = env::args().collect();
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--phase" => {
                i += 1;
                phase = args.get(i).cloned().unwrap_or_else(|| usage());
            }
            "--suite" => {
                i += 1;
                suite = PathBuf::from(args.get(i).unwrap_or_else(|| usage()));
            }
            "--patch-json" => {
                i += 1;
                patch_json = Some(PathBuf::from(args.get(i).unwrap_or_else(|| usage())));
                run_patch = false;
            }
            "--run-patch" => run_patch = true,
            "--model" => {
                i += 1;
                model = args.get(i).cloned().unwrap_or_else(|| usage());
            }
            "--patch-body" => {
                i += 1;
                patch_body = args.get(i).cloned().unwrap_or_else(|| usage());
            }
            "--strength" => {
                i += 1;
                strength = args
                    .get(i)
                    .and_then(|s| s.parse().ok())
                    .unwrap_or_else(|| usage());
            }
            "--max-new" => {
                i += 1;
                max_new = args
                    .get(i)
                    .and_then(|s| s.parse().ok())
                    .unwrap_or_else(|| usage());
            }
            "--merge-mode" => {
                i += 1;
                merge_mode = match args.get(i).map(|s| s.as_str()).unwrap_or_else(|| usage()) {
                    "deploy" => MergeMode::Deploy,
                    "oracle" => MergeMode::Oracle,
                    _ => usage(),
                };
            }
            "--out" => {
                i += 1;
                out = Some(PathBuf::from(args.get(i).unwrap_or_else(|| usage())));
            }
            "--cpu" => use_cpu = true,
            "-h" | "--help" => usage(),
            other => {
                eprintln!("unknown arg: {other}");
                usage();
            }
        }
        i += 1;
    }

    Cli {
        phase,
        suite,
        patch_json,
        run_patch,
        model,
        patch_body,
        strength,
        max_new,
        merge_mode,
        out,
        use_cpu,
    }
}

fn main() {
    let cli = parse_args();

    let suite = load_suite(&cli.suite).unwrap_or_else(|e| {
        eprintln!("load suite: {e}");
        std::process::exit(1);
    });
    let prompts = select_prompts(&suite, &cli.phase).unwrap_or_else(|e| {
        eprintln!("select prompts: {e}");
        std::process::exit(1);
    });
    let corpus = select_corpus(&suite, &cli.phase).unwrap_or_else(|e| {
        eprintln!("select corpus: {e}");
        std::process::exit(1);
    });

    let cfg = HybridConfig {
        merge_mode: cli.merge_mode,
        pipeline_v4: true,
        ..HybridConfig::default()
    };

    eprintln!(
        "valhalla_hybrid {} phase={} prompts={} corpus={} merge={:?} patch={} stem_parallel={}",
        HYBRID_PRODUCT_VERSION,
        cli.phase,
        prompts.len(),
        corpus.len(),
        cli.merge_mode,
        if cli.run_patch { "generate" } else { "json" },
        cfg.stem_parallel,
    );

    let result = if cli.run_patch {
        let patch_cfg = PatchLmConfig {
            model_dir: resolve_model_dir(&cli.model),
            patch_body: cli.patch_body,
            strength: cli.strength,
            max_new: cli.max_new,
            export_cycles: 1,
            use_cpu: true,
            pipeline_v4: true,
        };
        eprintln!("model_dir: {}", patch_cfg.model_dir.display());
        run_hybrid_eval_full(&prompts, &corpus, &cfg, &patch_cfg).unwrap_or_else(|e| {
            eprintln!("hybrid eval full: {e}");
            std::process::exit(1);
        })
    } else {
        let patch_path = cli.patch_json.unwrap_or_else(|| {
            eprintln!("need --patch-json or --run-patch");
            usage();
        });
        let patch_rows = load_patch_rows(&patch_path).unwrap_or_else(|e| {
            eprintln!("load patch: {e}");
            std::process::exit(1);
        });
        run_hybrid_eval(&prompts, &corpus, &patch_rows, &cfg).unwrap_or_else(|e| {
            eprintln!("hybrid eval: {e}");
            std::process::exit(1);
        })
    };

    let ts = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let merge_label = match cli.merge_mode {
        MergeMode::Deploy => "deploy",
        MergeMode::Oracle => "oracle",
    };
    let mode_tag = if cli.run_patch { "full" } else { "reuse" };
    let out_json = cli.out.unwrap_or_else(|| {
        PathBuf::from(format!(
            "reports/hybrid_valhalla_lm/hybrid_rust_{mode_tag}_{}_{merge_label}_{ts}.json",
            cli.phase
        ))
    });
    if let Some(parent) = out_json.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    let json = serde_json::to_string_pretty(&result).expect("serialize");
    std::fs::write(&out_json, &json).expect("write output");
    eprintln!(
        "hybrid acc: {:.2}% ({}/{}) -> {}",
        result.summary.hybrid.acc * 100.0,
        result.summary.hybrid.correct,
        result.summary.hybrid.n,
        out_json.display()
    );
    println!("{}", json);
}

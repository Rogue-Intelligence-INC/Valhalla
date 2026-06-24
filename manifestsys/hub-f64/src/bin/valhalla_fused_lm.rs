//! Fused Valhalla LM — one patched Qwen forward per question (no hybrid memory router).

use std::env;
use std::path::PathBuf;
use std::time::SystemTime;

use hub_f64::fair_suite::{load_suite, select_corpus, select_prompts};
use hub_f64::fused_lm::{run_fused_eval, FUSED_PRODUCT_VERSION};
use hub_f64::patch_lm::{resolve_model_dir, PatchLmConfig, DEFAULT_MAX_NEW, DEFAULT_STRENGTH};

fn usage() -> ! {
    eprintln!(
        "Usage: valhalla_fused_lm --phase smoke|val|test [options]\n\
         Single LLM product: pipeline v4 Valhalla patch + Qwen generate.\n\
         MCQ: 4-way logprob batch (no memory track, no hybrid router).\n\
         Options: [--suite PATH] [--model PATH|HF_ID] [--patch-body triad]\n\
                  [--strength 0.08] [--max-new 192] [--out PATH] [--cpu]"
    );
    std::process::exit(2);
}

struct Cli {
    phase: String,
    suite: PathBuf,
    model: String,
    patch_body: String,
    strength: f64,
    max_new: usize,
    out: Option<PathBuf>,
    use_cpu: bool,
}

fn parse_args() -> Cli {
    let mut phase = "test".to_string();
    let mut suite = PathBuf::from("tools/fair_benchmark/fair_benchmark_suite.json");
    let mut model = "Qwen/Qwen2.5-0.5B-Instruct".to_string();
    let mut patch_body = "triad".to_string();
    let mut strength = DEFAULT_STRENGTH;
    let mut max_new = DEFAULT_MAX_NEW;
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
        model,
        patch_body,
        strength,
        max_new,
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

    let patch_cfg = PatchLmConfig {
        model_dir: resolve_model_dir(&cli.model),
        patch_body: cli.patch_body,
        strength: cli.strength,
        max_new: cli.max_new,
        export_cycles: 1,
        use_cpu: cli.use_cpu,
        pipeline_v4: true,
        ..PatchLmConfig::default()
    };

    eprintln!(
        "valhalla_fused_lm {} phase={} prompts={} corpus={} model={}",
        FUSED_PRODUCT_VERSION,
        cli.phase,
        prompts.len(),
        corpus.len(),
        patch_cfg.model_dir.display(),
    );

    let result = run_fused_eval(&prompts, &corpus, &patch_cfg).unwrap_or_else(|e| {
        eprintln!("fused eval: {e}");
        std::process::exit(1);
    });

    let ts = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let out_json = cli.out.unwrap_or_else(|| {
        PathBuf::from(format!(
            "reports/fused_valhalla_lm/fused_{}_{ts}.json",
            cli.phase
        ))
    });
    if let Some(parent) = out_json.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    let json = serde_json::to_string_pretty(&result).expect("serialize");
    std::fs::write(&out_json, &json).expect("write output");
    eprintln!(
        "fused acc: {:.2}% ({}/{}) open={}/{} mcq={}/{} numeric={}/{} -> {}",
        result.summary.total.acc * 100.0,
        result.summary.total.correct,
        result.summary.total.n,
        result.summary.open.correct,
        result.summary.open.n,
        result.summary.mcq.correct,
        result.summary.mcq.n,
        result.summary.numeric.correct,
        result.summary.numeric.n,
        out_json.display()
    );
    println!("{}", json);
}

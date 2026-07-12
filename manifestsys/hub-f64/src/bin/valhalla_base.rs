//! ValhallaBase unified CLI — JSON · ask · demo · serve

use std::env;
use std::fs;
use std::io::{self, Read, Write};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::process;

use hub_f64::valhalla_base::VALHALLA_BASE_PROTOCOL;
use hub_f64::valhalla_base_bench::{
    default_corpus_dirs, run_local_corpus_demo, run_lara_fatewithv2, run_multihop_fate_rag,
    run_multihop_hybrid_rag, run_multihop_rag, run_needle_session, run_longchain_fate_final_test,
    run_v1_extensions, write_report,
};
use hub_f64::fate_parallel_longchain::{run_fate_parallel_longchain_smoke, run_locomo_routing_ab};
use hub_f64::fate_longchain_unified::{run_fate_longchain_unified, CorpusBuildMode, UnifiedRouting};
use hub_f64::fate_pretrain_holdout::run_fate_pretrain_holdout;
use hub_f64::valhalla_base_eval::{run_frames_slice, run_locomo_slice, run_longbench_slice, EvalControl};
use hub_f64::valhalla_base_client::ValhallaBaseClient;
use hub_f64::valhalla_base_http;
use hub_f64::valhalla_base_protocol::{
    example_names, example_request, execute_request_json, serialize_error, validate_request_json,
    ValhallaBaseErrorBody,
};

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() {
    hub_f64::ensure_production_runtime();
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        match args[1].as_str() {
            "ask" => {
                cmd_ask(&args[2..]);
                return;
            }
            "demo" => {
                cmd_demo(&args[2..]);
                return;
            }
            "serve" => {
                cmd_serve(&args[2..]);
                return;
            }
            "help" | "--help" | "-h" if args.len() == 2 => {
                print_help();
                return;
            }
            _ => {}
        }
    }
    cmd_json(&args[1..]);
}

fn print_help() {
    eprintln!(
        r#"valhalla_base {VERSION} — protocol {VALHALLA_BASE_PROTOCOL}

SUBCOMMANDS (Rust-native — prefer over Python wrappers):
  ask TEXT [--context LINE]... [--score-type open|mcq] [--native]
  demo v1|needle|needle-prod|longchain-fate|corpus|locomo|frames|multihop|multihop-fate [--fast] [--repo ROOT] [--limit N] [--verbose|-v] [--checkpoint PATH] [--resume PATH] [--pause-file PATH]
  serve [--port 8787] [--pretty] [--with-dashboard]
  demo fate-* | multihop-hybrid | ... (see docs/valhalla/REPRODUCTION.md)

JSON MODE (stdin/stdout — backward compatible):
  valhalla_base [--pretty] [--json-error] [request.json]
  valhalla_base --example open|session|mcq|multimodal|proactive|production
  valhalla_base --validate FILE

RUST API (in-process, no subprocess):
  hub_f64::ValhallaBaseClient
  hub_f64::execute_request / execute_request_json

ARTIFACTS:
  include/valhalla_base.h
  schemas/valhalla-base-v1.*.schema.json
  docs/valhalla/VALHALLA_BASE_CLI.md
"#
    );
}

fn fail(err: ValhallaBaseErrorBody, json_error: bool, pretty: bool) -> ! {
    eprintln!("{}: {}", VALHALLA_BASE_PROTOCOL, err.error);
    if json_error {
        let out = serialize_error(&err, pretty);
        let _ = io::stdout().write_all(out.as_bytes());
        let _ = io::stdout().write_all(b"\n");
    }
    process::exit(1);
}

fn cmd_ask(args: &[String]) {
    let mut context: Vec<String> = Vec::new();
    let mut score_type = "open".to_string();
    let mut native = false;
    let mut question: Option<String> = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--context" | "-c" => {
                i += 1;
                if let Some(l) = args.get(i) {
                    context.push(l.clone());
                }
            }
            "--score-type" => {
                i += 1;
                if let Some(s) = args.get(i) {
                    score_type = s.clone();
                }
            }
            "--native" => native = true,
            "-h" | "--help" => {
                eprintln!("usage: valhalla_base ask TEXT [--context LINE]... [--score-type open|mcq] [--native]");
                return;
            }
            other if !other.starts_with('-') && question.is_none() => {
                question = Some(other.to_string());
            }
            other => {
                fail(
                    ValhallaBaseErrorBody::new(
                        hub_f64::valhalla_base_protocol::ValhallaBaseErrorKind::Validate,
                        format!("unknown ask flag or extra arg: {other}"),
                    ),
                    false,
                    false,
                );
            }
        }
        i += 1;
    }
    let text = question.unwrap_or_else(|| {
        fail(
            ValhallaBaseErrorBody::new(
                hub_f64::valhalla_base_protocol::ValhallaBaseErrorKind::Validate,
                "ask requires QUESTION text",
            ),
            false,
            false,
        );
    });
    if context.is_empty() {
        fail(
            ValhallaBaseErrorBody::new(
                hub_f64::valhalla_base_protocol::ValhallaBaseErrorKind::Validate,
                "ask requires at least one --context LINE",
            ),
            false,
            false,
        );
    }

    let mut client = ValhallaBaseClient::production();
    if native {
        client.config.decode = hub_f64::valhalla_base::ValhallaBaseDecode::Native;
    }
    client.set_context(context.iter().map(|s| s.as_str()));
    match client.ask(text, &score_type, "q0") {
        Ok(ans) => println!("{ans}"),
        Err(e) => fail(e, false, false),
    }
}

fn cmd_demo(args: &[String]) {
    let name = args.first().map(|s| s.as_str()).unwrap_or("");
    let fast = args.iter().any(|a| a == "--fast");
    let repo = args
        .windows(2)
        .find(|w| w[0] == "--repo")
        .map(|w| PathBuf::from(&w[1]))
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../..")
                .canonicalize()
                .unwrap_or_else(|_| PathBuf::from("."))
        });
    let reports = repo.join("reports/valhalla_inference");
    let eval_ctrl = EvalControl::from_cli_args(args);

    let result = match name {
        "v1" => run_v1_extensions().map(|doc| {
            write_report(&reports.join("valhalla_base_v1_extensions.json"), &doc).ok();
            doc
        }),
        "needle" => {
            let depths: Vec<usize> = if fast {
                vec![5, 10]
            } else {
                vec![5, 10, 20, 30]
            };
            let production = args.iter().any(|a| a == "--production");
            run_needle_session(&depths, fast, production).map(|doc| {
                write_report(&reports.join("needle_session_v1_test.json"), &doc).ok();
                doc
            })
        }
        "needle-prod" => {
            let depths: Vec<usize> = if fast {
                vec![5, 10]
            } else {
                vec![5, 10, 20, 30]
            };
            run_needle_session(&depths, fast, true).map(|doc| {
                write_report(&reports.join("needle_session_prod_rust.json"), &doc).ok();
                doc
            })
        }
        "longchain-fate" => {
            let passes = args
                .windows(2)
                .find(|w| w[0] == "--pretrain-passes")
                .and_then(|w| w[1].parse().ok())
                .unwrap_or(hub_f64::fate_orchestrator::default_pretrain_passes());
            let depth = args
                .windows(2)
                .find(|w| w[0] == "--depth")
                .and_then(|w| w[1].parse().ok())
                .unwrap_or(if fast { 10 } else { 30 });
            run_longchain_fate_final_test(passes, depth, fast).map(|doc| {
                write_report(&reports.join("longchain_fate_final_test.json"), &doc).ok();
                doc
            })
        }
        "fate-parallel-longchain" => {
            let passes = args
                .windows(2)
                .find(|w| w[0] == "--pretrain-passes")
                .and_then(|w| w[1].parse().ok())
                .unwrap_or(2);
            let max_lines = args
                .windows(2)
                .find(|w| w[0] == "--max-lines")
                .and_then(|w| w[1].parse().ok())
                .unwrap_or(0);
            run_fate_parallel_longchain_smoke(passes, max_lines).map(|rep| {
                let doc = serde_json::json!({
                    "protocol": rep.protocol,
                    "n_items": rep.n_items,
                    "baseline_pass": rep.baseline_pass,
                    "parallel_pass": rep.parallel_pass,
                    "pretrain_passes": rep.pretrain_passes,
                    "delta_pp": (rep.parallel_pass - rep.baseline_pass) * 100.0,
                    "rows": rep.rows,
                });
                write_report(&reports.join("fate_parallel_longchain_test.json"), &doc).ok();
                doc
            })
        }
        "fate-routing-ab" => {
            let limit = args
                .windows(2)
                .find(|w| w[0] == "--limit")
                .and_then(|w| w[1].parse().ok())
                .unwrap_or(10);
            let max_lines = args
                .windows(2)
                .find(|w| w[0] == "--max-lines")
                .and_then(|w| w[1].parse().ok())
                .unwrap_or(64);
            let chunk = args
                .windows(2)
                .find(|w| w[0] == "--chunk-size")
                .and_then(|w| w[1].parse().ok())
                .unwrap_or(0);
            let passes = args
                .windows(2)
                .find(|w| w[0] == "--pretrain-passes")
                .and_then(|w| w[1].parse().ok())
                .unwrap_or(2);
            let low_resource = args.iter().any(|a| a == "--low-resource");
            let bundle = repo.join("tools/standard_benchmarks/bundled/locomo_qa_slice30.json");
            run_locomo_routing_ab(&bundle, limit, max_lines, chunk, fast, passes, low_resource).map(|rep| {
                let doc = serde_json::json!({
                    "protocol": rep.protocol,
                    "n_items": rep.n_items,
                    "low_resource": low_resource,
                    "baseline_pass": rep.baseline_pass,
                    "parallel_pass": rep.parallel_pass,
                    "targeted_pass": rep.targeted_pass,
                    "avg_hub_spread": rep.avg_hub_spread,
                    "delta_parallel_pp": (rep.parallel_pass - rep.baseline_pass) * 100.0,
                    "delta_targeted_pp": (rep.targeted_pass - rep.baseline_pass) * 100.0,
                    "corpus_mode": rep.corpus_mode,
                    "scoring": "score_qa_algebra (qa_adaptive)",
                    "rows": rep.rows,
                });
                write_report(&reports.join("fate_routing_ab_locomo.json"), &doc).ok();
                doc
            })
        }
        "fate-pretrain-holdout" => {
            let train_n = args
                .windows(2)
                .find(|w| w[0] == "--train")
                .and_then(|w| w[1].parse().ok())
                .unwrap_or(5);
            let test_n = args
                .windows(2)
                .find(|w| w[0] == "--test")
                .and_then(|w| w[1].parse().ok())
                .unwrap_or(5);
            let max_lines = args
                .windows(2)
                .find(|w| w[0] == "--max-lines")
                .and_then(|w| w[1].parse().ok())
                .unwrap_or(280);
            let chunk = args
                .windows(2)
                .find(|w| w[0] == "--chunk-size")
                .and_then(|w| w[1].parse().ok())
                .unwrap_or(0);
            let max_epochs = args
                .windows(2)
                .find(|w| w[0] == "--max-epochs")
                .and_then(|w| w[1].parse().ok())
                .unwrap_or(24);
            let low_resource = args.iter().any(|a| a == "--low-resource");
            let bundle = repo.join("tools/standard_benchmarks/bundled/locomo_qa_slice30.json");
            run_fate_pretrain_holdout(
                &bundle,
                train_n,
                test_n,
                max_lines,
                chunk,
                fast || low_resource,
                max_epochs,
                low_resource,
            )
                .map(|rep| {
                    let doc = serde_json::json!({
                        "protocol": rep.protocol,
                        "pretrain_mode": rep.pretrain_mode,
                        "train_n": rep.train_n,
                        "test_n": rep.test_n,
                        "train_skipped_no_evidence": rep.train_skipped_no_evidence,
                        "test_skipped_no_evidence": rep.test_skipped_no_evidence,
                        "pretrain_epochs": rep.pretrain_epochs,
                        "train_pass": rep.train_pass,
                        "test_pass": rep.test_pass,
                        "converged": rep.converged,
                        "max_epochs": rep.max_epochs,
                        "final_hub_spread": rep.final_hub_spread,
                        "low_resource": rep.low_resource,
                        "corpus_lines": rep.corpus_lines,
                        "ingest_passes": rep.ingest_passes,
                        "fragment_count": rep.fragment_count,
                        "total_walk_steps": rep.total_walk_steps,
                        "epoch_log": rep.epoch_log,
                        "train_rows": rep.train_rows,
                        "test_rows": rep.test_rows,
                    });
                    write_report(&reports.join("fate_pretrain_holdout.json"), &doc).ok();
                    doc
                })
        }
        "fate-longchain-unified" => {
            let max_lines = args
                .windows(2)
                .find(|w| w[0] == "--max-lines")
                .and_then(|w| w[1].parse().ok())
                .unwrap_or(0);
            let chunk = args
                .windows(2)
                .find(|w| w[0] == "--chunk-size")
                .and_then(|w| w[1].parse().ok())
                .unwrap_or(64);
            let pretrain_passes = args
                .windows(2)
                .find(|w| w[0] == "--pretrain-passes")
                .and_then(|w| w[1].parse().ok())
                .unwrap_or(0);
            let max_epochs = args
                .windows(2)
                .find(|w| w[0] == "--max-epochs")
                .and_then(|w| w[1].parse().ok())
                .unwrap_or(0);
            let low_resource = args.iter().any(|a| a == "--low-resource");
            let hybrid = args.iter().any(|a| a == "--hybrid");
            let parallel = args.iter().any(|a| a == "--parallel");
            let minimal_corpus = args.iter().any(|a| a == "--minimal-corpus");
            let fragment_full = args.iter().any(|a| a == "--fragment-full");
            let routing = if hybrid {
                UnifiedRouting::Hybrid
            } else if parallel {
                UnifiedRouting::Parallel
            } else {
                UnifiedRouting::Targeted
            };
            let corpus_build =
                CorpusBuildMode::resolve(minimal_corpus, fragment_full);
            let bundle = repo.join("tools/standard_benchmarks/bundled/locomo_qa_slice30.json");
            let spec = args
                .windows(2)
                .find(|w| w[0] == "--spec")
                .map(|w| repo.join(&w[1]))
                .unwrap_or_else(|| {
                    repo.join("tools/standard_benchmarks/bundled/fate_longchain_unified_locomo.json")
                });
            run_fate_longchain_unified(
                &bundle,
                Some(&spec),
                max_lines,
                chunk,
                fast || low_resource,
                low_resource,
                routing,
                pretrain_passes,
                max_epochs,
                corpus_build,
            )
            .map(|rep| {
                let doc = serde_json::json!({
                    "protocol": rep.protocol,
                    "corpus_mode": rep.corpus_mode,
                    "corpus_build": rep.corpus_build,
                    "routing": rep.routing,
                    "low_resource": rep.low_resource,
                    "corpus_lines": rep.corpus_lines,
                    "final_hub_spread": rep.final_hub_spread,
                    "pretrain_passes": rep.pretrain_passes,
                    "pretrain_max_epochs": rep.pretrain_max_epochs,
                    "pretrain_epochs": rep.pretrain_epochs,
                    "pretrain_pass": rep.pretrain_pass,
                    "pretrain_converged": rep.pretrain_converged,
                    "pretrain_epoch_log": rep.pretrain_epoch_log,
                    "pretrain_rows": rep.pretrain_rows,
                    "anchor_baseline_pass": rep.anchor_baseline_pass,
                    "anchor_reprobe_pass": rep.anchor_reprobe_pass,
                    "theme_drift_rate": rep.theme_drift_rate,
                    "anchor_retention_rate": rep.anchor_retention_rate,
                    "context_pollution_rate": rep.context_pollution_rate,
                    "holdout_pass": rep.holdout_pass,
                    "interleave_pass": rep.interleave_pass,
                    "composite_score": rep.composite_score,
                    "fairness": rep.fairness,
                    "anchor_drift_rows": rep.anchor_drift_rows,
                    "phase_rows": rep.phase_rows,
                });
                write_report(&reports.join("fate_longchain_unified.json"), &doc).ok();
                doc
            })
        }
        "locomo" => {
            let limit = args
                .windows(2)
                .find(|w| w[0] == "--limit")
                .and_then(|w| w[1].parse().ok())
                .unwrap_or(if fast { 3 } else { 0 });
            let max_lines = args
                .windows(2)
                .find(|w| w[0] == "--max-lines")
                .and_then(|w| w[1].parse().ok())
                .unwrap_or(64);
            let chunk = args
                .windows(2)
                .find(|w| w[0] == "--chunk-size")
                .and_then(|w| w[1].parse().ok())
                .unwrap_or(0);
            let bundle = repo.join("tools/standard_benchmarks/bundled/locomo_qa_slice30.json");
            run_locomo_slice(&bundle, limit, max_lines, chunk, fast, &eval_ctrl).map(|rep| {
                let doc = serde_json::json!({
                    "protocol": rep.protocol,
                    "n": rep.n,
                    "correct": rep.correct,
                    "accuracy": rep.accuracy,
                    "evidence_scoring": rep.evidence_scoring,
                    "qa_scoring": rep.qa_scoring,
                    "scoring": "score_qa_algebra (qa_adaptive)",
                    "dynamic_fragments": rep.dynamic_fragments,
                    "fragment_mode": rep.fragment_mode,
                    "max_lines": max_lines,
                    "chunk_size": chunk,
                    "paused": rep.paused,
                    "questions_done": rep.questions_done,
                    "rows": rep.rows.iter().map(|r| serde_json::json!({
                        "id": r.id,
                        "correct": r.correct,
                        "answer": r.answer,
                        "session_lines_used": r.session_lines_used,
                    })).collect::<Vec<_>>(),
                });
                write_report(&reports.join("locomo_v1_rust_test.json"), &doc).ok();
                doc
            })
        }
        "longbench" => {
            let limit = args
                .windows(2)
                .find(|w| w[0] == "--limit")
                .and_then(|w| w[1].parse().ok())
                .unwrap_or(0);
            let max_corpus = args
                .windows(2)
                .find(|w| w[0] == "--max-corpus-lines")
                .and_then(|w| w[1].parse().ok())
                .unwrap_or(0);
            let low_resource = args.iter().any(|a| a == "--low-resource");
            let mcq_light = !args.iter().any(|a| a == "--no-mcq-light");
            let bundle =
                repo.join("tools/standard_benchmarks/bundled/longbench_v2_mcq_short20.json");
            run_longbench_slice(&bundle, limit, max_corpus, fast, low_resource, mcq_light).map(|rep| {
                let doc = serde_json::json!({
                    "protocol": rep.protocol,
                    "warmup_mode": rep.warmup_mode,
                    "n": rep.n,
                    "correct": rep.correct,
                    "accuracy": rep.accuracy,
                    "dynamic_fragments": rep.dynamic_fragments,
                    "fragment_mode": rep.fragment_mode,
                    "max_corpus_lines": max_corpus,
                    "mcq_light": mcq_light,
                    "rows": rep.rows.iter().map(|r| serde_json::json!({
                        "id": r.id,
                        "correct": r.correct,
                        "answer": r.answer,
                        "domain": r.domain,
                        "difficulty": r.difficulty,
                        "context_truncated": r.context_truncated,
                    })).collect::<Vec<_>>(),
                });
                write_report(&reports.join("longbench_v2_v1_test.json"), &doc).ok();
                doc
            })
        }
        "frames" => {
            let limit = args
                .windows(2)
                .find(|w| w[0] == "--limit")
                .and_then(|w| w[1].parse().ok())
                .unwrap_or(if fast { 5 } else { 0 });
            let max_corpus = args
                .windows(2)
                .find(|w| w[0] == "--max-corpus-lines")
                .and_then(|w| w[1].parse().ok())
                .unwrap_or(0);
            let offline_wiki = args.iter().any(|a| a == "--offline-wiki");
            let bundle = if fast && limit > 0 && limit <= 3 {
                repo.join("tools/standard_benchmarks/bundled/frames_slice_smoke3.json")
            } else {
                repo.join("tools/standard_benchmarks/bundled/frames_slice25.json")
            };
            run_frames_slice(&repo, &bundle, limit, max_corpus, fast, offline_wiki, &eval_ctrl).map(|rep| {
                let doc = serde_json::json!({
                    "protocol": rep.protocol,
                    "n": rep.n,
                    "correct": rep.correct,
                    "accuracy": rep.accuracy,
                    "dynamic_fragments": rep.dynamic_fragments,
                    "fragment_mode": rep.fragment_mode,
                    "fragment_only": rep.fragment_only,
                    "wiki_items_fetched": rep.wiki_items_fetched,
                    "source": rep.source,
                    "citation": rep.citation,
                    "baseline_no_rag": rep.baseline_no_rag,
                    "baseline_oracle": rep.baseline_oracle,
                    "max_corpus_lines": max_corpus,
                    "paused": rep.paused,
                    "questions_done": rep.questions_done,
                    "rows": rep.rows.iter().map(|r| serde_json::json!({
                        "id": r.id,
                        "correct": r.correct,
                        "answer": r.answer,
                        "gold": r.gold,
                        "corpus_lines": r.corpus_lines,
                        "wiki_fetched": r.wiki_fetched,
                    })).collect::<Vec<_>>(),
                });
                write_report(&reports.join("frames_v1_rust_test.json"), &doc).ok();
                doc
            })
        }
        "corpus" => {
            let dirs = default_corpus_dirs(&repo);
            run_local_corpus_demo(&dirs).map(|doc| {
                write_report(&reports.join("local_corpus_demo_rust.json"), &doc).ok();
                doc
            })
        }
        "multihop" => {
            let limit = args
                .windows(2)
                .find(|w| w[0] == "--limit")
                .and_then(|w| w[1].parse().ok())
                .unwrap_or(if fast { 5 } else { 0 });
            let bundle = repo.join("tools/standard_benchmarks/bundled/multihop_rag_slice25.json");
            run_multihop_rag(&bundle, limit, fast, 0).map(|doc| {
                write_report(&reports.join("multihop_rag_v2_test.json"), &doc).ok();
                doc
            })
        }
        "multihop-fate" => {
            let limit = args
                .windows(2)
                .find(|w| w[0] == "--limit")
                .and_then(|w| w[1].parse().ok())
                .unwrap_or(if fast { 5 } else { 0 });
            let bundle = repo.join("tools/standard_benchmarks/bundled/multihop_rag_slice25.json");
            run_multihop_fate_rag(&bundle, limit, fast, 0).map(|doc| {
                write_report(&reports.join("multihop_rag_fate_test.json"), &doc).ok();
                doc
            })
        }
        "multihop-hybrid" => {
            let limit = args
                .windows(2)
                .find(|w| w[0] == "--limit")
                .and_then(|w| w[1].parse().ok())
                .unwrap_or(if fast { 5 } else { 0 });
            let bundle = repo.join("tools/standard_benchmarks/bundled/multihop_rag_slice25.json");
            run_multihop_hybrid_rag(&bundle, limit, fast, 0).map(|doc| {
                write_report(&reports.join("multihop_rag_hybrid_test.json"), &doc).ok();
                doc
            })
        }
        "lara" | "lara-fatewithv2" => {
            let limit = args
                .windows(2)
                .find(|w| w[0] == "--limit")
                .and_then(|w| w[1].parse().ok())
                .unwrap_or(if fast { 5 } else { 0 });
            let bundle = repo.join("tools/standard_benchmarks/bundled/lara_rag_vs_lc_slice20.json");
            run_lara_fatewithv2(&bundle, limit, fast).map(|doc| {
                write_report(&reports.join("lara_fatewithv2_test.json"), &doc).ok();
                doc
            })
        }
        "" | "help" | "--help" | "-h" => {
            eprintln!("usage: valhalla_base demo v1|needle|needle-prod|longchain-fate|corpus|locomo|frames|longbench|lara|multihop|multihop-fate|multihop-hybrid [--fast] [--production] [--repo ROOT] [--limit N] [--max-lines N] [--max-corpus-lines N] [--mcq-light|--no-mcq-light] [--chunk-size N] [--pretrain-passes N] [--depth N] [--offline-wiki] [--verbose|-v] [--checkpoint PATH] [--checkpoint-every N] [--resume PATH] [--pause-file PATH]");
            return;
        }
        other => Err(ValhallaBaseErrorBody::new(
            hub_f64::valhalla_base_protocol::ValhallaBaseErrorKind::Validate,
            format!("unknown demo {other:?}; use v1, needle, corpus, multihop, or multihop-fate"),
        )),
    };

    match result {
        Ok(doc) => {
            println!("{}", serde_json::to_string_pretty(&doc).expect("serialize"));
            if let Some(v) = doc.get("verdict").and_then(|v| v.as_str()) {
                eprintln!("verdict={v}");
            }
            if let Some(v) = doc.get("paused").and_then(|v| v.as_bool()) {
                if v {
                    eprintln!("paused=true (checkpoint saved — use --resume to continue)");
                }
            }
            if let (Some(c), Some(n)) = (
                doc.get("correct").and_then(|v| v.as_u64()),
                doc.get("n").and_then(|v| v.as_u64()),
            ) {
                eprintln!("score={c}/{n}");
            }
        }
        Err(e) => fail(e, false, false),
    }
}

fn cmd_serve(args: &[String]) {
    let port: u16 = args
        .windows(2)
        .find(|w| w[0] == "--port")
        .and_then(|w| w[1].parse().ok())
        .or_else(|| {
            env::var("VALHALLA_BASE_PORT")
                .ok()
                .and_then(|s| s.parse().ok())
        })
        .unwrap_or(8787);
    let pretty = args.iter().any(|a| a == "--pretty");
    let with_dashboard = args.iter().any(|a| a == "--with-dashboard")
        || env::var("VALHALLA_SERVE_DASHBOARD")
            .ok()
            .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
            .unwrap_or(false);
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
    if let Err(e) = rt.block_on(valhalla_base_http::serve(addr, pretty, with_dashboard)) {
        fail(
            ValhallaBaseErrorBody::new(
                hub_f64::valhalla_base_protocol::ValhallaBaseErrorKind::Io,
                format!("serve: {e}"),
            ),
            false,
            false,
        );
    }
}

fn read_input(path: Option<&str>) -> Result<String, ValhallaBaseErrorBody> {
    match path {
        Some(p) => fs::read_to_string(p).map_err(|e| {
            ValhallaBaseErrorBody::new(
                hub_f64::valhalla_base_protocol::ValhallaBaseErrorKind::Io,
                format!("read {p}: {e}"),
            )
        }),
        None => {
            let mut input = String::new();
            io::stdin().read_to_string(&mut input).map_err(|e| {
                ValhallaBaseErrorBody::new(
                    hub_f64::valhalla_base_protocol::ValhallaBaseErrorKind::Io,
                    format!("read stdin: {e}"),
                )
            })?;
            Ok(input)
        }
    }
}

fn cmd_json(args: &[String]) {
    let mut pretty = false;
    let mut json_error = false;
    let mut file: Option<String> = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-h" | "--help" => {
                print_help();
                return;
            }
            "-V" | "--version" => {
                println!("valhalla_base {VERSION} protocol={VALHALLA_BASE_PROTOCOL}");
                return;
            }
            "--pretty" => pretty = true,
            "--json-error" => json_error = true,
            "-e" | "--example" => {
                i += 1;
                let name = args.get(i).map(|s| s.as_str()).unwrap_or("");
                match example_request(name) {
                    Ok(req) => {
                        let out = if pretty {
                            serde_json::to_string_pretty(&req).expect("serialize")
                        } else {
                            serde_json::to_string(&req).expect("serialize")
                        };
                        println!("{out}");
                        return;
                    }
                    Err(e) => {
                        eprintln!(
                            "unknown --example {name:?}; available: {}",
                            example_names().join(", ")
                        );
                        fail(e, json_error, pretty);
                    }
                }
            }
            "--validate" => {
                i += 1;
                let path = args.get(i).map(|s| s.as_str()).unwrap_or("");
                if path.is_empty() {
                    fail(
                        ValhallaBaseErrorBody::new(
                            hub_f64::valhalla_base_protocol::ValhallaBaseErrorKind::Validate,
                            "--validate requires a file path",
                        ),
                        json_error,
                        pretty,
                    );
                }
                let raw = read_input(Some(path)).unwrap_or_else(|e| fail(e, json_error, pretty));
                match validate_request_json(&raw) {
                    Ok(req) => {
                        eprintln!(
                            "{}: validate OK — context={} questions={} proactive_idle={}",
                            VALHALLA_BASE_PROTOCOL,
                            req.context.len() + req.context_items.len(),
                            req.questions.len(),
                            req.proactive_idle
                        );
                        return;
                    }
                    Err(e) => fail(e, json_error, pretty),
                }
            }
            arg if arg.starts_with('-') => {
                fail(
                    ValhallaBaseErrorBody::new(
                        hub_f64::valhalla_base_protocol::ValhallaBaseErrorKind::Validate,
                        format!("unknown flag {arg}"),
                    ),
                    json_error,
                    pretty,
                );
            }
            path => file = Some(path.to_string()),
        }
        i += 1;
    }

    let raw = read_input(file.as_deref()).unwrap_or_else(|e| fail(e, json_error, pretty));
    if raw.trim().is_empty() {
        print_help();
        fail(
            ValhallaBaseErrorBody::new(
                hub_f64::valhalla_base_protocol::ValhallaBaseErrorKind::Validate,
                "empty input; pass JSON on stdin or provide request.json",
            ),
            json_error,
            pretty,
        );
    }

    match execute_request_json(&raw, pretty) {
        Ok(out) => {
            io::stdout().write_all(out.as_bytes()).expect("write stdout");
            io::stdout().write_all(b"\n").ok();
        }
        Err(e) => fail(e, json_error, pretty),
    }
}

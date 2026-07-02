//! ValhallaBase unified CLI — JSON · ask · demo · serve

use std::env;
use std::fs;
use std::io::{self, Read, Write};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::process;

use hub_f64::valhalla_base::VALHALLA_BASE_PROTOCOL;
use hub_f64::valhalla_base_bench::{
    default_corpus_dirs, run_local_corpus_demo, run_needle_session, run_v1_extensions,
    write_report,
};
use hub_f64::valhalla_base_client::ValhallaBaseClient;
use hub_f64::valhalla_base_http;
use hub_f64::valhalla_base_protocol::{
    example_names, example_request, execute_request_json, serialize_error, validate_request_json,
    ValhallaBaseErrorBody,
};

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() {
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
  demo v1|needle|corpus [--fast] [--repo ROOT]
  serve [--port 8787] [--pretty]

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
            run_needle_session(&depths, fast).map(|doc| {
                write_report(&reports.join("needle_session_v1_test.json"), &doc).ok();
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
        "" | "help" | "--help" | "-h" => {
            eprintln!("usage: valhalla_base demo v1|needle|corpus [--fast] [--repo ROOT]");
            return;
        }
        other => Err(ValhallaBaseErrorBody::new(
            hub_f64::valhalla_base_protocol::ValhallaBaseErrorKind::Validate,
            format!("unknown demo {other:?}; use v1, needle, or corpus"),
        )),
    };

    match result {
        Ok(doc) => {
            println!("{}", serde_json::to_string_pretty(&doc).expect("serialize"));
            if let Some(v) = doc.get("verdict").and_then(|v| v.as_str()) {
                eprintln!("verdict={v}");
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
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
    if let Err(e) = rt.block_on(valhalla_base_http::serve(addr, pretty)) {
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

//! Valhalla-as-base CLI — stdin JSON → stdout JSON.
//!
//! Context is the substrate; LM is optional decode head (runtime patch per context bundle).

use std::io::{self, Read, Write};

use hub_f64::valhalla_base::{run_valhalla_base, ValhallaBaseRequest, VALHALLA_BASE_PROTOCOL};

fn main() {
    let mut input = String::new();
    io::stdin()
        .read_to_string(&mut input)
        .expect("read stdin");
    let req: ValhallaBaseRequest = serde_json::from_str(&input).unwrap_or_else(|e| {
        eprintln!("{VALHALLA_BASE_PROTOCOL}: parse request: {e}");
        eprintln!(
            "Expected JSON: {{\"context\": [\"...\"], \"questions\": [{{\"id\",\"text\",\"score_type\"}}], \"config\": {{...}}}}"
        );
        std::process::exit(1);
    });
    eprintln!(
        "valhalla_base {} context={} questions={} proactive_idle={} decode={:?}",
        VALHALLA_BASE_PROTOCOL,
        req.context.len(),
        req.questions.len(),
        req.proactive_idle,
        req.config.decode
    );
    match run_valhalla_base(&req) {
        Ok(resp) => {
            let out = serde_json::to_string_pretty(&resp).expect("serialize");
            io::stdout().write_all(out.as_bytes()).expect("write");
            io::stdout().write_all(b"\n").ok();
        }
        Err(e) => {
            eprintln!("valhalla_base error: {e}");
            std::process::exit(1);
        }
    }
}

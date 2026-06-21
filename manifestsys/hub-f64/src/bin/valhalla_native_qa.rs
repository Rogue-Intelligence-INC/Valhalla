//! Tier B native Q&A CLI — JSON stdin/stdout, no HuggingFace.
//!
//! Single question: `NativeQaRequest`
//! Batch incubation: JSON with `questions` array + `corpus_ports`

use std::io::{self, Read};

use hub_f64::native_qa::{
    run_native_qa, run_native_qa_batch, NativeQaBatchRequest, NativeQaRequest,
};

fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).expect("read stdin");
    let v: serde_json::Value = serde_json::from_str(&input).expect("parse JSON stdin");

    if v.get("questions")
        .and_then(|q| q.as_array())
        .map(|a| !a.is_empty())
        .unwrap_or(false)
    {
        let req: NativeQaBatchRequest =
            serde_json::from_value(v).expect("parse NativeQaBatchRequest JSON");
        let resp = run_native_qa_batch(&req);
        println!("{}", serde_json::to_string_pretty(&resp).expect("serialize"));
        return;
    }

    let req: NativeQaRequest = serde_json::from_str(&input).expect("parse NativeQaRequest JSON");
    let resp = run_native_qa(&req);
    println!("{}", serde_json::to_string_pretty(&resp).expect("serialize"));
}

//! Single Fate QA — unified RelAddr + fatewithv2 stack.
//!
//! Ingress/egress: [`crate::fate_io`] (generic adapters).
//! Decode: ValhallaBase with `open_fate` + `multihop_v2` (former peak hybrid arm).
//! Internal land: FateRelAddr — absolute IDs only at ingress/egress.

use crate::fate_adaptive_algebra::FateAlgebraState;
use crate::fate_io::{apply_unified_solution, fate_ingress};
use crate::fate_parallel_longchain::{run_parallel_pipes, spread_from_client};
use crate::valhalla_base::ValhallaBaseQuestion;
use crate::valhalla_base_client::{RunOptions, ValhallaBaseClient};
use crate::valhalla_base_protocol::ValhallaBaseErrorBody;

#[derive(Debug, Clone, Copy)]
pub struct FateUnifiedQaOpts {
    pub isolated: bool,
    pub reanchor: bool,
    pub parallel_fallback: bool,
    /// When true, RelAddr-ingress retrieved lines before decode (generic entry).
    pub use_fate_ingress: bool,
}

impl Default for FateUnifiedQaOpts {
    fn default() -> Self {
        Self {
            isolated: false,
            reanchor: false,
            parallel_fallback: true,
            use_fate_ingress: true,
        }
    }
}

impl FateUnifiedQaOpts {
    pub fn persistent() -> Self {
        Self::default()
    }

    pub fn holdout_isolated() -> Self {
        Self {
            isolated: true,
            reanchor: false, // keep inherited session so FateRelAddr sees full corpus
            parallel_fallback: true,
            use_fate_ingress: true,
        }
    }

    pub fn pretrain_low_resource() -> Self {
        Self {
            isolated: false,
            reanchor: false,
            parallel_fallback: false,
            use_fate_ingress: false,
        }
    }
}

/// Decode on slim worker; unified solution flags + optional RelAddr ingress.
pub fn run_fate_unified_qa(
    client: &mut ValhallaBaseClient,
    corpus: &[String],
    q: &ValhallaBaseQuestion,
    opts: FateUnifiedQaOpts,
) -> Result<String, ValhallaBaseErrorBody> {
    let mut worker = client.spawn_fragment_qa_worker();
    apply_unified_solution(&mut worker.config);
    worker.config.fate_qa_feedback = false;

    if opts.use_fate_ingress && (opts.reanchor || worker.session().is_none()) {
        let state = FateAlgebraState::ambient();
        let bundle = fate_ingress(q, corpus, &state, None, None, None);
        if !bundle.lines.is_empty() {
            worker.ingest_retrieved_lines(&bundle.lines);
        } else if worker.session().is_none() {
            worker.set_context(corpus.iter().take(12).map(|s| s.as_str()));
        }
    }

    let answer = if worker.session().is_some() && !opts.reanchor {
        // Inherited session: FateRelAddr navigation on full corpus (no set_context wipe).
        let resp = worker.run(vec![q.clone()], RunOptions::default())?;
        resp.rows
            .first()
            .map(|r| r.answer.clone())
            .unwrap_or_default()
    } else if opts.parallel_fallback {
        let (ans, _, _, _) = run_parallel_pipes(corpus, q, &mut worker)?;
        ans
    } else {
        if worker.session().is_none() {
            worker.set_context(corpus.iter().take(12).map(|s| s.as_str()));
        }
        let resp = worker.run(vec![q.clone()], RunOptions::default())?;
        resp.rows
            .first()
            .map(|r| r.answer.clone())
            .unwrap_or_default()
    };

    let _ = spread_from_client(&worker);
    worker.teardown();
    client.trim_fate_rollbacks();
    client.shrink_ingress_buffers();
    Ok(answer)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opts_presets() {
        let h = FateUnifiedQaOpts::holdout_isolated();
        assert!(h.isolated && !h.reanchor && h.use_fate_ingress);
        let p = FateUnifiedQaOpts::persistent();
        assert!(!p.isolated && !p.reanchor && p.use_fate_ingress);
        let lr = FateUnifiedQaOpts::pretrain_low_resource();
        assert!(!lr.parallel_fallback && !lr.use_fate_ingress);
    }
}

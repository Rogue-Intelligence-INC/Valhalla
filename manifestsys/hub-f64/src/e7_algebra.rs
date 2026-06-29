//! E7-φ algebraic rationals for FOG gates and Fate corpus feedback.
//!
//! Integers only in definitions: r=7, δ=5, dim=133 → base_α=138.
//!   τ(k) = (r+δ)/(dim+δ+k)
//!   Δ(s₁) = s₁·δ/(dim+r)
//!   affinity⁺ = align·τ(0)     affinity⁻ = align·δ/(dim+r)

pub const R: usize = 7;
pub const DELTA: usize = 5;
pub const DIM: usize = 133;
pub const BASE_ALPHA: usize = DIM + DELTA;

/// τ(k) = (r+δ)/(dim+δ+k)
pub fn tau(keyword_count: usize) -> f64 {
    (R + DELTA) as f64 / (BASE_ALPHA + keyword_count) as f64
}

/// Δ_req(s₁) = s₁·δ/(dim+r)
pub fn delta_required(top_score: f64) -> f64 {
    top_score * DELTA as f64 / (DIM + R) as f64
}

/// δ/(dim+r) — negative coupling scale
pub fn neg_coupling() -> f64 {
    DELTA as f64 / (DIM + R) as f64
}

/// Fate QA pass intensity (replaces fitted 0.75)
pub fn qa_pass_intensity() -> f64 {
    tau(0)
}

/// Fate QA fail intensity (replaces fitted 0.25)
pub fn qa_fail_intensity() -> f64 {
    neg_coupling()
}

/// Corpus-line affinity update on correct retrieval
pub fn affinity_pos_delta(align: f64) -> f64 {
    align.max(0.0) * tau(0)
}

/// Corpus-line affinity penalty on wrong retrieval
pub fn affinity_neg_delta(align: f64) -> f64 {
    align.max(0.0) * neg_coupling()
}

/// Open rerank weight tied to question keyword count (same τ scale as FOG)
pub fn corpus_rerank_weight(keyword_count: usize) -> f64 {
    tau(keyword_count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tau_at_zero_matches_twelve_over_138() {
        let t = tau(0);
        assert!((t - 12.0 / 138.0).abs() < 1e-12);
    }

    #[test]
    fn delta_required_unit_score() {
        assert!((delta_required(1.0) - 5.0 / 140.0).abs() < 1e-12);
    }

    #[test]
    fn affinity_deltas_scale_with_align() {
        let a = 0.8;
        assert!((affinity_pos_delta(a) - a * 12.0 / 138.0).abs() < 1e-12);
        assert!((affinity_neg_delta(a) - a * 5.0 / 140.0).abs() < 1e-12);
    }
}

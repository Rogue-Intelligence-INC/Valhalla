# Hybrid 3.0 Rust — Test Report (2026-06-23)

**Product**: `hybrid-3.0-rust` · **Binary**: `target/release/valhalla_hybrid`  
**Suite**: `tools/fair_benchmark/fair_benchmark_suite.json` · **Phase**: test (145Q)  
**Patch rows**: reused from `hybrid_unified_test_20260623_0242.json` (Tier A LM answers)

## Summary

| Mode | Acc | Correct | vs Python 2.1 |
|------|-----|---------|---------------|
| **deploy** | **77.24%** | 112/145 | 77.2% ✓ |
| **oracle** | **80.00%** | 116/145 | 80.0% ✓ |

## By score type (deploy)

| Type | Acc |
|------|-----|
| open | 95.08% (58/61) |
| mcq | 52.17% (24/46) |
| numeric | 78.95% (30/38) |

## By score type (oracle)

| Type | Acc |
|------|-----|
| open | 95.08% (58/61) |
| mcq | 54.35% (25/46) |
| numeric | 86.84% (33/38) |

## Rust stack

| Module | Role |
|--------|------|
| `grinder.rs` | Python-compatible hash ports |
| `fair_score.rs` | `score_answer`, prompt formatting |
| `fair_suite.rs` | Suite load / phase selection |
| `hybrid.rs` | Memory tracks + deploy/oracle merge |
| `native_qa.rs` | `mcq_fate` request flag |

## Run

```bash
cargo build -p hub-f64 --release --bin valhalla_hybrid
./target/release/valhalla_hybrid --phase test \
  --patch-json reports/hybrid_valhalla_lm/hybrid_unified_test_20260623_0242.json \
  --merge-mode deploy
```

Artifacts: `hybrid_rust_test_deploy_20260623.json`, `hybrid_rust_test_oracle_20260623.json`

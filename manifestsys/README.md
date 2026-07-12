# manifestsys

| | |
|--|--|
| **Monorepo label** | **Product** |
| **Contents** | ValhallaBase runtime — Hub · Tile · StemCell · Fate (`hub-f64`, `valhalla_base`) |
| **Benchmarks** | [`../reports/valhalla_inference/`](../reports/valhalla_inference/) |
| **Map** | [`../REPO_MAP.md`](../REPO_MAP.md) |

Primary crate: [`hub-f64/`](hub-f64/) — build with `cargo build -p hub-f64 --release --features patch-lm --bin valhalla_base`.

## Open-QA stack (2026-07-13)

| Module | File | Role |
|--------|------|------|
| **fate_io** | `hub-f64/src/fate_io.rs` | RelAddr ingress · egress cites · unified solution flags |
| **open_qa_interface** | `hub-f64/src/open_qa_interface.rs` | Programmatic `OpenQaRequest` / `run_open_qa()` |
| **multihop_bench** | `hub-f64/src/multihop_bench.rs` | MultiHop / LaRA JSON benchmark runners |
| **elastic_resources** | `hub-f64/src/elastic_resources.rs` | CUDA-aware parallel + ingress budget |
| **valhalla_base_bench** | `hub-f64/src/valhalla_base_bench.rs` | Needle · LoCoMo · re-exports multihop bench |

Bench calls interface; interface calls `fate_io` + `native_qa_batch`. See [`docs/valhalla/VALHALLA_BASE_CLI.md`](../docs/valhalla/VALHALLA_BASE_CLI.md).

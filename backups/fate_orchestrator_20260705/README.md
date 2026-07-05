# Fate Orchestrator / Manifold v1 — backup 2026-07-05

Pre-change copies: `signal_ingress.rs`, `valhalla_base.rs`, `hub_lib.rs`, `fate_f64_lib.rs`

## Architecture (valhalla-manifold-v1)

| Component | Role |
|-----------|------|
| `SnapshotFragment` | Per-line / per-modality composable structure + Fate state |
| `ManifoldFragmentLibrary` | Offline pretrain JSON store + session reload |
| `FateOrchestrator` | Fate-owned adapt cycles (no Tile/Stem early break) |
| `TriadIncubationSnapshot` | Full manifold: fragments[], fate_convergence, hub_triple_fate, patch_vector |
| Live session | `VALHALLA_LIVE_SESSION=1` (default) — batch QA without per-question fork |
| Multi-fragment | `VALHALLA_MULTI_FRAGMENT=1` (default) — composable fragments |
| Fragment mode | `VALHALLA_FRAGMENT_MODE=density` (default) — Fate density-drop segments; `line` = per corpus line |

## Fragment modes

- **`density`** (default): segment when Fate marginal signal density / preference output rate drops
- **`line`**: one fragment per corpus line (legacy v1)
- Env: `VALHALLA_FRAGMENT_DENSITY_DROP=0.35`, `VALHALLA_FRAGMENT_MIN_LINES=2`, `VALHALLA_FRAGMENT_MAX_LINES=48`

## Query-conditioned merge (QA)

- `VALHALLA_QUERY_FRAGMENT_MERGE=1` (default): pick top-k fragments by question affinity, merge for QA substrate
- `VALHALLA_QUERY_FRAGMENT_TOP_K=3`
- Scoring: patch cosine + line-range affinity + keyword overlap on fragment label
- Batch QA response fields: `query_fragments`, `query_fragment_scores`; decode_path prefix `query_frag/kN/ids=...`

## Binaries

```bash
LIBRARY_PATH=/opt/cuda/targets/x86_64-linux/lib:$LIBRARY_PATH \
  cargo build --release -p hub-f64 \
  --bin manifold_orchestrator_smoke --bin manifold_pretrain --bin valhalla_base

./target/release/manifold_orchestrator_smoke
./target/release/manifold_pretrain --out reports/manifold_pretrain_library.json
./target/release/manifold_pretrain --corpus my_corpus.txt --out out.json
```

## Opt-out (legacy)

- `VALHALLA_FATE_ORCHESTRATE=0` — fixed incubation cycles
- `VALHALLA_LIVE_SESSION=0` — per-question snapshot fork in batch QA
- `VALHALLA_MULTI_FRAGMENT=0` — monolithic snapshot only

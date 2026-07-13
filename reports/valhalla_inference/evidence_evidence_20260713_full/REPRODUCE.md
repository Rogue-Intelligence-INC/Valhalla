# Valhalla Evidence Pack — Reproduce

**Tag:** `evidence_20260713_full`  
**Git SHA:** `c314282a4bc5a4ea9cdb3f5c89427e643b8433e3`  
**Packed:** 2026-07-13T11:39:11.480678+00:00  
**Backbone (industry calibration):** `Qwen/Qwen2.5-3B-Instruct`

## What this pack proves

1. **Suite** — bundled industry slices + runner scripts (see `suite_manifest.json`)
2. **Logs** — timestamped execution traces under `logs/`
3. **Reports** — machine-readable JSON under `reports/`
4. **Legitimacy** — `legitimacy_audit.json` marks citeable vs Tier-C (non-citeable)
5. **Provenance** — upstream dataset citations under `provenance/`
6. **Hashes** — `HASHES.sha256` for integrity

## One-shot industry suite (from repo root)

```bash
export RUSTFLAGS="-L /opt/cuda/lib64"
export LIBRARY_PATH="/opt/cuda/lib64:${LIBRARY_PATH:-}"
export HF_ENDPOINT="${HF_ENDPOINT:-https://hf-mirror.com}"
export VALHALLA_BASE_NO_SERVE=1
cargo build -p hub-f64 --release --bin valhalla_base
.venv-llm/bin/python3 tools/valhalla_inference/run_industry_benchmarks.py --all --fresh --verbose
.venv-llm/bin/python3 tools/valhalla_inference/audit_benchmark_legitimacy.py --json
.venv-llm/bin/python3 tools/valhalla_inference/pack_evidence_run.py --tag reproduce
```

## Ablation + competitor baselines

```bash
./tools/valhalla_inference/run_fundraise_ablation.sh --industry-only
.venv-llm/bin/python3 tools/valhalla_inference/run_competitor_compare.py --moderate
```

## Citeability rule

- Only reports with `citeable=true` in `legitimacy_audit.json` should appear on pitch homepage.
- FRAMES: run `./scripts/prefetch_frames_wiki.sh` (parasail HF wiki dump) before citing; without extracts = Tier C.
- TPI / NPPI are **mechanism** metrics (secondary), not industry SOTA claims.

## Competitor compare policy

See `COMPETITOR_COMPARE.md` in this pack.

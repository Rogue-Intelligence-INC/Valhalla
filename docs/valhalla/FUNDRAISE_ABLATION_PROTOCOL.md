# Fundraise Ablation Matrix — Protocol

**Protocol ID:** `fundraise-ablation-matrix-v1`  
**Purpose:** Paired industry + component ablation for fundraising / incubation-tool narrative.

## Arms (7)

| Arm | Description |
|-----|-------------|
| `full_context_lm` | Entire corpus in LM prompt each query |
| `vanilla_rag` | Traditional RAG block + question |
| `lm_patch` | Valhalla ingest + `decode=lm_patch` |
| `native` | Valhalla `decode=native` (structure decode) |
| `hybrid_full` | Production stack (Fate + multihop + triad) |
| `hybrid_no_fate` | Hybrid with `open_fate=0`, `multihop_fate=0` |
| `hybrid_tile` | Hybrid with `patch_body=tile` (no triad routing) |

## Benchmarks (moderate scale)

| Benchmark | Default n | Scoring |
|-----------|-----------|---------|
| LoCoMo | 15 / 30 | `score_short_answer` |
| MultiHop-RAG | 15 / 25 | `score_short_answer` |
| RAGTruth | 40 | faithfulness gate |
| LongBench-v2 MCQ | 10 / 20 | letter match |

**Backbone:** `Qwen/Qwen2.5-3B-Instruct` (industry calibration)

## Paradigm batteries (`--with-paradigm`)

- TPI-v2 (`--skip-cycles`, eval-limit 15)
- NPPI (full protocol)
- Token efficiency (with Valhalla append run)
- External Pareto matrix (LoCoMo 15 + LongBench 10)

## Commands

```bash
# Moderate industry + paradigm (recommended) — live tee + stop-on-error
./tools/valhalla_inference/run_fundraise_ablation.sh

# Industry ablation only
./tools/valhalla_inference/run_fundraise_ablation.sh --industry-only

# Smoke
./tools/valhalla_inference/run_fundraise_ablation.sh --fast

# Resume after pause / crash
./tools/valhalla_inference/run_fundraise_ablation.sh --resume

# Full scale
python3 tools/valhalla_inference/run_fundraise_ablation_matrix.py --with-paradigm --verbose
```

## Control (monitor · pause · error auto-stop)

| Action | How |
|--------|-----|
| Live monitor | `tail -f reports/valhalla_inference/fundraise_ablation_run.log` |
| Status | `cat reports/valhalla_inference/fundraise_ablation.status` |
| Pause | `touch reports/valhalla_inference/ablation.pause` |
| Resume | `rm …/ablation.pause && ./run_fundraise_ablation.sh --resume` |
| Error auto-stop | default `--stop-on-error` (abort cell + write status=error) |
| Continue on error | `./run_fundraise_ablation.sh --continue-on-error` |

CUDA / threads (ValhallaBase):
- `VALHALLA_CUDA_DEVICE` (default 0)
- `VALHALLA_USE_CPU=0` · Candle CUDA probe + libcudart
- CUDA default: serial outer LM workers (VRAM-safe); `VALHALLA_BENCH_PARALLEL=N` to fan out with infer lock
- Multihop demos honor `--verbose` / `--pause-file` / EvalControl

## Outputs

| File | Description |
|------|-------------|
| `reports/valhalla_inference/fundraise_ablation_matrix.json` | Full JSON (incremental) |
| `reports/valhalla_inference/FUNDRAISE_ABLATION_MATRIX.md` | Human-readable matrix |
| `docs/valhalla/FUNDRAISE_ABLATION_MATRIX.md` | Same, for GitHub docs |
| `reports/valhalla_inference/fundraise_ablation_run.log` | Live log (tee) |
| `reports/valhalla_inference/fundraise_ablation.status` | `state=running\|paused\|error\|ok` |
| `reports/valhalla_inference/ablation_ckpt/` | Per-cell resume checkpoint |

## Interpretation

- **Structure premium:** `hybrid_full` vs `vanilla_rag` on LoCoMo / MultiHop
- **Fate lift:** `hybrid_full` vs `hybrid_no_fate`
- **Triad lift:** `hybrid_full` vs `hybrid_tile`
- **Decode path:** `native` vs `lm_patch` vs `hybrid_full`

Paradigm batteries (TPI/NPPI) support the **incubation-tool** story; industry matrix supports **fundraise diligence**.

---

*Results table is auto-generated when the runner completes. See `fundraise_ablation_matrix.json` for latest partial results during long runs.*

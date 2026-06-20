# Valhalla Fair Benchmark

Decontaminated, split-aware benchmark — **memory + patch + generate** tracks.

See **[BENCHMARK_SPEC.md](./BENCHMARK_SPEC.md)** for full protocol.

## Quick start

```bash
cd /home/jouly/AI/Valhalla

python3 tools/fair_benchmark/build_fair_suite.py
python3 tools/fair_benchmark/validate_suite.py

RUSTFLAGS='-L /opt/cuda/lib64' cargo build -p hub-f64 --release \
  --bin valhalla_native_qa --bin valhalla_model_export

# Smoke (12Q · memory only · ~20s)
python3 tools/fair_benchmark/run_fair_benchmark.py --phase smoke

# Complete held-out test (145Q · all tracks)
python3 tools/fair_benchmark/run_fair_benchmark.py --phase test --full

# Sync to valhalla-transformer Viz/API
python3 tools/fair_benchmark/sync_to_valhalla_transformer.py
```

## Tracks

| Track | Binary | Protocol |
|-------|--------|----------|
| **memory_tier_b** | `valhalla_native_qa` | isolated per-Q + `triad_c1_persistent` batch |
| **patch_tier_a** | `valhalla_model_export` + HF | fresh reload per question, legacy@0.08 |
| **generate_baseline** | HF Qwen2.5-0.5B | zero-shot, fresh reload per question |

## Splits

| Split | N | Use |
|-------|---|-----|
| smoke | 12 | CI gate |
| val | 43 | tuning only |
| test | 145 | **primary report** |

## Outputs

- `reports/fair_benchmark/fair_benchmark_*.json`
- `reports/VALHALLA_FAIR_BENCHMARK_*.md`
- `valhalla-transformer/experiments/fair_benchmark_*.json` (after sync)

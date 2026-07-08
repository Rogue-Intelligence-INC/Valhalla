# Valhalla Base — Third-Party Reproduction Guide

**Protocol**: `valhalla-base-v1` · **Target**: independent verification without proprietary context

---

## 1. Prerequisites

| Requirement | Notes |
|-------------|--------|
| Linux x86_64 | Arch/Ubuntu tested |
| Rust 1.75+ | `rustup default stable` |
| Python 3.10+ | benchmark harness only |
| CUDA (optional) | GPU LM; set `VALHALLA_USE_CPU=1` for CPU-only smoke |
| HuggingFace cache | Qwen2.5-3B for full LM benchmarks |

---

## 2. One-command reproduction

```bash
git clone <repo-url> Valhalla && cd Valhalla
cp .env.example .env   # optional — edit paths
make repro
```

This runs: **build → demo v1 smoke → fate-core suite → Rust unit tests**.

Expected artifacts:

- `reports/valhalla_inference/` — JSON benchmark outputs
- `reports/valhalla_inference/suite_summaries/` — per-suite markdown

---

## 3. Step-by-step (manual)

### 3.1 Build

```bash
export RUSTFLAGS='-L /opt/cuda/lib64'   # omit if no CUDA
export VALHALLA_GIT_SHA=$(git rev-parse --short HEAD)
make build
```

Binary: `target/release/valhalla_base`

### 3.2 Triad smoke (Hub + Tile + StemCell)

```bash
./target/release/valhalla_base demo v1
# or
make smoke
```

### 3.3 Tile / StemCell visualization

**Standalone dashboard** (port 8765):

```bash
make dashboard
# Open http://127.0.0.1:8765/
```

**Integrated with infer API** (port 8787):

```bash
make serve-dash
# Open http://127.0.0.1:8787/dashboard/
```

Dashboard shows:

- Tile activation heatmap (per-tile neural activation)
- StemCell organ cluster graph (bubble size ∝ cell count)
- Incubation cycle trace (tile vs stem over optimize cycles)
- Hub quad routing + Fate preferences

### 3.4 HTTP API (programmatic)

```bash
make serve
curl -s http://127.0.0.1:8787/health | jq .
curl -s http://127.0.0.1:8787/v1/repro | jq .
curl -s http://127.0.0.1:8787/v1/protocol | jq .

# One-shot infer
./target/release/valhalla_base --example open | \
  ./target/release/valhalla_base --pretty --json-error
```

**Python wrapper** (auto-starts serve if needed):

```bash
python3 tools/valhalla_inference/vb.py health
python3 tools/valhalla_inference/vb.py demo v1
python3 tools/valhalla_inference/vb.py suite fate-core
```

### 3.5 Benchmark suites

```bash
python3 tools/valhalla_inference/run_suite_sequential.py --list
python3 tools/valhalla_inference/run_suite_sequential.py --suite fate-core
python3 tools/valhalla_inference/run_suite_sequential.py --all
```

| Suite | Contents |
|-------|----------|
| `fate-core` | parallel longchain, pretrain holdout, routing A/B, unified longchain |
| `rust-unit` | evidence scoring + theme drift unit tests |
| `standard-fast` | needle, locomo, frames, multihop, ragtruth, longbench, ruler, lara |
| `context-drift` | persistent-session topic drift battery |

---

## 4. Docker (CPU smoke path)

```bash
docker compose -f docker/docker-compose.yml build
docker compose -f docker/docker-compose.yml run --rm valhalla-base make repro
docker compose -f docker/docker-compose.yml up valhalla-serve
# → http://localhost:8787/dashboard/
```

---

## 5. Environment variables

See `.env.example`. Key variables:

| Variable | Default | Purpose |
|----------|---------|---------|
| `VALHALLA_USE_CPU` | `0` | Force CPU inference |
| `VALHALLA_LM_MODEL` | `Qwen/Qwen2.5-3B-Instruct` | HF model id |
| `VALHALLA_BASE_PORT` | `8787` | HTTP serve port |
| `VALHALLA_SERVE_DASHBOARD` | `0` | Mount `/dashboard` on serve |
| `VALHALLA_DASHBOARD_PORT` | `8765` | Standalone dashboard |
| `RUSTFLAGS` | `-L /opt/cuda/lib64` | CUDA link path |

---

## 6. Pass criteria (smoke)

| Check | Criterion |
|-------|-----------|
| Triad smoke | `overall_pass: true` in dashboard API |
| fate-core | 4/4 suite items exit 0 |
| rust-unit | 6/6 tests pass |
| theme_drift | `theme_drift_rate = 0` in unified longchain JSON |

Latest rollup: `reports/valhalla_inference/suite_summaries/INDEX_FULL_*.md`

---

## 7. Artifacts index

| Path | Description |
|------|-------------|
| `manifestsys/hub-f64/include/valhalla_base.h` | C ABI header |
| `manifestsys/hub-f64/schemas/` | JSON Schema |
| `docs/valhalla/VALHALLA_BASE_CLI.md` | Full CLI reference |
| `tools/valhalla_inference/run_suite_sequential.py` | Suite runner |
| `Makefile` | Build / serve / repro targets |

---

## 8. Licensing

Valhalla Base runtime is proprietary. Third-party reproduction requires written agreement — see `valhalla-unified-core/NON_OPEN_SOURCE.md`.

---

*Updated 2026-07-09 · Rogue Intelligence LNC.*

#!/usr/bin/env bash
# ValhallaBase — one-click highlight test (adapts to host: CPU/GPU, venv, CUDA paths)
#
# Validates product pillars on any Linux machine:
#   · Triad smoke          · RAG faithfulness (RAGTruth)
#   · Session needle       · Proactive gates (MAR/FIR)
#   · Fate holdout unit    · Optional backbone orthogonality (--full)
#   · Trainability L1/L2/L3 (--train-proof)
#
# Usage:
#   ./scripts/run_highlight_test.sh              # ~8–15 min (GPU) / ~20 min (CPU)
#   ./scripts/run_highlight_test.sh --full       # + independence panel + fate-core
#   ./scripts/run_highlight_test.sh --train-proof # + L2 pretrain holdout · L1 TPI · L3 bake
#   ./scripts/run_highlight_test.sh --cpu-only   # force CPU (no CUDA)
#   ./scripts/run_highlight_test.sh --skip-build # reuse existing valhalla_base binary
#
# Env overrides: VALHALLA_LM_MODEL, HF_ENDPOINT, RUSTFLAGS, VALHALLA_USE_CPU

set -euo pipefail

REPO="$(cd "$(dirname "$0")/.." && pwd)"
cd "$REPO"

FULL=0
TRAIN_PROOF=0
CPU_ONLY=0
SKIP_BUILD=0
for arg in "$@"; do
  case "$arg" in
    --full) FULL=1 ;;
    --train-proof) TRAIN_PROOF=1 ;;
    --cpu-only) CPU_ONLY=1 ;;
    --skip-build) SKIP_BUILD=1 ;;
    -h|--help)
      sed -n '2,15p' "$0"
      exit 0
      ;;
    *) echo "Unknown arg: $arg (try --help)" >&2; exit 2 ;;
  esac
done

# ── Python ─────────────────────────────────────────────────────────────────────
if [[ -x "${REPO}/.venv-llm/bin/python3" ]]; then
  PY="${REPO}/.venv-llm/bin/python3"
elif command -v python3 >/dev/null 2>&1; then
  PY=python3
else
  echo "ERROR: python3 not found (install 3.10+ or create .venv-llm)" >&2
  exit 1
fi

# ── CUDA / CPU ───────────────────────────────────────────────────────────────
detect_cuda_lib() {
  for d in \
    /opt/cuda/targets/x86_64-linux/lib \
    /opt/cuda/lib64 \
    /usr/local/cuda/lib64 \
    /usr/lib64; do
    if [[ -d "$d" ]] && ls "$d"/libcuda.so* &>/dev/null 2>&1 || [[ -f "$d"/libcudart.so ]]; then
      echo "$d"
      return 0
    fi
  done
  return 1
}

if [[ "$CPU_ONLY" -eq 1 ]]; then
  export VALHALLA_USE_CPU=1
elif [[ -z "${VALHALLA_USE_CPU:-}" ]]; then
  if command -v nvidia-smi >/dev/null 2>&1 && nvidia-smi -L &>/dev/null; then
    export VALHALLA_USE_CPU=0
  else
    export VALHALLA_USE_CPU=1
    echo "[env] No GPU detected → VALHALLA_USE_CPU=1"
  fi
fi

if [[ "${VALHALLA_USE_CPU}" == "1" ]]; then
  echo "[env] CPU inference mode"
else
  if CUDA_LIB="$(detect_cuda_lib 2>/dev/null || true)"; then
    export LIBRARY_PATH="${CUDA_LIB}${LIBRARY_PATH:+:$LIBRARY_PATH}"
    export RUSTFLAGS="${RUSTFLAGS:--L ${CUDA_LIB}}"
    echo "[env] CUDA lib: $CUDA_LIB"
  else
    export RUSTFLAGS="${RUSTFLAGS:--L /opt/cuda/lib64}"
    echo "[env] CUDA path fallback RUSTFLAGS=$RUSTFLAGS"
  fi
fi

export HF_ENDPOINT="${HF_ENDPOINT:-https://hf-mirror.com}"
export VALHALLA_LM_MODEL="${VALHALLA_LM_MODEL:-Qwen/Qwen2.5-0.5B-Instruct}"
export VALHALLA_INGRESS_RESOURCE_PERCENT="${VALHALLA_INGRESS_RESOURCE_PERCENT:-50}"
export VALHALLA_INGRESS_MAX_ITEMS="${VALHALLA_INGRESS_MAX_ITEMS:-48}"
export VALHALLA_LM_CACHE_MAX="${VALHALLA_LM_CACHE_MAX:-1}"
export VALHALLA_BASE_NO_SERVE="${VALHALLA_BASE_NO_SERVE:-1}"
export VALHALLA_GIT_SHA="${VALHALLA_GIT_SHA:-$(git rev-parse --short HEAD 2>/dev/null || echo dev)}"

VB="${REPO}/target/release/valhalla_base"
CARGO="${CARGO:-cargo}"
PKG=hub-f64
FEATURES=patch-lm

PASS=0
FAIL=0
SKIP=0
RESULTS=()

log() { echo ""; echo "════════════════════════════════════════════════════════"; echo "▶ $*"; echo "════════════════════════════════════════════════════════"; }

run_step() {
  local name="$1"
  shift
  local t0
  t0=$(date +%s)
  if "$@"; then
    local dt=$(( $(date +%s) - t0 ))
    echo "[OK]   $name (${dt}s)"
    PASS=$((PASS + 1))
    RESULTS+=("OK  $name")
    return 0
  else
    local dt=$(( $(date +%s) - t0 ))
    echo "[FAIL] $name (${dt}s)" >&2
    FAIL=$((FAIL + 1))
    RESULTS+=("FAIL $name")
    return 1
  fi
}

run_step_optional() {
  local name="$1"
  shift
  local t0
  t0=$(date +%s)
  if "$@"; then
    local dt=$(( $(date +%s) - t0 ))
    echo "[OK]   $name (${dt}s, optional)"
    PASS=$((PASS + 1))
    RESULTS+=("OK  $name")
  else
    local dt=$(( $(date +%s) - t0 ))
    echo "[SKIP] $name (${dt}s — missing weights or env)"
    SKIP=$((SKIP + 1))
    RESULTS+=("SKIP $name")
  fi
  return 0
}

echo "ValhallaBase highlight test"
echo "  REPO=$REPO"
echo "  PY=$PY"
echo "  LM=$VALHALLA_LM_MODEL"
echo "  CPU=$VALHALLA_USE_CPU  FULL=$FULL  TRAIN_PROOF=$TRAIN_PROOF"

# ── Build ────────────────────────────────────────────────────────────────────
if [[ "$SKIP_BUILD" -eq 0 ]] || [[ ! -x "$VB" ]]; then
  log "Build valhalla_base (release)"
  run_step "cargo build" \
    "$CARGO" build -p "$PKG" --release --bin valhalla_base --features "$FEATURES"
else
  echo "[skip] build (--skip-build, binary exists)"
fi

# ── Pillar tests ───────────────────────────────────────────────────────────
log "Triad smoke (demo v1)"
run_step "triad smoke" "$VB" demo v1

log "Fate holdout Rust unit"
run_step "fate holdout unit" \
  "$CARGO" test -p "$PKG" --lib --features "$FEATURES" fate_pretrain_holdout --release -- --nocapture

log "RAGTruth faithfulness (fast slice)"
run_step "ragtruth fast" \
  "$PY" tools/valhalla_inference/test_ragtruth_v1.py --fast

log "Needle session recall (fast)"
run_step "needle fast" \
  "$PY" tools/valhalla_inference/test_needle_session_v1.py --fast

log "Proactive gates (MAR / FIR)"
run_step "proactive gates" \
  "$PY" tools/valhalla_inference/test_proactive_fate_v2.py --gates-only

if [[ "$FULL" -eq 1 ]]; then
  log "Backbone orthogonality — practical six-class panel (fast)"
  run_step_optional "independence practical" \
    "$PY" tools/valhalla_inference/run_valhalla_independence.py --practical --fast --with-memory

  log "Fate-core suite (smoke — bounded RAM)"
  run_step_optional "fate-core smoke" \
    "$PY" tools/valhalla_inference/run_suite_sequential.py --suite fate-core
fi

if [[ "$TRAIN_PROOF" -eq 1 ]]; then
  log "Trainability proof (L1/L2/L3)"
  TPI_ARGS=()
  if [[ "${VALHALLA_USE_CPU}" == "1" ]]; then
    TPI_ARGS+=(--skip-tpi)
    echo "[hint] CPU mode → skipping L1 TPI transfer (use GPU for full chain)"
  fi
  run_step "train proof smoke" \
    "$PY" tools/valhalla_inference/test_train_proof_smoke.py "${TPI_ARGS[@]}"
fi

# ── Summary ────────────────────────────────────────────────────────────────
echo ""
echo "════════════════════════════════════════════════════════"
echo "Highlight test summary"
echo "  PASS=$PASS  FAIL=$FAIL  SKIP=$SKIP"
for r in "${RESULTS[@]}"; do echo "  $r"; done
echo "  Reports → ${REPO}/reports/valhalla_inference/"
echo "════════════════════════════════════════════════════════"

if [[ "$FAIL" -gt 0 ]]; then
  exit 1
fi
exit 0

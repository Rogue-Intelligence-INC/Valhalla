#!/usr/bin/env bash
# Full fundraise evidence suite — deps · legitimacy · competitor · token · pack
#
# Usage (IDE terminal):
#   ./scripts/run_evidence_suite.sh              # moderate competitor + pack
#   ./scripts/run_evidence_suite.sh --fast        # smoke
#   ./scripts/run_evidence_suite.sh --full        # + ablation industry
#   ./scripts/run_evidence_suite.sh --with-ablation
#
set -uo pipefail
REPO="$(cd "$(dirname "$0")/.." && pwd)"
cd "$REPO"

export RUSTFLAGS="${RUSTFLAGS:--L /opt/cuda/lib64}"
export LIBRARY_PATH="/opt/cuda/lib64:${LIBRARY_PATH:-}"
export HF_ENDPOINT="${HF_ENDPOINT:-https://hf-mirror.com}"
export VALHALLA_BASE_NO_SERVE=1
export VALHALLA_CUDA_DEVICE="${VALHALLA_CUDA_DEVICE:-0}"
export PYTHONUNBUFFERED=1

PY="${REPO}/.venv-llm/bin/python3"
[[ -x "$PY" ]] || PY=python3
REPORTS="$REPO/reports/valhalla_inference"
LOG="$REPORTS/evidence_suite_run.log"
PAUSE="$REPORTS/ablation.pause"
mkdir -p "$REPORTS"
: >"$LOG"

FAST=0
FULL=0
WITH_ABLATION=0
WITH_MEM0=1
for arg in "$@"; do
  case "$arg" in
    --fast) FAST=1 ;;
    --full) FULL=1; WITH_ABLATION=1 ;;
    --with-ablation) WITH_ABLATION=1 ;;
    --no-mem0) WITH_MEM0=0 ;;
  esac
done

log() { echo "[$(date -Iseconds)] $*" | tee -a "$LOG"; }

log "=== evidence suite start fast=$FAST ablation=$WITH_ABLATION mem0=$WITH_MEM0 ==="

# --- 0. deps ---
log "[0] install benchmark deps"
"$PY" -m pip install -q -r requirements-benchmark.txt 2>&1 | tee -a "$LOG" || true
"$PY" - <<'PY' 2>&1 | tee -a "$LOG"
import importlib
for m in ("transformers", "sentence_transformers", "mem0", "qdrant_client"):
    try:
        importlib.import_module(m if m != "sentence_transformers" else "sentence_transformers")
        print(f"  OK {m}")
    except Exception as e:
        print(f"  FAIL {m}: {e}")
PY

# --- 1. binary ---
if [[ ! -x "$REPO/target/release/valhalla_base" ]]; then
  log "[1] build valhalla_base"
  cargo build -p hub-f64 --release --bin valhalla_base 2>&1 | tee -a "$LOG"
else
  log "[1] valhalla_base present"
fi

# --- 2. legitimacy preflight ---
log "[2] legitimacy audit"
"$PY" tools/valhalla_inference/audit_benchmark_legitimacy.py 2>&1 | tee -a "$LOG"
"$PY" tools/valhalla_inference/audit_benchmark_legitimacy.py --json \
  >"$REPORTS/legitimacy_audit.json"

# --- 3. competitor compare ---
log "[3] competitor compare"
CMP_ARGS=(--moderate)
[[ "$FAST" -eq 1 ]] && CMP_ARGS=(--fast)
[[ "$WITH_MEM0" -eq 0 ]] && CMP_ARGS+=(--no-mem0)
"$PY" tools/valhalla_inference/run_competitor_compare.py "${CMP_ARGS[@]}" 2>&1 | tee -a "$LOG"
CMP_RC=${PIPESTATUS[0]}
[[ "$CMP_RC" -ne 0 ]] && log "ABORT competitor rc=$CMP_RC" && exit "$CMP_RC"

# --- 4. token efficiency ---
log "[4] token efficiency"
TOK=(tools/valhalla_inference/test_token_efficiency.py)
[[ "$FAST" -eq 0 ]] && TOK+=(--run-valhalla)
"$PY" "${TOK[@]}" 2>&1 | tee -a "$LOG" || log "WARN token efficiency non-zero"

# --- 5. optional ablation ---
if [[ "$WITH_ABLATION" -eq 1 ]]; then
  log "[5] fundraise ablation industry"
  ./tools/valhalla_inference/run_fundraise_ablation.sh --industry-only 2>&1 | tee -a "$LOG"
  AB_RC=${PIPESTATUS[0]}
  [[ "$AB_RC" -ne 0 && "$AB_RC" -ne 75 ]] && log "ABORT ablation rc=$AB_RC" && exit "$AB_RC"
else
  log "[5] skip ablation (pass --with-ablation or --full)"
fi

# --- 6. integrate + pack ---
log "[6] integrate + evidence pack"
"$PY" tools/valhalla_inference/integrate_valhalla_reports.py 2>&1 | tee -a "$LOG" || true
TAG="evidence_$(date +%Y%m%d)"
"$PY" tools/valhalla_inference/pack_evidence_run.py --tag "$TAG" --zip 2>&1 | tee -a "$LOG"

log "=== evidence suite DONE ==="
log "pack: $REPORTS/evidence_${TAG}/"
log "zip:  $REPORTS/evidence_${TAG}.zip"
log "log:  $LOG"
exit 0

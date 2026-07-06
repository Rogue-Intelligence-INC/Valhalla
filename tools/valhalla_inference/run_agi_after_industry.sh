#!/usr/bin/env bash
# Wait for industry benchmark, then run AGI/training proxies @ 3B.
set -uo pipefail
REPO="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$REPO"
LOG="${1:-reports/valhalla_inference/industry_3b_agi_20260706.log}"
WAIT_PATTERN="${WAIT_PATTERN:-run_industry_benchmarks.py --all}"

export HF_ENDPOINT="${HF_ENDPOINT:-https://hf-mirror.com}"
export LIBRARY_PATH="/opt/cuda/targets/x86_64-linux/lib:${LIBRARY_PATH:-}"
export RUSTFLAGS="-L /opt/cuda/lib64"
export VALHALLA_LM_MODEL="${VALHALLA_LM_MODEL:-Qwen/Qwen2.5-3B-Instruct}"
export VALHALLA_USE_CPU="${VALHALLA_USE_CPU:-0}"

echo "=== AGI QUEUE $(date -Iseconds) ===" | tee -a "$LOG"
echo "Waiting for: $WAIT_PATTERN" | tee -a "$LOG"

while pgrep -f "$WAIT_PATTERN" >/dev/null 2>&1; do
  step=$(ps aux | rg "test_" | rg -v "rg " | head -1 | awk '{for(i=11;i<=NF;i++) printf "%s ", $i; print ""}' || true)
  echo "[$(date -Iseconds)] industry still running … ${step:-}" | tee -a "$LOG"
  sleep 120
done

echo "[$(date -Iseconds)] industry done — starting AGI suite (--agi-only)" | tee -a "$LOG"
python3 tools/valhalla_inference/run_industry_benchmarks.py --agi-only 2>&1 | tee -a "$LOG"
rc=${PIPESTATUS[0]}
echo "=== AGI END $(date -Iseconds) exit=$rc ===" | tee -a "$LOG"
exit "$rc"

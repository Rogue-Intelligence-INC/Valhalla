#!/usr/bin/env bash
# MCQ scaling ladder: Qwen 0.5B → 3B → 7B via valhalla_base serve (no AGI).
set -euo pipefail
REPO="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$REPO"
LOG="${1:-reports/valhalla_inference/mcq_ladder_full_20260706.log}"

export HF_ENDPOINT="${HF_ENDPOINT:-https://hf-mirror.com}"
export LIBRARY_PATH="/opt/cuda/targets/x86_64-linux/lib:${LIBRARY_PATH:-}"
export RUSTFLAGS="-L /opt/cuda/lib64"
export VALHALLA_USE_CPU="${VALHALLA_USE_CPU:-0}"

echo "=== MCQ FULL LADDER $(date -Iseconds) ===" | tee "$LOG"
python3 -u -c "
import subprocess, sys
from pathlib import Path
REPO = Path('$REPO')
sys.path.insert(0, str(REPO / 'tools/valhalla_inference'))
from benchmark_env import benchmark_runtime
with benchmark_runtime() as env:
    print(f\"serve={env.get('VALHALLA_BASE_URL', 'subprocess')}\")
    sys.exit(subprocess.call(
        [sys.executable, str(REPO / 'tools/valhalla_inference/test_mcq_ladder_v1.py'),
         '--full-ladder', '--output', 'mcq_ladder_v1_full_test.json'],
        env=env,
    ))
" 2>&1 | tee -a "$LOG"
rc=${PIPESTATUS[0]}
echo "=== END $(date -Iseconds) exit=$rc ===" | tee -a "$LOG"
exit "$rc"

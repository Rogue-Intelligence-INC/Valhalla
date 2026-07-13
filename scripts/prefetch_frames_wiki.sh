#!/usr/bin/env bash
# Prefetch Wikipedia for FRAMES — prefer parasail HF dump, else live Wikipedia.
#
# Usage:
#   ./scripts/prefetch_frames_wiki.sh
#   ./scripts/prefetch_frames_wiki.sh --offline   # parasail/cache only
#   ./scripts/prefetch_frames_wiki.sh --live      # try Wikipedia API first
#
set -euo pipefail
REPO="$(cd "$(dirname "$0")/.." && pwd)"
cd "$REPO"

OFFLINE=0
PREFER=parasail
for arg in "$@"; do
  case "$arg" in
    --offline) OFFLINE=1 ;;
    --live) PREFER=live ;;
  esac
done

PY="${REPO}/.venv-llm/bin/python3"
[[ -x "$PY" ]] || PY=python3
export HF_ENDPOINT="${HF_ENDPOINT:-https://hf-mirror.com}"

echo "[prefetch] enrich FRAMES via parasail-ai/frames-benchmark-wikipedia (prefer=$PREFER) …"
CMD=("$PY" tools/standard_benchmarks/enrich_frames_wiki.py --prefer "$PREFER")
[[ "$OFFLINE" -eq 1 ]] && CMD+=(--offline)
"${CMD[@]}"

echo "[prefetch] OK — run:"
echo "  $PY tools/valhalla_inference/test_frames_v1.py --rust --fast"
echo "  # or full: $PY tools/valhalla_inference/test_frames_v1.py --rust"

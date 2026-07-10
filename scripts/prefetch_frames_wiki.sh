#!/usr/bin/env bash
# Prefetch Wikipedia extracts for FRAMES bundle (NAACL 2025 multi-hop RAG).
#
# Usage:
#   ./scripts/prefetch_frames_wiki.sh
#   ./scripts/prefetch_frames_wiki.sh --offline   # use cache only
#
# Requires outbound HTTPS to en.wikipedia.org (or pre-populated cache).

set -euo pipefail
REPO="$(cd "$(dirname "$0")/.." && pwd)"
cd "$REPO"

OFFLINE=0
for arg in "$@"; do
  case "$arg" in
    --offline) OFFLINE=1 ;;
  esac
done

PY="${REPO}/.venv-llm/bin/python3"
[[ -x "$PY" ]] || PY=python3

CMD=("$PY" tools/standard_benchmarks/download_standard_benchmarks.py --frames-n 25)
if [[ "$OFFLINE" -eq 1 ]]; then
  CMD+=(--offline)
fi

export HF_ENDPOINT="${HF_ENDPOINT:-https://hf-mirror.com}"
echo "[prefetch] FRAMES bundle + Wikipedia extracts …"
"${CMD[@]}"

"$PY" -c "
from tools.standard_benchmarks.common import frames_bundle_preflight
import json
p = frames_bundle_preflight('frames_slice25.json')
print(json.dumps(p, indent=2))
if not p['viable']:
    raise SystemExit('FRAMES preflight still not viable — check Wikipedia network')
"

echo "[prefetch] OK — run: python3 tools/valhalla_inference/test_frames_v1.py --rust --fast"

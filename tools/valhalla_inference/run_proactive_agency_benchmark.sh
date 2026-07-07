#!/usr/bin/env bash
# Proactive Agency Benchmark — 主动性 AI 完整测试
#
# Tracks:
#   A  高轮次对话 + 主动发言 + 回答补充
#   B  多模态同一问题 (text / TTS / image / tri-modal)
#   C  多语言不隔离 (EN/ZH/FR)
#
# Usage:
#   tools/valhalla_inference/run_proactive_agency_benchmark.sh --fast
#   tools/valhalla_inference/run_proactive_agency_benchmark.sh --mega --verbose
#   tools/valhalla_inference/run_proactive_agency_benchmark.sh --mega --turns 3000 --extend-chunk 64
#   tools/valhalla_inference/run_proactive_agency_benchmark.sh --turns 5000 --long-only
#   tools/valhalla_inference/run_proactive_agency_benchmark.sh --compare-base --download-dialogues
#   touch /tmp/vb_pause && tools/valhalla_inference/run_proactive_agency_benchmark.sh --resume

set -euo pipefail
REPO="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$REPO"

PY="${REPO}/.venv-llm/bin/python3"
[[ -x "$PY" ]] || PY=python3

export RUSTFLAGS="${RUSTFLAGS:--L /opt/cuda/lib64}"
export VALHALLA_USE_CPU="${VALHALLA_USE_CPU:-0}"

MODEL="${MODEL:-${REPO}/models/hf/Qwen2.5-3B-Instruct-Valhalla-TriModal-Fusion-v3-test-s0.10}"

exec "$PY" tools/valhalla_inference/run_proactive_agency_benchmark.py \
  --model "$MODEL" \
  "$@"

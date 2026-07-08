#!/usr/bin/env bash
# Finalize Valhalla baked models and push to GitHub via Git LFS.
#
# Usage:
#   tools/valhalla_model_bridge/push_baked_models_github.sh
#   tools/valhalla_model_bridge/push_baked_models_github.sh --dry-run
#
set -euo pipefail
REPO="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$REPO"

DRY=0
[[ "${1:-}" == "--dry-run" ]] && DRY=1

if ! command -v git-lfs >/dev/null 2>&1; then
  echo "git-lfs required. Install: sudo pacman -S git-lfs && git lfs install" >&2
  exit 1
fi

git lfs install

MODELS=(
  "models/hf/Qwen2.5-0.5B-Instruct-Valhalla-TriModal-Fusion-v3-smoke-s0.10"
  "models/hf/Qwen2.5-0.5B-Instruct-Valhalla-FreeMultimodal-Fusion-v2-test-s0.10"
  "models/hf/Qwen2.5-3B-Instruct-Valhalla-TriModal-Fusion-v3-test-s0.10"
)

PY="${REPO}/.venv-llm/bin/python3"
[[ -x "$PY" ]] || PY=python3

for d in "${MODELS[@]}"; do
  if [[ ! -f "$d/model.safetensors" ]]; then
    echo "SKIP (no weights): $d" >&2
    continue
  fi
  echo "Finalizing $d …" >&2
  "$PY" tools/valhalla_model_bridge/finalize_hf_repo.py "$d" --skip-verify
done

# LFS track (idempotent)
git lfs track "models/hf/*/model.safetensors" 2>/dev/null || true

for d in "${MODELS[@]}"; do
  [[ -d "$d" ]] || continue
  git add -f "$d"/*.safetensors "$d"/*.json "$d"/README.md "$d"/.gitattributes 2>/dev/null || true
  git add "$d"/tokenizer.json "$d"/tokenizer_config.json "$d"/chat_template.jinja 2>/dev/null || true
  git add "$d"/valhalla_manifest.json "$d"/hf_export.json 2>/dev/null || true
done

git add .gitattributes models/hf/.gitattributes 2>/dev/null || true

if [[ "$DRY" == 1 ]]; then
  git status --short models/hf/
  exit 0
fi

if git diff --cached --quiet; then
  echo "Nothing to commit for models/hf" >&2
  exit 0
fi

git commit -m "$(cat <<'EOF'
Publish Valhalla baked HF models via Git LFS.

TriModal v3 (0.5B smoke + 3B test) and FreeMultimodal v2 checkpoints for proactive agency benchmark reproduction.
EOF
)"

git push origin HEAD

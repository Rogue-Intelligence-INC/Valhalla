#!/usr/bin/env bash
# ValhallaBase CLI helper — delegates to Rust valhalla_base binary + vb.py
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
BIN="${VALHALLA_BASE_BIN:-$ROOT/target/release/valhalla_base}"
VB="${ROOT}/tools/valhalla_inference/vb.py"
export RUSTFLAGS="${RUSTFLAGS:--L /opt/cuda/lib64}"
export LIBRARY_PATH="/opt/cuda/targets/x86_64-linux/lib:${LIBRARY_PATH:-}"
export VALHALLA_USE_CPU="${VALHALLA_USE_CPU:-0}"
export VALHALLA_LM_MODEL="${VALHALLA_LM_MODEL:-Qwen/Qwen2.5-3B-Instruct}"

ensure_bin() {
  if [[ -x "$BIN" ]]; then return; fi
  echo "Building valhalla_base..." >&2
  cargo build -p hub-f64 --release --bin valhalla_base --manifest-path "$ROOT/manifestsys/hub-f64/Cargo.toml" --features patch-lm
}

cmd="${1:-help}"
shift || true

case "$cmd" in
  build) ensure_bin; echo "$BIN" ;;
  help|-h|--help) "$BIN" help 2>/dev/null || true
    echo ""
    echo "Extended: repro | dashboard | suite | vb"
    echo "  $0 repro          — make repro (full smoke path)"
    echo "  $0 dashboard      — Tile/Stem triad viz :8765"
    echo "  $0 serve-dash     — infer API + /dashboard :8787"
    echo "  $0 suite [name]   — run_suite_sequential.py"
    echo "  $0 vb ...         — python3 tools/valhalla_inference/vb.py"
    ;;
  repro) make -C "$ROOT" repro ;;
  dashboard) exec python3 "$VB" dashboard "$@" ;;
  serve-dash) ensure_bin; exec "$BIN" serve --with-dashboard "$@" ;;
  suite) exec python3 "$VB" suite "$@" ;;
  vb) exec python3 "$VB" "$@" ;;
  version|-V|--version) ensure_bin; "$BIN" --version ;;
  ask|demo|serve) ensure_bin; exec "$BIN" "$cmd" "$@" ;;
  example)
    ensure_bin
    name="${1:-open}"
    "$BIN" --example "$name" --pretty
    ;;
  validate)
    ensure_bin
    file="${1:?usage: valhalla_base_cli.sh validate FILE.json}"
    "$BIN" --validate "$file"
    ;;
  run)
    ensure_bin
    file="${1:-}"
    if [[ -n "$file" ]]; then "$BIN" --pretty --json-error "$file"
    else "$BIN" --pretty --json-error
    fi
    ;;
  open|session|mcq|multimodal|proactive|production)
    ensure_bin
    "$BIN" --example "$cmd" | "$BIN" --pretty --json-error
    ;;
  v1|needle|corpus)
    ensure_bin
    exec "$BIN" demo "$cmd" "$@"
    ;;
  fate-pretrain-holdout|fate-longchain-unified|fate-routing-ab|fate-parallel-longchain|longchain-fate|locomo|frames|multihop|multihop-fate|multihop-hybrid|needle-prod)
    ensure_bin
    exec "$BIN" demo "$cmd" "$@"
    ;;
  *)
    echo "Unknown: $cmd" >&2
    echo "Usage: $0 {ask|demo|serve|serve-dash|dashboard|repro|suite|vb|help}" >&2
    exit 1
    ;;
esac

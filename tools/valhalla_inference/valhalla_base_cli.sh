#!/usr/bin/env bash
# ValhallaBase CLI helper — delegates to Rust valhalla_base binary
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
BIN="${VALHALLA_BASE_BIN:-$ROOT/target/release/valhalla_base}"
export RUSTFLAGS="${RUSTFLAGS:--L /opt/cuda/lib64}"
export LIBRARY_PATH="/opt/cuda/targets/x86_64-linux/lib:${LIBRARY_PATH:-}"
export VALHALLA_USE_CPU="${VALHALLA_USE_CPU:-0}"
export VALHALLA_LM_MODEL="${VALHALLA_LM_MODEL:-Qwen/Qwen2.5-3B-Instruct}"

ensure_bin() {
  if [[ -x "$BIN" ]]; then return; fi
  echo "Building valhalla_base..." >&2
  cargo build -p hub-f64 --release --bin valhalla_base --manifest-path "$ROOT/manifestsys/hub-f64/Cargo.toml"
}

cmd="${1:-help}"
shift || true
ensure_bin

case "$cmd" in
  build) echo "$BIN" ;;
  help|-h|--help) "$BIN" help ;;
  version|-V|--version) "$BIN" --version ;;
  ask|demo|serve) "$BIN" "$cmd" "$@" ;;
  example)
    name="${1:-open}"
    "$BIN" --example "$name" --pretty
    ;;
  validate)
    file="${1:?usage: valhalla_base_cli.sh validate FILE.json}"
    "$BIN" --validate "$file"
    ;;
  run)
    file="${1:-}"
    if [[ -n "$file" ]]; then "$BIN" --pretty --json-error "$file"
    else "$BIN" --pretty --json-error
    fi
    ;;
  open|session|mcq|multimodal|proactive|production)
    "$BIN" --example "$cmd" | "$BIN" --pretty --json-error
    ;;
  v1|needle|corpus)
    "$BIN" demo "$cmd" "$@"
    ;;
  *)
    echo "ValhallaBase Rust CLI helper" >&2
    echo "Usage: $0 {ask|demo|serve|v1|needle|corpus|example|validate|run|build|help}" >&2
    exit 1
    ;;
esac

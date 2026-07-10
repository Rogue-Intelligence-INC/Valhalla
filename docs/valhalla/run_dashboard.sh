#!/usr/bin/env bash
# Valhalla triad dashboard — smoke aggregation + architecture view
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
export RUSTFLAGS="${RUSTFLAGS:--L /opt/cuda/lib64}"
export VALHALLA_DASHBOARD_PORT="${VALHALLA_DASHBOARD_PORT:-8765}"
cargo run --release -p hub-f64 --bin valhalla_dashboard

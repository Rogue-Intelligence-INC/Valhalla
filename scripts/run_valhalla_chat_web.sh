#!/usr/bin/env bash
# Start ValhallaBase serve + Flutter web chat UI.
#
# Usage:
#   ./scripts/run_valhalla_chat_web.sh              # serve + flutter run (HTML renderer)
#   ./scripts/run_valhalla_chat_web.sh --serve-only # backend only
#   ./scripts/run_valhalla_chat_web.sh --test       # flutter test (no serve)
#   ./scripts/run_valhalla_chat_web.sh --release  # build + static serve (recommended demo)
#
# For investor/demo: prefer ./scripts/demo_valhalla_chat.sh
#
# Env: VALHALLA_BASE_PORT (8787), VALHALLA_CHAT_WEB_PORT (8888), FLUTTER_ROOT

set -euo pipefail

REPO="$(cd "$(dirname "$0")/.." && pwd)"
CHAT="${REPO}/apps/valhalla_chat"
PORT="${VALHALLA_BASE_PORT:-8787}"
WEB_PORT="${VALHALLA_CHAT_WEB_PORT:-8888}"
VB="${REPO}/target/release/valhalla_base"
SERVE_ONLY=0
TEST_ONLY=0
RELEASE=0

for arg in "$@"; do
  case "$arg" in
    --serve-only) SERVE_ONLY=1 ;;
    --test) TEST_ONLY=1 ;;
    --release) RELEASE=1 ;;
    -h|--help)
      sed -n '2,14p' "$0"
      exit 0
      ;;
    *) echo "Unknown: $arg" >&2; exit 2 ;;
  esac
done

flutter_bin() {
  if [[ -n "${FLUTTER_ROOT:-}" && -x "${FLUTTER_ROOT}/bin/flutter" ]]; then
    echo "${FLUTTER_ROOT}/bin/flutter"
  elif [[ -x "${HOME}/flutter/bin/flutter" ]]; then
    echo "${HOME}/flutter/bin/flutter"
  elif command -v flutter >/dev/null 2>&1; then
    command -v flutter
  else
    return 1
  fi
}

setup_flutter_env() {
  if ! curl -sf --max-time 3 https://pub.dev/api/packages/http >/dev/null 2>&1; then
    export PUB_HOSTED_URL="${PUB_HOSTED_URL:-https://pub.flutter-io.cn}"
    export FLUTTER_STORAGE_BASE_URL="${FLUTTER_STORAGE_BASE_URL:-https://storage.flutter-io.cn}"
    echo "[flutter] using mirror: ${PUB_HOSTED_URL}"
  fi
}

if [[ "$TEST_ONLY" -eq 1 ]]; then
  FL="$(flutter_bin)" || { echo "ERROR: Flutter SDK not found. Set FLUTTER_ROOT or install Flutter." >&2; exit 1; }
  setup_flutter_env
  cd "$CHAT"
  "$FL" pub get
  "$FL" test
  exit 0
fi

if [[ "$RELEASE" -eq 1 ]]; then
  exec "${REPO}/scripts/demo_valhalla_chat.sh" "$@"
fi

if [[ ! -x "$VB" ]]; then
  echo "[build] valhalla_base …"
  RUSTFLAGS="${RUSTFLAGS:--L /opt/cuda/lib64}" \
    cargo build -p hub-f64 --release --bin valhalla_base --features patch-lm
fi

if ! curl -sf "http://127.0.0.1:${PORT}/health" >/dev/null 2>&1; then
  echo "[serve] valhalla_base on :${PORT} (with dashboard) …"
  "$VB" serve --port "$PORT" --with-dashboard &
  SERVE_PID=$!
  trap 'kill $SERVE_PID 2>/dev/null || true' EXIT
  for _ in $(seq 1 60); do
    curl -sf "http://127.0.0.1:${PORT}/health" >/dev/null 2>&1 && break
    sleep 1
  done
else
  echo "[serve] already running on :${PORT}"
fi

curl -s "http://127.0.0.1:${PORT}/health" | head -c 200
echo ""

if [[ "$SERVE_ONLY" -eq 1 ]]; then
  echo "Backend ready. Press Ctrl+C to stop."
  wait
fi

FL="$(flutter_bin)" || {
  echo ""
  echo "Flutter SDK not installed. Backend is running at http://127.0.0.1:${PORT}"
  echo "Install Flutter, then:"
  echo "  ./scripts/demo_valhalla_chat.sh"
  exit 1
}

setup_flutter_env
cd "$CHAT"
"$FL" pub get

# HTML renderer avoids CanvasKit CDN (gstatic) — blank page in restricted networks.
DEVICE="${FLUTTER_WEB_DEVICE:-web-server}"
exec "$FL" run -d "$DEVICE" \
  --web-port="$WEB_PORT" \
  --web-hostname=127.0.0.1 \
  --web-renderer html \
  --dart-define="VALHALLA_BASE_URL=http://127.0.0.1:${PORT}"

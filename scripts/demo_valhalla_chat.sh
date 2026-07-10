#!/usr/bin/env bash
# One-click ValhallaBase chat demo — backend + release Flutter web (HTML renderer).
#
# Usage:
#   ./scripts/demo_valhalla_chat.sh              # build (if needed) + serve + smoke test
#   ./scripts/demo_valhalla_chat.sh --no-build   # skip flutter build, serve existing web/
#   ./scripts/demo_valhalla_chat.sh --smoke-only # API smoke test only (no web server)
#
# Opens:
#   Chat UI:     http://127.0.0.1:${WEB_PORT}/
#   Backend:     http://127.0.0.1:${VALHALLA_BASE_PORT}/health
#   Triad dash:  http://127.0.0.1:${VALHALLA_BASE_PORT}/dashboard/
#
# Demo checklist (UI):
#   1. Confirm green "connected" in header after load
#   2. Click "Warmup" (sidebar) to ingest system context into Fate session
#   3. Ask: "What decode mode does ValhallaBase use?"
#   4. Toggle MULTIMODAL INGRESS → fill text/audio/image features → send
#   5. Inspect protocol log + Triad viz panels for session-native highlights

set -euo pipefail

REPO="$(cd "$(dirname "$0")/.." && pwd)"
CHAT="${REPO}/apps/valhalla_chat"
WEB_DIR="${CHAT}/build/web"
VB="${REPO}/target/release/valhalla_base"
API_PORT="${VALHALLA_BASE_PORT:-8787}"
WEB_PORT="${VALHALLA_CHAT_WEB_PORT:-8888}"
API_URL="http://127.0.0.1:${API_PORT}"
WEB_URL="http://127.0.0.1:${WEB_PORT}"

NO_BUILD=0
SMOKE_ONLY=0
SERVE_PID=""
WEB_PID=""

for arg in "$@"; do
  case "$arg" in
    --no-build) NO_BUILD=1 ;;
    --smoke-only) SMOKE_ONLY=1 ;;
    -h|--help)
      sed -n '2,20p' "$0"
      exit 0
      ;;
    *) echo "Unknown: $arg" >&2; exit 2 ;;
  esac
done

cleanup() {
  [[ -n "$WEB_PID" ]] && kill "$WEB_PID" 2>/dev/null || true
  [[ -n "$SERVE_PID" ]] && kill "$SERVE_PID" 2>/dev/null || true
}
trap cleanup EXIT INT TERM

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
  # pub.dev is often blocked; China mirror is the default fallback.
  if ! curl -sf --max-time 3 https://pub.dev/api/packages/http >/dev/null 2>&1; then
    export PUB_HOSTED_URL="${PUB_HOSTED_URL:-https://pub.flutter-io.cn}"
    export FLUTTER_STORAGE_BASE_URL="${FLUTTER_STORAGE_BASE_URL:-https://storage.flutter-io.cn}"
    echo "[flutter] using mirror: ${PUB_HOSTED_URL}"
  fi
}

ensure_backend() {
  if [[ ! -x "$VB" ]]; then
    echo "[build] valhalla_base …"
    RUSTFLAGS="${RUSTFLAGS:--L /opt/cuda/lib64}" \
      cargo build -p hub-f64 --release --bin valhalla_base --features patch-lm
  fi
  if curl -sf "${API_URL}/health" >/dev/null 2>&1; then
    echo "[serve] backend already on :${API_PORT}"
    return
  fi
  echo "[serve] starting valhalla_base on :${API_PORT} …"
  RUSTFLAGS="${RUSTFLAGS:--L /opt/cuda/lib64}" \
    "$VB" serve --port "$API_PORT" --with-dashboard &
  SERVE_PID=$!
  for _ in $(seq 1 60); do
    curl -sf "${API_URL}/health" >/dev/null 2>&1 && break
    sleep 1
  done
  curl -sf "${API_URL}/health" | head -c 240
  echo ""
}

smoke_test() {
  echo "[smoke] GET /health …"
  curl -sf "${API_URL}/health" >/dev/null

  echo "[smoke] POST /v1/infer (context QA) …"
  local qa
  qa="$(curl -sf -X POST "${API_URL}/v1/infer" \
    -H 'Content-Type: application/json' \
    -d '{
      "questions":[{"id":"q0","text":"What decode mode does ValhallaBase use?","score_type":"open"}],
      "context":[
        "ValhallaBase primary protocol is valhalla-base-v1.",
        "Production decode mode is hybrid with native retrieval for open QA."
      ],
      "config":{"decode":"hybrid","proactive_enabled":false}
    }')"
  echo "$qa" | python3 -c "import json,sys; d=json.load(sys.stdin); print('  ok=', d.get('ok', True), 'rows=', len(d.get('rows',[])))"

  echo "[smoke] POST /v1/infer (multimodal ingress) …"
  local mm
  mm="$(curl -sf -X POST "${API_URL}/v1/infer" \
    -H 'Content-Type: application/json' \
    -d '{
      "questions":[{"id":"q0","text":"Summarize the multimodal scene.","score_type":"open",
        "audio_features":[0.12,0.34,0.56,0.78],
        "image_features":[0.01,0.02,0.03,0.04]}],
      "context":["ValhallaBase supports context_items with text, audio_features, image_features."],
      "context_items":[{"text":"Lab demo: triad viz + protocol log visible."}],
      "config":{"decode":"hybrid","proactive_enabled":false}
    }')"
  echo "$mm" | python3 -c "import json,sys; d=json.load(sys.stdin); print('  ok=', d.get('ok', True), 'context_items=', 'context_items' in str(d))"

  echo "[smoke] POST /v1/infer (hybrid LM fallback — casual query) …"
  local casual
  casual="$(curl -sf -X POST "${API_URL}/v1/infer" \
    -H 'Content-Type: application/json' \
    -d '{
      "questions":[{"id":"q1","text":"hi there","score_type":"open"}],
      "context":["ValhallaBase primary protocol is valhalla-base-v1.","Production decode mode is hybrid with native retrieval for open QA."],
      "config":{"decode":"hybrid","proactive_enabled":false}
    }')"
  echo "$casual" | python3 -c "import json,sys; d=json.load(sys.stdin); a=(d.get('rows') or [{}])[0].get('answer',''); print('  ok=', not a.startswith('No confident match'), 'path=', (d.get('rows') or [{}])[0].get('decode_path','')[:60])"

  echo "[smoke] all API checks passed"
}

build_web() {
  local FL
  FL="$(flutter_bin)" || {
    echo "ERROR: Flutter SDK not found. Set FLUTTER_ROOT or install Flutter." >&2
    exit 1
  }
  setup_flutter_env
  cd "$CHAT"
  echo "[flutter] pub get …"
  "$FL" pub get
  echo "[flutter] build web (HTML renderer — no CanvasKit CDN) …"
  "$FL" build web --release --web-renderer html \
    --dart-define="VALHALLA_BASE_URL=${API_URL}"
}

serve_web() {
  if [[ ! -f "${WEB_DIR}/index.html" ]]; then
    echo "ERROR: ${WEB_DIR}/index.html missing. Run without --no-build." >&2
    exit 1
  fi
  if curl -sf "${WEB_URL}/" >/dev/null 2>&1; then
    echo "[web] already serving on :${WEB_PORT}"
    return
  fi
  echo "[web] static server on ${WEB_URL} …"
  cd "$WEB_DIR"
  python3 -m http.server "$WEB_PORT" --bind 127.0.0.1 &
  WEB_PID=$!
  sleep 0.5
  curl -sf "${WEB_URL}/" >/dev/null
  curl -sf "${WEB_URL}/main.dart.js" >/dev/null
  echo "[web] index + main.dart.js OK"
}

main() {
  ensure_backend
  smoke_test

  if [[ "$SMOKE_ONLY" -eq 1 ]]; then
    echo ""
    echo "Smoke-only complete. Backend: ${API_URL}"
    SERVE_PID=""
    exit 0
  fi

  if [[ "$NO_BUILD" -eq 0 ]] || [[ ! -f "${WEB_DIR}/main.dart.js" ]]; then
    build_web
  else
    echo "[flutter] skip build (--no-build, existing ${WEB_DIR})"
  fi

  serve_web

  echo ""
  echo "════════════════════════════════════════════════════════════"
  echo " ValhallaBase demo ready"
  echo "────────────────────────────────────────────────────────────"
  echo " Chat UI:    ${WEB_URL}/"
  echo " Backend:    ${API_URL}/health"
  echo " Dashboard:  ${API_URL}/dashboard/"
  echo "────────────────────────────────────────────────────────────"
  echo " Highlights to verify in UI:"
  echo "  • Session warmup → Fate lines ingested (sidebar metrics)"
  echo "  • Hybrid decode answer grounded in context"
  echo "  • Multimodal badge on user bubble (text+audio+image)"
  echo "  • Protocol log + Triad visualization panels"
  echo "════════════════════════════════════════════════════════════"
  echo "Press Ctrl+C to stop."

  if [[ -n "$WEB_PID" ]]; then
    wait "$WEB_PID"
  elif [[ -n "$SERVE_PID" ]]; then
    wait "$SERVE_PID"
  else
    wait
  fi
}

main "$@"

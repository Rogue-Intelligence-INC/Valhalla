# ValhallaBase Chat — Flutter Web UI

Interactive web frontend for **ValhallaBase** inference. Connects to the Rust HTTP server via `POST /v1/infer` and round-trips **session state** for multi-turn dialogue.

## Layout

| Region | Content |
|--------|---------|
| **Left / Chat tab** | Multi-turn dialogue · proactive nudges · multimodal badges |
| **Right / Triad tab** | Model state · Hub quad · Tile · StemCell · Proactive gate · Multimodal ingress |
| **Bottom** | Protocol log — curl CLI mirror + expandable JSON |

Wide screen (≥960px): chat + triad sidebar side-by-side.  
Narrow: tabs switch between Chat and Triad.

## Features

| Feature | API |
|---------|-----|
| Multi-turn chat | `session` round-trip |
| Proactive agency | `proactive_idle` + auto-after-turn |
| Multimodal | `context_items` / `append_items` + question features |
| Triad viz | `fate_telemetry` + `session.snapshot` + `/dashboard/api/state` |
| CLI log | Every request/response logged with curl mirror |

Start backend **with dashboard** for Tile/Stem smoke viz:

```bash
valhalla_base serve --port 8787 --with-dashboard
```

```
Flutter Web (browser)
    │  POST /v1/infer  ·  GET /health
    ▼
valhalla_base serve :8787
    └── Triad session + hybrid decode (Qwen LM head)
```

| Feature | Implementation |
|---------|----------------|
| Multi-turn chat | `session` from each response sent on next turn |
| Context ingest | System context lines on first turn; `append_lines` after |
| Fate warmup | Drawer → optional `fate_warmup_only` pre-pass |
| Connection | `/health` banner + retry |
| Decode trace | Shows `decode_path` per assistant message |

## Prerequisites

1. **ValhallaBase backend** built and serving:

```bash
cd /path/to/Valhalla
cargo build -p hub-f64 --release --bin valhalla_base --features patch-lm
./target/release/valhalla_base serve --port 8787
```

2. **Flutter SDK** (stable, web enabled):

```bash
flutter config --enable-web
```

China mirror (optional):

```bash
export PUB_HOSTED_URL=https://pub.flutter-io.cn
export FLUTTER_STORAGE_BASE_URL=https://storage.flutter-io.cn
```

## Quick start

```bash
# One script: serve + open Chrome
./scripts/run_valhalla_chat_web.sh

# Tests only
./scripts/run_valhalla_chat_web.sh --test
```

Manual:

```bash
cd apps/valhalla_chat
flutter pub get
flutter run -d chrome --dart-define=VALHALLA_BASE_URL=http://127.0.0.1:8787
```

Build static web:

```bash
flutter build web --dart-define=VALHALLA_BASE_URL=http://127.0.0.1:8787
# output: build/web/
```

## Tests

```bash
cd apps/valhalla_chat
flutter test
```

| File | Covers |
|------|--------|
| `test/valhalla_api_test.dart` | Request body builder · HTTP mock · health |
| `test/chat_controller_test.dart` | Session round-trip across turns |
| `test/widget_test.dart` | Message bubble UI |

## API contract

Default production config (matches Python `ValhallaBase`):

```json
{
  "decode": "hybrid",
  "follow_up_decode": "native_follow_up_aware",
  "fate_ingress_routing": "quad_cycle",
  "fate_qa_feedback": true
}
```

**First turn:**

```json
{
  "context": ["fact line 1", "fact line 2"],
  "questions": [{"id": "q0", "text": "…", "score_type": "open"}],
  "config": { … }
}
```

**Follow-up:**

```json
{
  "session": { "…from prior response…" },
  "append_lines": ["user utterance as corpus line"],
  "questions": [{"id": "q1", "text": "…", "score_type": "open"}],
  "config": { … }
}
```

See `manifestsys/hub-f64/examples/valhalla_base/` and `docs/valhalla/VALHALLA_BASE_CLI.md`.

## CORS & binding

- Server uses **permissive CORS** — browser calls from `localhost` work.
- Default bind: **`127.0.0.1:8787`** — same machine only.
- Flutter web and backend must run on the same host (or add a reverse proxy).

## Project layout

```
apps/valhalla_chat/
  lib/
    main.dart
    services/valhalla_api.dart    # HTTP client
    state/chat_controller.dart    # Session + messages
    screens/chat_screen.dart      # Main UI
    widgets/message_bubble.dart
  test/
  web/
```

---

*Rogue Intelligence LNC.*

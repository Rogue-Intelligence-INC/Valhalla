# ValhallaBase CLI & Programmatic API

**Protocol:** `valhalla-base-v1` · **Binary:** `valhalla_base` · **Updated:** 2026-07-13

Unified entry/exit for structure-first inference. **Rust-native path** (in-process client + demos + HTTP) replaces Python subprocess wrappers where possible.

| Artifact | Path |
|----------|------|
| **Rust session client** | [`manifestsys/hub-f64/src/valhalla_base_client.rs`](../manifestsys/hub-f64/src/valhalla_base_client.rs) |
| **Rust benchmarks** | [`manifestsys/hub-f64/src/valhalla_base_bench.rs`](../manifestsys/hub-f64/src/valhalla_base_bench.rs) (re-exports [`multihop_bench.rs`](../manifestsys/hub-f64/src/multihop_bench.rs)) |
| **Open-QA interface** | [`manifestsys/hub-f64/src/open_qa_interface.rs`](../manifestsys/hub-f64/src/open_qa_interface.rs) |
| **Fate I/O (ingress/egress)** | [`manifestsys/hub-f64/src/fate_io.rs`](../manifestsys/hub-f64/src/fate_io.rs) |
| **Elastic resources** | [`manifestsys/hub-f64/src/elastic_resources.rs`](../manifestsys/hub-f64/src/elastic_resources.rs) |
| **HTTP server** | [`manifestsys/hub-f64/src/valhalla_base_http.rs`](../manifestsys/hub-f64/src/valhalla_base_http.rs) |
| **Protocol header** | [`manifestsys/hub-f64/include/valhalla_base.h`](../manifestsys/hub-f64/include/valhalla_base.h) |
| **Request schema** | [`manifestsys/hub-f64/schemas/valhalla-base-v1.request.schema.json`](../manifestsys/hub-f64/schemas/valhalla-base-v1.request.schema.json) |
| **Response schema** | [`manifestsys/hub-f64/schemas/valhalla-base-v1.response.schema.json`](../manifestsys/hub-f64/schemas/valhalla-base-v1.response.schema.json) |
| **Error schema** | [`manifestsys/hub-f64/schemas/valhalla-base-v1.error.schema.json`](../manifestsys/hub-f64/schemas/valhalla-base-v1.error.schema.json) |
| **Examples** | [`manifestsys/hub-f64/examples/valhalla_base/`](../manifestsys/hub-f64/examples/valhalla_base/) |
| **Rust runtime** | [`manifestsys/hub-f64/src/valhalla_base.rs`](../manifestsys/hub-f64/src/valhalla_base.rs) |
| **Rust API layer** | [`manifestsys/hub-f64/src/valhalla_base_protocol.rs`](../manifestsys/hub-f64/src/valhalla_base_protocol.rs) |
| **Python wrapper** | [`tools/valhalla_inference/valhalla_base.py`](../tools/valhalla_inference/valhalla_base.py) |
| **Shell helper** | [`tools/valhalla_inference/valhalla_base_cli.sh`](../tools/valhalla_inference/valhalla_base_cli.sh) |

---

## Architecture (2026-07-13): bench vs interface

Benchmark runners **must not** embed native-QA batch logic. Use this stack:

```
multihop_bench / valhalla_base_bench  →  bundle · parallel · JSON
open_qa_interface                     →  OpenQaRequest · run_open_qa()
fate_io                               →  fate_ingress · fate_egress · fate_score_short_answer
native_qa                             →  run_native_qa_batch (RelAddr + Fate + v2)
elastic_resources                     →  ElasticResourcePlan · par_map_items
```

```bash
# MultiHop arms (slice25, full n=25)
valhalla_base demo multihop --limit 25          # v2 compose
valhalla_base demo multihop-fate --limit 25     # Fate adapt
valhalla_base demo multihop-hybrid --limit 25 # fatewithv2 (unified)
```

```rust
use hub_f64::{OpenQaRequest, OpenQaFlags, run_open_qa_scored};

let req = OpenQaRequest::new("MH_1", question, corpus)
    .with_flags(OpenQaFlags::unified());
let out = run_open_qa_scored(&req, gold);
```

Latest full industry run: [`EXPERIMENT_SUMMARY_20260713.md`](../../reports/valhalla_inference/EXPERIMENT_SUMMARY_20260713.md).

---

## 1. Build

```bash
export RUSTFLAGS='-L /opt/cuda/lib64'   # optional CUDA
cargo build -p hub-f64 --release --bin valhalla_base
# → target/release/valhalla_base
```

---

## 2. Rust-native subcommands (preferred)

```bash
# One-shot QA (in-process Rust — no Python)
valhalla_base ask "What is the protocol?" \
  --context "ValhallaBase primary protocol is valhalla-base-v1." --native

# Benchmarks (replaces test_valhalla_base_v1.py, test_needle_session_v1.py, run_local_corpus_demo.py)
valhalla_base demo v1
valhalla_base demo needle [--fast]
valhalla_base demo corpus [--repo /path/to/Valhalla]

# HTTP JSON API
valhalla_base serve --port 8787
valhalla_base serve --with-dashboard   # + Tile/Stem viz at /dashboard/

# Reproduction (third-party)
make repro
python3 tools/valhalla_inference/vb.py repro
# See REPRODUCTION.md

curl -s http://127.0.0.1:8787/health
curl -s http://127.0.0.1:8787/v1/repro
curl -s http://127.0.0.1:8787/dashboard/api/state   # triad viz JSON

# Standalone triad dashboard (Tile heatmap + Stem organ clusters)
valhalla_dashboard   # or: make dashboard  →  http://127.0.0.1:8765/

curl -s -X POST http://127.0.0.1:8787/v1/infer -H 'content-type: application/json' \
  -d @manifestsys/hub-f64/examples/valhalla_base/open_qa.json
```

### Rust in-process API

```rust
use hub_f64::{ValhallaBaseClient, RunOptions, production_config};

let mut client = ValhallaBaseClient::production();
client.set_context(["Fact A"]);
let ans = client.ask("What is Fact A?", "open", "q1")?;
```

---

## 3. JSON CLI (stdin/stdout · backward compatible)

```bash
valhalla_base --help
valhalla_base --version
valhalla_base --example open              # print example request JSON
valhalla_base --validate request.json     # parse + validate only
valhalla_base --pretty < request.json     # run inference, pretty stdout
valhalla_base --json-error request.json   # errors as JSON envelope on stdout
```

### Bundled examples

| Name | Scenario |
|------|----------|
| `open` | Open QA · hybrid decode |
| `session` | Fresh session ingest |
| `mcq` | MCQ · hybrid → LM logprob |
| `multimodal` | `context_items` text/audio/image |
| `proactive` | `proactive_idle` (empty questions) |
| `production` | Full production config stack |

```bash
valhalla_base --example production | valhalla_base --pretty
```

---

## 3. Complete command recipes

### 3.1 Open QA (native retrieval under hybrid)

```bash
valhalla_base --example open | valhalla_base --pretty --json-error
```

### 3.2 Session append (two-step)

```bash
# Step 1 — ingest + first question
valhalla_base --pretty --json-error \
  -e open 2>/dev/null | head -1 > /tmp/vb_step1.json
RESP=$(valhalla_base --pretty --json-error < /tmp/vb_step1.json)
echo "$RESP" | jq '.rows[0].answer'

# Step 2 — append without re-sending full corpus
jq -n \
  --argjson sess "$(echo "$RESP" | jq '.session')" \
  '{
    session: $sess,
    append_lines: ["New fact: proactive gate uses fate_signal_tick."],
    questions: [{id:"q2", text:"What does the proactive gate use?", score_type:"open"}],
    config: {decode:"hybrid", follow_up_decode:"native_follow_up_aware", fate_ingress_routing:"quad_cycle"}
  }' | valhalla_base --pretty --json-error
```

### 3.3 MCQ hybrid

```bash
valhalla_base --example mcq | valhalla_base --pretty
```

### 3.4 Proactive idle

Warm up with dialogue first, then:

```bash
valhalla_base --example proactive | valhalla_base --pretty
```

Production path: run 2+ QA turns via session, then `proactive_idle: true` with `session` round-trip.

### 3.5 Native-only (no LM load)

```bash
jq '.config.decode = "native"' \
  manifestsys/hub-f64/examples/valhalla_base/open_qa.json \
  | valhalla_base --pretty
```

### 3.6 Validate before batch jobs

```bash
valhalla_base --validate manifestsys/hub-f64/examples/valhalla_base/production_stack.json
```

---

## 4. Rust API (in-process)

```rust
use hub_f64::{
    execute_request, execute_request_json, production_config,
    ValhallaBaseRequest, ValhallaBaseQuestion, ValhallaContextItem,
};

let mut req = ValhallaBaseRequest::default();
req.context = vec!["Fact A".into()];
req.questions = vec![ValhallaBaseQuestion {
    id: "q1".into(),
    text: "What is Fact A?".into(),
    score_type: "open".into(),
    ..Default::default() // if Default impl added — else set fields
}];
req.config = production_config();

let resp = execute_request(&req)?;
// or
let json_out = execute_request_json(r#"{"context":["..."],"questions":[...]}"#, true)?;
```

Public re-exports live on `hub_f64::` root when feature `patch-lm` is enabled.

---

## 5. Python API

```python
from valhalla_base import ValhallaBase

base = ValhallaBase(decode="hybrid", follow_up_decode="native_follow_up_aware")
base.set_context(["ValhallaBase protocol is valhalla-base-v1."])
print(base.ask("What is the primary protocol name?"))
doc = base.run([{"id": "q1", "text": "...", "score_type": "open"}], append_only=True)
nudge = base.proactive_idle()
```

Uses subprocess to `target/release/valhalla_base --json-error`.

---

## 6. Request / response contract

### Entry (request)

| Field | Required | Purpose |
|-------|----------|---------|
| `context` | * | Text corpus lines (legacy) |
| `context_items` | * | Multimodal corpus |
| `append_lines` / `append_items` | * | Incremental session append |
| `session` | * | Round-trip state from prior response |
| `questions` | ✓† | QA batch |
| `proactive_idle` | | Empty questions + session → self-probe |
| `config` | | Decode / Fate / LM settings |

*At least one context source required. †Unless `proactive_idle=true`.

### Exit (response)

| Field | Purpose |
|-------|---------|
| `protocol` | Always `valhalla-base-v1` |
| `session` | **Must round-trip** for multi-turn |
| `rows[].answer` | Grounded answer + decode_path |
| `fate_telemetry` | Hub prefs / ingress stats |
| `proactive` | Nudge outcome when idle path fired |

### Exit (error with `--json-error`)

```json
{
  "protocol": "valhalla-base-v1",
  "ok": false,
  "error": "parse request: ...",
  "error_kind": "parse"
}
```

---

## 7. Production config

```json
{
  "decode": "hybrid",
  "follow_up_decode": "native_follow_up_aware",
  "fate_ingress_routing": "quad_cycle",
  "fate_qa_feedback": true,
  "fate_routing_free": false,
  "proactive_enabled": true,
  "proactive_min_fate_signals": 64,
  "incubation_cycles": 2,
  "lm_model": "Qwen/Qwen2.5-0.5B-Instruct",
  "patch_body": "triad",
  "strength": 0.08
}
```

Rust: `hub_f64::production_config()` · Example: `--example production`

---

## 8. Related binaries (not unified gateway)

| Binary | Use |
|--------|-----|
| `valhalla_base` | **Primary** — full ValhallaBase protocol |
| `valhalla_native_qa` | Native QA-only JSON CLI |
| `valhalla_hybrid` | Legacy hybrid pipeline |

New integrations should use **`valhalla_base` only**.

Platform doc: [VALHALLA_BASE_PLATFORM.md](https://github.com/Rogue-Intelligence-INC/valhalla-unified-core/blob/main/docs/VALHALLA_BASE_PLATFORM.md)

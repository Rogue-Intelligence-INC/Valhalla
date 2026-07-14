# ValhallaBase — 3-Minute Demo

**Goal:** Show corpus → structure snapshot → grounded answer → session append — **Rust-native first**.

**Prerequisites:** Rust 1.70+, optional CUDA (`RUSTFLAGS='-L /opt/cuda/lib64'`).

---

## 0. One-time setup

```bash
RUSTFLAGS='-L /opt/cuda/lib64' cargo build -p hub-f64 --release --bin valhalla_base
```

Full API reference: [`VALHALLA_BASE_CLI.md`](./VALHALLA_BASE_CLI.md)

---

## 1. v1 extensions smoke (~30s, Rust)

Hybrid routing · session append · multimodal:

```bash
valhalla_base demo v1
# → reports/valhalla_inference/valhalla_base_v1_extensions.json
```

---

## 2. Local corpus ingest (~60s, Rust)

```bash
git submodule update --init valhalla-unified-core
valhalla_base demo corpus
# → reports/valhalla_inference/local_corpus_demo_rust.json
```

Legacy Python (extended probes): `python3 tools/valhalla_inference/run_local_corpus_demo.py`

---

## 3. Needle session recall (~30s, Rust)

```bash
valhalla_base demo needle --fast    # depths 5,10
valhalla_base demo needle           # depths 5,10,20,30
# → reports/valhalla_inference/needle_session_v1_test.json
```

---

## 4. One-shot ask (Rust)

```bash
valhalla_base ask "What is the protocol?" \
  --context "ValhallaBase primary protocol is valhalla-base-v1." --native
```

---

## 5. HTTP serve

```bash
valhalla_base serve --port 8787
curl -s http://127.0.0.1:8787/health
curl -s -X POST http://127.0.0.1:8787/v1/infer \
  -H 'content-type: application/json' \
  -d @manifestsys/hub-f64/examples/valhalla_base/open_qa.json
```

---

## 6. Python-only batteries (still require Python harness)

```bash
pip install -e tools/valhalla_inference   # preferred import path
# Multimodal / U1-style: ValhallaBase(**production_multimodal_kwargs())
```

| Test | Command |
|------|---------|
| RAGTruth 40Q | `python3 tools/valhalla_inference/test_ragtruth_v1.py` |
| Follow-up 30-turn | `python3 tools/valhalla_inference/test_context_flexibility_goal.py` |
| Proactive Fate v2 | `python3 tools/valhalla_inference/test_proactive_fate_v2.py` |

Industry suite (needle via Rust): `python3 tools/valhalla_inference/run_industry_benchmarks.py --fast`

---

## Investor narrative (one line)

**ValhallaBase is a structure-first AI memory substrate that turns context into persistent Triad snapshots, enabling grounded retrieval, session learning, proactive routing, and hybrid LM decoding without LM weight updates.**

Product docs: [valhalla-unified-core](https://github.com/Rogue-Intelligence-INC/valhalla-unified-core)

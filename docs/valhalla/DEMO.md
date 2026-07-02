# ValhallaBase — 3-Minute Demo

**Goal:** Show corpus → structure snapshot → grounded answer → session append → proactive — without reading the full codebase.

**Prerequisites:** Rust 1.70+, Python 3.10+, optional CUDA (`RUSTFLAGS='-L /opt/cuda/lib64'`).

---

## 0. One-time setup

```bash
git submodule update --init valhalla-unified-core   # product docs corpus
RUSTFLAGS='-L /opt/cuda/lib64' cargo build -p hub-f64 --release --bin valhalla_base
```

---

## 1. Local corpus ingest + QA (~90s)

Zero-weight teach from local markdown, then eval with citations:

```bash
python3 tools/valhalla_inference/run_local_corpus_demo.py
```

Protocol: `local-corpus-demo-v1` · Report: `reports/valhalla_inference/local_corpus_demo.json`

---

## 2. RAGTruth faithfulness (~60s, 40Q subset)

```bash
python3 tools/valhalla_inference/test_ragtruth_v1.py
```

Tier A headline: **40/40 grounded** · full 100Q: `test_ragtruth_full_v1.py`

---

## 3. Needle session recall (~30s)

Append-then-ask long-range recall:

```bash
python3 tools/valhalla_inference/test_needle_session_v1.py
```

Tier A headline: **4/4 @ depth 30**

---

## 4. Follow-up native (~60s)

30-turn follow-up decode on structured session:

```bash
python3 tools/valhalla_inference/test_context_flexibility_goal.py
```

Headline: **96.7%** follow-up native

---

## 5. Proactive Fate (~60s)

Idle self-probe + grounded nudge (Fate `signal_tick` gate):

```bash
python3 tools/valhalla_inference/test_proactive_fate_v2.py
```

Headline: **71/72 · PPI 0.890**

---

## Investor narrative (one line)

**ValhallaBase is a structure-first AI memory substrate that turns context into persistent Triad snapshots, enabling grounded retrieval, session learning, proactive routing, and hybrid LM decoding without LM weight updates.**

Product docs: [valhalla-unified-core](https://github.com/Rogue-Intelligence-INC/valhalla-unified-core) · Benchmark interpretation: [`VALID_BENCHMARKS_INTERPRETATION_20260630.md`](../../reports/valhalla_inference/VALID_BENCHMARKS_INTERPRETATION_20260630.md)

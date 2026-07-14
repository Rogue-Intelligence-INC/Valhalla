# ValhallaBase — 3-Minute Live Demo Script (1080p)

**Audience:** Investors / technical evaluators  
**Runtime:** ~3:00  
**Stack:** `./scripts/demo_valhalla_chat.sh` · Chat UI `:8888` · Backend `:8787`

---

## Feature map (what each beat proves)

| Beat | Feature pillar | What to show |
|------|----------------|--------------|
| 1 | **Connected stack** | Green health · `valhalla-base-v1` · git/protocol in header |
| 2 | **Fate warmup** | System context → structure ingest · lines/tick counters move |
| 3 | **Hybrid grounded QA** | Native retrieval · `[corpus:N]` citation · hybrid decode path |
| 4 | **Session memory** | Follow-up without resending corpus · append-only wire |
| 5 | **Multimodal ingress** | `context_items` text + audio + image features · bubble badge |
| 6 | **Proactive agency** | Fate-gated nudge or honest abstain · PPI/MAR story |
| 7 | **Hybrid LM fallback** | Casual query when native abstains · still answers |
| 8 | **Triad transparency** | Hub quad · Tile/Stem · protocol log JSON |

---

## System context (paste in Settings drawer if empty)

```text
ValhallaBase primary protocol is valhalla-base-v1.
Production decode mode is hybrid with native retrieval for open QA.
Unique Proofs U1–U4 are the lead citeable suite (4/4 on 2026-07-14).
RAGTruth industry faithfulness is 38 out of 40 on the latest fast slice (88/100 on the full cache).
Longchain continuity: theme drift 0 percent and anchor retention 100 percent.
Proactive agency uses Fate signal_tick gating with MAR cross-domain abstain.
Session append sends ~64 bytes per turn instead of full prompt replay.
```

---

## Timed script

| Time | UI action | Say / type |
|------|-----------|------------|
| **0:00–0:12** | Open http://127.0.0.1:8888/ · confirm **Connected** | "ValhallaBase console — structure memory, not a fixed-size LM SKU." |
| **0:12–0:28** | Settings (gear) → verify context → **Fate warmup** | "Corpus incubates into Triad snapshot — watch lines ingested and fate tick." |
| **0:28–0:48** | Send Q1 | `What decode mode does ValhallaBase use?` |
| | *Expect* | Answer cites corpus · path contains `open_retrieval` or `native` |
| **0:48–1:05** | Send Q2 (session follow-up) | `What Unique Proof lead count was mentioned in our context?` |
| | *Expect* | `4/4` or `U1–U4` · session append in protocol log |
| **1:05–1:25** | Enable **MULTIMODAL INGRESS** → **Load demo vectors** → Send Q3 | `Summarize the scene for the ingress pipeline.` |
| | *Expect* | User bubble multimodal badge · `context_items` in protocol log |
| **1:25–1:45** | Enable **Proactive** toggle · **Auto after turn** ON → Send Q4 | `Tell me more about session append.` |
| | *Expect* | Grounded answer and/or proactive nudge bubble (Fate-gated) |
| **1:45–2:00** | Send Q5 | `hi there` |
| | *Expect* | Friendly reply via `hybrid/open_lm_fallback` — not "No confident match" |
| **2:00–2:25** | Scroll **Protocol log** · point at POST /v1/infer bodies | "Single endpoint — infer, warmup, session, proactive." |
| **2:25–2:45** | Triad sidebar: Hub quad · Tile/Stem metrics | "Hub routes ingress; Tile/Stem are incubated structure blocks." |
| **2:45–3:00** | Optional: open http://127.0.0.1:8787/dashboard/ | "Triad dashboard + citeable benchmarks in repo reports." |

---

## Question cheat-sheet (copy-paste)

```
What decode mode does ValhallaBase use?
What RAGTruth score was mentioned in our context?
Summarize the scene for the ingress pipeline.
Tell me more about session append.
hi there
```

---

## Automated recording

```bash
python3 scripts/run_demo_showcase.py
```

Validates every demo question against live `:8787` before a live presentation.


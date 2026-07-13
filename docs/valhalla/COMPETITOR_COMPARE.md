# Competitor Compare — What to Run (and Why)

**Goal:** Fair same-question baselines vs memory / RAG peers — not “beat every SOTA.”  
**Backbone lock:** `Qwen/Qwen2.5-3B-Instruct` (industry calibration only).  
**Evidence:** Results land in Pareto / ablation reports and are packed by `pack_evidence_run.py`.

---

## Who we compare to

| Peer | Category | What they optimize | Our fair compare angle |
|------|----------|--------------------|------------------------|
| **Full-context LM** | Baseline | Dump entire session into prompt | Token cost + accuracy upper bound |
| **Vanilla RAG** | Baseline | Corpus block / top-k in prompt | Most common “search solution” |
| **Mem0** | Product | Managed memory API | Same LoCoMo questions; memory add→search→LM |
| **Letta** | Product | Agent runtime + memory blocks | Optional API; else SKIP with reason |
| **Zep / Graphiti** | Product | Temporal knowledge graph | Optional / literature cite if no API key |
| **Valhalla −Fate / −Triad** | Ablation | Component off | Causal contribution (not a competitor) |
| **Valhalla hybrid full** | Us | Structure snapshot + Fate | Primary system under test |

Do **not** frame as “replace Mem0 API.” Frame as:

> Same questions · same backbone · memory/RAG systems vs structure substrate — accuracy **and** tokens/latency.

---

## Must-run matrix (P0)

| Benchmark | n | Systems | Why |
|-----------|---|---------|-----|
| **Unique-proof U1–U5** | micro | valhalla (+ peer SKIP) | **Primary wedge** — multimodal · proactive · TPI · backbone · cost@floor |
| **Token / multi-turn** | 30 turns | valhalla_append · rag_full_resend · rag_with_history | Cost wedge (requires U5 on-topic floor) |
| **RAGTruth-40** | 40 | valhalla · vanilla_rag (subset) | Faithfulness — our Tier A strength |
| **Ablation** | LoCoMo15×… | −Fate · −Triad · lm_patch · native | Prove structure ≠ “more tokens” |
| **LoCoMo** | 15 moderate / 30 full | full_context · vanilla_rag · valhalla · mem0? | Peers’ headline — **honest boundary**, not the lead claim |

```bash
./scripts/run_unique_proofs.sh --fast
```

See [`UNIQUE_VS_MEMORY_APIS.md`](./UNIQUE_VS_MEMORY_APIS.md) · [`UNIQUE_PROOF_MICROBENCH.md`](./UNIQUE_PROOF_MICROBENCH.md).

## Should-run (P1)

| Benchmark | n | Systems | Why |
|-----------|---|---------|-----|
| **MultiHop** | 15 | full_context · vanilla_rag · valhalla | Honest open multi-hop boundary |
| **LongBench-v2** | 10–20 | same | Honest long-context boundary |
| **Mem0 LoCoMo-10** | 10 | mem0 · valhalla · vanilla_rag | Competitor same-Q; keep small |

## Skip / defer

| Item | Reason |
|------|--------|
| Full Mem0 LongMemEval 94% replay | Needs their stack + disputed independent numbers |
| Zep full DMR | Needs Neo4j + API; cite paper until wired |
| Beat LongBench SOTA | Not our wedge |
| FRAMES without wiki | Tier C — run `./scripts/prefetch_frames_wiki.sh` (parasail dump) first |

---

## Commands (local IDE)

```bash
cd /home/jouly/AI/Valhalla
export RUSTFLAGS="-L /opt/cuda/lib64" LIBRARY_PATH=/opt/cuda/lib64
export VALHALLA_BASE_NO_SERVE=1 PYTHONUNBUFFERED=1
PY=.venv-llm/bin/python3

# 1) Internal structure ablations (LM / RAG / −Fate / −Triad)
./tools/valhalla_inference/run_fundraise_ablation.sh --industry-only

# 2) Competitor-oriented Pareto (+ Mem0 if installed)
$PY tools/valhalla_inference/run_competitor_compare.py --moderate
# with Mem0:
#   pip install mem0ai
#   $PY tools/valhalla_inference/run_competitor_compare.py --moderate --with-mem0

# 3) Token cost (architecture-level)
$PY tools/valhalla_inference/test_token_efficiency.py --run-valhalla

# 4) Pack uploadable proof
$PY tools/valhalla_inference/pack_evidence_run.py --tag competitor --zip
```

---

## Latest results — 2026-07-13 moderate

| Bench | full_context | vanilla_rag | valhalla | mem0 local ST |
|-------|--------------|-------------|----------|---------------|
| LoCoMo-15 | 9/15 | 9/15 | 0/15 | 5/15 |
| MultiHop-10 | 6/10 | 6/10 | 6/10 | 6/10 |
| LongBench-10 | 2/10 | 2/10 | 1/10 | 2/10 |

Full write-up: [`EVIDENCE_SUITE_SUMMARY_20260713.md`](../../reports/valhalla_inference/EVIDENCE_SUITE_SUMMARY_20260713.md) · [`COMPETITOR_COMPARE_RESULTS.md`](../../reports/valhalla_inference/COMPETITOR_COMPARE_RESULTS.md)

---

## How to report (honest)

| Say | Don’t say |
|-----|-----------|
| On LoCoMo-15 @ 3B, Valhalla vs vanilla RAG / full-context: acc + tok/q | We beat Mem0’s published 94% |
| Mem0 arm SKIPPED if no key — listed in matrix | Hide skipped competitors |
| Token efficiency: append vs full resend curve | “60–90% cheaper” without this run |
| Ablation Fate/Triad lifts in pp | Structure always wins MultiHop |
| LoCoMo competitor gap is honest (0/15 vs 9/15) | Structure wins session QA today |

---

## Mapping to fundraising narrative

- **Wedge page:** Unique capabilities (non-iso MM · proactive FOG · TPI rescue · backbone ortho · cost@floor)  
- **Diligence appendix:** unique-proof suite JSON + ablation + competitor same-Q (incl. LoCoMo gap)  
- **Incubation page:** TPI/NPPI (secondary; not competitor SOTA)

---

*Rogue Intelligence LNC.*

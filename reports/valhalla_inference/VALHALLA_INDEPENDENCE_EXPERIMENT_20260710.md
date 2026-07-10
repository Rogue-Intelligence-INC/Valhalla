# ValhallaBase Backbone Orthogonality — 2026-07-10

**Protocol:** `valhalla-independence-v1`  
**Runner:** `tools/valhalla_inference/run_valhalla_independence.py`  
**Backbone harness:** `tools/valhalla_inference/valhalla_backbone.py`

---

## Objective

Validate **structure-path orthogonality**: ValhallaBase open decode (`native` / hybrid Tier-B) must not depend on the LM backbone checkpoint.

| Check | Method |
|-------|--------|
| Open-path orthogonality | Per-backbone `native` + `hybrid` vs single `native` reference (byte-level answer match) |
| Trad baseline | Optional `trad` arm — expected to vary with backbone (sanity control) |
| Session-memory orthogonality | Needle recall: hybrid `lm_model` label vs `native` reference |
| Stack capability smokes | `core_fusion` / proactive gates / longchain — **fixed Qwen stack**, not swept per backbone |

---

## Backbone panel (cached)

| Class | Preset | HF checkpoint |
|-------|--------|---------------|
| RWKV | `rwkv7_0.1b` | `fla-hub/rwkv7-0.1B-g1` |
| MoE | `phi_tiny_moe` | `microsoft/Phi-tiny-MoE-instruct` |
| Dense | `qwen_3b` | `Qwen/Qwen2.5-3B-Instruct` |

---

## Results

| Layer | Metric | Verdict |
|-------|--------|---------|
| `open_path_orthogonality` | native+hybrid vs ref | **VALHALLA_BASE_INDEPENDENT** — 6/6 |
| `trad_baseline` | per-backbone trad | 4/4 each (no spread on this slice) |
| `session_memory_orthogonality` | needle d=5,10 | **MEMORY_INDEPENDENT** — 3/3 |
| `core_fusion_smoke` | 2Q @ Qwen3B | OK — CORE_FUSION_TIE |
| `proactive_gate_smoke` | gates-only | OK — GATES_OK |
| `longchain_memory_smoke` | theme_drift / composite | OK — composite **0.661**, drift **0%** (report cache on live timeout) |

All open items decode via `valhalla_base/native/.../open_retrieval_span` regardless of backbone preset.

---

## Resource controls

| Control | Purpose |
|---------|---------|
| `VALHALLA_INGRESS_RESOURCE_PERCENT=70` | Ingress budget cap |
| `VALHALLA_INGRESS_MAX_ITEMS=48/96` | Corpus ingress line limit (smoke / default) |
| `ValhallaLLM.unload()` | Release GPU between backbone trad runs |
| Subprocess timeout + serial smokes | Prevent longchain hang / OOM |
| `test_fate_longchain_unified.py --smoke` | 24-line corpus live attempt |
| Report cache fallback | Valid prior `fate_longchain_unified.json` when live run times out |

---

## Test layering (orthogonal vs capability)

| Suite | In primary orthogonality verdict? | Rationale |
|-------|-----------------------------------|-----------|
| open + session memory | Yes | Direct backbone orthogonality claim |
| core_fusion | No — `stack_capability_smokes` | Fixed LM + bake; not a backbone sweep |
| proactive gates | No — `stack_capability_smokes` | Hybrid stack + Fate signal gating |
| longchain unified | No — `stack_capability_smokes` | Heavy session benchmark; smoke + cache |

Primary verdict covers **open_path + session_memory** only. Capability smokes are regression appendix on a single production stack.

---

## Reproduce

```bash
# Primary orthogonality battery
VALHALLA_INGRESS_RESOURCE_PERCENT=70 VALHALLA_INGRESS_MAX_ITEMS=96 \
  .venv-llm/bin/python3 tools/valhalla_inference/run_valhalla_independence.py \
  --fast --with-trad --with-memory

# Stack capability smokes (serial, low memory)
.venv-llm/bin/python3 tools/valhalla_inference/run_valhalla_independence.py \
  --smokes-only --fast --extended

# Register ad-hoc checkpoint (no registry edit)
.venv-llm/bin/python3 tools/valhalla_inference/run_valhalla_independence.py \
  --hf org/model-id --fast --with-memory
```

**Artifact:** `reports/valhalla_inference/valhalla_independence_test.json`

---

## Backbone harness CLI

```bash
.venv-llm/bin/python3 tools/valhalla_inference/valhalla_backbone.py --list
.venv-llm/bin/python3 tools/valhalla_inference/valhalla_backbone.py --hf org/model --fast
.venv-llm/bin/python3 tools/valhalla_inference/valhalla_backbone.py --preset rwkv7_0.1b --preset phi_tiny_moe --fast
```

**Registry:** `tools/standard_benchmarks/backbone_registry.py`

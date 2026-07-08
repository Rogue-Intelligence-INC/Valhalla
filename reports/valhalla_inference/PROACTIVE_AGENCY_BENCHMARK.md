# Proactive Agency Benchmark

**Protocol:** `proactive-agency-benchmark-v1` + `proactive-fate-v2` (Track D)  
**Model:** `Qwen2.5-3B-Instruct-Valhalla-TriModal-Fusion-v3-test-s0.10`  
**Updated:** 2026-07-08

---

## Headline (full benchmark)

| Metric | Value |
|--------|-------|
| **Agency Index** | **0.639** (AGENCY_VIABLE) |
| Long-dialogue | n=**8** · pass **12.5%** · proactive gate **100%** |
| Multimodal | n=**10** · pass **90%** · consistency **0.90** |
| Crosslang CCU | **100%** (6 facts × EN/ZH/FR) |
| **PPI-v2 自动反应** | **0.650** · 68/72 pass · GATES_OK |
| MAR / FIR | **100% / 0%** |

**Agency formula:** `0.25·long + 0.20·multimodal + 0.20·crosslang + 0.35·PPI-v2`

---

## Run

```bash
# 标准 smoke
tools/valhalla_inference/run_proactive_agency_benchmark.sh --verbose

# 补齐版（10 multimodal + 5 LoCoMo + full PPI-v2）
tools/valhalla_inference/run_proactive_agency_full.sh --verbose

# 自动反应专项（handcraft long_dialogue + PPI-v2）
tools/valhalla_inference/run_proactive_agency_specialized.sh --verbose

# Mega 3000（基础设施压测，不计 Agency Index）
VALHALLA_BASE_URL=http://127.0.0.1:8788 \
  tools/valhalla_inference/run_proactive_agency_benchmark.sh \
  --mega --turns 3000 --extend-chunk 8 --segment-decodes 4 --verbose
```

---

## Reports

| File | Purpose |
|------|---------|
| [`proactive_agency_full.json`](./proactive_agency_full.json) | 补齐版完整跑分 |
| [`proactive_agency_specialized.json`](./proactive_agency_specialized.json) | 自动反应专项 |
| [`PROACTIVE_AGENCY_FULL_20260708.md`](./PROACTIVE_AGENCY_FULL_20260708.md) | 补齐版解读 |
| [`PROACTIVE_AGENCY_AUDIT_20260708.md`](./PROACTIVE_AGENCY_AUDIT_20260708.md) | 合法性/合理性/全面性审计 |

---

*Rogue Intelligence LNC.*

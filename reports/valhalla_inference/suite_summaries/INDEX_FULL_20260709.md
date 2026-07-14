> **HISTORICAL SNAPSHOT — not current diligence headline.** Canonical: [`../../docs/valhalla/CANONICAL_METRICS_20260714.md`](../../docs/valhalla/CANONICAL_METRICS_20260714.md).

# Valhalla Inference — 全套件运行汇总（2026-07-09 最终）

- **Runner:** `run_suite_sequential.py --all` + `run_industry_benchmarks.py --all` + `run_proactive_agency_full.sh`
- **日志:** [`run_complete_20260709.log`](run_complete_20260709.log)
- **评估:** [`PRODUCTIZATION_ASSESSMENT_20260709.md`](../PRODUCTIZATION_ASSESSMENT_20260709.md)

## 总览

| 套件 | 项数 | 状态 | 耗时 |
|------|------|------|------|
| **A fate-core** | 4/4 | ✅ PASS | ~2s |
| **B rust-unit** | 6/6 | ✅ PASS（E0063 已修） | <1s |
| **C standard-fast** | 8/8 | ✅ PASS | ~90s |
| **D context-drift** | 1/1 | ✅ PASS | ~70s |
| **E industry 3B** | 11/11 | ✅ PASS | ~41 min |
| **F proactive agency** | full | ✅ **AGENCY_STRONG 0.778** | ~1 min |

**全套件 exit:** 0（industry + proactive 链）· suite sequential 曾因 rust-unit 编译失败 exit=1，**已修复**

---

## 关键指标（融资引用）

| 能力 | 指标 |
|------|------|
| Grounded RAG | RAGTruth **100/100** · External holdout **15/15** |
| Needle / RULER | **100%** |
| Fate 长链 | theme_drift **0%** · holdout **100%** · composite **0.950** |
| Proactive Agency | **0.778 AGENCY_STRONG** · PPI-v2 **0.65** |
| LoCoMo (industry) | **14/30 (47%)** |
| MCQ 3B | **98%** |

---

## Per-suite 汇总

- [SUITE_fate-core_20260708T211736Z.md](SUITE_fate-core_20260708T211736Z.md)
- [SUITE_rust-unit_20260708T225405Z.md](SUITE_rust-unit_20260708T225405Z.md) ← **最新 PASS**
- [SUITE_standard-fast_20260708T211819Z.md](SUITE_standard-fast_20260708T211819Z.md)
- [SUITE_context-drift_20260708T211905Z.md](SUITE_context-drift_20260708T211905Z.md)
- Industry: [`industry_benchmarks_summary.json`](../industry_benchmarks_summary.json)
- Proactive: [`proactive_agency_full.json`](../proactive_agency_full.json)

```bash
python3 tools/valhalla_inference/run_suite_sequential.py --all
.venv-llm/bin/python3 tools/valhalla_inference/run_industry_benchmarks.py --all
tools/valhalla_inference/run_proactive_agency_full.sh --save-transcripts
```

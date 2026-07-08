# Proactive Agency Benchmark

**Protocol:** `proactive-agency-benchmark-v1` · **Model:** `/home/jouly/AI/Valhalla/models/hf/Qwen2.5-3B-Instruct-Valhalla-TriModal-Fusion-v3-test-s0.10`
**Generated:** 2026-07-08T20:56:41.526360+00:00

## Headline

| Metric | Value |
|--------|-------|
| **Agency Index** | **0.777** |
| Long-dialogue pass | 60.0% |
| Dialogue rounds | 24 |
| Supplement pass | 90.0% |
| Proactive pass | 100.0% |
| Multimodal consistency | 1.000 |
| Crosslang CCU | 100.0% |

## Automatic reaction (Track D · proactive-fate-v2)

| PPI-v2 | **0.650** |
| Trial pass rate | 94.4% |
| Gates OK | True |
| Unsolicited precision | 0.0 |

**Agency formula:** `0.25·long + 0.20·multimodal + 0.20·crosslang + 0.35·PPI-v2`


## Run

```bash
tools/valhalla_inference/run_proactive_agency_benchmark.sh --verbose
# Mega 3000 (memory-bounded)
VALHALLA_BASE_URL=http://127.0.0.1:8788 tools/valhalla_inference/run_proactive_agency_benchmark.sh \
  --mega --turns 3000 --extend-chunk 8 --segment-decodes 4 --verbose
```

Full report: [PROACTIVE_AGENCY_BENCHMARK.md](./PROACTIVE_AGENCY_BENCHMARK.md)


# Incubation / Trainable-LLM Benchmarks

**Generated:** 2026-07-10T21:37:19.115386+00:00

## Valhalla protocols (this run)

| Protocol | Result | Verdict | Citeable |
|----------|--------|---------|----------|
| valhalla-base-training-potential-v2 | TPI 0.755 (cached) | ['STRONG_INFERENCE_TIME_LEARNING', 'FATE_TRANSFER_RESCUES_SESSION'] | True |
| fate-pretrain-holdout-v3 | train 0.4 test 0.8 | HOLDOUT_STRONG | True |

## Literature mapping

- **MemoryBench** (arXiv:2510.17281): continual learning from user feedback in service time
  - Valhalla proxy: TPI-v2 B4 transfer + Fate session + proactive gates
- **CL-Bench** (tencent-community/CL-bench): in-context learning of novel rules without fine-tune
  - Valhalla proxy: cl-bench-slice-v1 + open_retrieval fair slice
- **TiC-LM** (arXiv:2504.02107): time-continual pretraining / forgetting-plasticity
  - Valhalla proxy: L3 bake + TPI curriculum (structure path; not web-scale CPT)
- **Continual Learning Bench** (arXiv:2606.05661): stateful gain across expert domains
  - Valhalla proxy: fate-longchain-unified + external-holdout

---

*Rogue Intelligence LNC.*

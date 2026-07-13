# Evidence suite — legitimacy · competitor · pack

One command for fundraise-grade proof:

```bash
./scripts/run_evidence_suite.sh          # moderate competitor + token + zip
./scripts/run_evidence_suite.sh --full   # + 7-arm ablation
./scripts/run_evidence_suite.sh --fast   # smoke
```

Requires: `.venv-llm` + `pip install -r requirements-benchmark.txt`

Outputs under `reports/valhalla_inference/`:

| File | Meaning |
|------|---------|
| `legitimacy_audit.json` | Citeable gate |
| `competitor_compare.json` | Same-Q baselines (+ mem0 local ST) |
| `fundraise_ablation_matrix.json` | 7-arm structure ablation (`--full`) |
| `evidence_<tag>/` + `.zip` | Uploadable pack |
| `evidence_suite_run.log` | Full log |
| `EVIDENCE_SUITE_SUMMARY_20260713.md` | Independent suite write-up |

### Latest completed run — 2026-07-13 `--full`

- **Done** 16:04+08 · legitimacy **11/11** · FRAMES wiki viable
- Competitor: LoCoMo 9/15 · 9/15 · **0/15** · 5/15; MultiHop all 6/10; LongBench 2/10 · 2/10 · 1/10 · 2/10
- Ablation: RAGTruth native/hybrid **34–37/40** vs LM/RAG **0/40**
- Pack: `evidence_evidence_20260713/` (+ zip)

See [`EVIDENCE_SUITE_SUMMARY_20260713.md`](../../reports/valhalla_inference/EVIDENCE_SUITE_SUMMARY_20260713.md) · parent [`README.md`](../../README.md) § Prove it.

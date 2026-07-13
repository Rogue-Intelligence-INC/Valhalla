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
| `evidence_<tag>/` + `.zip` | Uploadable pack |
| `evidence_suite_run.log` | Full log |

See parent [`README.md`](../../README.md) § Prove it.

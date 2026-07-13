# ValhallaBase — Product Docs Hub

**Updated:** 2026-07-14 · **Narrative:** unique capabilities first.

Canonical product documentation lives in the **[valhalla-unified-core](https://github.com/Rogue-Intelligence-INC/valhalla-unified-core)** repository (git submodule at `valhalla-unified-core/`). GitHub does **not** serve submodule files at paths under the main Valhalla tree — use the links below.

**ValhallaBase is not a memory-API clone and not a 3B LM product** — see [Product positioning](https://github.com/Rogue-Intelligence-INC/valhalla-unified-core/blob/main/docs/VALHALLA_BASE_PRODUCT_POSITIONING.md).

---

## Start here (unique-first)

| Doc | Link |
|-----|------|
| **Unique vs memory APIs** | [UNIQUE_VS_MEMORY_APIS.md](./UNIQUE_VS_MEMORY_APIS.md) |
| **Unique-proof protocol** | [UNIQUE_PROOF_MICROBENCH.md](./UNIQUE_PROOF_MICROBENCH.md) |
| **Unique-proof results** | [`UNIQUE_PROOF_SUITE.md`](../../reports/valhalla_inference/UNIQUE_PROOF_SUITE.md) |
| **Product positioning** | [VALHALLA_BASE_PRODUCT_POSITIONING.md](https://github.com/Rogue-Intelligence-INC/valhalla-unified-core/blob/main/docs/VALHALLA_BASE_PRODUCT_POSITIONING.md) |
| **Investor one-pager** | [INVESTOR_ONE_PAGER.md](https://github.com/Rogue-Intelligence-INC/valhalla-unified-core/blob/main/docs/INVESTOR_ONE_PAGER.md) |
| **VUC landing page** | [page/index.html](https://github.com/Rogue-Intelligence-INC/valhalla-unified-core/blob/main/page/index.html) |
| **Incubation capability** | [INCUBATION_CAPABILITY.md](https://github.com/Rogue-Intelligence-INC/valhalla-unified-core/blob/main/docs/INCUBATION_CAPABILITY.md) |
| **Platform reference** | [VALHALLA_BASE_PLATFORM.md](https://github.com/Rogue-Intelligence-INC/valhalla-unified-core/blob/main/docs/VALHALLA_BASE_PLATFORM.md) |
| **3-minute demo** | [DEMO_SCRIPT_3MIN_EN.md](./DEMO_SCRIPT_3MIN_EN.md) · [DEMO.md](./DEMO.md) |
| **Live web console** | [`scripts/demo_valhalla_chat.sh`](../../scripts/demo_valhalla_chat.sh) |

```bash
./scripts/run_unique_proofs.sh --fast
```

---

## Benchmarks & reports (this repo)

| Doc | Path |
|-----|------|
| **Unique proofs (latest)** | [`UNIQUE_PROOF_SUITE.md`](../../reports/valhalla_inference/UNIQUE_PROOF_SUITE.md) |
| **Evidence suite summary** | [`EVIDENCE_SUITE_SUMMARY_20260713.md`](../../reports/valhalla_inference/EVIDENCE_SUITE_SUMMARY_20260713.md) |
| **Fundraise pointer** | [`FUNDRAISE_BENCHMARK_LATEST.md`](../../reports/valhalla_inference/FUNDRAISE_BENCHMARK_LATEST.md) |
| **Integrated status** | [`VALHALLA_INTEGRATED_STATUS.md`](../../reports/valhalla_inference/VALHALLA_INTEGRATED_STATUS.md) |
| Industry / experiment summaries | [`BENCHMARK_SUMMARY_20260713.md`](../../reports/valhalla_inference/BENCHMARK_SUMMARY_20260713.md) · [`EXPERIMENT_SUMMARY_20260713.md`](../../reports/valhalla_inference/EXPERIMENT_SUMMARY_20260713.md) |
| Doc index | [`docs/README.md`](../README.md) |

---

## Clone note

```bash
git clone --recurse-submodules https://github.com/Rogue-Intelligence-INC/Valhalla.git
# or after clone:
git submodule update --init valhalla-unified-core
```

Local paths under `valhalla-unified-core/docs/` work after submodule init; web links on github.com/Valhalla should use the VUC URLs above.

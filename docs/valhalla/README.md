# ValhallaBase — Product Docs Hub

**Updated:** 2026-07-06

Canonical product documentation lives in the **[valhalla-unified-core](https://github.com/Rogue-Intelligence-INC/valhalla-unified-core)** repository (git submodule at `valhalla-unified-core/`). GitHub does **not** serve submodule files at paths under the main Valhalla tree — use the links below.

---

## Start here

| Doc | Link |
|-----|------|
| **Incubation capability** (TPI / NPPI · zero-weight learning) | [INCUBATION_CAPABILITY.md](https://github.com/Rogue-Intelligence-INC/valhalla-unified-core/blob/main/docs/INCUBATION_CAPABILITY.md) |
| **Platform reference** (runtime · Fate ingress · hybrid decode) | [VALHALLA_BASE_PLATFORM.md](https://github.com/Rogue-Intelligence-INC/valhalla-unified-core/blob/main/docs/VALHALLA_BASE_PLATFORM.md) |
| **Investor one-pager** | [INVESTOR_ONE_PAGER.md](https://github.com/Rogue-Intelligence-INC/valhalla-unified-core/blob/main/docs/INVESTOR_ONE_PAGER.md) |
| **VUC landing page** | [page/index.html](https://github.com/Rogue-Intelligence-INC/valhalla-unified-core/blob/main/page/index.html) |
| **3-minute demo** | [DEMO.md](./DEMO.md) (this repo) |

---

## Benchmarks & reports (this repo)

| Doc | Path |
|-----|------|
| Valid benchmark interpretation | [`reports/valhalla_inference/VALID_BENCHMARKS_INTERPRETATION_20260630.md`](../../reports/valhalla_inference/VALID_BENCHMARKS_INTERPRETATION_20260630.md) |
| **Industry 3B summary (2026-07-06)** | [`reports/valhalla_inference/INDUSTRY_3B_BENCHMARK_SUMMARY_20260706.md`](../../reports/valhalla_inference/INDUSTRY_3B_BENCHMARK_SUMMARY_20260706.md) |
| **Release 2026-07-06** | [RELEASE_20260706.md](./RELEASE_20260706.md) |
| **Manifold fragments v1** | [MANIFOLD_FRAGMENT_V1.md](./MANIFOLD_FRAGMENT_V1.md) |
| Full suite summary (2026-07-02) | [`reports/valhalla_inference/FULL_BENCHMARK_SUMMARY_20260702.md`](../../reports/valhalla_inference/FULL_BENCHMARK_SUMMARY_20260702.md) |
| MCQ scaling (0.5B→3B) | [`reports/valhalla_inference/MCQ_SCALING_POTENTIAL_20260706.md`](../../reports/valhalla_inference/MCQ_SCALING_POTENTIAL_20260706.md) |
| Doc index | [`docs/README.md`](../README.md) |

---

## Clone note

```bash
git clone --recurse-submodules https://github.com/Rogue-Intelligence-INC/Valhalla.git
# or after clone:
git submodule update --init valhalla-unified-core
```

Local paths under `valhalla-unified-core/docs/` work after submodule init; web links on github.com/Valhalla should use the VUC URLs above.

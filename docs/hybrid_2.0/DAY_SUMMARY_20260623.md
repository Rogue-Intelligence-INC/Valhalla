# Hybrid Valhalla — Day Summary (2026-06-23)

**Product line**: Hybrid Valhalla 2.0 · **Engine**: hybrid-4.0-rust-pipeline · **Benchmark**: fair-1.2 test (145Q)

---

## Executive summary

Today we closed the **MCQ deploy gap**: hybrid multiple-choice accuracy moved from **52.17% → 60.87%** (**24/46 → 28/46**) on the test slice, matching the patch-LM track on the final run. Baseline: `hybrid_v4_test_deploy_post_decode_fix.json`; final: `hybrid_v4_mcq_patch_strengthen_final.json`.

| Metric | Before (post-decode baseline) | After (final) | Δ |
|--------|-------------------------------|---------------|---|
| **Hybrid MCQ** | 24/46 (52.17%) | **28/46 (60.87%)** | **+4** |
| Patch LM MCQ | 27/46 (58.70%) | **28/46 (60.87%)** | +1 |
| Hybrid total (145Q) | 113/145 (77.93%) | **117/145 (80.69%)** | +4 |
| Open | 57/61 (93.44%) | 57/61 (93.44%) | — |
| Numeric | 32/38 (84.21%) | 32/38 (84.21%) | — |

---

## Engineering deliverables

### 1. Structure-guided patch LM (MCQ)

- **`patch_lm.rs`**: Per-question pipeline recompose → embed bias from option affinity + aggregate option energies; resolve answer via weighted vote (generation / affinity / aggregate), not blind trust of wrong LM output.
- **`PatchTrackRow`**: exports `mcq_patch_subtag`, `mcq_fate_spread`, `mcq_fate_letter`, `mcq_affinity_spread` for deploy gates.

### 2. Hybrid deploy router (MCQ)

- **`hybrid.rs`**: Tiered deploy—`mcq_option` disagreements can route to patch when signals agree; **latent path letter validation** (`pipeline_v4_mcq_X` protected only when X matches memory answer); **triangulated disagreement** (patch ≠ tile and ≠ stem) for solo-generation overrides.
- Parallel **secondary memory track** (stem-style) runs isolated; no cross-track letter confirmation.

### 3. Source signal fixes (affinity / option scores)

- **`task_decompose.rs`**: Letter-keyed option affinity; removed broken tile/fate index aliasing.
- **`mcq_fate.rs`**: Per-letter max energy map; demeaned `option_scores` aligned to MCQ options.
- **`pipeline.rs`**: Spread helpers, latent letter parse, merge requires minimum affinity spread before certifying latent MCQ paths.

### 4. Tests & reproduction

```bash
RUSTFLAGS='-L /opt/cuda/lib64' cargo build -p hub-f64 --release --bin valhalla_hybrid
./target/release/valhalla_hybrid --phase test --merge-mode deploy \
  --model checkpoints/valhalla_patched/Qwen2.5-0.5B-Instruct_valhalla_s0.08 \
  --out reports/hybrid_valhalla_lm/hybrid_v4_mcq_patch_strengthen_final.json
```

---

## Analysis notes (for DD / paper)

| Finding | Implication |
|---------|-------------|
| Pre-decode affinity alone ~9–15% | Keyword/corpus decode still carries memory MCQ; structure pre-decode needs option-comparator work |
| Patch LM alone 28/46 | Structure-guided patching is the MCQ ceiling today |
| Deploy +4 vs baseline | Routing policy matters as much as single-track accuracy |
| ~18 MCQ still wrong | Next lever: aggregate-phase option margin + comparator, not more LM scale |

---

## 中文摘要

- **Hybrid MCQ**：24/46 → **28/46**（+4），deploy 路由与 patch 双轨对齐。
- **全量 Hybrid**：117/145（**80.69%**），开放题 57/61（93.44%）、数字题 32/38（84.21%）持平。
- **三项落地**：option_scores/affinity 源头修复；fate spread 导出与 deploy 门控；latent 路径字母校验。
- **可复现 JSON**：`hybrid_v4_mcq_patch_strengthen_final.json`

---

**Next (paper / product)**: Frame as **persistent structure memory + hybrid routing** for external publication; expand fair benchmark audit pack for investors.

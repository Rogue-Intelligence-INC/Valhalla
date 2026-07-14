# Project completeness — Valhalla itself (2026-07-14)

**Focus:** product / engineering integrity — **not** fundraise posture.  
**Question this answers:** what is finished on the shipping path, what is still rough, what to fix next.

---

## Status map

| Area | State | Notes |
|------|-------|-------|
| **ValhallaBase runtime** (Rust `hub-f64`) | Solid | ask · demo · serve · protocol · client |
| **Python client** | Usable | `pip install -e tools/valhalla_inference` or `PYTHONPATH` |
| **Production defaults** | Aligned | hybrid · `quad_cycle` · FOG · **0.5B** default LM |
| **Evidence stack** | Strong | U1–U4 · scorecard · heavy invest · longchain continuity |
| **U5 cost@floor** | **Citeable (developing)** | `--fast`: ratio 0.291 · on-topic 55% · `COST_RATIO_PASS_ACC_DEVELOPING` |
| **Chat / dashboard** | Present | Flutter chat · triad dashboard |
| **Public VUC repo** | Docs + Pages | **Not** a standalone runtime — licensed monorepo required |
| **Holdout QA absolute** | **100%** (production fragment_full) | Continuity: drift 0% / retention 100% · interleave **75%** · composite ~0.88 |

---

## Closed this pass (integrity)

| Item | Fix |
|------|-----|
| Missing `.gitmodules` for VUC | Added |
| `make highlight` typo targets | Fixed |
| CLI / multimodal default LM → **0.5B** | Aligned |
| `REPRODUCTION.md` path · `REPO_MAP` examples | Fixed |
| Import without install path | `tools/valhalla_inference/pyproject.toml` |
| VUC README honesty | docs mirror, monorepo required |
| **`fragment_full` starved late evidence** | Do **not** truncate session before `fragment_full_capped`; pin evidence (e.g. D2:8 @~253); production cap **128** |
| Holdout land: schedule ≻ content | Holdout uses content RelAddr affinity + direct egress (skip sticky address-schedule in `worker.run`) |
| **U5 unique-proof citeability** | Fast path max-turns **20** → pass/citeable (acc developing) |
| Multimodal / U1 default | `production_multimodal_kwargs()` · `core_fusion=True` (base default stays off) |
| Fused loader stub message | Points to real binary path |

---

## Still open (project, ranked)

1. **U5 full floor** — on-topic **≥0.80** (not just fast 0.55) for lead-grade Unique Proof.  
2. **Native open decode + address schedule** — session phases still schedule-first; holdout bypasses via content egress. Optional: content-gate schedule for all open QA without hurting drift.  
3. **VUC as shippable SDK** — long path.  
4. **Holdout scoring false positives** — e.g. camping/temporal morphology can mark `LOCO_0_5` correct with weak land; tighten `score_qa_algebra` if citing holdout % strictly.

---

## One coherent “feel finished” path (monorepo)

```bash
git submodule update --init valhalla-unified-core
make build
make highlight
./scripts/run_unique_proofs.sh --fast --only U1,U2,U3,U4,U5
./scripts/demo_valhalla_chat.sh
pip install -e tools/valhalla_inference
# Multimodal / U1:
#   ValhallaBase(**production_multimodal_kwargs())
python3 tools/valhalla_inference/test_fate_longchain_unified.py --production
```

Architecture: `valhalla-unified-core/assets/architecture.svg` · `PRODUCT_ARCHITECTURE_20260714.md`

---

## Intentionally separate

Fundraise posture → `SOFT_RAISE_POSTURE_20260714.md` only when needed.  
This file stays about **whether the project itself is coherent and runnable**.

*Rogue Intelligence LNC.*

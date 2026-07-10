# Valhalla Monorepo — Directory Map

**ValhallaBase** = structure-trainable AI memory + hybrid inference.  
**This file classifies every top-level path.** Investors and evaluators should use only **Product** and **Product support** rows.

| Start here | Path |
|------------|------|
| One-click proof | [`scripts/run_highlight_test.sh`](scripts/run_highlight_test.sh) |
| Product docs | [`docs/valhalla/README.md`](docs/valhalla/README.md) · [valhalla-unified-core](https://github.com/Rogue-Intelligence-INC/valhalla-unified-core) |
| Latest integrated status | [`reports/valhalla_inference/VALHALLA_INTEGRATED_STATUS.md`](reports/valhalla_inference/VALHALLA_INTEGRATED_STATUS.md) |
| Web console | [`apps/valhalla_chat/`](apps/valhalla_chat/) |

---

## Product — ValhallaBase (ship / fund / evaluate)

| Path | Description |
|------|-------------|
| [`manifestsys/`](manifestsys/) | **Runtime core** — Hub · Tile · StemCell · Fate; `valhalla_base` binary (`hub-f64`) |
| [`systems/`](systems/) | **Triad metasystems** — Fate / Tile / StemCell f64 engines consumed by `manifestsys` |
| [`valhalla-unified-core/`](valhalla-unified-core/) | **Canonical product docs** (submodule) — positioning · protocols · investor memo |
| [`apps/valhalla_chat/`](apps/valhalla_chat/) | **Flutter web console** — Triad viz · proactive agency · protocol log |
| [`tools/valhalla_inference/`](tools/valhalla_inference/) | **Benchmark runners** — industry suite · independence · incubation · proactive |
| [`tools/standard_benchmarks/`](tools/standard_benchmarks/) | **Third-party harness** — RAGTruth · LoCoMo · FRAMES bundles |
| [`tools/fair_benchmark/`](tools/fair_benchmark/) | **Fair eval** — hybrid vs native · memory ablations |
| [`tools/valhalla_model_bridge/`](tools/valhalla_model_bridge/) | **L3 bake / incubation bridge** — HF artifact export |
| [`scripts/`](scripts/) | **Entry scripts** — `run_highlight_test.sh` · `run_full_valhalla_test.sh` |
| [`reports/valhalla_inference/`](reports/valhalla_inference/) | **Citeable benchmark JSON + summaries** |
| [`docs/valhalla/`](docs/valhalla/) | **Working product doc mirror** (GitHub-safe paths) |
| [`docker/`](docker/) | **Reproduction container** |
| [`Makefile`](Makefile) · [`REPRODUCTION.md`](REPRODUCTION.md) | Build · smoke · serve targets |

---

## Product support — required for reproduction, not the pitch

| Path | Description |
|------|-------------|
| [`models/`](models/) | Cached HF weights for benchmark calibration (0.5B / 3B / encoders) |
| [`checkpoints/`](checkpoints/) | Training / bake checkpoint storage |
| [`external_bench/`](external_bench/) | External comparator harness |
| [`examples/`](examples/) | Small API / protocol examples |
| [`utils/`](utils/) | Shared Rust/Python utilities used by product stack |
| [`src/`](src/) | Legacy workspace root crate (partial); prefer `manifestsys/hub-f64` |
| [`.env.example`](.env.example) | Environment template for serve / benchmarks |

---

## Theory & research — IP lineage, not the shipping binary

| Path | Description |
|------|-------------|
| [`s-wind-sim/`](s-wind-sim/) | **S-wind symbolic discovery** — φ-algebra research tooling; theory origin, not ValhallaBase runtime |
| [`papers/`](papers/) | **Academic manuscripts** — spacetime / E7 theory papers |
| [`swind/`](swind/) | S-wind library experiments |
| [`systems-f64/`](systems-f64/) · [`systems-rational/`](systems-rational/) | Early system prototypes superseded by `systems/` + `manifestsys/` |
| [`ai-incubator/`](ai-incubator/) | Pre-product incubation experiments |
| [`fate-organize/`](fate-organize/) | Fate file-organizer research demo |
| [`training_data/`](training_data/) | Downloaded corpora for offline experiments |

---

## Separate product lines — own SKU, own roadmap

| Path | Description |
|------|-------------|
| [`valhalla-crypto/`](valhalla-crypto/) | **Yaksha Cipher** — post-quantum / GPU crypto (not AI memory) |
| [`val-os/`](val-os/) · [`val-os-v5/`](val-os-v5/) · [`val-os-minimal/`](val-os-minimal/) · [`valos_v4/`](valos_v4/) | **Val-OS** — Wave-language OS experiments |
| [`val-os-v3-archive/`](val-os-v3-archive/) · [`valos_original/`](valos_original/) | Frozen Val-OS snapshots |
| [`valhalla压缩/`](valhalla压缩/) · [`wave/`](wave/) · [`wave-bin/`](wave-bin/) · [`wave-asm/`](wave-asm/) | **Wave / compression compiler** toolchain |
| [`website/`](website/) · [`website-new/`](website-new/) · [`enterprise-site/`](enterprise-site/) · [`rogue-intelligence-site/`](rogue-intelligence-site/) · [`rogueintelligence-site/`](rogueintelligence-site/) | Marketing / corporate site sources |
| [`pitch/`](pitch/) | Historical pitch decks |

---

## Archive & legacy — frozen; do not score for ValhallaBase

| Path | Description |
|------|-------------|
| [`archive/`](archive/) | **Legacy Wave / tile snapshots** |
| [`backups/`](backups/) | **Point-in-time code backups** |
| [`wave-v4-backup/`](wave-v4-backup/) | Wave v4 tree backup |
| [`experiments/`](experiments/) | One-off scripts without product protocol |
| [`patents/`](patents/) · [`patent_applications/`](patent_applications/) | **Patent drafts** — legal IP, not runtime |
| [`real_hw_test/`](real_hw_test/) · [`real_test/`](real_test/) · [`ui_demo/`](ui_demo/) · [`kernel_demo/`](kernel_demo/) · [`color_demo`](color_demo) · [`fate_boot`](fate_boot) · [`simple_boot/`](simple_boot/) · [`rt/`](rt/) | **Hardware / boot demos** — pre-ValhallaBase prototypes |
| [`valhalla-rational/`](valhalla-rational/) | Rational-precision experiment fork |
| [`output/`](output/) | Generated build / eval output scratch |

---

## Quick filter (investor view)

| Question | Answer |
|----------|--------|
| What is the product? | **ValhallaBase** — `manifestsys/` + `systems/` + inference tools |
| What proves it works? | `reports/valhalla_inference/` + `./scripts/run_highlight_test.sh` |
| What is *not* the product? | crypto · Val-OS · Wave compiler · physics sim · patents · archives · websites |
| Default LM size? | **0.5B** runtime; **3B** = industry calibration only |

---

*Rogue Intelligence Inc. · map maintained with product releases.*

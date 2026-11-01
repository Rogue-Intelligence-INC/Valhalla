# Valhalla

A signal-governed architecture for persistent structural learning, implemented
in Rust. Experience is written into a persistent, self-organising structure
(Tile and Stem populations under a global algebraic criterion, Fate), and
subsequent computation is routed through that structure. A language model
may be attached as an optional decoding head; it is not the object of study.

---

## Principal results

Three findings define the present state of the programme (November 2026).
Each is stated with the protocol that produced it.

**1. Structure carries the answer.**
On withheld multi-hop questions, trained bodies answer at 71–73% through
structure formed by their own dynamics. Removing every connection — with
stored bytes left intact — returns all 15 seeds to a single 45.5% floor:
a causal effect of **+25.7 percentage points**, replicated at
**+27.4 ± 0.9 pp** after a 6.7× scale-up (200 symbols). Because storage was
untouched, the effect is attributable to the network, not to retrieval.

**2. One body, from ingress to egress.**
A single deterministic programme (20 rounds, ≈55 s) takes facts, text,
code, image and audio in through Fate segmentation and scores three egress
channels separately, on the same body: stored-fact answers **100/100**
through a read-only interface, storage accounted **to the byte (×8 copies,
as designed)**, and full body state that freezes and thaws with identical
scores (F = C). The population reaches 7,130 tiles / 7,360 cells; the
global Fate equation remains observable throughout.

**3. A null result, reported as a result.**
Directed curricula were tested at two scales under criteria fixed in
advance and were not found: **+0.2 ± 1.4 pp** against a volume-matched
control, across three scoring instruments and 15 seeds. The curriculum
claim has been withdrawn. What survives is narrower and stated as such:
the body learns from contact volume, and direction operates one layer up,
through material selection and the Fate equation.

## Highlights

- **Endogenous organisation.** No task graph is supplied. At four training
  sizes (25–150 MB), the two populations assemble opposed topologies on
  their own: a sparse, hub-free Tile fabric (one giant component) and a
  dense Stem field whose unconnected fraction falls from 42% to 2%.
- **Persistence without retrieval.** Clearing stored bytes does not change
  answers; clearing connections does. The distinction is measured, not
  asserted.
- **Determinism.** Identical input and seed reproduce a body bit-for-bit;
  an independent rerun of the 25 MB body matched exactly.
- **Auditable storage.** Every byte fed to the system is accounted for in
  eight stored copies (two populations × four quad stores), decoupled
  from segmentation granularity, with zero measured storage forgetting
  (BWT +0.0 pp).
- **One programme.** Library tests, six check packages, the industry
  battery, the incubation proof, the end-to-end pipeline and the Tile/Stem
  monitoring page all run under a single Rust entry point, `valhalla`.

## The system

```text
multimodal ingress (text · image · audio · code, byte-wise)
    → Fate segmentation (cluster-equation convergence sets boundaries)
    → quad storage (Tile/Stem × BlackHole/MeshBrain/MultiNova/BaseForce)
    → native cycles (Tile/Stem cycles · global Fate tick · field settling)
    → egress, scored by channel:
        A  held-out material re-segmentation
        B  structural reachability over self-formed networks
        C  read-only Fate interface (react) over stored originals
```

| Component | Responsibility |
|---|---|
| Fate (`systems/metasystems/fate-f64`) | Signal/state governance; cluster equation; segmentation; global tick; read-only `react` interface |
| Tile (`manifestsys/tile-quattro`) | Sparse structural population; degree-capped connectivity; propagation |
| Stem (`manifestsys/stem-cell`) | Field-organised cell population; dense linking; field settling |
| Quad stores (`blackhole`, `meshbrain`, `multinova`, `base-force`) | Persistent byte storage, one instance per population per store |
| Hub (`manifestsys/hub-f64`) | Integration, evaluation programmes, unified entry point (`valhalla`) |
| Optional LM head | Decoding surface only; interchangeable and removable |

## Experiments

| Experiment | Question | Reported outcome |
|---|---|---|
| Causal battery (15 seeds, 2 scales) | Does self-formed structure causally support withheld answers? | +25.7 pp; +27.4 ± 0.9 pp at scale; ablation to a common floor |
| Persistence (F / I) | Is behaviour carried by structure or by stored bytes? | Freeze/thaw identical; byte-clearing inert; connection-clearing decisive |
| Scale run (160 MB) | What changes with corpus volume? | 20,127 tiles / 20,239 cells; storage exact; dense O(n²) field step identified as the binding constraint (~20 GB peak) |
| Directed cultivation (pre-specified; 15 seeds) | Does lesson order beat equal-volume exposure? | **Not detected**: +0.2 ± 1.4 pp (null) |
| Organisation census (25–150 MB) | What topology forms without design? | Opposed two-layer topologies; strength dispersion rises with scale (CV 0.78 → 1.27) |
| Industry battery (5 seeds) | Do fed relations enter the spontaneous network? | Storage 100%; network channel 54–76% by task family — self-generated analogues, not official datasets |
| Unique-capability suite U1–U4 (Python, Tier A) | Non-isolated multimodal ingress; gated proactive/abstain behaviour; pollution rescue; backbone independence | 4/4 within their named protocols, incl. U3 recovery **+38.7 pp** and TPI **0.755**; small slices (holdout n = 3 in places) |

## Why these results are reported this way

Credibility is treated as a property of procedure. The reporting practices
below are adapted from published norms — pre-specified analysis as
formalised in Registered Reports (Chambers et al., 2021) and the NeurIPS
reproducibility checklist (Pineau et al.) — and they constrain what can be
claimed here:

1. **Criteria before outcomes.** Decisive tests carry pass/fail criteria
   recorded in the run documents before execution. Outcomes are then
   reported against those criteria, including when they fail (the
   cultivation null is the worked example). These criteria were documented
   internally in advance; no external registry was used, and no formal
   preregistration is claimed.
2. **Controls that can only weaken the claim.** Headline numbers are
   accompanied by interventions designed to destroy them — connections
   cleared with storage intact, bytes cleared with structure intact,
   networks rewired at matched degree. Where such a control retains
   performance, the claim is narrowed accordingly (organisation is
   reported as formation, not optimality).
3. **Complete runs.** Results are reported per seed (15/15), with spread,
   not as a best run. Adverse readings produced during a run — for
   example, Tile cross-chain false reachability rising from 2.1% to
   12.5% by round 20 — remain in the record.
4. **Labelled provenance.** Where tasks are self-generated analogues of
   industry benchmarks, they are labelled as such wherever their numbers
   appear. Official-dataset standing is claimed only where an official
   protocol was run.
5. **Repeatability.** The system is deterministic; every headline run can
   be repeated with one command (§ Reproduction) and compared bit-for-bit.

Independent replication by a third party has not yet taken place and is
identified in the project documents as the next evidentiary step.

## Evidence classification

- **Class A — reproduced under current code:** the causal battery,
  persistence, byte-exact storage, zero storage forgetting, and the
  end-to-end pipeline above.
- **Class B — reproduced, protocol-bounded:** the industry battery
  (self-generated analogues), the U1–U4 suite and TPI 0.755 (named
  protocols, small slices), long-chain continuity (drift 0%, retention
  100% in the named `fragment_full` configuration), RAGTruth 38/40 under
  the Tier-A protocol, and the organisation census (formation, not
  optimality).
- **Class C — not established:** directed-curriculum gain (null result);
  topology specificity under new question types; paraphrase-level
  answering (the `react` interface returns stored originals verbatim and
  multi-hop chaining is performed by the caller); death/cleanup dynamics;
  state-of-the-art standing on open-domain benchmarks (LoCoMo and
  MultiHop measurements are reported as boundaries in the evidence
  documents). Class C items appear in no quantitative claim.

Canonical metrics: [`docs/valhalla/CANONICAL_METRICS_20260714.md`](docs/valhalla/CANONICAL_METRICS_20260714.md) ·
Evidence map: [`docs/valhalla/EVIDENCE_MAP_20261101.md`](docs/valhalla/EVIDENCE_MAP_20261101.md) ·
Per-run records: [`docs/valhalla/TEST_RUNS_20261101.md`](docs/valhalla/TEST_RUNS_20261101.md)

## Reproduction

```powershell
cargo build --release -p hub-f64 --bin valhalla
.\target\release\valhalla.exe            # L0 library tests · L1 check packages
                                         # L2 industry battery · L3 pipeline
.\target\release\valhalla.exe check      # L0 + L1 only
.\target\release\valhalla.exe incubation # L2 incubation proof (15 seeds, ~8 min)
.\target\release\valhalla.exe viz --open # Tile/Stem monitoring page
```

Each item writes a complete log to `logs/suite_<timestamp>/` and reports
PASS/FAIL by exit code. The monitoring page (`viz_out/index.html`) is a
single self-contained file generated from a live run; its field scatter
displays birth positions assigned by the birth formula — a direct mapping
of segment statistics — and is labelled accordingly.

Run specification: [`docs/valhalla/RUN_MODE.md`](docs/valhalla/RUN_MODE.md) ·
Pipeline details: [`docs/valhalla/FULL_FLOW_RUNBOOK.md`](docs/valhalla/FULL_FLOW_RUNBOOK.md)

## Repository structure

| Path | Contents |
|---|---|
| `systems/metasystems/` | Fate, quad stores, and related meta-systems |
| `manifestsys/` | Tile, Stem, chamber, storage, and the hub (binaries, unified entry) |
| `manifestsys/hub-f64/src/bin/` | Evaluation and pipeline programmes, incl. `valhalla`, `valhalla_full_flow`, `valhalla_viz`, `valhalla_incubation_proof`, `valhalla_industry_battery` |
| `docs/valhalla/` | Protocols, canonical metrics, evidence map, run records |
| `tools/` | Inference tooling and the legacy PowerShell suite wrapper |

## Requirements

- Rust (2021 edition); a CUDA toolkit is optional but expected on the
  reference workstation (`RUSTFLAGS=-L C:\Ai\cuda-libx64` on Windows).
- Python 3.10+ only for the legacy Tier-A inference tooling.

## References for reporting practice

- Chambers, C. D. et al. "The past, present and future of Registered
  Reports." *Nature Human Behaviour* 5 (2021).
  [doi:10.1038/s41562-021-01193-7](https://doi.org/10.1038/s41562-021-01193-7)
- Pineau, J. et al. "Improving Reproducibility in Machine Learning
  Research." *JMLR* 22 (2021); NeurIPS Paper Checklist,
  [neurips.cc/public/guides/PaperChecklist](https://neurips.cc/public/guides/PaperChecklist)
- Andrade, C. "How to write a good abstract for a scientific paper."
  *Indian Journal of Psychiatry* 53 (2011).
  [PMC3136027](https://pmc.ncbi.nlm.nih.gov/articles/PMC3136027/)

## Status and licence

Valhalla is active research software under development by Rogue
Intelligence Inc. The results above describe the system as of
1 November 2026 and will be revised as the Class C items are resolved.
Runtime licensing terms are stated in the repository notices; benchmark
harnesses and reports follow the notices attached to them.

---

*Rogue Intelligence Inc. · [github.com/Rogue-Intelligence-INC](https://github.com/Rogue-Intelligence-INC)*

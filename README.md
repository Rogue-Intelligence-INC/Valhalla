# Valhalla

**A signal-governed architecture for persistent structural learning.**

Valhalla is an implemented research system, written primarily in Rust, that
investigates a specific question: can compact, stateful signal interactions
modify what a system computes next, and can reusable structure form from
those interactions? The system is organised around a persistent structural
substrate (Tile and Stem populations), a global algebraic criterion (Fate),
and a set of quad storage components (BlackHole, MeshBrain, MultiNova,
BaseForce). A language model may be attached as an optional decoding head;
it is not the object of study.

This document states what the system is, what has been measured, under
which protocols, and what has not been established. Quantitative claims are
bounded by the protocols named alongside them.

---

## 1. System overview

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

Design commitments:

- **One ingress path.** All material enters through Fate's segmentation;
  storage accounting is decoupled from segment ("birth") granularity and
  is exact to the byte (eight stored copies: two populations × four quads).
- **Structure is endogenous.** Network edges arise only from the system's
  own dynamics (propagation, field linking, inheritance). No task graph
  is supplied.
- **A live global criterion.** A single quadratic cluster equation,
  adapted during operation, governs segmentation and global ticks. Its
  coefficients are observable at all times and are reported, not tuned
  per experiment.
- **Determinism.** Given identical input and seed, runs reproduce
  bit-for-bit; body state (structure, storage, equation) can be frozen
  and restored without change in measured behaviour.

## 2. Architecture

| Component | Responsibility |
|---|---|
| Fate (`systems/metasystems/fate-f64`) | Signal/state governance; cluster equation; segmentation; global tick; read-only `react` interface over stored items |
| Tile (`manifestsys/tile-quattro`) | Sparse structural population; degree-capped connectivity; propagation |
| Stem (`manifestsys/stem-cell`) | Field-organised cell population; dense linking; field settling |
| Quad stores (`blackhole`, `meshbrain`, `multinova`, `base-force`) | Persistent byte storage, one instance per population per store |
| Hub (`manifestsys/hub-f64`) | Integration, evaluation programmes, unified entry point (`valhalla`) |
| Optional LM head | Decoding surface only; interchangeable and removable |

## 3. Evidence

Results are classified by reproducibility status. Class C items appear in
no quantitative claim elsewhere.

### Class A — reproduced under current code (Rust core, November 2026)

| Result | Protocol | Value |
|---|---|---|
| Structural causation | Withheld multi-hop questions over self-formed structure; connection ablation, storage intact; 15 seeds, two scales (30 / 200 symbols) | **L = +25.7 pp / +27.4 ± 0.9 pp** above the ablated floor |
| Persistence | Freeze / thaw of full body state | Scores identical (F = C; I = C) |
| Storage accounting | End-to-end consolidated pipeline | Byte-exact, ×8 copies as designed |
| Storage forgetting | Backward-transfer measure on stored bytes | BWT +0.0 pp |
| End-to-end pipeline | 20-round consolidated run, egress scored at rounds 10 and 20 | Stored-fact interface 100/100; network true-pair reachability 98.5% |

### Class B — reproduced, protocol-bounded

| Result | Bound |
|---|---|
| Industry battery (storage 100%, network 54–76% across task families) | Self-generated task analogues, not official datasets |
| Unique-capability suite U1–U4 (4/4), incl. U3 pollution rescue **+38.7 pp**, TPI **0.755** | Named internal protocols; small slices (holdout n = 3 in places) |
| Long-chain continuity | Theme drift 0%, retention 100% in the named `fragment_full` configuration |
| RAGTruth subset | 38/40 under the Tier-A protocol |
| Spontaneous organisation census | Establishes that a two-layer network forms with opposed topologies; does not establish optimality (degree-matched rewiring retains task performance) |

### Class C — not established

- **Directed curricula.** A pre-registered test at two scales found no
  curriculum-ordering gain over equal-volume unstructured exposure
  (**+0.2 ± 1.4 pp**, null). "Directed cultivation" in this project denotes
  material selection and Fate-layer steering only.
- **Topology specificity** of the self-formed networks under new question
  types (instrumentation in progress).
- **Paraphrase-level answering.** The Fate `react` interface returns stored
  originals verbatim; multi-hop chaining is currently performed by the
  caller. Paraphrase egress is untested.
- **Death / cleanup dynamics** in the structural populations.
- **Official benchmark standing.** LoCoMo and MultiHop results are reported
  as boundary measurements in the evidence documents; no state-of-the-art
  claim is made for open-domain question answering, and the system is not
  presented as a Transformer replacement or as general intelligence.

Canonical metrics: [`docs/valhalla/CANONICAL_METRICS_20260714.md`](docs/valhalla/CANONICAL_METRICS_20260714.md) ·
Evidence map: [`docs/valhalla/EVIDENCE_MAP_20261101.md`](docs/valhalla/EVIDENCE_MAP_20261101.md) ·
Per-run records: [`docs/valhalla/TEST_RUNS_20261101.md`](docs/valhalla/TEST_RUNS_20261101.md)

## 4. Reproduction

The full system is exercised by a single Rust programme (no shell scripts
required):

```powershell
cargo build --release -p hub-f64 --bin valhalla
.\target\release\valhalla.exe            # full suite: L0 library tests,
                                         # L1 check packages, L2 industry
                                         # battery, L3 end-to-end pipeline
.\target\release\valhalla.exe check      # L0 + L1 only
.\target\release\valhalla.exe incubation # L2 incubation proof (15 seeds, ~8 min)
.\target\release\valhalla.exe viz --open # Tile/Stem monitoring page
```

Each item writes a complete log to `logs/suite_<timestamp>/` and reports
PASS/FAIL by exit code. The monitoring page (`viz_out/index.html`) is a
single self-contained file generated from a live run; its field scatter
shows birth positions assigned by the birth formula (a direct mapping of
segment statistics), and is labelled as such — it is not presented as
emergent geometry.

Run specification: [`docs/valhalla/RUN_MODE.md`](docs/valhalla/RUN_MODE.md) ·
Pipeline details: [`docs/valhalla/FULL_FLOW_RUNBOOK.md`](docs/valhalla/FULL_FLOW_RUNBOOK.md)

## 5. Repository structure

| Path | Contents |
|---|---|
| `systems/metasystems/` | Fate, quad stores, and related meta-systems |
| `manifestsys/` | Tile, Stem, chamber, storage, and the hub (binaries, unified entry) |
| `manifestsys/hub-f64/src/bin/` | Evaluation and pipeline programmes, incl. `valhalla`, `valhalla_full_flow`, `valhalla_viz`, `valhalla_incubation_proof`, `valhalla_industry_battery` |
| `docs/valhalla/` | Protocols, canonical metrics, evidence map, run records |
| `tools/` | Inference tooling and the legacy PowerShell suite wrapper |

## 6. Requirements

- Rust (2021 edition); CUDA toolkit optional but expected on the reference
  workstation (`RUSTFLAGS=-L C:\Ai\cuda-libx64` on Windows).
- Python 3.10+ only for the legacy Tier-A inference tooling.

## 7. Status and licence

Valhalla is active research software under development by Rogue
Intelligence Inc. Results above describe the state of the system as of
1 November 2026 and are expected to be revised as the open items in §3
are resolved. Runtime licensing terms are stated in the repository
notices; benchmark harnesses and reports follow the notices attached to
them.

---

*Rogue Intelligence Inc. · [github.com/Rogue-Intelligence-INC](https://github.com/Rogue-Intelligence-INC)*

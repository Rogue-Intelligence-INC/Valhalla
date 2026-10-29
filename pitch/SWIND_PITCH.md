# S-Wind

**An exact-arithmetic engine that proposes, tests, and falsifies mathematical conjectures — with a human in the loop.**

Rogue Intelligence LNC — Proprietary & Confidential

---

## One-Liner

Give S-wind a sequence of numbers. It tells you — exactly, over ℚ, no floating point in the core engine:

1. **Known?** It is matched against 130 built-in OEIS sequences (183 OEIS b-files loadable). Known results are labelled and **never counted as discoveries**.
2. **Fittable?** A fixed-order algebraic search (recurrences → closed forms → identities → equation solving → rational functions → multiplicative structure), then knowledge-base pattern classes, produces a candidate closed form.
3. **Survivable?** Candidates pass through fit-quality gates (degrees of freedom, extrapolation consistency, novelty) and are graded — never asserted. Survivors are queued for **human review**.

S-wind proposes; humans dispose. Full runtime specification: `ARCHITECTURE.md`.

---

## The Problem

Automated science has a credibility gap:

- Pattern miners produce correlations that collapse out-of-sample.
- Generative AI produces plausible-but-unverifiable claims at scale.
- Almost no tooling is built to **kill its own hypotheses cheaply** and keep an audit trail.

The bottleneck in conjecture-driven research is not generating ideas — it is **triaging** them: exact checking, known-result matching, and falsification, before a human spends an hour.

---

## How It Runs (three modes, honestly separated)

| Mode | Entry point | What happens |
|---|---|---|
| **Exploration** | `Fate::explore` | Structure extraction → known-sequence check → algebraic search → pattern classes with a DoF ≥ 2 non-triviality gate → signal-pool comparison (cross-domain isomorphism, generalization) |
| **Verified exploration** | `Fate::explore_with_verification` | Adds fit analysis, extrapolation verification, six-way discovery grading, and auto-archiving of candidates with DoF-based confidence (0.7–0.9) |
| **Self-loop** | `Fate::explore_with_self_loop` | Keeps working unsettled signals: a 12-transform chain (Δ, Δ², Σ, \|·\|, n·f(n), ratio, f², sgn, MA(3), Δ³, pairwise product, mod 10) with signature-based closure detection — up to 1,000 rounds or true exhaustion, declared **MYSTERY** |

A second, independent runtime (**Voyager**) turns signals into equations with complete step-by-step derivation traces and emits audit-ready LaTeX/Markdown reports.

Across calls, Fate **remembers**: every processed signal joins a cross-domain pool, and per-pattern success statistics re-order future trials — the engine's search strategy adapts to what has worked.

---

## What S-Wind Does (verified 2026-10-30)

| Capability | Status |
|---|---|
| Exact rational arithmetic (EPR) | Fuzz-tested against a Python big-integer oracle: 2,675 cases, zero failures (one boundary bug at 2¹²⁷ disclosed in appendix; fix designed) |
| Algebraic pattern fitting | Six-method algebraic search + pattern classes fit correctly across the probe suite (arithmetic / geometric / recurrence / polynomial / reciprocal / periodic) |
| Known-sequence recognition | 130 built-in sequences recognized instantly, never counted as discoveries |
| Continued exploration | Prime-gap signal: 16 rounds / 15 transforms before settling; exhausted signals are declared MYSTERY with the tested hypothesis space reported |
| Candidate & review tracks | Fate candidate pool (auto-archived, promotable at confidence ≥ 0.8 with support ≥ 3) plus a KnowledgeBase `PendingReview` queue (human verify → JSON persistence) — connecting them end-to-end is milestone M1 |
| Deterministic & self-contained | 56/56 unit tests pass from a clean clone; fixed-seed evaluation is bit-reproducible |

---

## The Falsification Engine

We pointed S-wind at our own favourite conjecture — "a unique quartic polynomial explains α⁻¹ ≈ 137":

- **146,410** candidate polynomials tested
- **146,317 falsified** by the system's own exhaustive search
- The tool's own printed conclusion: *uniqueness fails; the pattern is not special*

Blind tests and negative controls (Möbius, random sequences) ship in the repository and **reject** over-claims.

**We sell the referee, not the oracle.** A conjecture engine whose first job is to disprove is the one researchers can trust to pre-screen at scale.

---

## What S-Wind Is Not (disclosed up front)

- It has **not** discovered a validated physical law. Earlier "discovery" claims did not survive our own falsification runs and are withdrawn.
- The fast exploration path labels algebraic hits optimistically; only the **verified path** grades candidates. Verification itself has known bugs (appendix) — hence human review, always.
- Earlier generated paper catalogues contained simulated optima; they are **not** research output and are excluded from this deck.

We would rather show you the list of what it killed than a list of what it "found".

---

## Market

**Beachhead: conjecture triage for research & engineering teams**

- Mathematics / physics groups: sequence mining, identity hunting, pre-screening before human proof effort
- Engineering R&D: exact parameter-space screening with reproducible, auditable runs
- Verification-adjacent tooling: deterministic symbolic checking where f64 pipelines are unacceptable

Expansion: symbolic regression as a service; curated candidate feeds with human-verified labels (data asset).

---

## Business Model (proprietary licensing)

S-wind is closed-source, licensed commercially:

1. **Research licenses** — academic groups (evaluation terms in `LICENSE.md`)
2. **Enterprise licenses & API** — industrial R&D screening pipelines
3. **Custom exploration engagements** — bespoke conjecture-mining runs with audit trails
4. **Verified-candidate data products** — human-reviewed candidate feeds (post-M1)

---

## Traction (engineering, honestly stated)

- 64k-line Rust codebase; 87 runnable binaries; development continuous since 2026-01
- Core engine test-verified (56/56); candidate workflow demonstrated end-to-end
- Self-falsification suite ships in-repo (`swind_unified_proof`)
- Hardening completed 2026-10-30: zero machine-specific paths, all reported counts computed at runtime, deterministic CLI evaluation, self-contained build; main Valhalla workspace reproducibility restored (embedded-repo setup script, `cargo test -p s-wind-sim` = 56/56 from a fresh clone)

---

## Roadmap (what funding buys)

| Milestone | Deliverable | Why it matters |
|---|---|---|
| M1 | Connect Fate candidate pool → pending-review queue (persistence + dedup) | Closes the propose→review loop end-to-end |
| M2 | Rewrite verification gates on structured formulas (Expr), hold-out validation | Candidates become trustworthy enough to price |
| M3 | Fix disclosed arithmetic boundary bug; property-based fuzz in CI | "Exact" becomes an audited claim |
| M4 | Independent replication package (pinned commit, one-command build, public benchmark) | Third parties can check every number in this deck |

---

## Team

**Rogue Intelligence LNC**

- Research lab behind the Valhalla programme (S-wind, Wave, Valhalla Unified Core)
- Working style: negative controls first, claims second — this deck is an example

---

## The Ask

**Seed: $2M, milestone-tranched against M1–M4.**

- 40% Engineering (verification rewrite, pipeline closure)
- 30% Independent validation partnerships
- 20% Infrastructure (benchmark & replication hosting)
- 10% Operations

**Contact**: licensing@rogue-intelligence.com

---

## Appendix — Known Limitations (full disclosure)

1. `ExtendedInt` division/display bug for values in [2¹²⁷, 2¹²⁸) (affects EPR factorials ≥ 34!); root-caused, fix designed (M3).
2. Cross-domain verification branches on formula strings and extrapolates self-generated points; structured rewrite with true holdout is M2.
3. Known-sequence matching lacks a minimum-length guard (empty/short signals can match vacuously); fix queued with M2.
4. Self-loop explores by transformation only; formula-based signal extension described in a 2026-01 internal experiment is not wired in current code, and that experiment's results are not reproducible on HEAD — not claimed here.
5. Space "containers" are per-discovery metadata records, not persistent contexts (persistence is M1); the integrated `full_exploration` path converts EPR → i128 for Voyager/Space and degrades silently beyond i128.

*Reproducibility: clone this repository, run `cargo test --release` (56/56). Runtime specification: `ARCHITECTURE.md`. Every number in this deck traces to a test, a probe, or a shipped binary.*

---

**S-wind: the conjecture refinery.**
© 2026 Rogue Intelligence LNC. All Rights Reserved. Proprietary and confidential — do not redistribute.

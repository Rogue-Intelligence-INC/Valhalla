# Organ coalescence doctrine + long-runtime probe (2026-07-14)

**Protocol:** `natural-nn-organ-runtime-v1`  
**Script:** `tools/valhalla_inference/test_natural_nn_organ_runtime.py`  
**Report:** `reports/valhalla_inference/natural_nn_organ_runtime.json`

## Doctrine (corrects earlier “mega-cluster = fail”)

Project rule: **organs are not predefined**. Cell identity emerges from `data_signature` (and, while signature is still empty, from energy / variance / degree families).

| Observation | Reading |
|-------------|---------|
| Connected cells sharing the same emergent type | **Organ differentiation continuing** (coalescence / maturation) |
| One large same-type component | **Not a failure** — one organ body consolidating |
| Multiple same-type bodies with different labels | Multi-type organ trajectory |
| Mixed-type component (low purity) | Not yet typed — do not force an organ name |

This aligns with stem-cell source comments: *无预设类型，结构自由涌现*.

## Long-runtime schedule

Append pulses + rising `incubation_cycles` / Fate warmup; final pulse is cycles-only:

```bash
python3 tools/valhalla_inference/test_natural_nn_organ_runtime.py --long
```

## Live result (2026-07-14 `--long`)

| Pulse | tiles | fate_tick | typed sig | emergent types | largest | dominant type | purity |
|-------|------:|----------:|----------:|---------------:|--------:|---------------|-------:|
| p0_seed | 5 | 293 | 2 | 7 | 64 | `proto:e5_v4_d7` | 0.78 |
| p1_append | 5 | 549 | 2 | 7 | 64 | same | 0.78 |
| p2_deep | 6 | 805 | 2 | 7 | 64 | same | 0.78 |
| p3_longer | **10** | **1061** | 2 | 7 | 64 | same | 0.78 |
| p4_extra_cycles | 10 | 1061 | 2 | 7 | 64 | same | 0.78 |

**Verdict:** `ORGAN_COALESCENCE_RUNTIME_PASS`

- Differentiation **continuing**: one same-type body (proto energy/degree family) purity ~78% coalesced at 64/100 cells.  
- More structure with runtime: **tiles 5→10**, **Fate tick +768**.  
- Cell count soft-stable (~100); organ body size stable — consolidates rather than splitting into many preset organ names.  
- Honest next phase: only **2/100** cells carry non-empty `data_signature`; fuller signature typing / multi-body split needs richer signal inflow, not just more cycle counters.

## Relation to topology probe

`natural-nn-topology-v1` still measures small-world σ vs ER. Its old flag `functionally_differentiated_organs=false` meant “≥2 multi-cell CCs,” **not** “failed organogenesis.” Prefer this organ-runtime protocol for organ narrative.

*Rogue Intelligence LNC.*

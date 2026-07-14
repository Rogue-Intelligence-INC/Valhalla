# Natural NN causal protocol (strengthened) — 2026-07-14

**Protocol:** `natural-nn-causal-v1`  
**Script:** `tools/valhalla_inference/test_natural_nn_causal_v1.py`  
**Report:** `reports/valhalla_inference/natural_nn_causal_v1.json`

## Why this is stronger

| Gap in earlier probes | Strengthening | Literature |
|----------------------|---------------|------------|
| σ vs ER only | **Degree-preserving config edge-swap** null + p-value | Downes / culture surrogates |
| Density-biased σ | **SWP φ** vs lattice + random (supporting) | Muldoon et al. 2016 |
| Morphology only | **Organ-channel lesion vs rewire** | Soligo module intervention |
| Single run | **Multi-seed** (default 3) | Developmental statistics |

## Arms

1. **intact** — incubated Stem/Tile graph  
2. **organ_lesion** — remove edges incident to largest coalesced CC; zero member signatures  
3. **degree_rewire** — edge-swap null preserving degrees  

## Pass rule

- **Morphology:** majority of seeds reject config-null on clustering (p≤0.05)  
- **Causal:** organ-channel score (native weight 0.12) drops under lesion **more than** under rewire; structure mass collapses; lesion edges ≪ intact  

## Live result (3 seeds)

| Metric | Value |
|--------|------:|
| Verdict | **NATURAL_NN_CAUSAL_PASS** |
| Non-random vs config | **3/3** (p_cfg = 0) |
| Causal lesion > rewire | **3/3** |
| Mean organ-channel drop (lesion) | 0.0081 → 0 |
| Mean organ-channel drop (rewire) | 0 |
| Mean structure-mass drop (lesion) | **234.5** |
| Mean structure-mass drop (rewire) | **0** |
| φ_SWP mean | ≈0.34 (supporting; Muldoon often cites ≳0.6 — not sole gate) |

## Reproduce

```bash
python3 tools/valhalla_inference/test_natural_nn_causal_v1.py --seeds 3
python3 tools/valhalla_inference/test_natural_nn_causal_v1.py --seeds 5 --live-ask
```

## Honest bounds

- Organ channel uses hash text signatures (not LM embeddings); **direction** of lesion effect is the claim.  
- No preset cortical organ names.  
- Not a Mem0 online topology bake-off.

*Rogue Intelligence LNC.*

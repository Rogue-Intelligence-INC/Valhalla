# Hybrid 2.1 deploy merge — test report

**Date**: 2026-06-23  
**Phase**: test · 145Q  
**Backup**: `backups/hybrid-2.0-pre-2.1-20260623/` (hybrid-2.0 code + JSON)

## Change summary

| | hybrid-2.0 | hybrid-2.1 (default) |
|---|------------|----------------------|
| Version | `hybrid-2.0` | `hybrid-2.1` |
| Merge | **oracle** — pick first track with `correct==True` | **deploy** — routable without gold |
| MCQ | oracle best of tile/triad | tile_fate → triad (static) |
| Numeric policy id | `hybrid_max_all` | `numeric_confidence` |
| Numeric rule | oracle best of 3 tracks | native `direct_math`/`word_problem` → else patch fallback → else best native |
| `correct` field | copied from winning track | **re-scored** via `score_answer()` on chosen answer |

Legacy oracle: `--merge-mode oracle` or `VALHALLA_HYBRID_MERGE_MODE=oracle`.

## Results (same memory run + fair patch reuse)

| Mode | Total | open | mcq | numeric |
|------|-------|------|-----|---------|
| **deploy (2.1)** | **77.2%** (112/145) | 95.1% | 52.2% | **79.0%** |
| oracle (2.0) | 80.0% (116/145) | 95.1% | 54.4% | 86.8% |

**Source mix (deploy)**: tile_fate 63 · patch_lm 82  
**Source mix (oracle)**: tile_fate 72 · triad 1 · patch_lm 72

### Why deploy is lower (expected)

- **Numeric −7.8pp**: 11 items where oracle picked patch after native wrong; deploy uses patch **fallback** only when native lacks high-confidence path (`direct_math`/`word_problem`). Hard GSM/MATH items native misses still get patch, but 3 items differ vs oracle tie-break.
- **MCQ −2.2pp**: 1 item (`MCQ_ECON_01`) where triad right, tile wrong — deploy always shows tile.

Deploy numbers match **runtime-deployable** semantics (no gold peek).

## Reproduce

```bash
# hybrid-2.1 deploy (default)
python3 tools/fair_benchmark/run_hybrid_valhalla_lm.py --phase test --merge-mode deploy \
  --skip-patch --reuse-json reports/fair_benchmark/fair_benchmark_test_20260620_0859.json

# hybrid-2.0 oracle (legacy)
python3 tools/fair_benchmark/run_hybrid_valhalla_lm.py --phase test --merge-mode oracle \
  --skip-patch --reuse-json reports/fair_benchmark/fair_benchmark_test_20260620_0859.json
```

Full run with live patch (needs torch + GPU):

```bash
python3 tools/fair_benchmark/run_hybrid_valhalla_lm.py --phase test --merge-mode deploy
```

## Artifacts

- Deploy JSON: `reports/hybrid_valhalla_lm/hybrid_unified_test_20260623_0242.json` (last write wins if both modes run same minute)

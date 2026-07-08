# MCQ Ladder — Memory-Safe GPU F16 (2026-07-08)

**Protocol:** `mcq-ladder-v1` · `valhalla_hybrid_lm_patch` · 46 fair MCQ  
**Stack:** CUDA · `VALHALLA_LM_F16=1` · per-model serve restart · isolated session per question  
**Script:** `tools/valhalla_inference/run_mcq_full_ladder.sh`

---

## Results (measured)

| Model | Params | Correct | Accuracy |
|-------|--------|---------|----------|
| Qwen2.5-0.5B-Instruct | 0.5B | 29/46 | **63.0%** |
| Qwen2.5-3B-Instruct | 3.0B | 44/46 | **95.7%** |
| Qwen2.5-7B-Instruct | 7.0B | 45/46 | **97.8%** |

**Verdict:** `MCQ_LADDER_SCALES`

F16 GPU run matches 2026-07-07 FP32 ladder within ±1 question per size. **3B remains production default** (best accuracy/latency/VRAM tradeoff vs 7B marginal gain).

---

## Engineering summary

| Item | Detail |
|------|--------|
| **Memory control** | `--memory-safe`: fresh `valhalla_base serve` per model — VRAM released between 0.5B / 3B / 7B |
| **GPU F16** | `VALHALLA_LM_F16=1` halves LM weight memory |
| **Serve readiness** | `wait_infer_ready()` — fate_warmup probe until weights loaded |
| **Session isolation** | Per-question `clear_context()` — shared session snapshots exceeded ~32MB HTTP limit (~Q14+) |
| **HTTP retry** | `valhalla_base.py` retries broken pipe / connection reset |

### Root cause (industry suite MCQ skip)

Industry `--all` @ 3B skipped MCQ after LoCoMo: serve `broken pipe` from session snapshot inflation. Same class as proactive agency 3000-round mega — fixed by segment reset / isolated eval.

---

## Reproduce

```bash
export VALHALLA_USE_CPU=0 VALHALLA_LM_F16=1
export LIBRARY_PATH=/opt/cuda/targets/x86_64-linux/lib:$LIBRARY_PATH
export RUSTFLAGS='-L /opt/cuda/lib64'

tools/valhalla_inference/run_mcq_full_ladder.sh
```

**Artifact:** [`mcq_ladder_v1_full_test.json`](./mcq_ladder_v1_full_test.json) · log: [`mcq_ladder_mem_gpu_run.log`](./mcq_ladder_mem_gpu_run.log)

---

*Rogue Intelligence LNC.*

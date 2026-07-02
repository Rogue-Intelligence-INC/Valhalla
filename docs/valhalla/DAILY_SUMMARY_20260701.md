# Valhalla Inference — Daily Summary (2026-07-01)

## 变更摘要

| 模块 | 变更 |
|------|------|
| **`valhalla_base.rs`** | Proactive 时间门：对话轮数 → **Fate `signal_tick` 差值**；`fate_baseline_tick` + `proactive_min_fate_signals`（默认 64） |
| **`signal_ingress.rs`** | `TriadIncubationSnapshot.fate_signal_tick`；seed 恢复 logical time |
| **`fate-f64`** | `seed_from_hub_snapshot(..., logical_time)` |
| **`valhalla_base.py`** | 后续 `ask()` 仅传 session，避免重 incubate 重置 tick |

## 实验结果（proactive-fate-v2）

| 协议 | 脚本 | 结果 |
|------|------|------|
| proactive-fate-v2 | `test_proactive_fate_v2.py` | **71/72 (98.6%)** · PPI-v2 **0.890** |

| 指标 | 值 |
|------|-----|
| UP on-topic | 100% |
| MAR mismatch abstain | 88.2% |
| FIR | 3.1% |
| Gates | 3/3 |
| Handcraft | 7/7 |
| Fair holdout | 61/62 |

**机制：** `(fate_signal_tick − fate_baseline) ≥ 64` + spread gate → self-probe → FOG → nudge  
**Gate 示例：** `fate_signals=37<min_fate_signals=64`（1-turn）；`idle_ready` @ 74 signals（2 QA）

## 报告

- `reports/valhalla_inference/proactive_fate_v2_test.json`
- `reports/valhalla_inference/PROACTIVE_FATE_V2_REPORT.md`

## 复现

```bash
RUSTFLAGS="-L /opt/cuda/lib64" cargo build -p hub-f64 --release --bin valhalla_base
python3 tools/valhalla_inference/test_proactive_fate_v2.py
python3 tools/valhalla_inference/run_confidence_report.py
```

## 文档更新

- `FATE_E7_ALGEBRA_AND_MEMORY.md` · `EXPERIMENT_RECORD_INFERENCE_PARADIGM.md` · `FATE_OUTPUT_GATE.md`
- `DISTINCTIVE_CAPABILITIES_CN.md` · `VALID_BENCHMARKS_INTERPRETATION_20260630.md`
- `docs/README.md` · root `README.md` · `valhalla-unified-core/README.md`

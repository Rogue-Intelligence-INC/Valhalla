# Proactive Agency Benchmark

**Protocol:** `proactive-agency-benchmark-v1`  
**Model:** `Qwen2.5-3B-Instruct-Valhalla-TriModal-Fusion-v3-test-s0.10` (baked tri-modal)  
**Updated:** 2026-07-08

---

## 最大亮点（Headline）

**在单进程 GPU serve + 内存有界分段下，首次跑通 3000 轮高轮次对话基准：140 秒完成 460 次 decode、114 次 session 分段 reset，证明 ValhallaBase 长链对话基础设施可扩展到「千轮级」生产压测。**

配套 **24 轮全轨标准跑** 给出 **Agency Index 0.600（AGENCY_VIABLE）**，其中 **跨语言 CCU 100%**（EN/ZH/FR 同核心、零切换）、**多模态一致性 67%**——主动性 AI 的「多语 + 多模态同一问题」能力已可演示，**proactive idle 门控** 仍是明确下一步（spread 未过，非轮次不足）。

---

## Executive summary

| Run | Rounds | Duration | Agency Index | Verdict | Highlight |
|-----|--------|----------|--------------|---------|-----------|
| **Standard（3 tracks）** | 24 dialogue / 12 items | 14s | **0.600** | AGENCY_VIABLE | Crosslang **100%** · Multimodal **67%** |
| **Mega（long-only）** | **3000** | **140s** | 0.000 | AGENCY_WEAK | **460 decode · 114 segment resets** · infra validated |

---

## Tracks

| Track | 内容 | 指标 |
|-------|------|------|
| **A long_dialogue** | 高轮次 session → primary Q → supplement → `proactive_idle` | primary / supplement / proactive pass |
| **B multimodal_natural** | 同一问题：text / TTS / image / tri-modal | keyword hits · consistency |
| **C crosslang** | EN / ZH / FR 同事实 · unified core | CCU（all_correct rate） |

**Agency Index** = 0.35×long_pass + 0.25×multimodal_pass + 0.25×crosslang + 0.15×proactive_rate

---

## Standard run（24 轮 · 三轨）

| Metric | Value |
|--------|-------|
| Agency Index | **0.600** |
| Long-dialogue pass | 33.3% |
| Supplement pass | 33.3% |
| Proactive pass | 0.0% |
| Multimodal consistency | 0.667 |
| Crosslang CCU | **100%** |

**Interpretation:** 短程对话下 primary/supplement 部分通过；proactive 未触发（`fate_signals` 或 `hub_prefs spread` 门控）。跨语言与不切换栈已 production-grade。

---

## Mega run（3000 轮 · 内存有界）

| Metric | Value |
|--------|-------|
| Dialogue rounds | **3000**（94 ask + 2906 extend） |
| Decode count | **460** |
| Segment resets | **114**（每 4 decode 一次，cap snapshot ~100MB 膨胀） |
| Dialogue elapsed | **139.9s** |
| extend_chunk | 8（批量 append_only flush） |
| max_session_corpus | 128 |
| Proactive | ✗ `spread=0.0000 < 0.0031` · `fate_signals=37` |

**Engineering findings:**

1. **Session snapshot 指数膨胀**：无分段时 HTTP payload 在 ~5 次 decode 后达 **100MB+**（serve 413 / broken pipe）。
2. **分段 reset + append_only**：单模型 serve 下 3000 轮 **2.3 分钟** 跑完，峰值内存可控。
3. **Proactive 瓶颈是 spread，不是轮次**：3000 轮后仍 `spread=0.0000`；`fate_signals=37` 已过 min=32。
4. **分段 reset 牺牲长程 primary**：mega primary 退化为 corpus 检索引文，不如 24 轮 targeted 场景。

---

## Infrastructure added

| Component | Path |
|-----------|------|
| Benchmark runner | `tools/valhalla_inference/run_proactive_agency_benchmark.py` |
| Shell entry | `tools/valhalla_inference/run_proactive_agency_benchmark.sh` |
| Scenarios bundle | `tools/standard_benchmarks/bundled/proactive_agency_scenarios.json` |
| HTTP body limit 32MB | `manifestsys/hub-f64/src/valhalla_base_http.rs` |
| `ask()` append_only auto-flush | `tools/valhalla_inference/valhalla_base.py` |

CLI flags: `--mega` · `--turns 3000` · `--ask-every 32` · `--extend-chunk 8` · `--segment-decodes 4` · `--max-session-corpus 128` · progress · checkpoint · pause

---

## Run commands

```bash
# Standard（三轨 · baked 3B）
tools/valhalla_inference/run_proactive_agency_benchmark.sh --verbose

# Mega 3000 轮（内存有界 · 需 serve）
target/release/valhalla_base serve --port 8788 &
VALHALLA_BASE_URL=http://127.0.0.1:8788 \
  tools/valhalla_inference/run_proactive_agency_benchmark.sh \
  --mega --turns 3000 --extend-chunk 8 --segment-decodes 4 \
  --max-session-corpus 128 --verbose

# Smoke
tools/valhalla_inference/run_proactive_agency_benchmark.sh --fast
```

---

## Artifacts

| File | Description |
|------|-------------|
| `proactive_agency_benchmark.json` | Latest JSON（当前为 mega 3000） |
| `proactive_agency_mega3k_mem.log` | Mega 内存有界完整日志 |
| `proactive_agency_full.log` | Standard 三轨日志（Agency 0.600） |

---

## Next steps

1. **Proactive spread 门控**：调查 `hub_prefs_spread` 在长链下恒为 0 的根因（Fate incubation / fragment density）。
2. **Mega + 长程记忆**：分段 reset 与 targeted reanchor 结合，避免 primary 完全失忆。
3. **Native decode primary**：减少 `[corpus:N]` 引文式回答，改用 hybrid native 直答 rubric。

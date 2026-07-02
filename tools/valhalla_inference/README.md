# Valhalla Inference Loader

Valhalla 各模块 ↔ Transformer 的统一加载层（vLLM 风格 API + **Valhalla-as-base**）。

## Valhalla-as-base（上下文灵活基座）

Valhalla **同时作为基座**：每请求动态 `context` → Triad 孵化 → 结构快照 → 可选 LM decode。

| 对比 | 标准 LLM | ValhallaBase |
|------|----------|--------------|
| Context | 拼进 prompt 窗口 | ingress → Hub/Tile/Stem 结构 |
| 换 context | 改 prompt | `set_context()` / 换 bundle，重建 fingerprint |
| LM 角色 | 唯一基座 | **decode head**（runtime patch） |
| 无 LM | 不可用 | `decode=native` Tier B 记忆检索 |

```bash
cargo build -p hub-f64 --release --bin valhalla_base
python3 tools/valhalla_inference/test_valhalla_base_context.py
python3 tools/valhalla_inference/test_valhalla_base_context.py --decode native
```

协议: `valhalla-base-v1` · `manifestsys/hub-f64/src/valhalla_base.rs`

### v1 扩展（1 hybrid · 2 session · 3 multimodal）

| # | 能力 | 用法 |
|---|------|------|
| 1 | **Hybrid 路由** | `decode=hybrid` — open→native，mcq/numeric→LM logprob |
| 2 | **持久 session** | `session` + `append_lines` — 增量 context，不重发全量语料 |
| 3 | **多模态 context** | `context_items[]` — text / audio_features / image_features ingress |

```bash
python3 tools/valhalla_inference/test_valhalla_base_v1.py
```

```python
base = ValhallaBase(decode="hybrid")
base.set_context(["Fact A"])
base.ask("open question?", score_type="open")
base.append_context("Fact B")  # incremental session
base.run(questions, append_only=True)
base.append_multimodal(text="...", audio_features=[...], image_features=[...])
```

## 模块栈 → Transformer

| 模块 | 作用 | 写入 Transformer 的方式 |
|------|------|-------------------------|
| **hub** | Fate 四元路由 | `quad_layer_scales` → 24 层 LayerNorm 分组缩放 |
| **tile** | 块签名 | → `patch_vector` → hidden embed 增量 |
| **stemcell** | 细胞图 | → 融合进 `patch_vector` |
| **triad** | 三体并行孵化 | `build_model_patch()` → bake / runtime |
| **dl_front** | 多模态 ingress | LM 前 384-d → 三端口（非 generate 路径） |
| **patch_lm** | Candle Qwen2 | safetensors 加载 + 权重 surgery |
| **fused_lm** | 单轨 LM 产品 | baked forward + MCQ logprob |

## 加载模式

```
                    ┌─────────────────────────────────────┐
  Corpus / 多模态   │  Hub + Tile + Stem (Triad)          │
                    └──────────────┬──────────────────────┘
                                   │ model_patch
                    ┌──────────────▼──────────────────────┐
                    │  bake → model.safetensors           │
                    │  + valhalla_manifest.json           │
                    └──────────────┬──────────────────────┘
           ┌───────────────────────┼───────────────────────┐
           ▼                       ▼                       ▼
   transformers              vLLM (可选)           valhalla_fused_lm
   AutoModel...              预 bake 当 Qwen2       Rust Candle baked_lm
```

| 模式 | 条件 | 说明 |
|------|------|------|
| **baked_hf** | 有 manifest，`runtime_patch=false` | 结构已在权重里；vLLM 可直接 serve |
| **runtime_patch** | 裸 Qwen | 每题 Triad → 运行时 patch |
| **vllm_compat** | 预 bake 目录 | `vllm serve models/hf/...` 当普通 Qwen2 |

## 快速测试

```bash
source .venv-llm/bin/activate
export HF_ENDPOINT=https://hf-mirror.com

# vLLM 风格 API（transformers 后端）
python3 tools/valhalla_inference/test_model_load.py

# 指定引擎
python3 tools/valhalla_inference/test_model_load.py --engine transformers

# 若已装 vLLM
pip install vllm   # 可选
python3 tools/valhalla_inference/test_model_load.py --try-vllm

# + Rust fused_lm smoke
python3 tools/valhalla_inference/test_model_load.py --fused-smoke
```

## Python API（类 vLLM）

```python
import sys
sys.path.insert(0, "tools/valhalla_inference")
from loader import ValhallaLLM

llm = ValhallaLLM("models/hf/Qwen2.5-0.5B-Instruct-Valhalla-FreeMultimodal-Fusion-v2-test-s0.10")
out = llm.generate(["What is Newton's first law?"])
print(out[0].text)
```

## vLLM CLI（预 bake 模型）

Baked 权重已是标准 Qwen2 HF 格式，`valhalla_manifest.json` 被忽略：

```bash
vllm serve models/hf/Qwen2.5-0.5B-Instruct-Valhalla-FreeMultimodal-Fusion-v2-test-s0.10 \
  --trust-remote-code --max-model-len 4096
```

Hybrid 路由 / 每题 runtime patch **不在** vLLM 图内；需外部 orchestrator 或 pre-bake。

## 报告

`reports/valhalla_inference/model_load_test.json`

## 推理范式实验（零权重 / 非 fine-tune）

完整记录见 `valhalla-unified-core/docs/EXPERIMENT_RECORD_INFERENCE_PARADIGM.md`。

| 脚本 | 协议 | 指标 |
|------|------|------|
| `test_valhalla_base_training_potential.py --protocol v2` | training-potential-v2 | TPI-v2 |
| `test_new_paradigm_potential.py` | new-paradigm-v1 | NPPI |
| `test_context_topic_drift.py` | context-topic-drift | swap / drift |
| `run_local_corpus_demo.py` | local-corpus-demo-v1 | 本机语料 ingest |

```bash
# 新范式潜力（fair holdout + 合成 novel + 负对照）
python3 tools/valhalla_inference/test_new_paradigm_potential.py

# 本机 markdown 语料 → teach → 生成对比
python3 tools/valhalla_inference/run_local_corpus_demo.py \
  --corpus-dir valhalla-unified-core/docs

| `test_fate_output_gate.py` | fate-output-gate-v2 | FOG precision/coverage |
| `test_proactive_fate_v2.py` | proactive-fate-v2 | PPI-v2 / UP / MAR · **Fate signal_tick warmup** |

```bash
# 统一置信区间报告（Wilson + bootstrap，汇总全部实验 JSON）
python3 tools/valhalla_inference/run_confidence_report.py

# 补 traditional_lm 对照 + 刷新 TPI/NPPI（需 .venv-llm + HF Qwen cache）
.venv-llm/bin/python3 tools/valhalla_inference/test_valhalla_base_training_potential.py --only-rag
.venv-llm/bin/python3 tools/valhalla_inference/test_new_paradigm_potential.py --only-n5

# 主动 Fate — 全量 fair holdout（--fair-limit 0 = 全部 open_retrieval）
python3 tools/valhalla_inference/test_proactive_fate_v2.py --fair-limit 0

# 跨语言同核心（不切换栈）+ 外部 holdout
python3 tools/valhalla_inference/test_crosslang_core_unified.py
python3 tools/valhalla_inference/test_external_holdout.py

# MCQ 专项（fair-1.2 n=46）+ max_pick 路由
python3 tools/valhalla_inference/test_mcq_coverage_v1.py --fast
.venv-llm/bin/python3 tools/valhalla_inference/test_mcq_coverage_v1.py --only-trad-lm

# Token / wire 效率（30 轮对照）
python3 tools/valhalla_inference/test_token_efficiency.py

# Fate 输出门控（emit / abstain / defer）
python3 tools/valhalla_inference/test_fate_output_gate.py
```

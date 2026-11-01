# LLM 分解增强：证据评估与孵化复现方案（2026-11-01）

## 被评估的对象

分解管线（`tools/model_decompose/`、`TRANSFORMER_USE.md`）：现成 Qwen
解码层按四段映射 Hub/Tile/Stem 端口 → TriadSession 内 Fate 多轮适应
闭合（`model_recompose_ready`：aggregate ≥0.85、decompose ≥0.75、Fate
退出探索）→ 权重手术写回新检查点（四元 scale 乘 LayerNorm + 结构向量
按 strength 加 embed_tokens 行；无损失、无反向传播；原检查点只读）。
已落盘产物：`models/hf/Qwen2.5-0.5B-Instruct-Valhalla-*-s0.10`
（含 valhalla_manifest.json：cycles、corpus_lines、patch 指纹、
weight_delta_norm ≈298）。能力读数汇总于
`DECOMPOSE_RECOMPOSE_CAPABILITY.md`，原始逐题记录在
`reports/valhalla_inference/`、`reports/fair_benchmark/`。

## 证据盘点（按本项目 A/B/C 口径）

| 读数 | 协议/规模 | 对照 | 主要弱点 |
|---|---|---|---|
| MCQ 结构补丁+路由 30/46 vs 传统 RAG 13/46（+37.0pp） | mcq-coverage-v1，46 题 | 同骨干 RAG | n=46；strength（0.08/0.10）为事后选定值，无预先声明；无 sham 对照 |
| 同规模 MCQ hybrid 65% vs 对照 15%（+50pp） | 日报 §7 | 有 | 同上；逐题记录在 fair_benchmark |
| 多模态融合 12 题 8/12→9/12 | context-core-fusion-v2 | 有 | n=12，只许作轶事级旁证 |
| 一次孵化 Tile 22/46 vs 逐题重载 13/46 | MCQ 同题集 | 有 | 与上行同批题，不独立 |
| U3 抗污染 +38.7pp（87.1% vs 48.4%） | CANONICAL Tier A | 有（污染 dump） | 已在案 Class B，本评估不重复计价 |

**裁定**：分解增强可以作为 **Class B 应用层证据**被引用，命题须限于
"结构导出的补丁能在具名小协议上改变 LM 行为且方向有益"。**不能**
用作成立新计算范式的证据，原因有三：(1) 效果载体是权重手术后的 LM，
不是身体自身的计算；(2) 关键对照缺失——没有"随机结构向量、同范数"
的 sham bake，无法区分收益来自结构信息还是来自任意同量级扰动；
(3) strength 与题集规模使选择偏差无法排除。另外注意已烘焙模型的
manifest 显示 corpus_lines 仅 4 行量级——"训练量"表述对外必须按
manifest 实数，不许放大。

## 在孵化（Rust 身体）上复现的方案

分两期，判据预先声明，任一期不过则该线封存为 Class B 不再投入。

**E-LLM-A · sham 对照复现（先行，便宜）**
在原 Python 管线、原题集（MCQ 46，协议 mcq-coverage-v1 不变）上补
三臂：(i) 真补丁（原 strength，先验固定 0.10，不许看结果回调）；
(ii) sham：随机向量、同 weight_delta_norm；(iii) scale-only（只乘
LayerNorm、不加结构向量）。判据：真补丁高于 sham ≥10pp 才许进入
B 期；否则"结构特异性"在该管线未检出，分解线整体降为工程轶事。

**E-LLM-B · Rust 孵化身体复现（桥接，本体工作）**
现状如实记录：Rust 孵化身体（TileQuattroComplete/StemCellComplete）
与烘焙所用的 TriadSession 是两套结构，**目前没有从孵化身体导出
补丁张量的桥**。B 期即建此桥并检验：
1. 以 MCQ 知识语料培养孵化身体（Fate 分割入口 + 四元记账，与
   full_flow 同喂法），冻结身体（save_full 同格式存档）；
2. 由身体导出补丁：以符号→出生单位映射与连接统计生成结构向量，
   导出规则在看测试结果前写死并随代码提交；
3. 同 strength 烘焙 Qwen2.5-0.5B，四臂对照：base / RAG / 真补丁 /
   sham（同范数随机）。判据：真补丁 > sham 且 > RAG，逐题记录
   全量落盘；多语料种子 ≥3 个，各臂同题。
通过则证据升格为"身体结构可导出并增强外部 LM"的跨系统 Class A
候选（仍非范式证据，范式支点在 E1 拓扑特异性）；不过则维持
Class B 并注明桥接未检出。

## 与主线实验的排序关系

E0（统一预算）已毕且定稿保级。E-LLM-A 排在 E1 之前跑（成本低、
能先把分解线的计价定死）；E-LLM-B 排在 E2（原生闭环）之后——
若身体连自定位答题都未立，先做导出补丁容易把外部映射的功劳
记到身体头上，重蹈标签泄漏的覆辙。

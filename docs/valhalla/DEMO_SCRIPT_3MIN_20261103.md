# Valhalla — 3 分钟产品演示与演讲稿（2026-11-03）

**受众**：投资人 / 技术评估者 · **时长**：约 3:00
**载体**：deck v1.6（第 1–3、10 页）+ 终端 + 监控页 `viz_out/index.html`
**口径**：全部数字为 2026-11-03 现行代码实测（基线 e1ff0b3）；
未成立项按稿内标注说，不临场加码。

---

## 演示前准备（提前 10 分钟）

```powershell
cd C:\Ai\rogue-products\Valhalla
# 清残留环境（重要：上一轮 VH_*/VALHALLA_* 会污染）
Get-ChildItem env:VH_*,env:VALHALLA_* | ForEach-Object { Remove-Item "env:$($_.Name)" }
$env:RUSTFLAGS="-L C:\Ai\cuda-libx64"
$env:PATH="C:\Ai\cuda-libx64;C:\Program Files\NVIDIA GPU Computing Toolkit\CUDA\v13.4\bin;$env:PATH"
# 预构建并预跑一遍孵化（保证现场二进制与缓存就绪）
cargo build --release -p hub-f64 --bin valhalla --bin valhalla_incubation_proof
.\target\release\valhalla.exe viz --rounds 20   # 生成 viz_out/index.html
```

- 浏览器预开三页：deck v1.6、组织站 Valhalla 子页、
  `viz_out/index.html`（监控页）。
- 终端字号调大；现场演示命令只跑种子 1（数秒内出结果）。
- **兜底**：若现场构建/运行异常，直接打开
  `reproduce/logs/` 最新日志与监控页讲，每个节拍都有
  对应静态材料（deck 第 10 页即同一组数字）。

---

## 分镜与台词（EN 主讲稿 ≈ 430 词 / 3 分钟）

### 0:00–0:35 问题与定位
**[SCREEN]** deck 第 2 页 THE PROBLEM → 第 3 页 THE SOLUTION

> "Deployed AI has three structural problems. It is frozen after
> deployment — every improvement needs a new training run. It
> generates answers it cannot show the source of — hallucination
> is the default failure mode. And everything it knows lives in a
> fragile context window.
>
> Valhalla is our answer: a **rules-first, self-forming neural
> network**. Explicit algebraic rules govern a persistent body
> that grows its own structure. Experience is written into that
> structure, and all later computation runs through it. A language
> model can be attached as a decoder later — it is not the engine."

### 0:35–1:25 现场演示：结构承载答案（核心一分钟）
**[SCREEN]** 终端。**[TYPE]**（种子 1，现场跑）

```powershell
$env:VALHALLA_PROOF_SEEDS='1'
.\target\release\valhalla_incubation_proof.exe
```

**[指着输出讲]** 依次指 C（完整身体）→ G（拆连接）→ I（清存储）三行。注意：种子 1 单种子读数更高（实测 C 74.2 / G 45.5 / I 74.2，约 6 秒跑完），台词报的 +11.6 是 15 种子汇总——指输出时先说"这是单种子示意"，结论落在汇总数上，不要拿单种子读数当结论。

> "This body was cultivated on chained facts, then asked questions
> it never saw. Watch three numbers. Intact body: it answers
> through its own structure. Now we clear **every connection** —
> stored bytes untouched — and the score collapses to the 45.5%
> floor. Clear the stored bytes instead, keep the connections:
> nothing changes.
>
> Across 15 seeds that difference is **+11.6 points** — the
> capability lives in the structure the body formed, not in
> retrieved text. And because the read-only output returns stored
> originals verbatim, nothing in this channel is free-generated.
> That is hallucination engineered out — in the measured channel,
> and we say exactly that."

### 1:25–1:55 结构长什么样
**[SCREEN]** `viz_out/index.html` 监控页：种群曲线 → 方程漂移 → 逐料出生

> "This is the same body growing: nine thousand tiles, 123
> thousand nerves after fifty rounds of eight kinds of material —
> facts, text, code, image, audio — all through one entrance.
> Parallel strands are bundled into single nerves by the Fate
> equation; that one mechanism cut the connection count by 99.8%
> and made scoring a hundred times cheaper. The governing equation
> itself drifts observably as material changes — 0.867 to 0.716
> across the run — and it only ever learns from material it has
> already judged."

### 1:55–2:30 证据纪律（诚实节拍，不跳过）
**[SCREEN]** deck 第 10 页 PRINCIPAL RESULTS / 子页 Evidence 节

> "We price every claim by its evidence, and we publish the zeros.
> Continual-learning forgetting: minus 0.7 points — essentially
> eliminated. Directed curricula: **not detected**, at two scales.
> Topology specificity: **not established** — we say so on the same
> page as the positive results. Long-range composition is our open
> deficit, 28.9% at four-to-five hops, and fixing it is the next
> milestone, with its criterion written down before we run it."

### 2:30–3:00 复现与融资收尾
**[SCREEN]** 终端一行命令 + deck 融资页

```powershell
.\reproduce\reproduce.ps1   # 全套：构建→库测→检查→电池→孵化15→50轮→逐项核对
```

> "Everything I just showed regenerates from one command in the
> public repo — build, tests, fifteen seeds, fifty rounds, and it
> checks its own headline numbers, sixteen out of sixteen. Don't
> take my word for any of it; run it.
>
> We are raising a pre-seed of 500K to 1M dollars against
> pre-registered milestones: the streaming field step, long-range
> recovery, and partner evaluations. Valhalla is a rules-first,
> self-forming neural network — and unlike most claims you'll hear
> this week, ours comes with the ablation that could have killed
> it, published either way."

---

## 中文版台词（≈650 字 / 3 分钟）

**0:00–0:35**｜"部署后的 AI 有三个结构性毛病：冻住了——每次
进步都要重新训练；会编——答不出也要答，幻觉是默认失效模
式；记性全在一个脆弱的上下文窗口里。Valhalla 是我们的回
答：**基于规则的自发生神经网络**。显式代数规则治理一个持
久身体，经验写进它自发生长的结构，之后的计算全部经由这
个结构。语言模型以后可以接作解码头，但它不是引擎。"

**0:35–1:25**｜（跑种子 1）"这个身体在链式事实上培养，然
后回答它没见过的题。看三个数：完整身体靠自己的结构答题；
把**连接全清掉、存储字节不动**，分数塌回 45.5% 下限；反
过来清存储、留连接，毫无变化。15 个种子上这个差是
**+11.6 分**——能力住在身体形成的结构里，不在检索的文本
里。而且只读出口逐字返回存储原文，这个通道里没有任何自
由生成。这就是把幻觉从构造上消除——在测量的通道内，我
们只说这么多。"

**1:25–1:55**｜（监控页）"这是同一个身体在长：八种料五十
轮后 9180 个 Tile、12.3 万条神经，全走一个入口。平行连
接束被 Fate 方程集束成单条神经——连接数降 99.8%，评分
便宜两个数量级。治理方程本身随材料可观测地漂移，而且它
只从自己判定过的材料里学。"

**1:55–2:30**｜"我们按证据给每个声称定价，零结果照发：持
续学习遗忘 −0.7 分，基本消除；定向课程增益，两规模**未
检出**；拓扑特异性**未成立**——和正面结果写在同一页。长
程组合是我们公开的欠账，4–5 跳 28.9%，是下一个里程碑，
判据先写好再跑。"

**2:30–3:00**｜"刚才的一切，在公开仓库里一条命令全部再
生并自核对，16 项读数全中。不必信我——自己跑。我们正按
预注册里程碑融 50 万到 100 万美元 pre-seed：流式场步、长
程恢复、伙伴评估。Valhalla 是基于规则的自发生神经网
络——而且和这周你听到的多数声称不同，我们的声称附带那
个本可以杀死它的消融实验，无论结果如何都已发表。"

---

## 问答兜底（被追问时）

| 追问 | 回答（口径） |
|---|---|
| 和 RAG / 微调的区别？ | RAG 把经验放在计算基底外；Valhalla 把经验写进结构且计算经由结构路由——消融连接与消融存储的对照（G/I 两臂）就是为区分这两者设计的。 |
| 幻觉真的解决了？ | 只说测量通道内：只读出口逐字返回存储原文、无自由生成；开放域生成不在声称内。释义级答题未成立（页面 Class C）。 |
| 为什么 L 只有 +11.6，旧版 +26.8？ | 旧数是硬编码时代基线，已退役并写明原因（集束代表制压低远距离）。现行数是当前代码的完整复跑，不挑版本报。 |
| 规模能上去吗？ | 图成本已由集束解决；场步仍是稠密 n×n，是明示的工程墙，流式场步是融资里程碑第一项。 |
| 能现场跑通吗？ | 预构建失败时切 `reproduce/logs/` 与监控页；全套一键包在参考机实测 16/16、exit 0（2026-11-03 三轮）。 |

# 四元自主重组 预注册（2026-11-02，用户定向）

## 指令与分账

用户指令：让四元按照自己的规则进行重组、删除、修改，这是必须的、是基础。

分账：四个元系统（BlackHole/MeshBrain/MultiNova/BaseForce）原版只有
整份保存（keep）+ 各自的信号函数，从不对自己存的条目动手。本文件
定义的"每元用自己的信号函数决定对自家条目的操作"为**用户定向新增
的接线**，不是原版行为，不冒充原设计。上一轮已落的
stored_split/stored_join/stored_modify/stored_remove 是原语，本轮让
四元在周期内自主调用它们。

## 设计（每元只用自己的函数与自己的配置常数）

每元新增 `reorganize()`：对自家 stored 做一遍单趟维护，决策全部由
该元自己的信号函数对条目字节统计给出；操作经自家原语执行；返回
计数报告（splits/joins/removed/modified/bytes_before/bytes_after）。
统计量：条目字节均值 m；归一 x=m/255（字节原生量程，仓内
valhalla压缩 有 byte/255 先例）；内部分割点为条目自身的最大均值
移位点（与上一轮观察探针同统计，两侧各留 ≥8B）。

- BlackHole（GravityField）：距离=m（与其 keep 同尺）。
  删除：is_within_horizon(m) 的条目被事件视界吞没，stored_remove。
  重组：删除后引力最强（calculate_gravity 最大）的一条与相邻条
  塌缩 join，每趟至多一次。
- MeshBrain（MeshBrainSystem）：智能 s=mean*fluctuation_intensity
  （与其 generate_signal 的 intelligence_level 同式，不进
  signal_buffer——内部维护不冒充外部信号）。
  分解：条目内两半的 s 相差超过 fluctuation_intensity 比例带
  （|s1-s2| > 0.3*max）且长度 ≥32，在最大移位点 split。
  重组：相邻两条 s 落在同一涨落带内且合并后 ≤8192B（一次喂入
  单位上限），join。
- MultiNova（MultinovaProcessor）：process_evolution(x)，
  is_nova（x 超过自家 nova_threshold 0.8）的条目爆发：原地
  stored_modify 为前半段并插入后半段（记 modified+split，字节
  守恒）；非 nova 不动。
- BaseForce（BaseForceCoordinator）：协调去冗——与前面任一条
  逐字相同的重复条目 stored_remove，只留第一份。

## 接线（基础，不是探针）

Tile 的 process_tiles_cycle、Stem 的 process_cells_cycle 每周期
末各自调用一次 quad_reorganize()（四元各维护 Tile/Stem 里的自家
那份）。四份自此按各自规则演化，**入账仍是同文八份，周期后不再
保证同文**——这是本设计的直接后果，如实记账；上一轮的锁步原语
quad_split_entry/quad_join_entry 保留，其长度核对在分化后会正当
拒绝。Fate 不参与
判定（全局度量与观察层不否决身体动力学，与既有铁律一致）。

## 预先声明的观察判据（不许事后改口径）

喂混合料（事实条、文本片、图片块、静默块、逐字重复块、亮块）
后连跑周期，四元自家报告须显示各自规则在对应材料上触发：
BH 删除静默、BF 删除重复、MN 爆发亮条、MB 分解异质条；
每元 bytes_after == bytes_before − 被删字节（删除外字节守恒）；
连跑 10 周期后条目数变化收敛（末 3 周期每元净变化 ≤2 条，
不许无限切分或合并塌缩）；身体照常喂入与周期；
valhalla check 六件不许因本改动转红（存储记账数字变化允许，
须在 TEST_RUNS 换基线并注明机制原因，图结构数字——孵化 C/L、
指纹的 tiles/cells/边——应逐字不变，因为重组只动四元 stored
账本，不动细胞/Tile 本体；若图数字变了即是接线漏进了动力学，
按 bug 查）。

## 不做

不改四个系统的信号函数与配置常数；不让 Fate 审批四元操作；
不做跨元投票或共识合并；不追求任何"重组产生智能"的表述，
本轮只立基础机制并观察其守恒与收敛。

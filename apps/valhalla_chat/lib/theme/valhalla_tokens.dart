/// Valhalla Triad quad labels (Hub / Fate routing).
const kHubQuadLabels = [
  ('BlackHole', 'Fate 黑洞 · 收敛锚点', 0xFFEF4444),
  ('MeshBrain', 'MeshBrain · 网状推理', 0xFF06B6D4),
  ('MultiNova', 'MultiNova · 多源爆发', 0xFFF59E0B),
  ('BaseForce', 'BaseForce · 基态力场', 0xFF22C55E),
];

/// Component annotations shown in UI tooltips.
const kTriadAnnotations = {
  'hub': 'Hub (Fate 四元): 语料经 grinder ingress 后路由到四个 Fate 腔；hub_prefs 为偏好权重。',
  'tile': 'Tile: 块级神经签名 — completed_tiles 为孵化完成的结构块。',
  'stem': 'StemCell: 细胞图器官 — stem_cells 为被动 Fate 耦合的细胞簇。',
  'fate_tick': 'fate_signal_tick: Fate 逻辑时钟；主动性门控需 tick − baseline ≥ 64。',
  'proactive': 'proactive_idle: Fate 门控自探 — FOG 决定是否发出 nudge，MAR 保证不错域。',
  'multimodal': 'context_items: text · audio_features · image_features 并行 ingress。',
  'protocol': 'POST /v1/infer — 所有推理、session 续接、warmup、proactive 均走此端点。',
};

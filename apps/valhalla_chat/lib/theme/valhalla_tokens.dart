/// Valhalla Triad quad labels (Hub / Fate routing).
const kHubQuadLabels = [
  ('BlackHole', 'Fate black hole · convergence anchor', 0xFFEF4444),
  ('MeshBrain', 'MeshBrain · mesh reasoning', 0xFF06B6D4),
  ('MultiNova', 'MultiNova · multi-source burst', 0xFFF59E0B),
  ('BaseForce', 'BaseForce · baseline force field', 0xFF22C55E),
];

/// Component annotations shown in UI tooltips.
const kTriadAnnotations = {
  'hub':
      'Hub (Fate quad): corpus routes through grinder ingress into four Fate chambers; hub_prefs are routing weights.',
  'tile':
      'Tile: block-level neural signatures — completed_tiles are incubated structural blocks.',
  'stem':
      'StemCell: cellular organ graph — stem_cells are passively Fate-coupled cell clusters.',
  'fate_tick':
      'fate_signal_tick: Fate logical clock; proactive gating requires tick − baseline ≥ 64.',
  'proactive':
      'proactive_idle: Fate-gated self-probe — FOG decides nudge emission; MAR prevents cross-domain misfires.',
  'multimodal':
      'context_items: parallel ingress for text · audio_features · image_features.',
  'protocol':
      'POST /v1/infer — all inference, session continuation, warmup, and proactive calls use this endpoint.',
};

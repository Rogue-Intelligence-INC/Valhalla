import 'package:flutter/material.dart';

import '../models/app_models.dart';
import '../theme/valhalla_tokens.dart';

class AnnotatedSection extends StatelessWidget {
  const AnnotatedSection({
    super.key,
    required this.title,
    required this.annotationKey,
    required this.child,
    this.trailing,
  });

  final String title;
  final String annotationKey;
  final Widget child;
  final Widget? trailing;

  @override
  Widget build(BuildContext context) {
    final note = kTriadAnnotations[annotationKey] ?? '';
    return Card(
      margin: EdgeInsets.zero,
      child: Padding(
        padding: const EdgeInsets.all(14),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                Expanded(
                  child: Row(
                    children: [
                      Text(
                        title,
                        style: Theme.of(context).textTheme.titleSmall?.copyWith(
                              letterSpacing: 0.8,
                              fontWeight: FontWeight.w600,
                            ),
                      ),
                      const SizedBox(width: 6),
                      Tooltip(
                        message: note,
                        preferBelow: false,
                        child: Icon(
                          Icons.info_outline,
                          size: 16,
                          color: Theme.of(context).colorScheme.outline,
                        ),
                      ),
                    ],
                  ),
                ),
                if (trailing != null) trailing!,
              ],
            ),
            if (note.isNotEmpty)
              Padding(
                padding: const EdgeInsets.only(top: 4, bottom: 10),
                child: Text(
                  note,
                  style: Theme.of(context).textTheme.bodySmall?.copyWith(
                        color: Theme.of(context).colorScheme.onSurfaceVariant,
                        height: 1.35,
                      ),
                ),
              )
            else
              const SizedBox(height: 8),
            child,
          ],
        ),
      ),
    );
  }
}

class ModelStatusPanel extends StatelessWidget {
  const ModelStatusPanel({super.key, required this.state});

  final ModelState state;

  @override
  Widget build(BuildContext context) {
    return AnnotatedSection(
      title: 'MODEL STATE',
      annotationKey: 'protocol',
      child: Wrap(
        spacing: 8,
        runSpacing: 8,
        children: [
          _Chip(label: 'protocol', value: state.protocol),
          _Chip(label: 'decode', value: state.decode),
          _Chip(label: 'lines', value: '${state.linesIngested}'),
          _Chip(label: 'fragments', value: '${state.fragmentCount}'),
          _Chip(label: 'fate_tick', value: '${state.fateSignalTick}'),
          _Chip(
            label: 'Δsignals',
            value: '${state.fateSignalsSinceBaseline}',
          ),
          _Chip(label: 'hub_spread', value: state.hubSpread.toStringAsFixed(3)),
          _Chip(label: 'affinity', value: '${state.affinityNonzero}'),
        ],
      ),
    );
  }
}

class _Chip extends StatelessWidget {
  const _Chip({required this.label, required this.value});

  final String label;
  final String value;

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 6),
      decoration: BoxDecoration(
        color: Theme.of(context).colorScheme.surfaceContainerHighest,
        borderRadius: BorderRadius.circular(8),
        border: Border.all(
          color: Theme.of(context).colorScheme.outline.withOpacity(0.3),
        ),
      ),
      child: RichText(
        text: TextSpan(
          style: Theme.of(context).textTheme.labelSmall,
          children: [
            TextSpan(
              text: '$label · ',
              style: TextStyle(
                color: Theme.of(context).colorScheme.onSurfaceVariant,
              ),
            ),
            TextSpan(
              text: value,
              style: const TextStyle(fontFamily: 'monospace', fontSize: 11),
            ),
          ],
        ),
      ),
    );
  }
}

class HubQuadPanel extends StatelessWidget {
  const HubQuadPanel({super.key, required this.prefs, required this.routed});

  final List<double> prefs;
  final List<int> routed;

  @override
  Widget build(BuildContext context) {
    return AnnotatedSection(
      title: 'HUB · FATE QUAD',
      annotationKey: 'hub',
      child: Column(
        children: List.generate(4, (i) {
          final label = kHubQuadLabels[i];
          final p = i < prefs.length ? prefs[i] : 0.0;
          final r = i < routed.length ? routed[i] : 0;
          return Padding(
            padding: const EdgeInsets.only(bottom: 8),
            child: Row(
              children: [
                SizedBox(
                  width: 88,
                  child: Text(
                    label.$1,
                    style: TextStyle(
                      fontSize: 11,
                      color: Color(label.$3),
                      fontWeight: FontWeight.w600,
                    ),
                  ),
                ),
                Expanded(
                  child: ClipRRect(
                    borderRadius: BorderRadius.circular(4),
                    child: LinearProgressIndicator(
                      value: p.clamp(0.0, 1.0),
                      minHeight: 10,
                      backgroundColor:
                          Theme.of(context).colorScheme.surfaceContainerHighest,
                      color: Color(label.$3),
                    ),
                  ),
                ),
                const SizedBox(width: 8),
                SizedBox(
                  width: 72,
                  child: Text(
                    '${p.toStringAsFixed(2)} · $r',
                    style: const TextStyle(fontFamily: 'monospace', fontSize: 10),
                    textAlign: TextAlign.right,
                  ),
                ),
              ],
            ),
          );
        }),
      ),
    );
  }
}

class TileStemPanel extends StatelessWidget {
  const TileStemPanel({
    super.key,
    required this.modelState,
    this.dashboard,
  });

  final ModelState modelState;
  final DashboardViz? dashboard;

  @override
  Widget build(BuildContext context) {
    return Column(
      children: [
        AnnotatedSection(
          title: 'TILE',
          annotationKey: 'tile',
          child: Row(
            children: [
              _StatBox(
                label: 'completed',
                value: '${modelState.tileCount}',
                color: const Color(0xFF6366F1),
              ),
              const SizedBox(width: 8),
              _StatBox(
                label: 'ops',
                value: '${modelState.tileOps}',
                color: const Color(0xFF818CF8),
              ),
              const SizedBox(width: 8),
              _StatBox(
                label: 'neurons (smoke)',
                value: dashboard?.available == true
                    ? '${dashboard!.tileNeurons.length}'
                    : '—',
                color: const Color(0xFFA5B4FC),
              ),
            ],
          ),
        ),
        const SizedBox(height: 10),
        AnnotatedSection(
          title: 'STEM CELL',
          annotationKey: 'stem',
          child: Row(
            children: [
              _StatBox(
                label: 'cells',
                value: '${modelState.stemCells}',
                color: const Color(0xFF22C55E),
              ),
              const SizedBox(width: 8),
              _StatBox(
                label: 'organs (smoke)',
                value: dashboard?.available == true
                    ? '${dashboard!.stemOrgans.length}'
                    : '—',
                color: const Color(0xFF4ADE80),
              ),
              const SizedBox(width: 8),
              _StatBox(
                label: 'hub_signals',
                value: dashboard?.available == true
                    ? '${dashboard!.hubSignals}'
                    : '—',
                color: const Color(0xFF06B6D4),
              ),
            ],
          ),
        ),
      ],
    );
  }
}

class _StatBox extends StatelessWidget {
  const _StatBox({
    required this.label,
    required this.value,
    required this.color,
  });

  final String label;
  final String value;
  final Color color;

  @override
  Widget build(BuildContext context) {
    return Expanded(
      child: Container(
        padding: const EdgeInsets.all(10),
        decoration: BoxDecoration(
          color: Theme.of(context).colorScheme.surfaceContainerHighest,
          borderRadius: BorderRadius.circular(8),
          border: Border(top: BorderSide(color: color, width: 3)),
        ),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              label,
              style: Theme.of(context).textTheme.labelSmall?.copyWith(
                    color: Theme.of(context).colorScheme.onSurfaceVariant,
                  ),
            ),
            const SizedBox(height: 4),
            Text(
              value,
              style: Theme.of(context).textTheme.titleMedium?.copyWith(
                    fontWeight: FontWeight.bold,
                    fontFeatures: [const FontFeature.tabularFigures()],
                  ),
            ),
          ],
        ),
      ),
    );
  }
}

class ProactivePanel extends StatelessWidget {
  const ProactivePanel({
    super.key,
    required this.state,
    required this.modelState,
    required this.onToggle,
    required this.onAutoToggle,
    required this.autoAfterTurn,
    required this.onTrigger,
    required this.loading,
  });

  final ProactiveState state;
  final ModelState modelState;
  final ValueChanged<bool> onToggle;
  final ValueChanged<bool> onAutoToggle;
  final bool autoAfterTurn;
  final VoidCallback onTrigger;
  final bool loading;

  @override
  Widget build(BuildContext context) {
    final p = state.lastOutcome;
    final signals = p?['fate_signals_since_baseline'] as int? ??
        modelState.fateSignalsSinceBaseline;
    final minSig = (p?['proactive_min_fate_signals'] as int? ?? state.minFateSignals)
        .clamp(1, 1 << 30);
    final gatePct = (signals / minSig).clamp(0.0, 1.0);

    return AnnotatedSection(
      title: 'PROACTIVE AGENCY',
      annotationKey: 'proactive',
      trailing: Switch(
        value: state.enabled,
        onChanged: loading ? null : onToggle,
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Row(
            children: [
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(
                      'Fate signals $signals / $minSig',
                      style: Theme.of(context).textTheme.bodySmall,
                    ),
                    const SizedBox(height: 6),
                    ClipRRect(
                      borderRadius: BorderRadius.circular(4),
                      child: LinearProgressIndicator(
                        value: gatePct,
                        minHeight: 8,
                        color: signals >= minSig
                            ? const Color(0xFF7EE787)
                            : const Color(0xFFF0B429),
                      ),
                    ),
                  ],
                ),
              ),
            ],
          ),
          const SizedBox(height: 10),
          Wrap(
            spacing: 8,
            runSpacing: 8,
            children: [
              FilterChip(
                label: const Text('Auto after turn'),
                selected: autoAfterTurn,
                onSelected: loading ? null : onAutoToggle,
              ),
              FilledButton.tonalIcon(
                onPressed: loading || !state.enabled ? null : onTrigger,
                icon: const Icon(Icons.bolt, size: 18),
                label: const Text('Proactive probe'),
              ),
            ],
          ),
          if (p != null) ...[
            const SizedBox(height: 10),
            _Chip(
              label: 'fog',
              value: p['fog_decision']?.toString() ?? '—',
            ),
            const SizedBox(height: 6),
            _Chip(
              label: 'emitted',
              value: '${p['emitted'] == true}',
            ),
          ],
        ],
      ),
    );
  }
}

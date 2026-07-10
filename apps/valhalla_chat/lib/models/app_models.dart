import 'dart:convert';

class ProtocolLogEntry {
  ProtocolLogEntry({
    required this.id,
    required this.timestamp,
    required this.direction,
    required this.method,
    required this.path,
    required this.summary,
    this.body,
    this.statusCode,
    this.durationMs,
    this.error = false,
  });

  final String id;
  final DateTime timestamp;
  /// `out` = client → server, `in` = server → client
  final String direction;
  final String method;
  final String path;
  final String summary;
  final Map<String, dynamic>? body;
  final int? statusCode;
  final int? durationMs;
  final bool error;

  String get cliLine {
    if (direction == 'out' && method == 'POST') {
      final payload = body != null ? jsonEncode(body) : '{}';
      return 'curl -s -X POST \$BASE$path \\\n  -H "Content-Type: application/json" \\\n  -d \'${payload.length > 120 ? '${payload.substring(0, 120)}…' : payload}\'';
    }
    if (direction == 'out' && method == 'GET') {
      return 'curl -s \$BASE$path';
    }
    return summary;
  }

  String get prettyBody {
    if (body == null) return '';
    const encoder = JsonEncoder.withIndent('  ');
    return encoder.convert(body);
  }
}

class ModelState {
  ModelState({
    this.protocol = '—',
    this.decode = '—',
    this.contextLines = 0,
    this.fingerprint = '—',
    this.linesIngested = 0,
    this.fragmentCount = 0,
    this.fateSignalTick = 0,
    this.fateBaselineTick = 0,
    this.hubPrefs = const [0, 0, 0, 0],
    this.hubRouted = const [0, 0, 0, 0],
    this.hubSpread = 0,
    this.tileCount = 0,
    this.tileOps = 0,
    this.stemCells = 0,
    this.affinityNonzero = 0,
    this.affinityMax = 0,
    this.features = const [],
    this.gitSha = '',
    this.version = '',
  });

  factory ModelState.fromInferResponse(Map<String, dynamic> raw) {
    final session = raw['session'] as Map<String, dynamic>? ?? {};
    final snap = session['snapshot'] as Map<String, dynamic>? ?? {};
    final telemetry = raw['fate_telemetry'] as Map<String, dynamic>?;

    List<double> prefs = _asDoubleList(snap['hub_prefs'] ?? telemetry?['hub_prefs']);
    if (prefs.length < 4) {
      prefs = [...prefs, ...List.filled(4 - prefs.length, 0.0)];
    }

    List<int> routed = _asIntList(snap['hub_routed'] ?? telemetry?['hub_routed']);

    final tiles = snap['completed_tiles'] as List<dynamic>? ?? [];
    final stems = snap['stem_cells'] as List<dynamic>? ?? [];
    final frags = session['fragments'] as List<dynamic>? ?? [];

    double spread = (telemetry?['hub_prefs_spread'] as num?)?.toDouble() ?? 0;
    if (spread == 0 && prefs.isNotEmpty) {
      spread = prefs.reduce((a, b) => a > b ? a : b) -
          prefs.reduce((a, b) => a < b ? a : b);
    }

    return ModelState(
      protocol: raw['protocol'] as String? ?? 'valhalla-base-v1',
      decode: raw['decode']?.toString() ?? 'hybrid',
      contextLines: raw['context_lines'] as int? ?? 0,
      fingerprint: session['fingerprint'] as String? ??
          raw['context_fingerprint'] as String? ??
          '—',
      linesIngested: session['lines_ingested'] as int? ?? 0,
      fragmentCount: frags.length,
      fateSignalTick: snap['fate_signal_tick'] as int? ?? 0,
      fateBaselineTick: session['fate_baseline_tick'] as int? ?? 0,
      hubPrefs: prefs.take(4).toList(),
      hubRouted: routed.length >= 4 ? routed.take(4).toList() : [0, 0, 0, 0],
      hubSpread: spread,
      tileCount: tiles.length,
      tileOps: snap['tile_total_operations'] as int? ?? 0,
      stemCells: stems.length,
      affinityNonzero: telemetry?['corpus_line_affinity_nonzero'] as int? ?? 0,
      affinityMax: (telemetry?['corpus_line_affinity_max'] as num?)?.toDouble() ?? 0,
      features: (raw['features'] as List<dynamic>?)
              ?.map((e) => e.toString())
              .toList() ??
          [],
    );
  }

  static List<double> _asDoubleList(dynamic v) {
    if (v is! List) return [];
    return v
        .map((e) => e is num ? e.toDouble() : double.tryParse('$e') ?? 0.0)
        .toList();
  }

  static List<int> _asIntList(dynamic v) {
    if (v is! List) return [];
    return v.map((e) => e is num ? e.toInt() : int.tryParse('$e') ?? 0).toList();
  }

  int get fateSignalsSinceBaseline =>
      (fateSignalTick - fateBaselineTick).clamp(0, 1 << 62);

  final String protocol;
  final String decode;
  final int contextLines;
  final String fingerprint;
  final int linesIngested;
  final int fragmentCount;
  final int fateSignalTick;
  final int fateBaselineTick;
  final List<double> hubPrefs;
  final List<int> hubRouted;
  final double hubSpread;
  final int tileCount;
  final int tileOps;
  final int stemCells;
  final int affinityNonzero;
  final double affinityMax;
  final List<String> features;
  final String gitSha;
  final String version;
}

class ProactiveState {
  const ProactiveState({
    this.enabled = true,
    this.lastOutcome,
    this.minFateSignals = 64,
  });

  final bool enabled;
  final Map<String, dynamic>? lastOutcome;
  final int minFateSignals;

  bool get gateOpen {
    final o = lastOutcome;
    if (o == null) return false;
    return o['triggered'] == true;
  }

  bool get emitted => lastOutcome?['emitted'] == true;

  String get nudge => lastOutcome?['nudge'] as String? ?? '';

  String get fogDecision => lastOutcome?['fog_decision'] as String? ?? '—';

  ProactiveState copyWith({
    bool? enabled,
    Map<String, dynamic>? lastOutcome,
    int? minFateSignals,
  }) {
    return ProactiveState(
      enabled: enabled ?? this.enabled,
      lastOutcome: lastOutcome ?? this.lastOutcome,
      minFateSignals: minFateSignals ?? this.minFateSignals,
    );
  }
}

class DashboardViz {
  DashboardViz({
    this.available = false,
    this.tileNeurons = const [],
    this.stemOrgans = const [],
    this.hubSignals = 0,
    this.overallPass = false,
  });

  factory DashboardViz.fromJson(Map<String, dynamic>? doc) {
    if (doc == null) return DashboardViz();
    final report = doc['report'] as Map<String, dynamic>? ?? doc;
    final viz = report['viz'] as Map<String, dynamic>? ?? {};
    return DashboardViz(
      available: true,
      hubSignals: report['hub_signals'] as int? ?? 0,
      overallPass: report['overall_pass'] == true,
      tileNeurons: (viz['tile_neurons'] as List<dynamic>? ?? [])
          .cast<Map<String, dynamic>>(),
      stemOrgans: (viz['stem_organs'] as List<dynamic>? ?? [])
          .cast<Map<String, dynamic>>(),
    );
  }

  final bool available;
  final int hubSignals;
  final bool overallPass;
  final List<Map<String, dynamic>> tileNeurons;
  final List<Map<String, dynamic>> stemOrgans;
}

class MultimodalDraft {
  MultimodalDraft({
    this.textNote = '',
    this.audioFeatures = '',
    this.imageFeatures = '',
    this.enabled = false,
  });

  String textNote;
  String audioFeatures;
  String imageFeatures;
  bool enabled;

  List<double>? parseFeatures(String raw) {
    final t = raw.trim();
    if (t.isEmpty) return null;
    try {
      return t.split(',').map((s) => double.parse(s.trim())).toList();
    } catch (_) {
      return null;
    }
  }

  List<Map<String, dynamic>> toContextItems(String userText) {
    final items = <Map<String, dynamic>>[];
    if (textNote.trim().isNotEmpty) {
      items.add({'text': textNote.trim()});
    }
    final audio = parseFeatures(audioFeatures);
    if (audio != null && audio.isNotEmpty) {
      items.add({'audio_features': audio});
    }
    final image = parseFeatures(imageFeatures);
    if (image != null && image.isNotEmpty) {
      items.add({'image_features': image});
    }
    if (items.isEmpty && userText.isNotEmpty) {
      items.add({'text': userText});
    }
    return items;
  }
}

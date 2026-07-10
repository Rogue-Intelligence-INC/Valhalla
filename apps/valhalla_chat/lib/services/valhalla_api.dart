import 'dart:convert';

import 'package:http/http.dart' as http;

import '../models/app_models.dart';

/// HTTP client for ValhallaBase `valhalla-base-v1` — POST /v1/infer.
class ValhallaApi {
  ValhallaApi({
    required String baseUrl,
    http.Client? client,
    Duration timeout = const Duration(seconds: 300),
  })  : baseUrl = baseUrl,
        _client = client ?? http.Client(),
        _timeout = timeout;

  String baseUrl;
  final http.Client _client;
  final Duration _timeout;

  Uri _uri(String path) {
    final normalized = baseUrl.endsWith('/')
        ? baseUrl.substring(0, baseUrl.length - 1)
        : baseUrl;
    return Uri.parse('$normalized$path');
  }

  Map<String, dynamic> productionConfig({
    bool proactiveEnabled = true,
    bool coreFusion = false,
    int proactiveMinFateSignals = 64,
  }) =>
      {
        'decode': 'hybrid',
        'follow_up_decode': 'native_follow_up_aware',
        'fate_ingress_routing': 'quad_cycle',
        'fate_qa_feedback': true,
        'proactive_enabled': proactiveEnabled,
        'proactive_min_fate_signals': proactiveMinFateSignals,
        'core_fusion': coreFusion,
      };

  Future<HealthStatus> health() async {
    final t0 = DateTime.now();
    final resp = await _client.get(_uri('/health')).timeout(_timeout);
    if (resp.statusCode != 200) {
      throw ValhallaApiException(
        'health failed: HTTP ${resp.statusCode}',
        kind: 'http',
      );
    }
    final doc = jsonDecode(resp.body) as Map<String, dynamic>;
    return HealthStatus(
      ok: doc['ok'] == true,
      protocol: doc['protocol'] as String? ?? 'unknown',
      version: doc['version'] as String? ?? '',
      gitSha: doc['git_sha'] as String? ?? '',
      raw: doc,
      durationMs: DateTime.now().difference(t0).inMilliseconds,
    );
  }

  Future<DashboardViz> fetchDashboardState() async {
    try {
      final resp = await _client
          .get(_uri('/dashboard/api/state'))
          .timeout(const Duration(seconds: 8));
      if (resp.statusCode != 200) return DashboardViz();
      return DashboardViz.fromJson(
        jsonDecode(resp.body) as Map<String, dynamic>,
      );
    } catch (_) {
      return DashboardViz();
    }
  }

  Map<String, dynamic> buildInferBody({
    required String questionText,
    String questionId = 'q0',
    Map<String, dynamic>? session,
    List<String>? contextLines,
    List<String>? appendLines,
    List<Map<String, dynamic>>? contextItems,
    List<Map<String, dynamic>>? appendItems,
    Map<String, dynamic>? config,
    bool fateWarmupOnly = false,
    bool sessionExtendOnly = false,
    bool proactiveIdle = false,
    List<double>? questionAudioFeatures,
    List<double>? questionImageFeatures,
  }) {
    final questions = proactiveIdle || fateWarmupOnly || sessionExtendOnly
        ? <Map<String, dynamic>>[]
        : [
            {
              'id': questionId,
              'text': questionText,
              'score_type': 'open',
              if (questionAudioFeatures != null)
                'audio_features': questionAudioFeatures,
              if (questionImageFeatures != null)
                'image_features': questionImageFeatures,
            },
          ];

    final body = <String, dynamic>{
      'questions': questions,
      'config': config ?? productionConfig(),
    };
    if (session != null) body['session'] = session;
    if (contextLines != null && contextLines.isNotEmpty) {
      body['context'] = contextLines;
    }
    if (contextItems != null && contextItems.isNotEmpty) {
      body['context_items'] = contextItems;
    }
    if (appendLines != null && appendLines.isNotEmpty) {
      body['append_lines'] = appendLines;
    }
    if (appendItems != null && appendItems.isNotEmpty) {
      body['append_items'] = appendItems;
    }
    if (fateWarmupOnly) body['fate_warmup_only'] = true;
    if (sessionExtendOnly) body['session_extend_only'] = true;
    if (proactiveIdle) body['proactive_idle'] = true;
    return body;
  }

  Future<InferResult> infer(Map<String, dynamic> body) async {
    final t0 = DateTime.now();
    final resp = await _client
        .post(
          _uri('/v1/infer'),
          headers: {'Content-Type': 'application/json'},
          body: jsonEncode(body),
        )
        .timeout(_timeout);

    Map<String, dynamic> doc;
    try {
      doc = jsonDecode(resp.body) as Map<String, dynamic>;
    } catch (_) {
      throw ValhallaApiException(
        'invalid JSON: ${resp.body.substring(0, resp.body.length.clamp(0, 200))}',
        kind: 'parse',
      );
    }

    if (doc['ok'] == false || resp.statusCode >= 400) {
      throw ValhallaApiException(
        doc['error'] as String? ?? 'infer failed HTTP ${resp.statusCode}',
        kind: doc['error_kind'] as String? ?? 'runtime',
      );
    }

    final rows = doc['rows'] as List<dynamic>? ?? [];
    String answer = '';
    String? decodePath;
    if (rows.isNotEmpty) {
      final row = rows.first as Map<String, dynamic>;
      answer = row['answer'] as String? ?? '';
      decodePath = row['decode_path'] as String?;
    }

    final proactive = doc['proactive'] as Map<String, dynamic>?;
    if (proactive != null && proactive['nudge'] != null) {
      final nudge = proactive['nudge'] as String?;
      if (nudge != null && nudge.isNotEmpty && answer.isEmpty) {
        answer = nudge;
      }
    }

    return InferResult(
      answer: answer,
      decodePath: decodePath,
      session: doc['session'] as Map<String, dynamic>?,
      proactive: proactive,
      fateTelemetry: doc['fate_telemetry'] as Map<String, dynamic>?,
      raw: doc,
      requestBody: body,
      statusCode: resp.statusCode,
      durationMs: DateTime.now().difference(t0).inMilliseconds,
    );
  }

  Future<InferResult> ask({
    required String text,
    Map<String, dynamic>? session,
    List<String>? contextLines,
    List<String>? appendLines,
    List<Map<String, dynamic>>? contextItems,
    List<Map<String, dynamic>>? appendItems,
    String questionId = 'q0',
    Map<String, dynamic>? config,
    List<double>? questionAudioFeatures,
    List<double>? questionImageFeatures,
  }) {
    return infer(
      buildInferBody(
        questionText: text,
        questionId: questionId,
        session: session,
        contextLines: contextLines,
        appendLines: appendLines,
        contextItems: contextItems,
        appendItems: appendItems,
        config: config,
        questionAudioFeatures: questionAudioFeatures,
        questionImageFeatures: questionImageFeatures,
      ),
    );
  }

  Future<InferResult> proactiveIdle({
    required Map<String, dynamic> session,
    Map<String, dynamic>? config,
  }) {
    return infer(
      buildInferBody(
        questionText: '',
        session: session,
        proactiveIdle: true,
        config: config ?? productionConfig(proactiveEnabled: true),
      ),
    );
  }

  Future<Map<String, dynamic>?> warmupSession(
    List<String> contextLines, {
    Map<String, dynamic>? config,
  }) async {
    final result = await infer(
      buildInferBody(
        questionText: '',
        contextLines: contextLines,
        fateWarmupOnly: true,
        config: {
          ...?config,
          ...productionConfig(),
          'fate_warmup_enabled': true,
          'fate_warmup_cycles': 4,
        },
      ),
    );
    return result.session;
  }

  void dispose() => _client.close();
}

class HealthStatus {
  const HealthStatus({
    required this.ok,
    required this.protocol,
    required this.version,
    required this.gitSha,
    this.raw,
    this.durationMs,
  });

  final bool ok;
  final String protocol;
  final String version;
  final String gitSha;
  final Map<String, dynamic>? raw;
  final int? durationMs;
}

class InferResult {
  const InferResult({
    required this.answer,
    required this.session,
    required this.raw,
    this.decodePath,
    this.proactive,
    this.fateTelemetry,
    this.requestBody,
    this.statusCode,
    this.durationMs,
  });

  final String answer;
  final String? decodePath;
  final Map<String, dynamic>? session;
  final Map<String, dynamic>? proactive;
  final Map<String, dynamic>? fateTelemetry;
  final Map<String, dynamic> raw;
  final Map<String, dynamic>? requestBody;
  final int? statusCode;
  final int? durationMs;
}

class ValhallaApiException implements Exception {
  ValhallaApiException(this.message, {this.kind = 'unknown'});

  final String message;
  final String kind;

  @override
  String toString() => 'ValhallaApiException($kind): $message';
}

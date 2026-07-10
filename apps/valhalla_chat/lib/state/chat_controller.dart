import 'package:flutter/foundation.dart';
import 'package:uuid/uuid.dart';

import '../models/app_models.dart';
import '../models/chat_message.dart';
import '../services/valhalla_api.dart';

class ChatController extends ChangeNotifier {
  ChatController({required ValhallaApi api})
      : _api = api,
        baseUrl = api.baseUrl;

  final ValhallaApi _api;
  final _uuid = const Uuid();
  bool _disposed = false;

  void _safeNotify() {
    if (!_disposed) notifyListeners();
  }

  final List<ChatMessage> messages = [];
  final List<ProtocolLogEntry> protocolLog = [];
  Map<String, dynamic>? _session;
  List<String> _systemContext = [];

  bool loading = false;
  bool connected = false;
  String? connectionError;
  String baseUrl;

  ModelState modelState = ModelState();
  ProactiveState proactiveState = const ProactiveState();
  DashboardViz dashboardViz = DashboardViz();
  MultimodalDraft multimodal = MultimodalDraft();
  bool autoProactiveAfterTurn = true;
  bool coreFusion = false;
  bool showProtocolLog = true;

  ChatController.withBaseUrl(String url)
      : _api = ValhallaApi(baseUrl: url),
        baseUrl = url;

  ValhallaApi get api => _api;

  void setShowProtocolLog(bool v) {
    showProtocolLog = v;
    _safeNotify();
  }

  void setBaseUrl(String url) {
    baseUrl = url;
    _api.baseUrl = url.endsWith('/') ? url.substring(0, url.length - 1) : url;
    _safeNotify();
  }

  void setSystemContext(String text) {
    _systemContext = text
        .split('\n')
        .map((l) => l.trim())
        .where((l) => l.isNotEmpty)
        .toList();
    _safeNotify();
  }

  void setProactiveEnabled(bool v) {
    proactiveState = proactiveState.copyWith(enabled: v);
    _safeNotify();
  }

  void setAutoProactiveAfterTurn(bool v) {
    autoProactiveAfterTurn = v;
    _safeNotify();
  }

  void setCoreFusion(bool v) {
    coreFusion = v;
    _safeNotify();
  }

  void setMultimodalEnabled(bool v) {
    multimodal.enabled = v;
    _safeNotify();
  }

  void updateMultimodal(MultimodalDraft draft) {
    multimodal = draft;
    _safeNotify();
  }

  void clearProtocolLog() {
    protocolLog.clear();
    _safeNotify();
  }

  void _logOut(String path, Map<String, dynamic> body, String summary) {
    protocolLog.add(
      ProtocolLogEntry(
        id: _uuid.v4(),
        timestamp: DateTime.now(),
        direction: 'out',
        method: path.contains('infer') ? 'POST' : 'GET',
        path: path,
        summary: summary,
        body: body,
      ),
    );
  }

  void _logIn(
    String path,
    Map<String, dynamic>? body, {
    required String summary,
    int? statusCode,
    int? durationMs,
    bool error = false,
  }) {
    protocolLog.add(
      ProtocolLogEntry(
        id: _uuid.v4(),
        timestamp: DateTime.now(),
        direction: 'in',
        method: 'RESP',
        path: path,
        summary: summary,
        body: body,
        statusCode: statusCode,
        durationMs: durationMs,
        error: error,
      ),
    );
  }

  Map<String, dynamic> _config() => _api.productionConfig(
        proactiveEnabled: proactiveState.enabled,
        coreFusion: coreFusion,
        proactiveMinFateSignals: proactiveState.minFateSignals,
      );

  Future<void> checkConnection() async {
    try {
      _logOut('/health', {}, 'GET /health');
      final h = await _api.health();
      connected = h.ok;
      connectionError = null;
      modelState = ModelState(
        protocol: h.protocol,
        version: h.version,
        gitSha: h.gitSha,
      );
      _logIn('/health', h.raw, summary: 'ok=${h.ok} ${h.durationMs}ms');
      await _refreshDashboard();
    } on ValhallaApiException catch (e) {
      connected = false;
      connectionError = e.message;
      _logIn('/health', null, summary: e.message, error: true);
    } catch (e) {
      connected = false;
      connectionError = e.toString();
      _logIn('/health', null, summary: e.toString(), error: true);
    }
    _safeNotify();
  }

  Future<void> _refreshDashboard() async {
    dashboardViz = await _api.fetchDashboardState();
    _safeNotify();
  }

  void _applyInferResult(InferResult result) {
    _session = result.session;
    modelState = ModelState.fromInferResponse(result.raw);
    if (result.proactive != null) {
      proactiveState = proactiveState.copyWith(lastOutcome: result.proactive);
    }
  }

  Future<void> warmup() async {
    if (_systemContext.isEmpty) return;
    loading = true;
    _safeNotify();
    try {
      final body = _api.buildInferBody(
        questionText: '',
        contextLines: _systemContext,
        fateWarmupOnly: true,
        config: {
          ..._config(),
          'fate_warmup_enabled': true,
          'fate_warmup_cycles': 4,
        },
      );
      _logOut('/v1/infer', body, 'fate_warmup_only');
      final result = await _api.infer(body);
      _applyInferResult(result);
      _logIn('/v1/infer', result.raw,
          summary: 'warmup lines=${modelState.linesIngested}',
          statusCode: result.statusCode,
          durationMs: result.durationMs);
      _appendSystem(
        'Fate warmup · ${modelState.linesIngested} lines · tick=${modelState.fateSignalTick}',
      );
    } on ValhallaApiException catch (e) {
      _appendSystem('Warmup failed: ${e.message}', error: true);
      _logIn('/v1/infer', null, summary: e.message, error: true);
    } finally {
      loading = false;
      _safeNotify();
    }
  }

  Future<void> sendMessage(String text) async {
    final trimmed = text.trim();
    if (trimmed.isEmpty || loading) return;

    final modalities = <String>[];
    if (multimodal.enabled) {
      if (multimodal.textNote.trim().isNotEmpty) modalities.add('text');
      if (multimodal.parseFeatures(multimodal.audioFeatures) != null) {
        modalities.add('audio');
      }
      if (multimodal.parseFeatures(multimodal.imageFeatures) != null) {
        modalities.add('image');
      }
    }

    messages.add(
      ChatMessage(
        id: _uuid.v4(),
        role: ChatRole.user,
        text: trimmed,
        timestamp: DateTime.now(),
        hasMultimodal: modalities.isNotEmpty,
        modalities: modalities,
      ),
    );
    loading = true;
    _safeNotify();

    try {
      final isFirstTurn = _session == null;
      List<Map<String, dynamic>>? contextItems;
      List<Map<String, dynamic>>? appendItems;
      List<String>? contextLines;
      List<String>? appendLines;

      if (multimodal.enabled && modalities.isNotEmpty) {
        final items = multimodal.toContextItems(trimmed);
        if (isFirstTurn) {
          contextItems = items;
          if (_systemContext.isNotEmpty) {
            contextLines = _systemContext;
          }
        } else {
          appendItems = items;
        }
      } else {
        if (isFirstTurn && _systemContext.isNotEmpty) {
          contextLines = _systemContext;
        } else if (_session != null) {
          appendLines = [trimmed];
        }
      }

      final body = _api.buildInferBody(
        questionText: trimmed,
        questionId: 'q_${messages.length}',
        session: _session,
        contextLines: contextLines,
        appendLines: appendLines,
        contextItems: contextItems,
        appendItems: appendItems,
        config: _config(),
        questionAudioFeatures: multimodal.enabled
            ? multimodal.parseFeatures(multimodal.audioFeatures)
            : null,
        questionImageFeatures: multimodal.enabled
            ? multimodal.parseFeatures(multimodal.imageFeatures)
            : null,
      );
      _logOut('/v1/infer', body, 'ask: ${trimmed.length > 40 ? '${trimmed.substring(0, 40)}…' : trimmed}');

      final result = await _api.infer(body);
      _applyInferResult(result);
      _logIn(
        '/v1/infer',
        result.raw,
        summary: '${result.decodePath ?? "infer"} ${result.durationMs}ms',
        statusCode: result.statusCode,
        durationMs: result.durationMs,
      );

      messages.add(
        ChatMessage(
          id: _uuid.v4(),
          role: ChatRole.assistant,
          text: result.answer.isEmpty ? '(empty response)' : result.answer,
          timestamp: DateTime.now(),
          decodePath: result.decodePath,
          hasMultimodal: modalities.isNotEmpty,
          modalities: modalities,
        ),
      );
      connected = true;
      connectionError = null;

      if (autoProactiveAfterTurn &&
          proactiveState.enabled &&
          _session != null) {
        await _runProactiveInternal(silent: true);
      }
    } on ValhallaApiException catch (e) {
      messages.add(
        ChatMessage(
          id: _uuid.v4(),
          role: ChatRole.assistant,
          text: e.message,
          timestamp: DateTime.now(),
          error: true,
        ),
      );
      _logIn('/v1/infer', null, summary: e.message, error: true);
      connected = false;
      connectionError = e.message;
    } catch (e) {
      messages.add(
        ChatMessage(
          id: _uuid.v4(),
          role: ChatRole.assistant,
          text: e.toString(),
          timestamp: DateTime.now(),
          error: true,
        ),
      );
    } finally {
      loading = false;
      _safeNotify();
    }
  }

  Future<void> triggerProactive() async {
    if (_session == null) {
      _appendSystem('Proactive requires session — send a message or warmup first.',
          error: true);
      _safeNotify();
      return;
    }
    loading = true;
    _safeNotify();
    await _runProactiveInternal(silent: false);
    loading = false;
    _safeNotify();
  }

  Future<void> _runProactiveInternal({required bool silent}) async {
    if (_session == null || !proactiveState.enabled) return;
    try {
      final body = _api.buildInferBody(
        questionText: '',
        session: _session,
        proactiveIdle: true,
        config: _config(),
      );
      _logOut('/v1/infer', body, 'proactive_idle');
      final result = await _api.proactiveIdle(
        session: _session!,
        config: _config(),
      );
      _applyInferResult(result);
      final p = result.proactive ?? {};
      _logIn(
        '/v1/infer',
        result.raw,
        summary:
            'proactive emitted=${p['emitted']} fog=${p['fog_decision']}',
        statusCode: result.statusCode,
        durationMs: result.durationMs,
      );

      if (p['emitted'] == true || (p['nudge'] as String?)?.isNotEmpty == true) {
        messages.add(
          ChatMessage(
            id: _uuid.v4(),
            role: ChatRole.proactive,
            text: p['nudge'] as String? ??
                p['probe_question'] as String? ??
                'Proactive nudge',
            timestamp: DateTime.now(),
            isProactive: true,
            fogDecision: p['fog_decision'] as String?,
            decodePath: 'proactive_idle',
          ),
        );
      } else if (!silent) {
        _appendSystem(
          'Proactive gate: no nudge (fog=${p['fog_decision'] ?? "—"}, '
          'signals=${p['fate_signals_since_baseline'] ?? modelState.fateSignalsSinceBaseline})',
        );
      }
    } on ValhallaApiException catch (e) {
      if (!silent) {
        _appendSystem('Proactive failed: ${e.message}', error: true);
      }
      _logIn('/v1/infer', null, summary: e.message, error: true);
    }
  }

  void clearChat() {
    messages.clear();
    _session = null;
    modelState = ModelState(
      protocol: modelState.protocol,
      version: modelState.version,
      gitSha: modelState.gitSha,
    );
    proactiveState = proactiveState.copyWith(lastOutcome: null);
    _safeNotify();
  }

  void _appendSystem(String text, {bool error = false}) {
    messages.add(
      ChatMessage(
        id: _uuid.v4(),
        role: ChatRole.system,
        text: text,
        timestamp: DateTime.now(),
        error: error,
      ),
    );
  }

  @override
  void dispose() {
    _disposed = true;
    _api.dispose();
    super.dispose();
  }
}

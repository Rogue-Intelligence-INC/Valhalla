import 'dart:convert';

import 'package:flutter_test/flutter_test.dart';
import 'package:http/http.dart' as http;
import 'package:http/testing.dart';

import 'package:valhalla_chat/services/valhalla_api.dart';

void main() {
  group('ValhallaApi.buildInferBody', () {
    late ValhallaApi api;

    setUp(() {
      api = ValhallaApi(baseUrl: 'http://127.0.0.1:8787');
    });

    test('first turn includes context and question', () {
      final body = api.buildInferBody(
        questionText: 'What is the protocol?',
        contextLines: ['ValhallaBase primary protocol is valhalla-base-v1.'],
      );

      expect(body['context'], ['ValhallaBase primary protocol is valhalla-base-v1.']);
      expect(body['questions'], hasLength(1));
      expect(body['config']['proactive_enabled'], isTrue);
    });

    test('proactive_idle clears questions', () {
      final body = api.buildInferBody(
        questionText: '',
        session: {'corpus': []},
        proactiveIdle: true,
      );
      expect(body['proactive_idle'], isTrue);
      expect(body['questions'], isEmpty);
    });

    test('multimodal context_items', () {
      final body = api.buildInferBody(
        questionText: 'mm?',
        contextItems: [
          {'text': 'note'},
          {'audio_features': [0.1, 0.2]},
        ],
      );
      expect(body['context_items'], hasLength(2));
    });
  });

  group('ValhallaApi.infer', () {
    test('parses success with proactive', () async {
      final client = MockClient((_) async {
        return http.Response(
          jsonEncode({
            'session': {'lines_ingested': 1},
            'fate_telemetry': {'hub_prefs': [0.1, 0.2, 0.3, 0.4]},
            'rows': [{'answer': 'ok', 'decode_path': 'native_retrieval'}],
            'proactive': {'emitted': false, 'fog_decision': 'abstain'},
          }),
          200,
        );
      });

      final api = ValhallaApi(baseUrl: 'http://127.0.0.1:8787', client: client);
      final result = await api.ask(text: 'Hello');
      expect(result.answer, 'ok');
      expect(result.proactive?['fog_decision'], 'abstain');
    });
  });
}

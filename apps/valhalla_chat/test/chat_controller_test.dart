import 'dart:convert';

import 'package:flutter_test/flutter_test.dart';
import 'package:http/http.dart' as http;
import 'package:http/testing.dart';

import 'package:valhalla_chat/services/valhalla_api.dart';
import 'package:valhalla_chat/state/chat_controller.dart';

void main() {
  test('sendMessage round-trips session from infer response', () async {
    var call = 0;
    final client = MockClient((request) async {
      call += 1;
      if (call == 1) {
        return http.Response(
          jsonEncode({
            'session': {'corpus': ['ctx'], 'lines_ingested': 1},
            'rows': [
              {'answer': 'first answer', 'decode_path': 'native_retrieval'},
            ],
          }),
          200,
        );
      }
      final body = jsonDecode(request.body) as Map<String, dynamic>;
      expect(body.containsKey('session'), isTrue);
      expect(body['append_lines'], isNotNull);

      return http.Response(
        jsonEncode({
          'session': {'corpus': ['ctx', 'follow'], 'lines_ingested': 2},
          'rows': [
            {'answer': 'second answer', 'decode_path': 'hybrid'},
          ],
        }),
        200,
      );
    });

    final api = ValhallaApi(baseUrl: 'http://127.0.0.1:8787', client: client);
    final ctrl = ChatController(api: api);
    ctrl.setAutoProactiveAfterTurn(false);
    ctrl.setSystemContext('ctx line');

    await ctrl.sendMessage('first question');
    expect(ctrl.messages.last.text, 'first answer');

    await ctrl.sendMessage('second question');
    expect(ctrl.messages.last.text, 'second answer');
    expect(call, 2);
    ctrl.dispose();
  });

  test('dispose suppresses notifyListeners after teardown', () {
    final api = ValhallaApi(baseUrl: 'http://127.0.0.1:8787');
    final ctrl = ChatController(api: api);
    var notifications = 0;
    ctrl.addListener(() => notifications += 1);
    ctrl.dispose();
    ctrl.setProactiveEnabled(false);
    expect(notifications, 0);
  });
}

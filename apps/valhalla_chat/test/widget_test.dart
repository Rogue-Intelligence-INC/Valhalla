import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:valhalla_chat/models/chat_message.dart';
import 'package:valhalla_chat/widgets/message_bubble.dart';

void main() {
  testWidgets('MessageBubble renders user and assistant text', (tester) async {
    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(
          body: Column(
            children: [
              MessageBubble(
                message: ChatMessage(
                  id: '1',
                  role: ChatRole.user,
                  text: 'Hello Valhalla',
                  timestamp: DateTime.now(),
                ),
              ),
              MessageBubble(
                message: ChatMessage(
                  id: '2',
                  role: ChatRole.assistant,
                  text: 'valhalla-base-v1',
                  timestamp: DateTime.now(),
                  decodePath: 'native_retrieval',
                ),
              ),
            ],
          ),
        ),
      ),
    );

    expect(find.text('Hello Valhalla'), findsOneWidget);
    expect(find.text('valhalla-base-v1'), findsOneWidget);
    expect(find.text('native_retrieval'), findsOneWidget);
  });
}

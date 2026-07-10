import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'package:valhalla_chat/models/app_models.dart';
import 'package:valhalla_chat/widgets/triad_panels.dart';

void main() {
  testWidgets('ProactivePanel does not crash when minFateSignals is zero', (tester) async {
    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(
          body: ProactivePanel(
            state: const ProactiveState(minFateSignals: 0),
            modelState: ModelState(fateSignalTick: 0, fateBaselineTick: 0),
            autoAfterTurn: false,
            loading: false,
            onToggle: (_) {},
            onAutoToggle: (_) {},
            onTrigger: () {},
          ),
        ),
      ),
    );

    expect(find.textContaining('Fate signals'), findsOneWidget);
    expect(tester.takeException(), isNull);
  });
}

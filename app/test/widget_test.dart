// End-to-end widget smoke: the real DictaApp boots under a ProviderScope and
// renders its localised shell. No dart:io transport is reached — the engine
// seam is overridden with a fake.
import 'package:dicta_app/core/engine.dart';
import 'package:dicta_app/core/providers.dart';
import 'package:dicta_app/main.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';

import 'fake_dicta_engine.dart';

void main() {
  testWidgets('DictaApp boots and renders the first shell destination',
      (tester) async {
    await tester.pumpWidget(
      ProviderScope(
        overrides: [
          engineProvider.overrideWithValue(
            FakeDictaEngine(
              const DictaStatus(connected: true, state: DaemonState.idle),
            ),
          )
        ],
        child: const DictaApp(),
      ),
    );

    // Dashboard is the first NavigationRailDestination in shell.dart; the
    // label comes from app_ru.arb via the committed generated l10n sources.
    expect(find.text('Дашборд'), findsOneWidget);
  });
}

// HIST-02/HIST-03: journal browsing, substring search and the three row
// actions. Navigation taps the rail before asserts — offstage tabs are hidden
// from find.text by default (skipOffstage).
import 'dart:async';

import 'package:dicta_app/core/engine.dart';
import 'package:dicta_app/features/history/history.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:intl/intl.dart';

import 'fake_dicta_engine.dart';

Future<void> tick(WidgetTester tester) async {
  await tester.pump(const Duration(milliseconds: 100));
  await tester.pump();
}

Future<void> captureTextViaStop(WidgetTester tester) async {
  await tester.tap(find.byKey(const Key('dashStopButton')));
  await tester.pump();
  await tester.pump();
}

Future<void> goToHistory(WidgetTester tester) async {
  await tester.tap(find.text('История'));
  await tick(tester);
}

bool rowButtonEnabled(WidgetTester tester, String key) =>
    tester.widget<ElevatedButton>(find.byKey(Key(key))).onPressed != null;

void main() {
  testWidgets('captured dictation is listed with an HH:mm timestamp',
      (tester) async {
    final fake = FakeDictaEngine(
      const DictaStatus(connected: true, state: DaemonState.recording),
      stopReply: const DictaStatus(
        connected: true,
        state: DaemonState.idle,
        text: 'привет',
      ),
    );
    ProviderContainer? container;
    await tester.pumpWidget(
      buildFakeApp(fake, onCopy: (_) {}, captureContainer: (c) => container = c),
    );
    await tick(tester);
    await captureTextViaStop(tester);
    await goToHistory(tester);

    final at = container!.read(historyProvider).first.at;
    expect(find.text('привет'), findsOneWidget);
    expect(find.text(DateFormat.Hm('ru').format(at)), findsOneWidget);

    await tester.pumpWidget(const SizedBox.shrink());
  });

  testWidgets('search is case-insensitive substring, no debounce, empty = all',
      (tester) async {
    final fake = FakeDictaEngine(
      const DictaStatus(connected: true, state: DaemonState.recording),
      stopReply: const DictaStatus(
        connected: true,
        state: DaemonState.idle,
        text: 'мир',
      ),
    );
    await tester.pumpWidget(buildFakeApp(fake, onCopy: (_) {}));
    await tick(tester);
    await captureTextViaStop(tester);
    fake.stopReply = const DictaStatus(
      connected: true,
      state: DaemonState.idle,
      text: 'солнце',
    );
    await captureTextViaStop(tester);
    await goToHistory(tester);

    expect(find.text('мир'), findsOneWidget);
    expect(find.text('солнце'), findsOneWidget);

    await tester.enterText(find.byKey(const Key('historySearchField')), 'МИР');
    await tester.pump();
    expect(find.text('мир'), findsOneWidget);
    expect(find.text('солнце'), findsNothing);

    await tester.enterText(find.byKey(const Key('historySearchField')), 'луна');
    await tester.pump();
    expect(find.text('ничего не найдено'), findsOneWidget);

    await tester.enterText(find.byKey(const Key('historySearchField')), '');
    await tester.pump();
    expect(find.text('мир'), findsOneWidget);
    expect(find.text('солнце'), findsOneWidget);

    await tester.pumpWidget(const SizedBox.shrink());
  });

  testWidgets('copy uses the seam on explicit click and reports locally',
      (tester) async {
    final fake = FakeDictaEngine(
      const DictaStatus(connected: true, state: DaemonState.recording),
      stopReply: const DictaStatus(
        connected: true,
        state: DaemonState.idle,
        text: 'привет',
      ),
    );
    final copied = <String>[];
    await tester.pumpWidget(buildFakeApp(fake, onCopy: copied.add));
    await tick(tester);
    await captureTextViaStop(tester);
    await goToHistory(tester);

    await tester.tap(find.byKey(const Key('histCopyButton-1')));
    await tester.pump();
    await tester.pump();

    expect(copied, <String>['привет']);
    expect(find.text('скопировано'), findsOneWidget);

    await tester.pumpWidget(const SizedBox.shrink());
  });

  testWidgets('re-insert goes through inject and is gated while in flight',
      (tester) async {
    final gate = Completer<void>();
    final fake = FakeDictaEngine(
      const DictaStatus(connected: true, state: DaemonState.recording),
      stopReply: const DictaStatus(
        connected: true,
        state: DaemonState.idle,
        text: 'привет',
      ),
      injectGate: gate,
    );
    await tester.pumpWidget(buildFakeApp(fake, onCopy: (_) {}));
    await tick(tester);
    await captureTextViaStop(tester);
    await goToHistory(tester);

    expect(rowButtonEnabled(tester, 'histInjectButton-1'), isTrue);
    await tester.tap(find.byKey(const Key('histInjectButton-1')));
    await tester.pump();
    expect(fake.injectTexts, <String>['привет']);
    expect(rowButtonEnabled(tester, 'histInjectButton-1'), isFalse);

    gate.complete();
    await tester.pump();
    await tester.pump();
    expect(find.text('текст вставлен'), findsOneWidget);

    await tester.pumpWidget(const SizedBox.shrink());
  });

  testWidgets('delete removes the entry locally and reports it',
      (tester) async {
    final fake = FakeDictaEngine(
      const DictaStatus(connected: true, state: DaemonState.recording),
      stopReply: const DictaStatus(
        connected: true,
        state: DaemonState.idle,
        text: 'привет',
      ),
    );
    ProviderContainer? container;
    await tester.pumpWidget(
      buildFakeApp(fake, onCopy: (_) {}, captureContainer: (c) => container = c),
    );
    await tick(tester);
    await captureTextViaStop(tester);
    await goToHistory(tester);

    await tester.tap(find.byKey(const Key('histDeleteButton-1')));
    await tester.pump();
    await tester.pump();

    expect(container!.read(historyProvider), isEmpty);
    expect(find.text('привет'), findsNothing);
    expect(find.text('удалено'), findsOneWidget);
    expect(find.text('журнал пуст'), findsOneWidget);

    await tester.pumpWidget(const SizedBox.shrink());
  });

  testWidgets('empty journal states the honest A4 limitation', (tester) async {
    final fake = FakeDictaEngine(
      const DictaStatus(connected: true, state: DaemonState.idle),
    );
    await tester.pumpWidget(buildFakeApp(fake, onCopy: (_) {}));
    await tick(tester);
    await goToHistory(tester);

    expect(find.text('журнал пуст'), findsOneWidget);
    expect(
      find.text(
        'записи появляются после диктовок из приложения — глобальный хоткей демона приложению не виден',
      ),
      findsOneWidget,
    );

    await tester.pumpWidget(const SizedBox.shrink());
  });
}

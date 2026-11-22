// DASH-03: the card shows only text captured from our own action replies,
// copies through a seam on an explicit click, and re-injects exact text.
import 'dart:async';

import 'package:dicta_app/core/engine.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'fake_dicta_engine.dart';

Future<void> tick(WidgetTester tester) async {
  await tester.pump(const Duration(milliseconds: 100));
  await tester.pump();
}

bool enabled(WidgetTester tester, String key) =>
    tester.widget<ElevatedButton>(find.byKey(Key(key))).onPressed != null;

Future<void> captureTextViaStop(WidgetTester tester) async {
  await tester.tap(find.byKey(const Key('dashStopButton')));
  await tester.pump();
  await tester.pump();
}

void main() {
  testWidgets('honest empty state, then text from the stop reply',
      (tester) async {
    final fake = FakeDictaEngine(
      const DictaStatus(connected: true, state: DaemonState.idle),
    );
    await tester.pumpWidget(buildFakeApp(fake, onCopy: (_) {}));
    await tick(tester);

    expect(find.text('последний текст'), findsOneWidget);
    expect(find.text('нет распознанного текста'), findsOneWidget);
    expect(
      find.text('текст появится после остановки записи из приложения'),
      findsOneWidget,
    );
    expect(enabled(tester, 'dashCopyButton'), isFalse);
    expect(enabled(tester, 'dashInjectButton'), isFalse);

    fake.statusValue = const DictaStatus(
      connected: true,
      state: DaemonState.recording,
    );
    fake.stopReply = const DictaStatus(
      connected: true,
      state: DaemonState.idle,
      text: 'привет',
    );
    await tick(tester);
    await captureTextViaStop(tester);

    expect(find.text('привет'), findsOneWidget);
    expect(find.text('нет распознанного текста'), findsNothing);
    expect(enabled(tester, 'dashCopyButton'), isTrue);
    expect(enabled(tester, 'dashInjectButton'), isTrue);
    expect(find.text('вставка идёт в активное окно'), findsOneWidget);

    await tester.pumpWidget(const SizedBox.shrink());
  });

  testWidgets('copy calls the seam once and reports locally', (tester) async {
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

    await tester.tap(find.byKey(const Key('dashCopyButton')));
    await tester.pump();
    await tester.pump();

    expect(copied, <String>['привет']);
    expect(fake.injectTexts, isEmpty);
    expect(find.text('скопировано'), findsOneWidget);

    await tester.pumpWidget(const SizedBox.shrink());
  });

  testWidgets('inject sends exact text once, disables while in flight, '
      'shows the daemon error verbatim', (tester) async {
    final gate = Completer<void>();
    final fake = FakeDictaEngine(
      const DictaStatus(connected: true, state: DaemonState.recording),
      stopReply: const DictaStatus(
        connected: true,
        state: DaemonState.idle,
        text: 'привет',
      ),
      injectGate: gate,
      injectReply: const DictaStatus(
        connected: true,
        state: DaemonState.idle,
        error: 'вставка не удалась: доступ запрещён',
      ),
    );
    await tester.pumpWidget(buildFakeApp(fake, onCopy: (_) {}));
    await tick(tester);
    await captureTextViaStop(tester);

    await tester.tap(find.byKey(const Key('dashInjectButton')));
    await tester.pump();
    expect(fake.injectTexts, <String>['привет']);
    expect(enabled(tester, 'dashInjectButton'), isFalse);

    gate.complete();
    await tester.pump();
    await tester.pump();
    expect(find.text('ошибка вставки'), findsOneWidget);
    expect(find.text('вставка не удалась: доступ запрещён'), findsOneWidget);

    await tester.pumpWidget(const SizedBox.shrink());
  });

  testWidgets('successful inject confirms with a transient snackbar',
      (tester) async {
    final fake = FakeDictaEngine(
      const DictaStatus(connected: true, state: DaemonState.recording),
      stopReply: const DictaStatus(
        connected: true,
        state: DaemonState.idle,
        text: 'привет',
      ),
    );
    await tester.pumpWidget(buildFakeApp(fake, onCopy: (_) {}));
    await tick(tester);
    await captureTextViaStop(tester);

    expect(find.text('текст вставлен'), findsNothing);
    await tester.tap(find.byKey(const Key('dashInjectButton')));
    await tester.pump();
    await tester.pump();

    expect(fake.injectTexts, <String>['привет']);
    expect(find.text('текст вставлен'), findsOneWidget);
    expect(find.text('ошибка вставки'), findsNothing);

    await tester.pumpWidget(const SizedBox.shrink());
  });

  testWidgets('empty captured text keeps both actions disabled',
      (tester) async {
    final fake = FakeDictaEngine(
      const DictaStatus(connected: true, state: DaemonState.recording),
      stopReply: const DictaStatus(
        connected: true,
        state: DaemonState.idle,
        text: '',
      ),
    );
    await tester.pumpWidget(buildFakeApp(fake, onCopy: (_) {}));
    await tick(tester);
    await captureTextViaStop(tester);

    expect(find.text('нет распознанного текста'), findsOneWidget);
    expect(enabled(tester, 'dashCopyButton'), isFalse);
    expect(enabled(tester, 'dashInjectButton'), isFalse);
    expect(fake.injectTexts, isEmpty);

    await tester.pumpWidget(const SizedBox.shrink());
  });
}

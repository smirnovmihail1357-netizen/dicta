// HIST-01 tracer: the journal is fed ONLY by the app's stop seam.
// Inject echoes, failed stops and whitespace-only replies must not add entries.
import 'package:dicta_app/core/engine.dart';
import 'package:dicta_app/features/history/history.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';

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

void main() {
  testWidgets('a stop reply with text adds exactly one journal entry',
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
      buildFakeApp(fake, captureContainer: (c) => container = c),
    );
    await tick(tester);
    await captureTextViaStop(tester);

    final entries = container!.read(historyProvider);
    expect(entries, hasLength(1));
    expect(entries.first.text, 'привет');
    expect(entries.first.id, 1);
    expect(entries.first.at, isA<DateTime>());

    await tester.pumpWidget(const SizedBox.shrink());
  });

  testWidgets('inject echo does not create a second entry', (tester) async {
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
      buildFakeApp(fake, captureContainer: (c) => container = c),
    );
    await tick(tester);
    await captureTextViaStop(tester);

    await tester.tap(find.byKey(const Key('dashInjectButton')));
    await tester.pump();
    await tester.pump();

    expect(container!.read(historyProvider), hasLength(1));
    expect(fake.injectTexts, <String>['привет']);

    await tester.pumpWidget(const SizedBox.shrink());
  });

  testWidgets('failed and whitespace-only stops never enter the journal',
      (tester) async {
    final fake = FakeDictaEngine(
      const DictaStatus(connected: true, state: DaemonState.recording),
      stopReply: const DictaStatus(
        connected: true,
        state: DaemonState.idle,
        error: 'ошибка остановки',
      ),
    );
    ProviderContainer? container;
    await tester.pumpWidget(
      buildFakeApp(fake, captureContainer: (c) => container = c),
    );
    await tick(tester);
    await captureTextViaStop(tester);
    expect(container!.read(historyProvider), isEmpty);

    fake.stopReply = const DictaStatus(
      connected: true,
      state: DaemonState.idle,
      text: '   ',
    );
    await captureTextViaStop(tester);
    expect(container!.read(historyProvider), isEmpty);

    await tester.pumpWidget(const SizedBox.shrink());
  });

  testWidgets('two captures are ordered newest-first with monotonic ids',
      (tester) async {
    final fake = FakeDictaEngine(
      const DictaStatus(connected: true, state: DaemonState.recording),
      stopReply: const DictaStatus(
        connected: true,
        state: DaemonState.idle,
        text: 'первый',
      ),
    );
    ProviderContainer? container;
    await tester.pumpWidget(
      buildFakeApp(fake, captureContainer: (c) => container = c),
    );
    await tick(tester);
    await captureTextViaStop(tester);

    fake.stopReply = const DictaStatus(
      connected: true,
      state: DaemonState.idle,
      text: 'второй',
    );
    await captureTextViaStop(tester);

    final entries = container!.read(historyProvider);
    expect(entries, hasLength(2));
    expect(entries[0].text, 'второй');
    expect(entries[1].text, 'первый');
    expect(entries[0].id, 2);
    expect(entries[1].id, 1);
    expect(entries[0].id, greaterThan(entries[1].id));

    await tester.pumpWidget(const SizedBox.shrink());
  });
}

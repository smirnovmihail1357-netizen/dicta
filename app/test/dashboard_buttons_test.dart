// DASH-02: the button matrix is honest in every daemon state and the spinner
// follows the polled state, not the button's Future.
import 'dart:async';

import 'package:dicta_app/core/engine.dart';
import 'package:dicta_app/features/dashboard/dicta_control.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';

import 'fake_dicta_engine.dart';

Future<void> tick(WidgetTester tester) async {
  await tester.pump(const Duration(milliseconds: 100));
  await tester.pump();
}

// M3 hierarchy uses Elevated/Filled.tonal/Outlined: assert against their
// shared base so a button's visual variant does not break the matrix test.
bool enabled(WidgetTester tester, String key) =>
    tester.widget<ButtonStyleButton>(find.byKey(Key(key))).onPressed != null;

void main() {
  testWidgets('start/stop/cancel matrix across all four states', (tester) async {
    final fake = FakeDictaEngine(
      const DictaStatus(connected: true, state: DaemonState.idle),
    );
    await tester.pumpWidget(buildFakeApp(fake));
    await tick(tester);

    expect(enabled(tester, 'dashStartButton'), isTrue);
    expect(enabled(tester, 'dashStopButton'), isFalse);
    expect(enabled(tester, 'dashCancelButton'), isFalse);

    fake.statusValue = const DictaStatus(
      connected: true,
      state: DaemonState.recording,
    );
    await tick(tester);
    expect(enabled(tester, 'dashStartButton'), isFalse);
    expect(enabled(tester, 'dashStopButton'), isTrue);
    expect(enabled(tester, 'dashCancelButton'), isTrue);

    fake.statusValue = const DictaStatus(
      connected: true,
      state: DaemonState.transcribing,
    );
    await tick(tester);
    expect(enabled(tester, 'dashStartButton'), isFalse);
    expect(enabled(tester, 'dashStopButton'), isFalse);
    expect(enabled(tester, 'dashCancelButton'), isFalse);
    expect(find.text('распознаю…'), findsOneWidget);
    expect(find.byType(CircularProgressIndicator), findsOneWidget);

    fake.statusValue = const DictaStatus.disconnected();
    await tick(tester);
    expect(enabled(tester, 'dashStartButton'), isFalse);
    expect(enabled(tester, 'dashStopButton'), isFalse);
    expect(enabled(tester, 'dashCancelButton'), isFalse);

    await tester.pumpWidget(const SizedBox.shrink());
  });

  testWidgets('spinner dies on the next polled idle tick', (tester) async {
    final fake = FakeDictaEngine(
      const DictaStatus(connected: true, state: DaemonState.transcribing),
    );
    await tester.pumpWidget(buildFakeApp(fake));
    await tick(tester);
    expect(find.text('распознаю…'), findsOneWidget);

    fake.statusValue = const DictaStatus(
      connected: true,
      state: DaemonState.idle,
    );
    await tick(tester);
    expect(find.text('распознаю…'), findsNothing);
    expect(find.byType(CircularProgressIndicator), findsNothing);

    await tester.pumpWidget(const SizedBox.shrink());
  });

  testWidgets('in-flight stop disables its button until resolved',
      (tester) async {
    final gate = Completer<void>();
    final fake = FakeDictaEngine(
      const DictaStatus(connected: true, state: DaemonState.recording),
      stopGate: gate,
    );
    await tester.pumpWidget(buildFakeApp(fake));
    await tick(tester);
    expect(enabled(tester, 'dashStopButton'), isTrue);

    await tester.tap(find.byKey(const Key('dashStopButton')));
    await tester.pump();
    expect(enabled(tester, 'dashStopButton'), isFalse);

    gate.complete();
    await tester.pump();
    await tester.pump();
    expect(enabled(tester, 'dashStopButton'), isTrue);

    await tester.pumpWidget(const SizedBox.shrink());
  });

  testWidgets('cancel notice is suppressed, other cancel errors are shown '
      'under a neutral heading (WR-01/WR-04)', (tester) async {
    ProviderContainer? container;
    final noticeFake = FakeDictaEngine(
      const DictaStatus(connected: true, state: DaemonState.recording),
      // Byte-identical to crates/dictad/src/main.rs:947.
      cancelReply: const DictaReply(ok: false, error: 'запись отменена'),
    );
    await tester.pumpWidget(buildFakeApp(noticeFake,
        captureContainer: (c) => container = c));
    await tick(tester);

    await tester.tap(find.byKey(const Key('dashCancelButton')));
    await tester.pump();
    await tester.pump();

    expect(container!.read(dictaControlProvider).lastError, isNull);
    expect(find.text('запись отменена'), findsNothing);
    expect(find.text('ошибка команды'), findsNothing);

    await tester.pumpWidget(const SizedBox.shrink());

    final failureFake = FakeDictaEngine(
      const DictaStatus(connected: true, state: DaemonState.recording),
      cancelReply: const DictaReply(ok: false, error: 'демон не отвечает'),
    );
    await tester.pumpWidget(buildFakeApp(failureFake,
        captureContainer: (c) => container = c));
    await tick(tester);

    await tester.tap(find.byKey(const Key('dashCancelButton')));
    await tester.pump();
    await tester.pump();

    expect(container!.read(dictaControlProvider).lastError, 'демон не отвечает');
    expect(find.text('демон не отвечает'), findsOneWidget);
    expect(find.text('ошибка команды'), findsOneWidget);
    expect(find.text('ошибка вставки'), findsNothing);

    await tester.pumpWidget(const SizedBox.shrink());
  });

  testWidgets('stop reply text is captured; start clears it', (tester) async {
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
        buildFakeApp(fake, captureContainer: (c) => container = c));
    await tick(tester);

    await tester.tap(find.byKey(const Key('dashStopButton')));
    await tester.pump();
    await tester.pump();
    expect(container!.read(dictaControlProvider).lastText, 'привет');

    fake.statusValue = const DictaStatus(
      connected: true,
      state: DaemonState.idle,
    );
    await tick(tester);
    await tester.tap(find.byKey(const Key('dashStartButton')));
    await tester.pump();
    await tester.pump();
    expect(container!.read(dictaControlProvider).lastText, isNull);

    await tester.pumpWidget(const SizedBox.shrink());
  });
}

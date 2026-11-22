// DASH-01: the REAL DictaStatusNotifier poller drives the dashboard through
// 100 ms ticks. Only the engine seam is overridden; no provider state is
// injected directly.
import 'package:dicta_app/core/engine.dart';
import 'package:dicta_app/features/dashboard/level_history.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';

import 'fake_dicta_engine.dart';

Future<void> tick(WidgetTester tester) async {
  await tester.pump(const Duration(milliseconds: 100));
  await tester.pump();
}

void main() {
  testWidgets('poller drives state label, live level and mm:ss', (tester) async {
    final fake = FakeDictaEngine(
      const DictaStatus(connected: true, state: DaemonState.idle),
    );
    await tester.pumpWidget(buildFakeApp(fake));
    await tick(tester);
    expect(find.text('состояние: ожидание'), findsOneWidget);

    fake.statusValue = const DictaStatus(
      connected: true,
      state: DaemonState.recording,
      level: 0.42,
      seconds: 65.3,
    );
    await tick(tester);
    expect(find.text('состояние: запись'), findsOneWidget);
    expect(
      tester
          .widget<LinearProgressIndicator>(find.byType(LinearProgressIndicator))
          .value,
      0.42,
    );
    expect(find.text('время 01:05'), findsOneWidget);

    fake.statusValue = const DictaStatus(
      connected: true,
      state: DaemonState.transcribing,
    );
    await tick(tester);
    expect(find.text('состояние: распознавание'), findsOneWidget);

    await tester.pumpWidget(const SizedBox.shrink());
  });

  testWidgets('only recording ticks feed the level history', (tester) async {
    final fake = FakeDictaEngine(
      const DictaStatus(connected: true, state: DaemonState.idle),
    );
    ProviderContainer? container;
    await tester.pumpWidget(
        buildFakeApp(fake, captureContainer: (c) => container = c));
    await tick(tester);
    expect(container!.read(levelHistoryProvider), isEmpty);
    final captured = container!;

    fake.statusValue = const DictaStatus(
      connected: true,
      state: DaemonState.recording,
      level: 0.2,
    );
    await tick(tester);
    fake.statusValue = const DictaStatus(
      connected: true,
      state: DaemonState.recording,
      level: 0.6,
    );
    await tick(tester);
    expect(captured.read(levelHistoryProvider), <double>[0.2, 0.6]);

    fake.statusValue = const DictaStatus(
      connected: true,
      state: DaemonState.transcribing,
    );
    await tick(tester);
    expect(captured.read(levelHistoryProvider), <double>[0.2, 0.6]);

    fake.statusValue = const DictaStatus(
      connected: true,
      state: DaemonState.recording,
      level: 0.9,
    );
    await tick(tester);
    expect(captured.read(levelHistoryProvider), <double>[0.9]);

    await tester.pumpWidget(const SizedBox.shrink());
  });

  testWidgets('a throwing status() disconnects, backs off, then recovers '
      '(WR-04)', (tester) async {
    final fake = FakeDictaEngine(
      const DictaStatus(connected: true, state: DaemonState.idle),
    );
    await tester.pumpWidget(buildFakeApp(fake));
    await tick(tester);
    expect(find.text('демон недоступен'), findsNothing);

    fake.statusError = StateError('socket gone');
    await tester.pump(const Duration(milliseconds: 100));
    await tester.pump();
    expect(find.text('демон недоступен'), findsOneWidget);
    final attemptsAtFailure = fake.statusCount;
    expect(attemptsAtFailure, greaterThan(0));

    // The 200 ms backoff holds the immediately following tick: no retry storm.
    await tester.pump(const Duration(milliseconds: 50));
    await tester.pump();
    expect(fake.statusCount, attemptsAtFailure);

    // Let the backoff elapse, clear the fault, and require a successful retry.
    await tester.pump(const Duration(milliseconds: 200));
    await tester.pump();
    fake.statusError = null;
    await tester.pump(const Duration(milliseconds: 200));
    await tester.pump();
    expect(find.text('демон недоступен'), findsNothing);
    expect(fake.statusCount, greaterThan(attemptsAtFailure));

    await tester.pumpWidget(const SizedBox.shrink());
  });

  testWidgets('unknown state string renders the honest idle label',
      (tester) async {
    final fake = FakeDictaEngine(
      DictaStatus.fromReplyLine(
        '{"state":"paused","level":0.0,"seconds":0.0,"text":null,"error":null}',
      ),
    );
    await tester.pumpWidget(buildFakeApp(fake));
    await tick(tester);
    expect(find.text('состояние: ожидание'), findsOneWidget);

    await tester.pumpWidget(const SizedBox.shrink());
  });
}

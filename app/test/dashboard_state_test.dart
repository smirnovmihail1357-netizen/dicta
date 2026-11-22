// DASH-01: the REAL DictaStatusNotifier poller drives the dashboard through
// 100 ms ticks. Only the engine seam is overridden; no provider state is
// injected directly.
import 'package:dicta_app/core/engine.dart';
import 'package:dicta_app/core/providers.dart';
import 'package:dicta_app/features/dashboard/level_history.dart';
import 'package:dicta_app/features/shell/shell.dart';
import 'package:dicta_app/l10n/generated/app_localizations.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';

import 'fake_dicta_engine.dart';

Widget buildStateTestApp(
  FakeDictaEngine fake,
  void Function(ProviderContainer container) captureContainer,
) {
  return ProviderScope(
    overrides: [engineProvider.overrideWithValue(fake)],
    child: Builder(
      builder: (context) {
        captureContainer(ProviderScope.containerOf(context));
        return MaterialApp(
          localizationsDelegates: AppLocalizations.localizationsDelegates,
          supportedLocales: AppLocalizations.supportedLocales,
          locale: const Locale('ru'),
          home: const ShellWidget(),
        );
      },
    ),
  );
}

Future<void> tick(WidgetTester tester) async {
  await tester.pump(const Duration(milliseconds: 100));
  await tester.pump();
}

void main() {
  testWidgets('poller drives state label, live level and mm:ss', (tester) async {
    final fake = FakeDictaEngine(
      const DictaStatus(connected: true, state: DaemonState.idle),
    );
    await tester.pumpWidget(buildStateTestApp(fake, (_) {}));
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
    await tester.pumpWidget(buildStateTestApp(fake, (c) => container = c));
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

  testWidgets('unknown state string renders the honest idle label',
      (tester) async {
    final fake = FakeDictaEngine(
      DictaStatus.fromReplyLine(
        '{"state":"paused","level":0.0,"seconds":0.0,"text":null,"error":null}',
      ),
    );
    await tester.pumpWidget(buildStateTestApp(fake, (_) {}));
    await tick(tester);
    expect(find.text('состояние: ожидание'), findsOneWidget);

    await tester.pumpWidget(const SizedBox.shrink());
  });
}

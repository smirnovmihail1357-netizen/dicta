// DASH-02: the button matrix is honest in every daemon state and the spinner
// follows the polled state, not the button's Future.
import 'dart:async';

import 'package:dicta_app/core/engine.dart';
import 'package:dicta_app/core/providers.dart';
import 'package:dicta_app/features/dashboard/dicta_control.dart';
import 'package:dicta_app/features/shell/shell.dart';
import 'package:dicta_app/l10n/generated/app_localizations.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';

import 'fake_dicta_engine.dart';

Widget buildButtonsTestApp(
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

bool enabled(WidgetTester tester, String key) =>
    tester.widget<ElevatedButton>(find.byKey(Key(key))).onPressed != null;

void main() {
  testWidgets('start/stop/cancel matrix across all four states', (tester) async {
    final fake = FakeDictaEngine(
      const DictaStatus(connected: true, state: DaemonState.idle),
    );
    await tester.pumpWidget(buildButtonsTestApp(fake, (_) {}));
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
    await tester.pumpWidget(buildButtonsTestApp(fake, (_) {}));
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
    await tester.pumpWidget(buildButtonsTestApp(fake, (_) {}));
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
    await tester.pumpWidget(buildButtonsTestApp(fake, (c) => container = c));
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

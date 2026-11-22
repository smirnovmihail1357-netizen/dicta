// DIAG-01: automatic rows come from status/settings only; hotkey and paste
// rows are honest manual hints that never send a command.
import 'dart:convert';
import 'dart:io';

import 'package:dicta_app/core/engine.dart';
import 'package:dicta_app/core/providers.dart';
import 'package:dicta_app/features/shell/shell.dart';
import 'package:dicta_app/l10n/generated/app_localizations.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';

import 'fake_dicta_engine.dart';

Widget buildDiagnosticsApp(FakeDictaEngine fake) {
  return ProviderScope(
    overrides: [engineProvider.overrideWithValue(fake)],
    child: MaterialApp(
      localizationsDelegates: AppLocalizations.localizationsDelegates,
      supportedLocales: AppLocalizations.supportedLocales,
      locale: const Locale('ru'),
      home: const ShellWidget(),
    ),
  );
}

Future<void> settle(WidgetTester tester) async {
  await tester.pump(const Duration(milliseconds: 100));
  await tester.pump();
  await tester.pump();
}

void main() {
  late Map<String, dynamic> fixture;

  setUpAll(() {
    final raw = File('test/fixtures/settings_reply.json').readAsStringSync();
    fixture = jsonDecode(raw) as Map<String, dynamic>;
  });

  testWidgets('connected and model ready: three green automatic rows',
      (tester) async {
    final fake = FakeDictaEngine(
      const DictaStatus(connected: true, state: DaemonState.idle),
      fetchSettingsReply: fixture,
    );
    await tester.pumpWidget(buildDiagnosticsApp(fake));
    await settle(tester);

    expect(find.text('диагностика'), findsOneWidget);
    expect(find.text('демон жив'), findsOneWidget);
    expect(find.text('сокет отвечает'), findsOneWidget);
    expect(find.text('модель готова'), findsOneWidget);
    expect(find.byIcon(Icons.check_circle), findsNWidgets(3));
    expect(find.byIcon(Icons.error), findsNothing);
    expect(find.byIcon(Icons.info_outline), findsNWidgets(2));
    expect(find.text('запусти: systemctl --user start dictad'), findsNothing);
    expect(find.text('хоткей — проверь вручную: dictactl toggle'),
        findsOneWidget);
    expect(find.text('вставку проверь живой диктовкой'), findsOneWidget);
    expect(fake.setCalls, isEmpty);
    expect(fake.injectTexts, isEmpty);

    await tester.pumpWidget(const SizedBox.shrink());
  });

  testWidgets('reloading is a neutral state, not an error', (tester) async {
    final settings = fixture['settings'] as Map<String, dynamic>;
    final reloading = <String, dynamic>{
      ...fixture,
      'settings': <String, dynamic>{...settings, 'reloading': true},
    };
    final fake = FakeDictaEngine(
      const DictaStatus(connected: true, state: DaemonState.idle),
      fetchSettingsReply: reloading,
    );
    await tester.pumpWidget(buildDiagnosticsApp(fake));
    await settle(tester);

    expect(find.text('модель загружается…'), findsOneWidget);
    expect(find.text('модель готова'), findsNothing);
    expect(find.byIcon(Icons.error), findsNothing);
    expect(find.byIcon(Icons.check_circle), findsNWidgets(2));

    await tester.pumpWidget(const SizedBox.shrink());
  });

  testWidgets('disconnected: red rows and the daemon start hint',
      (tester) async {
    final fake = FakeDictaEngine(
      const DictaStatus.disconnected(),
      fetchSettingsReply: fixture,
    );
    await tester.pumpWidget(buildDiagnosticsApp(fake));
    await settle(tester);

    expect(find.text('запусти: systemctl --user start dictad'), findsOneWidget);
    expect(find.byIcon(Icons.error), findsNWidgets(3));
    expect(find.byIcon(Icons.check_circle), findsNothing);
    expect(fake.setCalls, isEmpty);
    expect(fake.injectTexts, isEmpty);

    await tester.pumpWidget(const SizedBox.shrink());
  });

  testWidgets('recheck performs one poll and one settings refetch',
      (tester) async {
    final fake = FakeDictaEngine(
      const DictaStatus(connected: true, state: DaemonState.idle),
      fetchSettingsReply: fixture,
    );
    await tester.pumpWidget(buildDiagnosticsApp(fake));
    await settle(tester);

    await tester.ensureVisible(find.byKey(const Key('diagRecheckButton')));
    await tester.pump();
    final fetchesBefore = fake.fetchCount;
    final statusesBefore = fake.statusCount;
    await tester.tap(find.byKey(const Key('diagRecheckButton')));
    await tester.pump();

    expect(fake.fetchCount, fetchesBefore + 1);
    expect(fake.statusCount, statusesBefore + 1);
    expect(fake.setCalls, isEmpty);
    expect(fake.injectTexts, isEmpty);

    await tester.pumpWidget(const SizedBox.shrink());
  });
}

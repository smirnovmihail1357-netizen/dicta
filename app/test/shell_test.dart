// APP-03 proof: the shell is driven only through the DictaEngine seam.
// No dart:io import and no DaemonEngine reference anywhere in this file.
import 'package:dicta_app/core/engine.dart';
import 'package:dicta_app/core/providers.dart';
import 'package:dicta_app/features/shell/shell.dart';
import 'package:dicta_app/l10n/app_localizations.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';

class FakeDictaEngine implements DictaEngine {
  FakeDictaEngine(this.snapshot);

  final DictaStatus snapshot;

  @override
  Future<DictaStatus> status() async => snapshot;

  @override
  Future<DictaReply> start({String? app}) async => const DictaReply(ok: true);

  @override
  Future<DictaReply> stop({String? app}) async => const DictaReply(ok: true);

  @override
  Future<DictaReply> toggle({String? app}) async => const DictaReply(ok: true);

  @override
  Future<DictaReply> cancel() async => const DictaReply(ok: true);

  @override
  Future<Map<String, dynamic>> fetchSettings() async => <String, dynamic>{};

  @override
  Future<DictaReply> set(String key, String value) async =>
      const DictaReply(ok: true);

  @override
  Stream<DictaStatus> get statusStream => Stream<DictaStatus>.empty();
}

Widget buildTestApp(FakeDictaEngine fake) {
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

void main() {
  testWidgets('four Russian destinations render; no banner while connected',
      (tester) async {
    await tester.pumpWidget(buildTestApp(FakeDictaEngine(
      const DictaStatus(connected: true, state: DaemonState.idle),
    )));
    await tester.pump(const Duration(milliseconds: 300));

    expect(find.text('Дашборд'), findsOneWidget);
    expect(find.text('История'), findsOneWidget);
    expect(find.text('Модели'), findsOneWidget);
    expect(find.text('Настройки'), findsOneWidget);
    expect(find.text('демон недоступен'), findsNothing);

    await tester.pumpWidget(const SizedBox.shrink());
  });

  testWidgets('daemon-unavailable banner appears on a disconnected snapshot',
      (tester) async {
    await tester.pumpWidget(
      buildTestApp(FakeDictaEngine(const DictaStatus.disconnected())),
    );
    await tester.pump(const Duration(milliseconds: 300));

    expect(find.text('демон недоступен'), findsOneWidget);

    await tester.pumpWidget(const SizedBox.shrink());
  });
}

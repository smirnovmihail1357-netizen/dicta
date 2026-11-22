// End-to-end widget smoke: the real DictaApp boots under a ProviderScope and
// renders its localised shell. No dart:io transport is reached — the engine
// seam is overridden with a fake.
import 'package:dicta_app/core/engine.dart';
import 'package:dicta_app/core/providers.dart';
import 'package:dicta_app/main.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';

class _FakeEngine implements DictaEngine {
  @override
  Future<DictaStatus> status() async =>
      const DictaStatus(connected: true, state: DaemonState.idle);

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

void main() {
  testWidgets('DictaApp boots and renders the first shell destination',
      (tester) async {
    await tester.pumpWidget(
      ProviderScope(
        overrides: [engineProvider.overrideWithValue(_FakeEngine())],
        child: const DictaApp(),
      ),
    );

    // Dashboard is the first NavigationRailDestination in shell.dart; the
    // label comes from app_ru.arb via the committed generated l10n sources.
    expect(find.text('Дашборд'), findsOneWidget);
  });
}

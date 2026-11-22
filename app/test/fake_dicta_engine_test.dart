// Contract test for the shared fake factory: every scenario hook declared for
// Tasks 2-6 must behave here so the scenario tests never clone the fake.
import 'dart:async';

import 'package:dicta_app/core/engine.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'fake_dicta_engine.dart';

void main() {
  test('status() returns the initial snapshot and reflects mutation', () async {
    final fake = FakeDictaEngine(
      const DictaStatus(connected: true, state: DaemonState.idle),
    );
    expect((await fake.status()).state, DaemonState.idle);

    fake.statusValue = const DictaStatus(
      connected: true,
      state: DaemonState.recording,
      level: 0.5,
      seconds: 12.0,
    );
    final next = await fake.status();
    expect(next.state, DaemonState.recording);
    expect(next.level, 0.5);
  });

  test('set() records exactly one (key, value) pair per call', () async {
    final fake = FakeDictaEngine(
      const DictaStatus(connected: true, state: DaemonState.idle),
    );
    expect(fake.setCalls, isEmpty);

    await fake.set('min_level', '0.42');

    expect(fake.setCalls.length, 1);
    expect(fake.setCalls.single.key, 'min_level');
    expect(fake.setCalls.single.value, '0.42');
  });

  test('fetchSettings() returns the configured reply and counts calls',
      () async {
    final settings = <String, dynamic>{
      'min_level': 0.35,
      'auto_stop': true,
    };
    final fake = FakeDictaEngine(
      const DictaStatus(connected: true, state: DaemonState.idle),
      fetchSettingsReply: settings,
    );
    expect(fake.fetchCount, 0);

    final first = await fake.fetchSettings();
    final second = await fake.fetchSettings();

    expect(first, same(settings));
    expect(second, same(settings));
    expect(fake.fetchCount, 2);
  });

  test('stop() stays in flight until stopGate is resolved', () async {
    final gate = Completer<void>();
    final fake = FakeDictaEngine(
      const DictaStatus(connected: true, state: DaemonState.recording),
      stopGate: gate,
    );

    var done = false;
    final pending = fake.stop().then((_) => done = true);
    await Future<void>.delayed(Duration.zero);
    expect(done, isFalse);

    gate.complete();
    await pending;
    expect(done, isTrue);
  });

  testWidgets('the shared fake drives the real shell widget', (tester) async {
    final fake = FakeDictaEngine(
      const DictaStatus(connected: true, state: DaemonState.idle),
    );
    await tester.pumpWidget(buildFakeApp(fake));
    await tester.pump(const Duration(milliseconds: 300));

    expect(find.text('Дашборд'), findsOneWidget);
    expect(find.text('демон недоступен'), findsNothing);

    await tester.pumpWidget(const SizedBox.shrink());
  });
}

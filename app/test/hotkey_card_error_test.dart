// IN-02 residual: hotkeyConfigTextProvider promotes a protocol error to an
// AsyncError, keeps the file fallback for transport failures, and the poll
// branch reports the disconnect cause (error runtimeType only, no payload).
import 'dart:io';

import 'package:dicta_app/core/engine.dart';
import 'package:dicta_app/core/providers.dart';
import 'package:dicta_app/features/settings/hotkey_card.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';

import 'fake_dicta_engine.dart';

void main() {
  late Directory dir;
  late String fallbackPath;
  late String daemonPath;

  setUp(() {
    dir = Directory.systemTemp.createTempSync('dicta_hk_error_');
    fallbackPath = '${dir.path}/fallback.toml';
    daemonPath = '${dir.path}/from_daemon.toml';
    File(fallbackPath).writeAsStringSync('hotkey = "alt+v"\n');
    File(daemonPath).writeAsStringSync('hotkey = "ctrl+alt+v"\n');
  });

  tearDown(() => dir.deleteSync(recursive: true));

  ProviderContainer containerFor(FakeDictaEngine fake) => ProviderContainer(
        overrides: [
          engineProvider.overrideWithValue(fake),
          hotkeyConfigPathProvider.overrideWithValue(fallbackPath),
        ],
      );

  test('reply error field becomes AsyncError, not a file fallback', () async {
    final fake = FakeDictaEngine(
      const DictaStatus(connected: true, state: DaemonState.idle),
      fetchSettingsReply: <String, dynamic>{
        'error': 'model not found',
        'settings': <String, dynamic>{'config_path': daemonPath},
      },
    );
    final container = containerFor(fake);
    addTearDown(container.dispose);

    await expectLater(
      container.read(hotkeyConfigTextProvider.future),
      throwsA(
        isA<StateError>().having(
          (error) => error.message,
          'message',
          contains('model not found'),
        ),
      ),
    );
  });

  test('transport failure falls back to the injected config file', () async {
    final fake = FakeDictaEngine(
      const DictaStatus(connected: true, state: DaemonState.idle),
    );
    // fetchSettingsError is a settable field, not a constructor parameter.
    fake.fetchSettingsError = SocketException('dead daemon');
    final container = containerFor(fake);
    addTearDown(container.dispose);

    final text = await container.read(hotkeyConfigTextProvider.future);
    expect(text, contains('hotkey = "alt+v"'));
  });

  test('healthy reply wins over the fallback path', () async {
    final fake = FakeDictaEngine(
      const DictaStatus(connected: true, state: DaemonState.idle),
      fetchSettingsReply: <String, dynamic>{
        'settings': <String, dynamic>{'config_path': daemonPath},
      },
    );
    final container = containerFor(fake);
    addTearDown(container.dispose);

    final text = await container.read(hotkeyConfigTextProvider.future);
    expect(text, contains('hotkey = "ctrl+alt+v"'));
  });

  test('poll reports the disconnect cause without any payload', () async {
    final fake = FakeDictaEngine(
      const DictaStatus(connected: true, state: DaemonState.idle),
      statusError: SocketException('SECRET PAYLOAD MUST NOT LEAK'),
    );
    final container = ProviderContainer(
      overrides: [engineProvider.overrideWithValue(fake)],
    );
    addTearDown(container.dispose);

    await container.read(dictaStatusProvider.notifier).poll();
    final snapshot = container.read(dictaStatusProvider);

    expect(snapshot.connected, isFalse);
    expect(snapshot.error, 'SocketException');
    expect(snapshot.error, isNot(contains('SECRET PAYLOAD')));
  });
}

// APP-04 golden: the captured daemon reply is the one true 17-key contract.
import 'dart:convert';
import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:dicta_app/core/settings.dart';

void main() {
  late Map<String, dynamic> fixture;
  late SettingsReply settings;

  setUpAll(() {
    final raw = File('test/fixtures/settings_reply.json').readAsStringSync();
    fixture = jsonDecode(raw) as Map<String, dynamic>;
    settings = SettingsReply.fromReply(fixture).settings;
  });

  test('captured fixture deserializes field-by-field (20 fields)', () {
    expect(settings.language, 'ru');
    expect(settings.injectMode, 'paste');
    expect(settings.minLevel, closeTo(0.35, 1e-6));
    expect(settings.autoStop, isFalse);
    expect(settings.autoStopLevel, closeTo(0.2, 1e-6));
    expect(settings.autoStopAfterMs, 1500);
    expect(settings.autoStopGraceMs, 1200);
    expect(settings.maxRecordMs, 60000);
    expect(settings.useGpu, isTrue);
    expect(settings.gpuDevice, 0);
    expect(settings.nThreads, 8);
    expect(settings.beamSize, 1);
    expect(settings.pasteKeys, 'ctrl+v');
    expect(settings.clipboardRestoreMs, 2500);
    expect(settings.pasteDelayMs, 150);
    expect(settings.atspiAllowAnyRole, isFalse);
    expect(settings.model,
        '/home/wardreg/.local/share/whisper-cpp/ggml-medium-q5_0.bin');
    expect(settings.models.length, 3);
    expect(settings.reloading, isFalse);
    expect(settings.configPath, '/home/wardreg/.config/dicta/config.toml');
  });

  test('kEditableKeys is the fixture settings keys minus three extras', () {
    final rawKeys = (fixture['settings'] as Map<String, dynamic>).keys.toSet();
    final expected = rawKeys.difference({'models', 'reloading', 'config_path'});
    expect(kEditableKeys.length, 17);
    expect(kEditableKeys, expected);
    expect(kEngineKeys.length, 6);
  });

  test('unknown keys tolerated, missing keys default', () {
    final parsed = SettingsReply.fromJson({'min_level': 0.5, 'future_key': 1});
    expect(parsed.minLevel, closeTo(0.5, 1e-6));
    expect(parsed.language, 'ru');
  });

  test('poisoned replies yield defaults without throwing', () {
    for (final reply in <Map<String, dynamic>>[
      {'settings': 'garbage'},
      {'settings': {'min_level': 'many'}},
      {'settings': {'auto_stop': null}},
      <String, dynamic>{},
    ]) {
      final result = SettingsReply.fromReply(reply);
      expect(result.settings.language, 'ru');
      expect(result.settings.minLevel, 0.0);
    }
  });

  test('top-level error is surfaced from the reply', () {
    expect(SettingsReply.fromReply({'error': 'bad value'}).error, 'bad value');
  });
}

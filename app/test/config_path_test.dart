// Parity tests for the read-only config helpers. The format is copied verbatim
// from the frozen Rust/TOML reference (config.rs:173-178, config.example.toml:80).
import 'dart:io';

import 'package:dicta_app/core/config_path.dart';
import 'package:flutter_test/flutter_test.dart';

void main() {
  final example = File('test/fixtures/config_example_toml.txt').readAsStringSync();
  final live = File('test/fixtures/config_live_no_hotkey.txt').readAsStringSync();

  group('extractHotkey (flat TOML, no package)', () {
    test('example fixture yields the verbatim evdev combination', () {
      expect(extractHotkey(example), 'ctrl+alt+v');
    });

    test('live-like config without a hotkey line yields null (daemon off)', () {
      expect(extractHotkey(live), isNull);
    });

    test('explicit off is returned verbatim', () {
      expect(extractHotkey('hotkey = "off"'), 'off');
    });

    test('leading spaces and extra spaces around = still match', () {
      expect(extractHotkey('   hotkey   =   "alt+v"'), 'alt+v');
    });

    test('TOML literal string in single quotes (WR-04)', () {
      expect(extractHotkey("hotkey = 'alt+v'"), 'alt+v');
    });

    test('single quotes with leading spaces and explicit off (WR-04)', () {
      expect(extractHotkey("   hotkey   =   'off'"), 'off');
    });

    test('single-quoted literal after a section header (WR-04)', () {
      expect(extractHotkey("[hotkey]\nhotkey='ctrl+alt+v'"), 'ctrl+alt+v');
    });

    test('unquoted garbage is not a value', () {
      expect(extractHotkey('hotkey = off'), isNull);
    });

    test('line-based: a section header changes nothing', () {
      expect(extractHotkey('[audio]\nhotkey="x"'), 'x');
    });
  });

  group('resolveConfigPath (parity with config.rs:173-178)', () {
    test('non-empty override wins', () {
      expect(
        resolveConfigPath(
          configOverride: '/tmp/custom.toml',
          xdgConfigHome: '/xdg',
          home: '/home/u',
        ),
        '/tmp/custom.toml',
      );
    });

    test('empty override is unset, not a zero-length path (IN-01)', () {
      expect(
        resolveConfigPath(
          configOverride: '',
          xdgConfigHome: '/xdg',
          home: '/home/u',
        ),
        '/xdg/dicta/config.toml',
      );
    });

    test('relative XDG_CONFIG_HOME is ignored, dirs parity (IN-01)', () {
      expect(
        resolveConfigPath(
          configOverride: '',
          xdgConfigHome: 'relative/xdg',
          home: '/home/u',
        ),
        '/home/u/.config/dicta/config.toml',
      );
      // A relative non-empty override is NOT an XDG setting: config.rs's
      // var_os("DICTA_CONFIG") accepts it as a path verbatim.
      expect(
        resolveConfigPath(
          configOverride: 'relative/config.toml',
          xdgConfigHome: '/xdg',
          home: '/home/u',
        ),
        'relative/config.toml',
      );
    });

    test('no XDG falls back to HOME/.config', () {
      expect(
        resolveConfigPath(
          configOverride: '',
          xdgConfigHome: '',
          home: '/home/u',
        ),
        '/home/u/.config/dicta/config.toml',
      );
    });

    test('no HOME uses the honest last resort', () {
      expect(
        resolveConfigPath(
          configOverride: '',
          xdgConfigHome: '',
          home: '',
        ),
        'dicta-config.toml',
      );
    });
  });
}

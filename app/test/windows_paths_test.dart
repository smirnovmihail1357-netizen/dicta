// WIN-05 contract tests — no Rust Windows reference exists yet. These record
// the phase-6 path CONTRACT with injectable env values; the existing Linux
// XDG resolution is covered by the unchanged config_path/hf_parse suites.
import 'package:dicta_app/core/config_path.dart';
import 'package:dicta_app/core/hf/hf_catalog.dart';
import 'package:flutter_test/flutter_test.dart';

void main() {
  group('windowsConfigPath (%APPDATA% contract)', () {
    test('uses the injected roaming AppData directory', () {
      expect(
        windowsConfigPath(appData: r'C:\Users\u\AppData\Roaming'),
        r'C:\Users\u\AppData\Roaming\dicta\config.toml',
      );
    });

    test('empty AppData falls back to the honest relative path', () {
      expect(windowsConfigPath(appData: ''), r'dicta\config.toml');
    });

    test('missing AppData falls back to the honest relative path', () {
      // On Linux CI APPDATA is absent; no Unix path is substituted.
      expect(windowsConfigPath(appData: null), r'dicta\config.toml');
    });
  });

  group('resolveConfigPath precedence', () {
    test('DICTA_CONFIG override wins over every platform branch', () {
      expect(
        resolveConfigPath(
          configOverride: r'D:\custom\dicta.toml',
          appData: r'C:\Users\u\AppData\Roaming',
          xdgConfigHome: '/xdg',
          home: '/home/u',
        ),
        r'D:\custom\dicta.toml',
      );
    });
  });

  group('destDirFrom (%LOCALAPPDATA% contract)', () {
    test('uses the injected local app-data directory', () {
      expect(
        destDirFrom('ggml.bin', null, '/home/u', r'C:\Users\u\AppData\Local'),
        r'C:\Users\u\AppData\Local\whisper-cpp',
      );
    });

    test('empty localAppData keeps the existing XDG branch', () {
      expect(
        destDirFrom('ggml.bin', '/data', '/home/u', ''),
        '/data/whisper-cpp',
      );
    });

    test('null localAppData keeps the existing XDG branch', () {
      expect(
        destDirFrom('ggml.bin', '/data', '/home/u', null),
        '/data/whisper-cpp',
      );
    });

    test('explicit model parent path wins over localAppData', () {
      expect(
        destDirFrom('/opt/models/ggml.bin', null, '/home/u',
            r'C:\Users\u\AppData\Local'),
        '/opt/models',
      );
    });

    test('explicit Windows model parent wins over localAppData', () {
      expect(
        destDirFrom(r'C:\models\ggml.bin', null, '/home/u',
            r'C:\Users\u\AppData\Local'),
        r'C:\models',
      );
    });

    test('Linux HOME fallback is unchanged (regression)', () {
      expect(
        destDirFrom('ggml.bin', null, '/home/u', null),
        '/home/u/.local/share/whisper-cpp',
      );
    });
  });
}

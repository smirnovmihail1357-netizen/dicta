// APP-02 / APP-06 framing regression tests (pure unit tests, no live socket).
import 'dart:convert';

import 'package:flutter_test/flutter_test.dart';
import 'package:dicta_app/core/engine.dart';

void main() {
  group('set payload framing (APP-06)', () {
    test('quotes, backslashes and newlines round-trip byte-exact', () {
      const value = 'он сказал "да\n"';
      final line = buildSetLine('paste_keys', value);
      expect(
        line,
        '{"cmd":"set","key":"paste_keys","value":"он сказал \\"да\\n\\""}\n',
      );
      final decoded = jsonDecode(line) as Map<String, dynamic>;
      expect(decoded['value'], value);
    });

    test('empty value emits a valid JSON line with an empty string', () {
      final line = buildSetLine('paste_keys', '');
      expect(line, '{"cmd":"set","key":"paste_keys","value":""}\n');
      expect((jsonDecode(line) as Map<String, dynamic>)['value'], '');
    });

    test('status line has the exact wire form', () {
      expect(buildStatusLine(), '{"cmd":"status"}\n');
    });
  });

  group('reply line parsing', () {
    test('valid status line parses into a connected idle snapshot', () {
      const line =
          '{"state":"idle","level":0.0,"seconds":0.0,"text":null,"error":null,"panel":false}';
      final status = DictaStatus.fromReplyLine(line);
      expect(status.connected, isTrue);
      expect(status.state, DaemonState.idle);
      expect(status.level, 0.0);
    });

    test('garbage and partial lines yield disconnected, never throw', () {
      for (final bad in ['{not json', '', '{"state":', '[]']) {
        expect(DictaStatus.fromReplyLine(bad).connected, isFalse);
      }
    });
  });
}

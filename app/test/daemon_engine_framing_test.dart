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

    test('stop reply with text parses into a snapshot without losing text', () {
      const line =
          '{"state":"idle","level":0.0,"seconds":0.0,"text":"привет","error":null,"panel":false}';
      final status = DictaStatus.fromReplyLine(line);
      expect(status.state, DaemonState.idle);
      expect(status.text, 'привет');
    });

    test('unknown state string falls back to idle without throwing', () {
      const line =
          '{"state":"paused","level":0.0,"seconds":0.0,"text":null,"error":null}';
      expect(DictaStatus.fromReplyLine(line).state, DaemonState.idle);
    });
  });

  group('inject payload framing (DASH-03)', () {
    test('quotes, backslashes and newlines round-trip byte-exact', () {
      const text = 'он сказал "да\n"';
      final line = buildInjectLine(text, null);
      final decoded = jsonDecode(line) as Map<String, dynamic>;
      expect(decoded['cmd'], 'inject');
      expect(decoded['text'], text);
      expect(decoded.containsKey('app'), isFalse);
    });

    test('empty text emits a valid JSON line with an empty field', () {
      final line = buildInjectLine('', null);
      expect(line, '{"cmd":"inject","text":""}\n');
      expect((jsonDecode(line) as Map<String, dynamic>)['text'], '');
    });

    test('app hint is included when provided', () {
      final decoded = jsonDecode(buildInjectLine('привет', 'gedit'))
          as Map<String, dynamic>;
      expect(decoded['app'], 'gedit');
    });
  });
}

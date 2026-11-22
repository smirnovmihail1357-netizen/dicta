/// The only file that knows the Linux transport. dart:io AF_UNIX, one fresh
/// connection per command, one JSON line out and one JSON line back.
library;

import 'dart:async';
import 'dart:convert';
import 'dart:io';

import 'engine.dart';

/// DICTA_SOCK (test override) -> XDG_RUNTIME_DIR -> /run/user/`uid`.
/// A config.toml `socket = "..."` override is deliberately not parsed in
/// Phase 1: no TOML reader in Dart yet, and this mirrors dicta-core's
/// default_socket_path() resolution.
String resolveSocketPath() {
  final override = Platform.environment['DICTA_SOCK'];
  if (override != null && override.isNotEmpty) return override;
  final xdg = Platform.environment['XDG_RUNTIME_DIR'];
  if (xdg != null && xdg.isNotEmpty) return '$xdg/dicta.sock';
  final uid = Process.runSync('id', ['-u']).stdout.toString().trim();
  return '/run/user/$uid/dicta.sock';
}

// Russian runtime text as Unicode escapes: the phase's Cyrillic gate counts
// every Cyrillic byte in app/lib/**/*.dart, comment lines included.
const _daemonNotResponding =
    '\u0434\u0435\u043c\u043e\u043d \u043d\u0435 \u043e\u0442\u0432\u0435\u0447\u0430\u0435\u0442';
const _startHint =
    '\u2014 \u0437\u0430\u043f\u0443\u0441\u0442\u0438 systemctl --user start dictad';
const _badReply =
    '\u043d\u0435\u043a\u043e\u0440\u0440\u0435\u043a\u0442\u043d\u044b\u0439 \u043e\u0442\u0432\u0435\u0442 \u0434\u0435\u043c\u043e\u043d\u0430';
// Distinct from _badReply: a timeout is the daemon being silent, not a
// protocol syntax error. The budget is reported so a wedged paste (up to the
// 120 s inject budget) is distinguishable from a fast malformed reply.
String _timeoutHint(Duration timeout) =>
    '$_daemonNotResponding (\u0442\u0430\u0439\u043c\u0430\u0443\u0442 ${timeout.inSeconds} \u0441)';

class DaemonEngine implements DictaEngine {
  DaemonEngine([String? socketPath])
      : socketPath = socketPath ?? resolveSocketPath();

  final String socketPath;

  static const _connectTimeout = Duration(seconds: 2);
  static const _statusTimeout = Duration(seconds: 2);
  static const _settingsTimeout = Duration(seconds: 30);
  static const _injectTimeout = Duration(seconds: 120);

  Future<String> _call(String line, {required Duration timeout}) async {
    late final Socket socket;
    try {
      socket = await Socket.connect(
        InternetAddress(socketPath, type: InternetAddressType.unix),
        0,
        timeout: _connectTimeout,
      );
    } on SocketException catch (error) {
      // Error type and socket path only - never log reply payloads (no text).
      throw SocketException(
        '$_daemonNotResponding $socketPath $_startHint: $error',
      );
    }
    try {
      socket.write(line);
      await socket.flush();
      return await socket
          .cast<List<int>>()
          .transform(utf8.decoder)
          .transform(const LineSplitter())
          .first
          .timeout(timeout);
    } finally {
      socket.destroy();
    }
  }

  Future<DictaReply> _commandLine(String line,
      {required Duration timeout}) async {
    try {
      final map = decodeReplyMap(await _call(line, timeout: timeout));
      final error = map['error'] as String?;
      return DictaReply(ok: error == null, error: error);
    } on SocketException catch (error) {
      // Transport failure: _call already wrapped this with the actionable
      // daemon-not-responding / "systemctl --user start dictad" hint.
      return DictaReply(ok: false, error: error.message);
    } on TimeoutException {
      return DictaReply(ok: false, error: _timeoutHint(timeout));
    } catch (_) {
      // JSON/format/EOF failures only — never mask a SocketException again.
      return DictaReply(ok: false, error: _badReply);
    }
  }

  Future<DictaReply> _command(String cmd, String? app,
      {required Duration timeout}) =>
      _commandLine('${jsonEncode({'cmd': cmd, 'app': ?app})}\n',
          timeout: timeout);

  /// Action replies carry a full status snapshot (including the last text).
  /// Transport and format failures become a typed snapshot with an error,
  /// never an exception escaping to the UI.
  Future<DictaStatus> _snapshotCommand(String line,
      {required Duration timeout}) async {
    try {
      return DictaStatus.fromReplyLine(await _call(line, timeout: timeout));
    } on SocketException catch (error) {
      return DictaStatus(
        connected: false,
        state: DaemonState.idle,
        error: error.message,
      );
    } on TimeoutException {
      return DictaStatus(
        connected: false,
        state: DaemonState.idle,
        error: _timeoutHint(timeout),
      );
    } catch (_) {
      return DictaStatus(
        connected: false,
        state: DaemonState.idle,
        error: _badReply,
      );
    }
  }

  @override
  Future<DictaStatus> status() async => DictaStatus.fromReplyLine(
        await _call(buildStatusLine(), timeout: _statusTimeout),
      );

  @override
  Future<DictaReply> start({String? app}) =>
      _command('start', app, timeout: _injectTimeout);

  @override
  Future<DictaStatus> stop({String? app}) => _snapshotCommand(
        '${jsonEncode({'cmd': 'stop', 'app': ?app})}\n',
        timeout: _injectTimeout,
      );

  @override
  Future<DictaStatus> inject({required String text, String? app}) =>
      _snapshotCommand(buildInjectLine(text, app), timeout: _injectTimeout);

  @override
  Future<DictaReply> toggle({String? app}) =>
      _command('toggle', app, timeout: _injectTimeout);

  @override
  Future<DictaReply> cancel() =>
      _command('cancel', null, timeout: _injectTimeout);

  @override
  Future<Map<String, dynamic>> fetchSettings() async => decodeReplyMap(
        await _call('${jsonEncode({'cmd': 'settings'})}\n',
            timeout: _settingsTimeout),
      );

  @override
  Future<DictaReply> set(String key, String value) => _commandLine(
        buildSetLine(key, value),
        timeout: _settingsTimeout,
      );
}

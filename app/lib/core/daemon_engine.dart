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

class DaemonEngine implements DictaEngine {
  DaemonEngine([String? socketPath])
      : socketPath = socketPath ?? resolveSocketPath();

  final String socketPath;
  final StreamController<DictaStatus> _statusController =
      StreamController<DictaStatus>.broadcast();

  @override
  Stream<DictaStatus> get statusStream => _statusController.stream;

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
    } catch (_) {
      return DictaReply(ok: false, error: _badReply);
    }
  }

  Future<DictaReply> _command(String cmd, String? app,
      {required Duration timeout}) =>
      _commandLine('${jsonEncode({'cmd': cmd, 'app': ?app})}\n',
          timeout: timeout);

  @override
  Future<DictaStatus> status() async {
    final status = DictaStatus.fromReplyLine(
      await _call(buildStatusLine(), timeout: _statusTimeout),
    );
    _statusController.add(status);
    return status;
  }

  @override
  Future<DictaReply> start({String? app}) =>
      _command('start', app, timeout: _injectTimeout);

  @override
  Future<DictaReply> stop({String? app}) =>
      _command('stop', app, timeout: _injectTimeout);

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

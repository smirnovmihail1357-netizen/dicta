/// The only file that knows the Windows named pipe transport. `\\.\pipe\dicta`,
/// one fresh connection per command, one JSON line out and one JSON line back.
library;

import 'dart:async';
import 'dart:convert';
import 'dart:io';

import 'package:dart_ipc/dart_ipc.dart';

import 'engine.dart';
import 'settings.dart';

// DACL contract (phase 7, PITFALLS.md:234): the pipe SERVER creates
// \\.\pipe\dicta with a security descriptor granting access ONLY to the
// current user SID. Phase 6 verifies this CONTRACT is recorded, not
// implemented — the client connects, it does not create the pipe.

/// The pipe name lives here and nowhere else (edge-probe WIN-05).
const kDictaPipe = '\\\\.\\pipe\\dicta';

/// DICTA_PIPE (test override) -> kDictaPipe. DICTA_SOCK is deliberately not
/// read: a Unix path on Windows would spoof the transport (threat T-6-01);
/// this override is a test-only seam and is not set in production.
String resolvePipePath() {
  final override = Platform.environment['DICTA_PIPE'];
  if (override != null && override.isNotEmpty) return override;
  return kDictaPipe;
}

// Russian runtime text as Unicode escapes: the phase's Cyrillic gate counts
// every Cyrillic byte in app/lib/**/*.dart, comment lines included.
const _daemonNotResponding =
    '\u0434\u0435\u043c\u043e\u043d \u043d\u0435 \u043e\u0442\u0432\u0435\u0447\u0430\u0435\u0442';
const _badReply =
    '\u043d\u0435\u043a\u043e\u0440\u0440\u0435\u043a\u0442\u043d\u044b\u0439 \u043e\u0442\u0432\u0435\u0442 \u0434\u0435\u043c\u043e\u043d\u0430';
// Distinct from _badReply: a timeout is the daemon being silent, not a
// protocol syntax error. The budget is reported so a wedged paste (up to the
// 120 s inject budget) is distinguishable from a fast malformed reply.
String _timeoutHint(Duration timeout) =>
    '$_daemonNotResponding (\u0442\u0430\u0439\u043c\u0430\u0443\u0442 ${timeout.inSeconds} \u0441)';

/// Transport failure with an actionable, payload-free message. IOException is
/// abstract with no public constructor, so the wrapped connect error is
/// carried by this private subtype instead.
class _PipeTransportException implements IOException {
  const _PipeTransportException(this.message);

  final String message;

  @override
  String toString() => 'PipeTransportException: $message';
}

String _transportMessage(Object error) =>
    error is _PipeTransportException ? error.message : error.toString();

class WindowsPipeEngine implements DictaEngine {
  WindowsPipeEngine([String? pipePath])
      : pipePath = pipePath ?? resolvePipePath();

  final String pipePath;

  static const _connectTimeout = Duration(seconds: 2);
  static const _statusTimeout = Duration(seconds: 2);
  static const _settingsTimeout = Duration(seconds: 30);
  static const _injectTimeout = Duration(seconds: 120);

  // dart_ipc.connect has no timeout parameter (public API is just `path`):
  // on Windows a connect to a missing pipe retries for ~5 s, so wrap it.
  Future<Socket> _connect() {
    final connectFuture = connect(pipePath);
    return connectFuture.timeout(_connectTimeout, onTimeout: () {
      // dart_ipc's retry loop cannot be cancelled: reap a late success so no
      // orphaned Win32 handle is left open for the phase 7 pipe server.
      connectFuture.then((socket) => socket.destroy()).ignore();
      throw TimeoutException('connect $pipePath', _connectTimeout);
    });
  }

  Future<String> _call(String line, {required Duration timeout}) async {
    late final Socket socket;
    try {
      socket = await _connect();
    } on TimeoutException {
      // Connect has its own 2 s budget - report THAT, not the command budget.
      // A silent pipe must be distinguishable from a wedged 120 s paste.
      throw _PipeTransportException(
        '$_daemonNotResponding $pipePath: ${_timeoutHint(_connectTimeout)}',
      );
    } on IOException catch (error) {
      // Error type and pipe path only - never log reply payloads (no text).
      throw _PipeTransportException(
        '$_daemonNotResponding $pipePath: ${error.runtimeType}',
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
    } on IOException catch (error) {
      // Transport failure: _call already wrapped this with the actionable
      // daemon-not-responding hint.
      return DictaReply(ok: false, error: _transportMessage(error));
    } on TimeoutException {
      return DictaReply(ok: false, error: _timeoutHint(timeout));
    } catch (_) {
      // JSON/format/EOF failures only — never mask a transport failure again.
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
    } on IOException catch (error) {
      return DictaStatus(
        connected: false,
        state: DaemonState.idle,
        error: _transportMessage(error),
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
  Future<DictaStatus> status() =>
      _snapshotCommand(buildStatusLine(), timeout: _statusTimeout);

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
  Future<SetReply> set(String key, String value) async {
    try {
      // Parse the FULL reply: note (text) + error + fresh settings snapshot
      // (dictad main.rs:730-739). _commandLine would drop note and settings.
      final map = decodeReplyMap(
        await _call(buildSetLine(key, value), timeout: _settingsTimeout),
      );
      return SetReply.fromReply(map);
    } on IOException catch (error) {
      // Error message only - never log the set payload (payload privacy).
      return SetReply(
        note: null,
        error: _transportMessage(error),
        settings: const SettingsReply(),
      );
    } on TimeoutException {
      return SetReply(
        error: _timeoutHint(_settingsTimeout),
        settings: const SettingsReply(),
      );
    } catch (_) {
      // JSON/format/EOF failures only — never mask a transport failure again.
      return SetReply(
        error: _badReply,
        settings: const SettingsReply(),
      );
    }
  }
}

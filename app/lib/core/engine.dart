/// Transport-agnostic seam between the UI and the dictad daemon.
///
/// One command is one JSON line out and one JSON line back. Every command
/// opens a fresh connection, so command ordering needs no guarantee and none
/// is implemented. Value validation stays in the daemon: this layer only
/// serializes and surfaces the reply's error field.
library;

import 'dart:convert';

import 'settings.dart';

enum DaemonState { idle, recording, transcribing }

class DictaStatus {
  const DictaStatus({
    required this.connected,
    required this.state,
    this.level = 0.0,
    this.seconds = 0.0,
    this.text,
    this.error,
  });

  const DictaStatus.disconnected()
      : connected = false,
        state = DaemonState.idle,
        level = 0.0,
        seconds = 0.0,
        text = null,
        error = null;

  /// Tolerant parse of one daemon reply line. Malformed or partial input
  /// becomes a disconnected snapshot instead of escaping as an exception.
  factory DictaStatus.fromReplyLine(String line) {
    try {
      final map = decodeReplyMap(line);
      final state = switch (map['state']) {
        'recording' => DaemonState.recording,
        'transcribing' => DaemonState.transcribing,
        _ => DaemonState.idle,
      };
      return DictaStatus(
        connected: true,
        state: state,
        level: (map['level'] as num?)?.toDouble() ?? 0.0,
        seconds: (map['seconds'] as num?)?.toDouble() ?? 0.0,
        text: map['text'] as String?,
        error: map['error'] as String?,
      );
    } catch (_) {
      return const DictaStatus.disconnected();
    }
  }

  final bool connected;
  final DaemonState state;
  final double level;
  final double seconds;
  final String? text;
  final String? error;
}

class DictaReply {
  const DictaReply({required this.ok, this.error});

  final bool ok;
  final String? error;
}

abstract interface class DictaEngine {
  Future<DictaStatus> status();
  Future<DictaReply> start({String? app});

  /// The reply is a snapshot: it is the only place the recognized text ever
  /// appears (status never carries text).
  Future<DictaStatus> stop({String? app});

  /// Inserts text into the active window with the daemon's 120 s budget.
  Future<DictaStatus> inject({required String text, String? app});

  Future<DictaReply> toggle({String? app});
  Future<DictaReply> cancel();

  /// Full decoded reply map; SettingsReply.fromReply consumes it (Task 4).
  Future<Map<String, dynamic>> fetchSettings();

  /// The reply carries a note and a fresh settings snapshot (C1) so the form
  /// can update without a refetch and engine-reload refusals stay visible.
  Future<SetReply> set(String key, String value);
}

Map<String, dynamic> decodeReplyMap(String line) {
  final decoded = jsonDecode(line);
  if (decoded is Map<String, dynamic>) return decoded;
  throw const FormatException('daemon reply is not a JSON object');
}

String buildStatusLine() => '${jsonEncode({'cmd': 'status'})}\n';

String buildSetLine(String key, String value) =>
    '${jsonEncode({'cmd': 'set', 'key': key, 'value': value})}\n';

String buildInjectLine(String text, String? app) =>
    '${jsonEncode({'cmd': 'inject', 'text': text, 'app': ?app})}\n';

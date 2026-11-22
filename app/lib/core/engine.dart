/// Transport-agnostic seam between the UI and the dictad daemon.
///
/// One command is one JSON line out and one JSON line back. Every command
/// opens a fresh connection, so command ordering needs no guarantee and none
/// is implemented. Value validation stays in the daemon: this layer only
/// serializes and surfaces the reply's error field.
library;

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

  factory DictaStatus.fromReplyLine(String line) => const DictaStatus.disconnected();

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
  Future<DictaReply> stop({String? app});
  Future<DictaReply> toggle({String? app});
  Future<DictaReply> cancel();
  Future<Object> fetchSettings();
  Future<DictaReply> set(String key, String value);
  Stream<DictaStatus> get statusStream;
}

String buildStatusLine() => '{}';

String buildSetLine(String key, String value) => '{}';

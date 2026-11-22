/// Command-side state for the dashboard. Recognized text is captured only
/// from our own stop/inject replies - status never carries text.
/// Transport stays in core; errors arrive as data, never as exceptions.
library;

import 'package:dicta_app/core/providers.dart';
import 'package:flutter/services.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

// Unicode escapes: the Cyrillic gate counts literals even in comments.
const _cancelNotice =
    '\u0437\u0430\u043f\u0438\u0441\u044c \u043e\u0442\u043c\u0435\u043d\u0435\u043d\u0430';
const _commandFailed =
    '\u043a\u043e\u043c\u0430\u043d\u0434\u0430 \u043d\u0435 \u0432\u044b\u043f\u043e\u043b\u043d\u0435\u043d\u0430';

class DictaControlState {
  const DictaControlState({
    this.commandInFlight = false,
    this.lastText,
    this.lastError,
  });

  final bool commandInFlight;
  final String? lastText;
  final String? lastError;

  DictaControlState copyWith({
    bool? commandInFlight,
    String? lastText,
    String? lastError,
    bool clearLastText = false,
    bool clearLastError = false,
  }) {
    return DictaControlState(
      commandInFlight: commandInFlight ?? this.commandInFlight,
      lastText: clearLastText ? null : lastText ?? this.lastText,
      lastError: clearLastError ? null : lastError ?? this.lastError,
    );
  }
}

class DictaControlNotifier extends Notifier<DictaControlState> {
  @override
  DictaControlState build() => const DictaControlState();

  Future<void> start() async {
    state = state.copyWith(commandInFlight: true, clearLastError: true);
    try {
      // start clears the daemon's last_text, so the captured text is stale.
      final reply = await ref.read(engineProvider).start();
      state = state.copyWith(
        commandInFlight: false,
        clearLastText: true,
        clearLastError: reply.error == null,
        lastError: reply.error,
      );
    } catch (_) {
      state = state.copyWith(
        commandInFlight: false,
        lastError: _commandFailed,
      );
    }
  }

  Future<void> stop() async {
    state = state.copyWith(commandInFlight: true, clearLastError: true);
    try {
      final snapshot = await ref.read(engineProvider).stop();
      state = state.copyWith(
        commandInFlight: false,
        clearLastText: snapshot.text == null,
        lastText: snapshot.text,
        clearLastError: snapshot.error == null,
        lastError: snapshot.error,
      );
    } catch (_) {
      state = state.copyWith(
        commandInFlight: false,
        lastError: _commandFailed,
      );
    }
  }

  Future<void> cancel() async {
    state = state.copyWith(commandInFlight: true, clearLastError: true);
    try {
      final reply = await ref.read(engineProvider).cancel();
      // The daemon's "recording cancelled" note is honest noise, not a fault.
      final notice = reply.error == _cancelNotice;
      state = state.copyWith(
        commandInFlight: false,
        clearLastError: reply.error == null || notice,
        lastError: notice ? null : reply.error,
      );
    } catch (_) {
      state = state.copyWith(
        commandInFlight: false,
        lastError: _commandFailed,
      );
    }
  }

  Future<void> inject(String text) async {
    state = state.copyWith(commandInFlight: true, clearLastError: true);
    try {
      final snapshot = await ref.read(engineProvider).inject(text: text);
      state = state.copyWith(
        commandInFlight: false,
        clearLastText: snapshot.text == null,
        lastText: snapshot.text,
        clearLastError: snapshot.error == null,
        lastError: snapshot.error,
      );
    } catch (_) {
      state = state.copyWith(
        commandInFlight: false,
        lastError: _commandFailed,
      );
    }
  }
}

final dictaControlProvider =
    NotifierProvider<DictaControlNotifier, DictaControlState>(
        DictaControlNotifier.new);

/// Copy seam over the SDK clipboard. Writing happens only on an explicit click.
final clipboardWriterProvider = Provider<void Function(String)>((ref) =>
    (text) => Clipboard.setData(ClipboardData(text: text)));

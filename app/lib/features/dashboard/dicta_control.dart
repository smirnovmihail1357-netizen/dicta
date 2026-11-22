/// Command-side state for the dashboard. Recognized text is captured only
/// from our own stop/inject replies - status never carries text.
/// The history journal captures only dictations made through the app.
/// Transport stays in core; errors arrive as data, never as exceptions.
library;

import 'package:dicta_app/core/providers.dart';
import 'package:dicta_app/features/history/history.dart';
import 'package:flutter/services.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

// Unicode escapes: the Cyrillic gate counts literals even in comments.
// The daemon's own notice stays a protocol value; the generic user-facing
// failure body lives in app_ru.arb (dashCommandFailedBody) and is read at the
// point of display, never stored as text in this file.
const _cancelNotice =
    '\u0437\u0430\u043f\u0438\u0441\u044c \u043e\u0442\u043c\u0435\u043d\u0435\u043d\u0430';

/// Which command produced [DictaControlState.lastError]. The last-text card
/// uses it to avoid blaming inject for a failed stop/cancel/start (WR-01).
enum DictaCommand { start, stop, cancel, inject }

class DictaControlState {
  const DictaControlState({
    this.commandInFlight = false,
    this.lastText,
    this.lastError,
    this.lastErrorCommand,
    this.genericFailure = false,
  });

  final bool commandInFlight;
  final String? lastText;
  final String? lastError;
  final DictaCommand? lastErrorCommand;

  /// True when a command threw before the daemon could supply an error body.
  /// The card renders the localised dashCommandFailedBody for this case.
  final bool genericFailure;

  DictaControlState copyWith({
    bool? commandInFlight,
    String? lastText,
    String? lastError,
    DictaCommand? lastErrorCommand,
    bool? genericFailure,
    bool clearLastText = false,
    bool clearLastError = false,
  }) {
    return DictaControlState(
      commandInFlight: commandInFlight ?? this.commandInFlight,
      lastText: clearLastText ? null : lastText ?? this.lastText,
      lastError: clearLastError ? null : lastError ?? this.lastError,
      // Command travels with its error: never leave a stale inject label.
      lastErrorCommand: clearLastError
          ? null
          : lastErrorCommand ?? this.lastErrorCommand,
      genericFailure:
          clearLastError ? false : genericFailure ?? this.genericFailure,
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
        lastErrorCommand: DictaCommand.start,
      );
    } catch (_) {
      state = state.copyWith(
        commandInFlight: false,
        genericFailure: true,
        lastErrorCommand: DictaCommand.start,
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
        lastErrorCommand: DictaCommand.stop,
      );
      // The stop reply is the only honest capture point: status never carries
      // text. Empty/whitespace text never lands in the journal (A3/A4).
      if (snapshot.error == null &&
          snapshot.text != null &&
          snapshot.text!.trim().isNotEmpty) {
        ref.read(historyProvider.notifier).add(snapshot.text!);
      }
    } catch (_) {
      state = state.copyWith(
        commandInFlight: false,
        genericFailure: true,
        lastErrorCommand: DictaCommand.stop,
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
        lastErrorCommand: DictaCommand.cancel,
      );
    } catch (_) {
      state = state.copyWith(
        commandInFlight: false,
        genericFailure: true,
        lastErrorCommand: DictaCommand.cancel,
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
        lastErrorCommand: DictaCommand.inject,
      );
    } catch (_) {
      state = state.copyWith(
        commandInFlight: false,
        genericFailure: true,
        lastErrorCommand: DictaCommand.inject,
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

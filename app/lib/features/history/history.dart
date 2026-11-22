/// In-memory journal of dictations made through the app.
/// Deliberately not persisted: memory is the MVP boundary (HIST-04),
/// persistence is out of scope for this milestone. Entries are appended only
/// from the DictaControlNotifier.stop() seam; the status stream never carries
/// recognized text (A4).
library;

import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:intl/intl.dart';

/// Immutable journal entry. No copyWith: a recorded dictation is final.
class HistoryEntry {
  const HistoryEntry({
    required this.id,
    required this.text,
    required this.at,
  });

  final int id;
  final String text;
  final DateTime at;
}

class HistoryNotifier extends Notifier<List<HistoryEntry>> {
  @override
  List<HistoryEntry> build() => const <HistoryEntry>[];

  void add(String text) {
    final entry = HistoryEntry(
      // The id is derived from the head of the list: no mutable counter to
      // drift out of sync with the state (newest-first, A1).
      id: state.isEmpty ? 1 : state.first.id + 1,
      text: text,
      at: DateTime.now(),
    );
    state = [entry, ...state];
  }

  void remove(int id) {
    state = state.where((entry) => entry.id != id).toList();
  }
}

final historyProvider =
    NotifierProvider<HistoryNotifier, List<HistoryEntry>>(HistoryNotifier.new);

class HistoryQueryNotifier extends Notifier<String> {
  @override
  String build() => '';

  void set(String value) => state = value;
}

final historyQueryProvider =
    NotifierProvider<HistoryQueryNotifier, String>(HistoryQueryNotifier.new);

/// Minute-precision timestamp for a journal entry (tabular figures are applied
/// by the screen, not by the formatter).
String formatEntryTime(DateTime at, String localeName) =>
    DateFormat.Hm(localeName).format(at);

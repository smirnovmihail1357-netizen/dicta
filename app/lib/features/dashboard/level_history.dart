/// In-memory ring buffer of live signal samples for the calibration histogram.
/// History is deliberately not persisted: it is a live view of the last ~20 s.
library;

import 'package:flutter_riverpod/flutter_riverpod.dart';

class LevelHistoryNotifier extends Notifier<List<double>> {
  static const capacity = 200;

  @override
  List<double> build() => const <double>[];

  void push(double level) {
    final next = <double>[...state, level];
    state = next.length > capacity
        ? next.sublist(next.length - capacity)
        : next;
  }

  void clear() => state = const <double>[];
}

final levelHistoryProvider =
    NotifierProvider<LevelHistoryNotifier, List<double>>(LevelHistoryNotifier.new);

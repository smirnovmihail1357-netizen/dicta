// Ring buffer contract for the calibration histogram (DASH-04).
import 'package:dicta_app/features/dashboard/level_history.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';

void main() {
  test('push preserves sample order and clear empties the buffer', () {
    final container = ProviderContainer();
    addTearDown(container.dispose);
    final history = container.read(levelHistoryProvider.notifier);

    history.push(0.1);
    history.push(0.7);
    expect(container.read(levelHistoryProvider), <double>[0.1, 0.7]);

    history.clear();
    expect(container.read(levelHistoryProvider), isEmpty);
  });

  test('capacity is 200: the oldest samples are dropped first', () {
    final container = ProviderContainer();
    addTearDown(container.dispose);
    final history = container.read(levelHistoryProvider.notifier);

    for (var i = 0; i < 205; i++) {
      history.push(i / 1000);
    }

    final samples = container.read(levelHistoryProvider);
    expect(samples.length, 200);
    expect(samples.first, 5 / 1000);
    expect(samples.last, 204 / 1000);
  });
}

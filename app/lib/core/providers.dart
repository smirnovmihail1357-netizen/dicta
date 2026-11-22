import 'dart:async';

import 'package:flutter_riverpod/flutter_riverpod.dart';

import 'daemon_engine.dart';
import 'engine.dart';

final engineProvider = Provider<DictaEngine>((ref) {
  return DaemonEngine(resolveSocketPath());
});

final dictaStatusProvider =
    NotifierProvider<DictaStatusNotifier, DictaStatus>(DictaStatusNotifier.new);

/// Owns the 100 ms poll timer. Riverpod 3 has no legacy notifier/mounted API,
/// so reentrancy is guarded by a plain bool and the timer by ref.onDispose.
class DictaStatusNotifier extends Notifier<DictaStatus> {
  Timer? _timer;
  bool _inFlight = false;
  Duration _backoff = const Duration(milliseconds: 200);

  @override
  DictaStatus build() {
    _timer = Timer.periodic(const Duration(milliseconds: 100), (_) => poll());
    ref.onDispose(() => _timer?.cancel());
    return const DictaStatus.disconnected();
  }

  Future<void> poll() async {
    if (_inFlight) return;
    _inFlight = true;
    try {
      state = await ref.read(engineProvider).status();
      _backoff = const Duration(milliseconds: 200);
    } catch (_) {
      state = const DictaStatus.disconnected();
      await Future<void>.delayed(_backoff);
      final doubled = _backoff * 2;
      _backoff = doubled > const Duration(seconds: 2)
          ? const Duration(seconds: 2)
          : doubled;
    } finally {
      _inFlight = false;
    }
  }
}

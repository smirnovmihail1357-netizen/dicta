import 'dart:async';

import 'package:flutter_riverpod/flutter_riverpod.dart';

import 'daemon_engine.dart';
import 'engine.dart';
import 'settings.dart';

final engineProvider = Provider<DictaEngine>((ref) {
  final engine = DaemonEngine(resolveSocketPath());
  ref.onDispose(engine.dispose);
  return engine;
});

final dictaStatusProvider =
    NotifierProvider<DictaStatusNotifier, DictaStatus>(DictaStatusNotifier.new);

/// Settings are fetched on demand and refreshed explicitly (after set or a
/// diagnostics recheck) - never polled. The only parser is SettingsReply.fromReply.
final settingsSnapshotProvider = FutureProvider<SettingsReply>((ref) async {
  final reply = await ref.read(engineProvider).fetchSettings();
  return SettingsReply.fromReply(reply).settings;
});

/// Owns the 100 ms poll timer. Riverpod 3 has no legacy notifier/mounted API,
/// so reentrancy is guarded by a plain bool and the timer by ref.onDispose.
class DictaStatusNotifier extends Notifier<DictaStatus> {
  Timer? _timer;
  bool _inFlight = false;
  bool _disposed = false;
  Duration _backoff = const Duration(milliseconds: 200);

  @override
  DictaStatus build() {
    _disposed = false;
    _timer = Timer.periodic(const Duration(milliseconds: 100), (_) => poll());
    ref.onDispose(() {
      _disposed = true;
      _timer?.cancel();
    });
    return const DictaStatus.disconnected();
  }

  Future<void> poll() async {
    if (_inFlight || _disposed) return;
    _inFlight = true;
    try {
      final snapshot = await ref.read(engineProvider).status();
      if (_disposed) return;
      state = snapshot;
      _backoff = const Duration(milliseconds: 200);
    } catch (_) {
      if (_disposed) return;
      state = const DictaStatus.disconnected();
      await Future<void>.delayed(_backoff);
      if (_disposed) return;
      final doubled = _backoff * 2;
      _backoff = doubled > const Duration(seconds: 2)
          ? const Duration(seconds: 2)
          : doubled;
    } finally {
      _inFlight = false;
    }
  }
}

import 'dart:async';

import 'package:flutter_riverpod/flutter_riverpod.dart';

import 'daemon_engine.dart';
import 'engine.dart';
import 'hf/download_engine.dart';
import 'hf/hf_client.dart';
import 'settings.dart';
import 'tray.dart';
import 'window_shell.dart';

/// DaemonEngine holds no disposable resource: every command opens and closes
/// its own socket, and the retired statusStream broadcast controller was its
/// only long-lived state (phase 2 IN-04).
final engineProvider = Provider<DictaEngine>((ref) =>
    DaemonEngine(resolveSocketPath()));

final dictaStatusProvider =
    NotifierProvider<DictaStatusNotifier, DictaStatus>(DictaStatusNotifier.new);

/// HF transport seam - overridden by FakeHfClient in widget tests.
final hfClientProvider = Provider<HfClient>((_) => HttpHfClient());

/// Download transport seam - overridden by FakeDownloadEngine in tests.
final downloadEngineProvider =
    Provider<DownloadEngine>((_) => HttpDownloadEngine());

/// Tray plugin seam - overridden by FakeTrayService in tests.
final trayServiceProvider = Provider<TrayService>((_) => TrayManagerService());

/// Window plugin seam - overridden by FakeWindowShell in tests.
final windowShellProvider = Provider<WindowShell>((_) => WindowManagerShell());

/// VRAM limit is UI-only state (2..32 GB, default 24). There is no daemon key:
/// this value is NEVER sent through engine.set (CONTEXT:53).
final vramLimitGbProvider =
    NotifierProvider<VramLimitNotifier, double>(VramLimitNotifier.new);

class VramLimitNotifier extends Notifier<double> {
  @override
  double build() => 24.0;

  void set(double value) => state = value.clamp(2.0, 32.0);
}

/// Settings are fetched on demand and refreshed explicitly (after set or a
/// diagnostics recheck) - never polled. The only parser is SettingsReply.fromReply.
final settingsSnapshotProvider = FutureProvider<SettingsReply>((ref) async {
  final reply = await ref.read(engineProvider).fetchSettings();
  final parsed = SettingsReply.fromReply(reply);
  // IN-02: fromReply returns (settings, error). Never silently discard the
  // error and hand out all-default settings - promote it to the provider's
  // AsyncError so diagnostics can render a distinct failure row. Note the
  // throwing contract: fetchSettings() itself may propagate a raw
  // SocketException/TimeoutException on a transport/timeout failure.
  if (parsed.error != null) {
    throw StateError('settings reply error: ${parsed.error}');
  }
  return parsed.settings;
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

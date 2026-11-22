/// Composition-root helper for the SNI tray. Lives beside main.dart — NOT in
/// core/ — because it imports features/ (dictaControlProvider and, from Task 3,
/// selectedTabProvider). No core file imports features/.
///
/// Only prod-main constructs this; buildFakeApp/buildTestApp never touch a
/// platform channel. No timers, sockets or second poll loop live here: the icon
/// and menu ride on the existing dictaStatusProvider / DictaControlNotifier.
library;

import 'package:dicta_app/core/providers.dart';
import 'package:dicta_app/core/tray.dart';
import 'package:dicta_app/core/window_shell.dart';
import 'package:dicta_app/features/dashboard/dicta_control.dart';
import 'package:dicta_app/features/shell/shell.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

class TrayController {
  TrayController(this._container, this._tray, this._window);

  final ProviderContainer _container;
  final TrayService _tray;
  final WindowShell _window;

  /// Last icon installed, so the 100 ms status tick only swaps on transitions.
  TrayIconState? _lastIcon;

  Future<void> start() async {
    // Icon and menu before anything else (labels are set inside the service).
    await _tray.start();
    await _tray.setIconState(TrayIconState.idle);
    _lastIcon = TrayIconState.idle;

    // The icon rides on the EXISTING 100 ms status poll — no new timer/socket.
    _container.listen(dictaStatusProvider, (_, next) {
      final target =
          next.connected ? iconFor(next.state) : TrayIconState.idle;
      if (target == _lastIcon) return;
      _lastIcon = target;
      _tray.setIconState(target);
    });

    // TRAY-02: the X is a vetoed close request, not an exit. destroy() bypasses
    // the veto, so it is deliberately reachable ONLY from the Exit item below.
    await _window.preventClose();
    _window.onCloseRequest(() => _window.hide());

    // Linux SNI: the panel owns the click and opens the exported menu, so the
    // "left click = show/hide" intent is a menu item (accepted deviation,
    // orchestrator 2026-11-22). All commands go through the existing control
    // notifier with its commandInFlight serialization — no direct engine use.
    _tray.onMenuShowHide(() async {
      if (await _window.isVisible()) {
        await _window.hide();
      } else {
        await _window.show();
      }
    });
    _tray.onMenuStart(
        () => _container.read(dictaControlProvider.notifier).start());
    _tray.onMenuStop(
        () => _container.read(dictaControlProvider.notifier).stop());
    _tray.onMenuCancel(
        () => _container.read(dictaControlProvider.notifier).cancel());
    _tray.onMenuSettings(() async {
      await _window.show();
      // Rail order is fixed by phase 1: Settings is destination 3.
      _container.read(selectedTabProvider.notifier).select(3);
    });
    _tray.onMenuExit(() async {
      await _tray.dispose();
      await _window.destroy();
    });
  }
}

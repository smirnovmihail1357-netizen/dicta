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
import 'package:dicta_app/l10n/generated/app_localizations.dart';
import 'package:flutter/material.dart'
    show GlobalKey, Locale, ScaffoldMessengerState, SnackBar, Text;
import 'package:flutter_riverpod/flutter_riverpod.dart';

class TrayController {
  TrayController(
    this._container,
    this._tray,
    this._window, {
    this.messengerKey,
  });

  final ProviderContainer _container;
  final TrayService _tray;
  final WindowShell _window;
  final GlobalKey<ScaffoldMessengerState>? messengerKey;

  /// Last icon installed, so the 100 ms status tick only swaps on transitions.
  TrayIconState? _lastIcon;

  /// WR-01: the native menu is a fourth command path beside the dashboard
  /// buttons. A double click must not send a second command to the daemon.
  /// Dart's single-threaded event loop makes this check-and-set atomic between
  /// two callbacks fired before the first command resolves.
  bool _menuCommandInFlight = false;

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
    _tray.onMenuStart(() => _command((n) => n.start()));
    _tray.onMenuStop(() => _command((n) => n.stop()));
    _tray.onMenuCancel(() => _command((n) => n.cancel()));
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

  /// WR-01 guard: drop a second click while the first command is still in
  /// flight. The flag is released in [finally] so a thrown command can never
  /// leave the menu permanently disabled.
  void _command(Future<void> Function(DictaControlNotifier notifier) run) {
    if (_menuCommandInFlight) return;
    _menuCommandInFlight = true;
    _runCommand(run);
  }

  Future<void> _runCommand(
    Future<void> Function(DictaControlNotifier notifier) run,
  ) async {
    try {
      // Respect a command already started from the dashboard too: both paths
      // share the notifier's one-at-a-time rule.
      if (_container.read(dictaControlProvider).commandInFlight) return;
      await run(_container.read(dictaControlProvider.notifier));
      await _reportCommandError();
    } finally {
      _menuCommandInFlight = false;
    }
  }

  /// WR-01: a daemon reply error must not be silent in the tray path. Show it
  /// through a snackbar while the window is visible; a hidden window has no
  /// honest surface (the SNI icon cannot carry an error) — skip deliberately.
  Future<void> _reportCommandError() async {
    final control = _container.read(dictaControlProvider);
    final error = control.lastError;
    if ((error == null || error.isEmpty) && !control.genericFailure) return;
    if (!await _window.isVisible()) return;

    final l10n = lookupAppLocalizations(const Locale('ru'));
    // Daemon error text is data; a command that threw before a reply uses the
    // localised body — same rule as the dashboard's error card.
    final body = control.genericFailure ? l10n.dashCommandFailedBody : error!;
    messengerKey?.currentState?.showSnackBar(SnackBar(content: Text(body)));
  }
}

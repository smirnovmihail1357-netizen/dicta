/// The SNI tray seam. The plugin transport lives only in this file; the
/// bootstrap and features talk to the abstract interface so tests can inject
/// FakeTrayService and never open a platform channel (mirror of
/// core/hf/hf_client.dart, APP-03).
library;

import 'package:dicta_app/l10n/generated/app_localizations.dart';
import 'package:flutter/material.dart' show Locale;

import 'engine.dart';
import 'package:tray_manager/tray_manager.dart'
    show
        ContextMenuTrigger,
        ImageAsset,
        Menu,
        MenuItem,
        MenuItemClickedEvent,
        MenuItemType,
        TrayIcon;

/// Only two visual states: recording AND transcribing share the red icon
/// (orchestrator decision 2026-11-22), everything else is the blue idle icon.
enum TrayIconState { idle, recording }

/// Pure state → icon mapping. The busy part of the recording cycle (recording
/// AND transcribing) shares the red icon; everything else is idle blue.
/// Disconnected snapshots are mapped by the caller (they have no red state).
TrayIconState iconFor(DaemonState state) => switch (state) {
      DaemonState.recording || DaemonState.transcribing =>
        TrayIconState.recording,
      _ => TrayIconState.idle,
    };

abstract interface class TrayService {
  /// Creates the icon, attaches the menu, makes it visible.
  Future<void> start();

  /// Sets the icon by name. Idempotent: repeated states are a no-op.
  Future<void> setIconState(TrayIconState state);

  void onMenuShowHide(void Function() cb);
  void onMenuStart(void Function() cb);
  void onMenuStop(void Function() cb);
  void onMenuCancel(void Function() cb);
  void onMenuSettings(void Function() cb);
  void onMenuExit(void Function() cb);

  Future<void> dispose();
}

/// The single place that knows `tray_manager`. Holding the [TrayIcon] reference
/// is mandatory: if it is garbage-collected the panel drops the icon.
class TrayManagerService implements TrayService {
  static const _idleAsset = 'assets/tray/idle.png';
  static const _recordingAsset = 'assets/tray/recording.png';

  TrayIcon? _trayIcon;
  TrayIconState? _lastIconState;
  final Map<String, void Function()> _callbacks = <String, void Function()>{};

  @override
  Future<void> start() async {
    final icon = TrayIcon.create();
    if (icon == null) throw StateError('Unable to create the tray icon');
    _trayIcon = icon;

    // Native menu labels live outside the widget tree: resolve them the same
    // way main.dart:14 resolves the window title (no literal Cyrillic in lib/).
    final l10n = lookupAppLocalizations(const Locale('ru'));

    icon
      ..icon = ImageAsset.fromAsset(_idleAsset)
      ..setTooltip(l10n.appTitle);
    _lastIconState = TrayIconState.idle;

    final menu = Menu.create();
    if (menu == null) throw StateError('Unable to create the tray menu');

    void addItem(String key, String label) {
      final item = MenuItem.createWithLabelAndType(label, MenuItemType.normal);
      if (item == null) return;
      item.addListener((event) {
        if (event is MenuItemClickedEvent) _callbacks[key]?.call();
      });
      menu.addItem(item);
    }

    // Actions | window | exit.
    addItem('start', l10n.trayStart);
    addItem('stop', l10n.trayStop);
    addItem('cancel', l10n.trayCancel);
    menu.addSeparator();
    addItem('showHide', l10n.trayShowHide);
    addItem('settings', l10n.traySettings);
    menu.addSeparator();
    addItem('exit', l10n.trayExit);

    icon
      ..setContextMenu(menu)
      // Linux default is `none`: without an explicit trigger KDE never shows
      // the exported menu (research Pitfall 1 / Anti-Pattern).
      ..setContextMenuTrigger(ContextMenuTrigger.rightClicked)
      ..setVisible(true);
  }

  @override
  Future<void> setIconState(TrayIconState state) async {
    // Guard by the last installed name: the 100 ms status tick must not spam
    // native icon swaps ten times a second (validation Sampling).
    //
    // WR-03 LIVE PROOF (2026-11-22, KDE Plasma 6 SNI host): with a live daemon
    // and the built bundle, gdbus Properties.Get(IconPixmap) on this item showed
    // 208 x #35689F blue / 0 x #E53935 red at idle, 0 / 208 after firing the
    // DBusMenu "start" item, and 208 / 0 again after "cancel". A plain
    // `trayIcon.icon = ImageAsset.fromAsset(...)` DOES repaint the panel, so the
    // A1 destroy+create fallback is deliberately NOT added: re-registering the
    // SNI item on every transition would only add risk. Repeat this probe if
    // the panel/SNI host changes.
    if (state == _lastIconState) return;
    final icon = _trayIcon;
    if (icon == null) return;
    icon.icon = ImageAsset.fromAsset(
      state == TrayIconState.recording ? _recordingAsset : _idleAsset,
    );
    _lastIconState = state;
  }

  @override
  void onMenuShowHide(void Function() cb) => _callbacks['showHide'] = cb;

  @override
  void onMenuStart(void Function() cb) => _callbacks['start'] = cb;

  @override
  void onMenuStop(void Function() cb) => _callbacks['stop'] = cb;

  @override
  void onMenuCancel(void Function() cb) => _callbacks['cancel'] = cb;

  @override
  void onMenuSettings(void Function() cb) => _callbacks['settings'] = cb;

  @override
  void onMenuExit(void Function() cb) => _callbacks['exit'] = cb;

  @override
  Future<void> dispose() async {
    final icon = _trayIcon;
    _trayIcon = null;
    _lastIconState = null;
    _callbacks.clear();
    // The menu is owned by the native icon; the process is exiting at this
    // point (it is called only from the menu's Exit item), so no explicit
    // menu.dispose() here — freeing an item inside its own callback is unsafe.
    icon?.dispose();
  }
}

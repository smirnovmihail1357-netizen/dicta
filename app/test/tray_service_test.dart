// TrayController unit coverage through the seams: no plugin channel is ever
// opened. The fake panel invokes the six registered menu callbacks directly.
import 'package:dicta_app/core/engine.dart';
import 'package:dicta_app/core/providers.dart';
import 'package:dicta_app/core/tray.dart';
import 'package:dicta_app/tray_bootstrap.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';

import 'fake_dicta_engine.dart';
import 'fake_tray_service.dart';
import 'fake_window_shell.dart';

void main() {
  late ProviderContainer container;
  late FakeTrayService tray;
  late FakeWindowShell window;
  late FakeDictaEngine daemon;

  Future<void> settle() => Future<void>.delayed(const Duration(milliseconds: 1));

  setUp(() async {
    tray = FakeTrayService();
    window = FakeWindowShell();
    daemon = FakeDictaEngine(
      const DictaStatus(connected: true, state: DaemonState.idle),
    );
    container = ProviderContainer(
      overrides: [
        engineProvider.overrideWithValue(daemon),
        trayServiceProvider.overrideWithValue(tray),
        windowShellProvider.overrideWithValue(window),
      ],
    );
    addTearDown(container.dispose);
    await TrayController(container, tray, window).start();
  });

  test('start boots the icon and vetoes the native close', () {
    expect(tray.startCount, 1);
    expect(tray.iconCalls, <TrayIconState>[TrayIconState.idle]);
    expect(window.preventCloseCount, 1);
    expect(window.onCloseCallback, isNotNull);
    for (final key in [
      'showHide',
      'start',
      'stop',
      'cancel',
      'settings',
      'exit',
    ]) {
      expect(tray.hasCallback(key), isTrue, reason: 'menu item $key unwired');
    }
  });

  test('TRAY-02: the X hides the window and never destroys it', () async {
    window.clickCloseButton();
    await settle();
    expect(window.hideCount, 1);
    expect(window.destroyCount, 0);
  });

  test('honest exit: tray disposed first, then window destroyed', () async {
    tray.clickExit();
    await settle();
    expect(tray.disposeCount, 1);
    expect(window.destroyCount, 1);
  });

  test('show/hide menu item toggles both directions', () async {
    // FakeWindowShell starts visible.
    tray.clickShowHide();
    await settle();
    expect(window.hideCount, 1);
    tray.clickShowHide();
    await settle();
    expect(window.showCount, 1);
    expect(window.visible, isTrue);
  });

  test('menu commands reach the fake daemon through DictaControlNotifier', () {
    tray.clickStart();
    expect(daemon.startCalls, 1);
    tray.clickStop();
    expect(daemon.stopCalls, 1);
    tray.clickCancel();
    expect(daemon.cancelCalls, 1);
  });

  test('settings menu item shows the window', () async {
    await window.hide();
    tray.clickSettings();
    await settle();
    expect(window.showCount, 1);
    expect(window.visible, isTrue);
  });
}

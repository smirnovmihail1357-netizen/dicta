// TrayController unit coverage through the seams: no plugin channel is ever
// opened. The fake panel invokes the six registered menu callbacks directly.
import 'package:dicta_app/core/engine.dart';
import 'package:dicta_app/core/providers.dart';
import 'package:dicta_app/core/tray.dart';
import 'package:dicta_app/features/shell/shell.dart';
import 'package:dicta_app/tray_bootstrap.dart';
import 'package:flutter/material.dart';
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

  test('menu commands reach the fake daemon through DictaControlNotifier',
      () async {
    // WR-01 serializes menu commands: wait for each one to resolve before
    // issuing the next, exactly as a user clicking a settled menu would.
    tray.clickStart();
    await settle();
    expect(daemon.startCalls, 1);
    tray.clickStop();
    await settle();
    expect(daemon.stopCalls, 1);
    tray.clickCancel();
    await settle();
    expect(daemon.cancelCalls, 1);
  });

  test('WR-01: a rapid double click issues only one command', () async {
    tray.clickStart();
    tray.clickStart();
    await settle();
    expect(daemon.startCalls, 1);

    // The guard is released on completion; a later deliberate click works.
    tray.clickStop();
    await settle();
    expect(daemon.stopCalls, 1);
  });

  testWidgets('WR-01: a reply error surfaces in a snackbar, not silence',
      (tester) async {
    final messengerKey = GlobalKey<ScaffoldMessengerState>();
    final errorDaemon = FakeDictaEngine(
      const DictaStatus(connected: true, state: DaemonState.idle),
      startReply: const DictaReply(ok: false, error: 'daemon refused'),
    );
    final errorTray = FakeTrayService();
    final errorContainer = ProviderContainer(overrides: [
      engineProvider.overrideWithValue(errorDaemon),
      trayServiceProvider.overrideWithValue(errorTray),
      windowShellProvider.overrideWithValue(window),
    ]);
    addTearDown(errorContainer.dispose);
    await TrayController(errorContainer, errorTray, window,
        messengerKey: messengerKey).start();

    await tester.pumpWidget(buildFakeApp(
      errorDaemon,
      scaffoldMessengerKey: messengerKey,
      home: const Scaffold(body: SizedBox.expand()),
    ));
    errorTray.clickStart();
    for (var i = 0; i < 5; i++) {
      await tester.pump();
      await tester.pump(const Duration(milliseconds: 20));
    }
    expect(find.text('daemon refused'), findsOneWidget);
  });

  test('settings menu item shows the window and selects the Settings tab',
      () async {
    await window.hide();
    tray.clickSettings();
    await settle();
    expect(window.showCount, 1);
    expect(window.visible, isTrue);
    expect(container.read(selectedTabProvider), 3);
  });

  test('icon follows status transitions only (guard, red on transcribing)',
      () async {
    final status = container.read(dictaStatusProvider.notifier);

    status.state = const DictaStatus(
        connected: true, state: DaemonState.recording);
    expect(tray.iconCalls.last, TrayIconState.recording);

    // Repeating the same state is a no-op: no 10 Hz icon spam.
    final count = tray.iconCalls.length;
    status.state = const DictaStatus(
        connected: true, state: DaemonState.recording);
    expect(tray.iconCalls.length, count);

    // Transcribing maps to red too; from idle it must install the red icon.
    status.state =
        const DictaStatus(connected: true, state: DaemonState.idle);
    expect(tray.iconCalls.last, TrayIconState.idle);
    status.state = const DictaStatus(
        connected: true, state: DaemonState.transcribing);
    expect(tray.iconCalls.last, TrayIconState.recording);
    expect(tray.iconCalls.length, count + 2);

    // recording -> transcribing are both red: the guard suppresses the swap.
    status.state = const DictaStatus(
        connected: true, state: DaemonState.recording);
    expect(tray.iconCalls.length, count + 2);

    status.state =
        const DictaStatus(connected: true, state: DaemonState.idle);
    expect(tray.iconCalls.last, TrayIconState.idle);

    // A dead daemon maps to idle blue and does not add a redundant call.
    final beforeDisconnect = tray.iconCalls.length;
    status.state = const DictaStatus.disconnected();
    expect(tray.iconCalls.last, TrayIconState.idle);
    expect(tray.iconCalls.length, beforeDisconnect);
  });
}

// HK-01 coverage: the card reads config.toml, never writes it, and is honest
// about the daemon's default "off". No plugin channels (buildFakeApp seam).
import 'dart:io';

import 'package:dicta_app/core/engine.dart';
import 'package:dicta_app/features/settings/hotkey_card.dart';
import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart' show RenderSemanticsAnnotations;
import 'package:flutter_test/flutter_test.dart';

import 'fake_dicta_engine.dart';

void main() {
  late Directory dir;
  late String examplePath;
  late String livePath;

  setUp(() {
    dir = Directory.systemTemp.createTempSync('dicta_hk_');
    examplePath = '${dir.path}/config_example.toml';
    livePath = '${dir.path}/config_live.toml';
    File(examplePath)
        .writeAsStringSync(File('test/fixtures/config_example_toml.txt')
            .readAsStringSync());
    File(livePath).writeAsStringSync(
        File('test/fixtures/config_live_no_hotkey.txt').readAsStringSync());
  });

  tearDown(() => dir.deleteSync(recursive: true));

  Widget cardFor(String path) => buildFakeApp(
        FakeDictaEngine(
          const DictaStatus(connected: true, state: DaemonState.idle),
          // fetchSettings() returns the full reply: SettingsReply.fromReply
          // reads the nested `settings` map (fixtures/settings_reply.json).
          fetchSettingsReply: <String, dynamic>{
            'settings': <String, dynamic>{'config_path': path},
          },
        ),
        home: const HotkeyCard(),
      );

  Future<void> pumpCard(WidgetTester tester, String path) async {
    await tester.pumpWidget(cardFor(path));
    // File.readAsString is real I/O: it only completes inside runAsync in the
    // fake-async widget-test zone, then the provider rebuild needs a pump.
    for (var i = 0; i < 15; i++) {
      await tester.runAsync(
          () => Future<void>.delayed(const Duration(milliseconds: 20)));
      await tester.pump();
    }
  }

  testWidgets('live-like config shows off, restart warning and DE table',
      (tester) async {
    await pumpCard(tester, livePath);

    expect(find.text('не задан (off)'), findsOneWidget);
    expect(find.text('изменение применится после рестарта демона'),
        findsOneWidget);
    expect(find.text('KWin: Alt+V — ./systemd/install.sh && '
            './adapters/kwin/install.sh'),
        findsOneWidget);
    expect(find.text('Hyprland: include конфигурации адаптера'),
        findsOneWidget);
    expect(find.text('Sway: include конфигурации адаптера'), findsOneWidget);
    expect(find.text("GNOME: apply.sh '<Ctrl><Alt>v'"), findsOneWidget);
  });

  testWidgets('example config shows the verbatim evdev combination',
      (tester) async {
    await pumpCard(tester, examplePath);

    expect(find.text('ctrl+alt+v'), findsOneWidget);
    expect(find.text('не задан (off)'), findsNothing);
  });

  testWidgets('unreadable config shows an honest error with retry',
      (tester) async {
    await pumpCard(tester, '${dir.path}/missing/config.toml');

    expect(find.text('не удалось прочитать конфиг демона'), findsOneWidget);
    expect(find.text('повторить'), findsOneWidget);
  });

  testWidgets('config read error is announced to AT (IN-03)', (tester) async {
    await pumpCard(tester, '${dir.path}/missing/config.toml');

    final errorRegion = find.byKey(const Key('hkConfigError'));
    expect(errorRegion, findsOneWidget);
    expect(
      tester.renderObject<RenderSemanticsAnnotations>(errorRegion)
          .properties
          .liveRegion,
      isTrue,
    );
  });

  testWidgets('card has no editable hotkey field', (tester) async {
    await pumpCard(tester, examplePath);

    expect(
      find.descendant(
        of: find.byKey(const Key('hotkeyCard')),
        matching: find.byType(TextField),
      ),
      findsNothing,
    );
  });
}

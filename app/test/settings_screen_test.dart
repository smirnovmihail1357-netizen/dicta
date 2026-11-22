// TRACER: one settings key end-to-end through the real seam
// (slider -> set -> SetReply snapshot -> form, no refetch).
import 'dart:async';
import 'dart:convert';
import 'dart:io';

import 'package:dicta_app/core/engine.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'fake_dicta_engine.dart';

Future<void> tick(WidgetTester tester) async {
  await tester.pump(const Duration(milliseconds: 100));
  await tester.pump();
}

Future<void> openSettings(WidgetTester tester) async {
  await tester.tap(find.text('Настройки'));
  await tester.pump();
}

Text readout(WidgetTester tester) =>
    tester.widget<Text>(find.byKey(const Key('confPasteDelayReadout')));

double sliderValue(WidgetTester tester) => tester
    .widget<Slider>(find.byKey(const Key('confPasteDelaySlider')))
    .value;

Map<String, dynamic> fixtureWith(
  Map<String, dynamic> base,
  Map<String, dynamic> overrides,
) {
  final settings = Map<String, dynamic>.from(
    base['settings'] as Map<String, dynamic>,
  );
  settings.addAll(overrides);
  return <String, dynamic>{...base, 'settings': settings};
}

void main() {
  late Map<String, dynamic> settingsFixture;

  setUpAll(() {
    final raw = File('test/fixtures/settings_reply.json').readAsStringSync();
    settingsFixture = jsonDecode(raw) as Map<String, dynamic>;
  });

  testWidgets('settings screen opens with the slider at the snapshot value',
      (tester) async {
    final fake = FakeDictaEngine(
      const DictaStatus(connected: true, state: DaemonState.idle),
      fetchSettingsReply: settingsFixture,
    );
    await tester.pumpWidget(buildFakeApp(fake));
    await tick(tester);
    await openSettings(tester);

    expect(sliderValue(tester), 150);
    expect(readout(tester).data, '150');

    await tester.pumpWidget(const SizedBox.shrink());
  });

  testWidgets('timed drag commits exactly one set with an integer value',
      (tester) async {
    final fake = FakeDictaEngine(
      const DictaStatus(connected: true, state: DaemonState.idle),
      fetchSettingsReply: settingsFixture,
    );
    await tester.pumpWidget(buildFakeApp(fake));
    await tick(tester);
    await openSettings(tester);

    final slider = find.byKey(const Key('confPasteDelaySlider'));
    await tester.ensureVisible(slider);
    await tester.pump();
    await tester.timedDrag(slider, const Offset(80, 0), const Duration(milliseconds: 300));
    await tester.pump();
    await tester.pump(const Duration(milliseconds: 100));

    expect(fake.setCalls.length, 1);
    expect(fake.setCalls.single.key, 'paste_delay_ms');
    expect(fake.setCalls.single.value, matches(r'^\d+$'));

    await tester.pumpWidget(const SizedBox.shrink());
  });

  testWidgets('success note is neutral status (never a red error) and the snapshot applies',
      (tester) async {
    // Real daemon shape: text is ALWAYS non-null on success, error stays null.
    const note =
        'paste_delay_ms = 800 — сохранено в /home/user/.config/dicta/config.toml';
    late final FakeDictaEngine fake;
    fake = FakeDictaEngine(
      const DictaStatus(connected: true, state: DaemonState.idle),
      fetchSettingsReply: settingsFixture,
      setReplyMap: const {
        'text': note,
        'settings': <String, dynamic>{'paste_delay_ms': 900},
      },
    );
    await tester.pumpWidget(buildFakeApp(fake));
    await tick(tester);
    await openSettings(tester);

    final slider = tester.widget<Slider>(find.byKey(const Key('confPasteDelaySlider')));
    slider.onChanged!(800);
    await tester.pump();
    final fetchCountBeforeSet = fake.fetchCount;
    slider.onChangeEnd!(800);
    await tester.pump();
    await tester.pump(const Duration(milliseconds: 100));

    expect(fake.setCalls.single.value, '800');
    // The canonical value comes from the reply, not from the local preview.
    expect(readout(tester).data, '900');
    expect(sliderValue(tester), 900);
    expect(fake.fetchCount, fetchCountBeforeSet);

    // The success note is shown verbatim in the liveRegion, but it is DATA:
    // neutral styling, not the red error line.
    expect(find.text(note), findsOneWidget);
    final status = find.byKey(const Key('confStatus'));
    final errorColor = Theme.of(tester.element(status)).colorScheme.error;
    expect(tester.widget<Text>(status).style?.color, isNot(errorColor));

    await tester.pumpWidget(const SizedBox.shrink());
  });

  testWidgets('refusal error is verbatim and the local slider stays put',
      (tester) async {
    const refusal = 'paste_delay_ms: 5000 вне диапазона 0..3000';
    final fake = FakeDictaEngine(
      const DictaStatus(connected: true, state: DaemonState.idle),
      fetchSettingsReply: settingsFixture,
      setReplyMap: const {'error': refusal},
    );
    await tester.pumpWidget(buildFakeApp(fake));
    await tick(tester);
    await openSettings(tester);

    final slider = tester.widget<Slider>(find.byKey(const Key('confPasteDelaySlider')));
    slider.onChanged!(900);
    await tester.pump();
    slider.onChangeEnd!(900);
    await tester.pump();

    expect(find.text(refusal), findsOneWidget);
    expect(sliderValue(tester), 900);
    // Only a real top-level error gets the red error styling.
    final status = find.byKey(const Key('confStatus'));
    final errorColor = Theme.of(tester.element(status)).colorScheme.error;
    expect(tester.widget<Text>(status).style?.color, errorColor);

    await tester.pumpWidget(const SizedBox.shrink());
  });

  // ---- Task 4: full 17-key form -------------------------------------------

  testWidgets('all four groups and every editable control are present',
      (tester) async {
    final fake = FakeDictaEngine(
      const DictaStatus(connected: true, state: DaemonState.idle),
      fetchSettingsReply: settingsFixture,
    );
    await tester.pumpWidget(buildFakeApp(fake));
    await tick(tester);
    await openSettings(tester);

    for (final title in ['запись', 'вставка', 'движок', 'система']) {
      expect(find.text(title), findsOneWidget);
    }
    for (final key in [
      'confMinLevelSlider',
      'confAutoStopSwitch',
      'confAutoStopLevelSlider',
      'confAutoStopAfterSlider',
      'confAutoStopGraceSlider',
      'confMaxRecordSlider',
      'confInjectModeSegments',
      'confPasteKeysPresets',
      'confPasteKeysField',
      'confClipboardRestoreSlider',
      'confPasteDelaySlider',
      'confAtspiSwitch',
      'confModelDropdown',
      'confUseGpuSwitch',
      'confGpuDeviceDropdown',
      'confThreadsSlider',
      'confBeamSlider',
      'confLanguageSegments',
      'confConfigPathField',
    ]) {
      expect(find.byKey(Key(key)), findsOneWidget, reason: key);
    }
    expect(
      tester.widget<TextField>(find.byKey(const Key('confConfigPathField')))
          .readOnly,
      isTrue,
    );

    await tester.pumpWidget(const SizedBox.shrink());
  });

  testWidgets('min_level slider commits exactly one formatted set on release',
      (tester) async {
    final fake = FakeDictaEngine(
      const DictaStatus(connected: true, state: DaemonState.idle),
      fetchSettingsReply: settingsFixture,
    );
    await tester.pumpWidget(buildFakeApp(fake));
    await tick(tester);
    await openSettings(tester);

    final slider = tester.widget<Slider>(find.byKey(const Key('confMinLevelSlider')));
    slider.onChanged!(0.42);
    await tester.pump();
    expect(fake.setCalls, isEmpty);
    slider.onChangeEnd!(0.42);
    await tester.pump();

    expect(fake.setCalls.length, 1);
    expect(fake.setCalls.single.key, 'min_level');
    expect(fake.setCalls.single.value, '0.42');

    await tester.pumpWidget(const SizedBox.shrink());
  });

  testWidgets('bool switch sends on/off strings', (tester) async {
    final fake = FakeDictaEngine(
      const DictaStatus(connected: true, state: DaemonState.idle),
      fetchSettingsReply: settingsFixture,
    );
    await tester.pumpWidget(buildFakeApp(fake));
    await tick(tester);
    await openSettings(tester);

    tester.widget<SwitchListTile>(find.byKey(const Key('confAutoStopSwitch')))
        .onChanged!(true);
    await tester.pump();
    expect(fake.setCalls.single.key, 'auto_stop');
    expect(fake.setCalls.single.value, 'on');

    tester.widget<SwitchListTile>(find.byKey(const Key('confAutoStopSwitch')))
        .onChanged!(false);
    await tester.pump();
    expect(fake.setCalls.last.value, 'off');

    await tester.pumpWidget(const SizedBox.shrink());
  });

  testWidgets('inject_mode segmented sends its four daemon strings',
      (tester) async {
    final fake = FakeDictaEngine(
      const DictaStatus(connected: true, state: DaemonState.idle),
      fetchSettingsReply: settingsFixture,
    );
    await tester.pumpWidget(buildFakeApp(fake));
    await tick(tester);
    await openSettings(tester);

    for (final value in ['paste', 'clipboard', 'stdout', 'atspi']) {
      tester
          .widget<SegmentedButton<String>>(
            find.byKey(const Key('confInjectModeSegments')),
          )
          .onSelectionChanged!({value});
      await tester.pump();
    }

    expect(fake.setCalls.length, 4);
    expect(
      fake.setCalls.map((call) => call.value).toList(),
      ['paste', 'clipboard', 'stdout', 'atspi'],
    );
    expect(fake.setCalls.every((call) => call.key == 'inject_mode'), isTrue);

    await tester.pumpWidget(const SizedBox.shrink());
  });

  testWidgets('ENGINE switch waits for confirm; decline sends nothing',
      (tester) async {
    final fake = FakeDictaEngine(
      const DictaStatus(connected: true, state: DaemonState.idle),
      fetchSettingsReply: settingsFixture,
    );
    await tester.pumpWidget(buildFakeApp(fake));
    await tick(tester);
    await openSettings(tester);

    tester.widget<SwitchListTile>(find.byKey(const Key('confUseGpuSwitch')))
        .onChanged!(false);
    await tester.pump();
    expect(find.byType(AlertDialog), findsOneWidget);
    expect(
      find.text(
        'изменение перезагрузит модель — на секунды диктовка будет недоступна',
      ),
      findsOneWidget,
    );
    expect(fake.setCalls, isEmpty);

    await tester.tap(find.text('отмена'));
    await tester.pump();
    expect(fake.setCalls, isEmpty);

    tester.widget<SwitchListTile>(find.byKey(const Key('confUseGpuSwitch')))
        .onChanged!(false);
    await tester.pump();
    await tester.tap(find.text('продолжить'));
    await tester.pump();
    await tester.pump(const Duration(milliseconds: 100));

    expect(fake.setCalls.length, 1);
    expect(fake.setCalls.single.key, 'use_gpu');

    await tester.pumpWidget(const SizedBox.shrink());
  });

  testWidgets('language is an ENGINE key and needs confirm too', (tester) async {
    final fake = FakeDictaEngine(
      const DictaStatus(connected: true, state: DaemonState.idle),
      fetchSettingsReply: settingsFixture,
    );
    await tester.pumpWidget(buildFakeApp(fake));
    await tick(tester);
    await openSettings(tester);

    tester
        .widget<SegmentedButton<String>>(
          find.byKey(const Key('confLanguageSegments')),
        )
        .onSelectionChanged!({'en'});
    await tester.pump();
    expect(fake.setCalls, isEmpty);
    await tester.tap(find.text('продолжить'));
    await tester.pump();

    expect(fake.setCalls.length, 1);
    expect(fake.setCalls.single.key, 'language');
    expect(fake.setCalls.single.value, 'en');

    await tester.pumpWidget(const SizedBox.shrink());
  });

  testWidgets('engine refusal arrives as note at error:None and is verbatim',
      (tester) async {
    const refusal =
        'идёт запись или распознавание — смену модели отложи; заверши диктовку (Alt+V) и повтори';
    final fake = FakeDictaEngine(
      const DictaStatus(connected: true, state: DaemonState.idle),
      fetchSettingsReply: fixtureWith(settingsFixture, {'reloading': false}),
      setReplyMap: <String, dynamic>{
        'text': refusal,
        'settings': settingsFixture['settings'],
      },
    );
    await tester.pumpWidget(buildFakeApp(fake));
    await tick(tester);
    await openSettings(tester);

    tester.widget<SwitchListTile>(find.byKey(const Key('confAutoStopSwitch')))
        .onChanged!(true);
    await tester.pump();

    expect(find.text(refusal), findsOneWidget);
    expect(find.byKey(const Key('confStatus')), findsOneWidget);

    await tester.pumpWidget(const SizedBox.shrink());
  });

  testWidgets('reloading snapshot shows a spinner and disables ENGINE controls',
      (tester) async {
    final fake = FakeDictaEngine(
      const DictaStatus(connected: true, state: DaemonState.idle),
      fetchSettingsReply: fixtureWith(settingsFixture, {'reloading': true}),
    );
    await tester.pumpWidget(buildFakeApp(fake));
    await tick(tester);
    await openSettings(tester);

    expect(find.byKey(const Key('confReloadSpinner')), findsOneWidget);
    expect(find.text('модель загружается…'), findsOneWidget);
    expect(
      tester.widget<SwitchListTile>(find.byKey(const Key('confUseGpuSwitch')))
          .onChanged,
      isNull,
    );

    await tester.pumpWidget(const SizedBox.shrink());
  });

  testWidgets('engine slider ranges match the daemon, not the reference UI',
      (tester) async {
    final fake = FakeDictaEngine(
      const DictaStatus(connected: true, state: DaemonState.idle),
      fetchSettingsReply: settingsFixture,
    );
    await tester.pumpWidget(buildFakeApp(fake));
    await tick(tester);
    await openSettings(tester);

    final threads =
        tester.widget<Slider>(find.byKey(const Key('confThreadsSlider')));
    expect(threads.min, 1);
    expect(threads.max, 64);
    final beam =
        tester.widget<Slider>(find.byKey(const Key('confBeamSlider')));
    expect(beam.min, 1);
    expect(beam.max, 10);

    await tester.pumpWidget(const SizedBox.shrink());
  });

  testWidgets('paste_keys presets and free text each commit one set',
      (tester) async {
    final fake = FakeDictaEngine(
      const DictaStatus(connected: true, state: DaemonState.idle),
      fetchSettingsReply: settingsFixture,
    );
    await tester.pumpWidget(buildFakeApp(fake));
    await tick(tester);
    await openSettings(tester);

    tester
        .widget<SegmentedButton<String>>(
          find.byKey(const Key('confPasteKeysPresets')),
        )
        .onSelectionChanged!({'ctrl+shift+v'});
    await tester.pump();
    expect(fake.setCalls.single.key, 'paste_keys');
    expect(fake.setCalls.single.value, 'ctrl+shift+v');

    tester
        .widget<TextField>(find.byKey(const Key('confPasteKeysField')))
        .onSubmitted!('super+v');
    await tester.pump();
    expect(fake.setCalls.last.value, 'super+v');

    await tester.pumpWidget(const SizedBox.shrink());
  });

  testWidgets('all four unit-suffixed sliders commit raw integers on release',
      (tester) async {
    final fake = FakeDictaEngine(
      const DictaStatus(connected: true, state: DaemonState.idle),
      fetchSettingsReply: settingsFixture,
    );
    await tester.pumpWidget(buildFakeApp(fake));
    await tick(tester);
    await openSettings(tester);

    final cases = [
      (
        key: 'confAutoStopAfterSlider',
        setting: 'auto_stop_after_ms',
        value: 2000.0,
        readout: '2000',
      ),
      (
        key: 'confAutoStopGraceSlider',
        setting: 'auto_stop_grace_ms',
        value: 1300.0,
        readout: '1300',
      ),
      (
        key: 'confMaxRecordSlider',
        setting: 'max_record_ms',
        value: 30000.0,
        // Displayed as seconds; the daemon still gets raw milliseconds.
        readout: '30',
      ),
      (
        key: 'confClipboardRestoreSlider',
        setting: 'clipboard_restore_ms',
        value: 2600.0,
        readout: '2600',
      ),
    ];

    for (var i = 0; i < cases.length; i++) {
      final item = cases[i];
      final finder = find.byKey(Key(item.key));
      await tester.ensureVisible(finder);
      await tester.pump();
      final slider = tester.widget<Slider>(finder);
      final before = fake.setCalls.length;

      // Drag: the readout follows the pending RAW value (no snap-back).
      slider.onChanged!(item.value);
      await tester.pump();
      expect(fake.setCalls.length, before, reason: '${item.key} no set during drag');
      final duringDrag =
          tester.widget<Text>(find.byKey(Key('${item.key}Readout'))).data ?? '';
      expect(duringDrag, contains(item.readout), reason: '${item.key} readout moved');

      // Release: exactly one set, carrying a bare integer the daemon accepts.
      slider.onChangeEnd!(item.value);
      await tester.pump();
      expect(fake.setCalls.length, before + 1, reason: '${item.key} one set per release');
      expect(fake.setCalls.last.key, item.setting);
      expect(fake.setCalls.last.value, matches(r'^\d+$'), reason: '${item.key} raw wire');
      expect(fake.setCalls.last.value, item.value.toInt().toString());
      // The value must parse as the daemon's u64/int - no unit suffix.
      expect(int.tryParse(fake.setCalls.last.value), isNotNull);
    }

    await tester.pumpWidget(const SizedBox.shrink());
  });

  testWidgets('reload polling stops and reports honestly when the daemon dies',
      (tester) async {
    final reloadingSnapshot = <String, dynamic>{
      ...settingsFixture['settings'] as Map<String, dynamic>,
      'reloading': true,
    };
    final fake = FakeDictaEngine(
      const DictaStatus(connected: true, state: DaemonState.idle),
      fetchSettingsReply: settingsFixture,
      // The set starts reload polling with reloading:true in its snapshot.
      setReplyMap: <String, dynamic>{
        'text': 'auto_stop = on — сохранено в /tmp/config.toml',
        'settings': reloadingSnapshot,
      },
    );
    await tester.pumpWidget(buildFakeApp(fake));
    await tick(tester);
    await openSettings(tester);

    tester.widget<SwitchListTile>(find.byKey(const Key('confAutoStopSwitch')))
        .onChanged!(true);
    await tester.pump();
    await tester.pump(const Duration(milliseconds: 100));
    expect(find.byKey(const Key('confReloadSpinner')), findsOneWidget);

    // The daemon goes away mid-reload.
    fake.fetchSettingsError = StateError('socket closed');
    await tester.pump(const Duration(seconds: 1));
    await tester.pump();
    expect(find.text('демон недоступен'), findsOneWidget);
    final countAfterFailure = fake.fetchCount;

    // The timer was stopped, not left to refetch-and-throw every second.
    await tester.pump(const Duration(seconds: 3));
    await tester.pump();
    expect(fake.fetchCount, countAfterFailure);

    await tester.pumpWidget(const SizedBox.shrink());
  });

  // ---- Phase-4 UI audit top-1: honest fetch states ------------------------

  testWidgets('a failed settings fetch shows a retry card, never fabricated defaults',
      (tester) async {
    final fake = FakeDictaEngine(
      const DictaStatus(connected: true, state: DaemonState.idle),
      // A daemon error reply promotes settingsSnapshotProvider to AsyncError.
      fetchSettingsReply: const <String, dynamic>{
        'error': 'демон недоступен',
      },
    );
    await tester.pumpWidget(buildFakeApp(fake));
    await tick(tester);
    await openSettings(tester);

    expect(find.byKey(const Key('confFetchError')), findsOneWidget);
    expect(find.text('не удалось получить настройки'), findsOneWidget);
    expect(find.byKey(const Key('confFetchRetryButton')), findsOneWidget);
    // No form built out of const SettingsReply() zeros: a dead daemon must
    // never look like a fully populated "current values" form.
    expect(find.byKey(const Key('confPasteDelaySlider')), findsNothing);
    expect(find.byKey(const Key('confInjectModeSegments')), findsNothing);
    expect(find.byKey(const Key('confModelDropdown')), findsNothing);
    // A failed fetch is a transport state, not a daemon note: no status line.
    expect(find.byKey(const Key('confStatus')), findsNothing);

    await tester.pumpWidget(const SizedBox.shrink());
  });

  testWidgets('retry after a failed settings fetch refetches and shows the form',
      (tester) async {
    final fake = FakeDictaEngine(
      const DictaStatus(connected: true, state: DaemonState.idle),
      fetchSettingsReply: const <String, dynamic>{
        'error': 'демон недоступен',
      },
    );
    await tester.pumpWidget(buildFakeApp(fake));
    await tick(tester);
    await openSettings(tester);

    expect(find.byKey(const Key('confFetchError')), findsOneWidget);
    final before = fake.fetchCount;

    // The daemon comes back before the retry tap.
    fake.fetchSettingsReply = settingsFixture;
    await tester.tap(find.byKey(const Key('confFetchRetryButton')));
    await tick(tester);

    expect(fake.fetchCount, greaterThan(before));
    expect(find.byKey(const Key('confFetchError')), findsNothing);
    expect(find.byKey(const Key('confPasteDelaySlider')), findsOneWidget);
    expect(sliderValue(tester), 150);

    await tester.pumpWidget(const SizedBox.shrink());
  });

  testWidgets('in-flight settings fetch shows a spinner, not a populated form',
      (tester) async {
    final gate = Completer<void>();
    final fake = FakeDictaEngine(
      const DictaStatus(connected: true, state: DaemonState.idle),
      fetchSettingsReply: settingsFixture,
      fetchSettingsGate: gate,
    );
    await tester.pumpWidget(buildFakeApp(fake));
    await tick(tester);
    await openSettings(tester);

    expect(find.byKey(const Key('confFetchSpinner')), findsOneWidget);
    expect(find.text('настройки загружаются…'), findsOneWidget);
    expect(find.byKey(const Key('confPasteDelaySlider')), findsNothing);

    gate.complete();
    await tick(tester);
    expect(find.byKey(const Key('confPasteDelaySlider')), findsOneWidget);

    await tester.pumpWidget(const SizedBox.shrink());
  });
}

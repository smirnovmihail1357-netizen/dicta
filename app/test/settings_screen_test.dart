// TRACER: one settings key end-to-end through the real seam
// (slider -> set -> SetReply snapshot -> form, no refetch).
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
}

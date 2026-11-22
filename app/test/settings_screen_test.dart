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

  testWidgets('form updates from the set snapshot with no refetch',
      (tester) async {
    late final FakeDictaEngine fake;
    fake = FakeDictaEngine(
      const DictaStatus(connected: true, state: DaemonState.idle),
      fetchSettingsReply: settingsFixture,
      setReplyMap: const {
        'text': null,
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

    await tester.pumpWidget(const SizedBox.shrink());
  });
}

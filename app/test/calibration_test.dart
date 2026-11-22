// DASH-04: one set(min_level) per drag gesture, marker from the settings
// snapshot, calibration legal while recording, daemon error shown as data.
import 'dart:convert';
import 'dart:io';

import 'package:dicta_app/core/engine.dart';
import 'package:dicta_app/features/dashboard/calibration_card.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'fake_dicta_engine.dart';

Future<void> tick(WidgetTester tester) async {
  await tester.pump(const Duration(milliseconds: 100));
  await tester.pump();
}

Future<void> dragMarkerTo(WidgetTester tester, double fraction) async {
  await tester.ensureVisible(find.byKey(const Key('calibrationSurface')));
  await tester.pump();
  final rect = tester.getRect(find.byKey(const Key('calibrationSurface')));
  await tester.dragFrom(
    rect.topLeft + Offset(rect.width * 0.1, rect.height / 2),
    Offset(rect.width * (fraction - 0.1), 0),
  );
  await tester.pump();
  await tester.pump();
}

void main() {
  late Map<String, dynamic> settingsFixture;

  setUpAll(() {
    final raw = File('test/fixtures/settings_reply.json').readAsStringSync();
    settingsFixture = jsonDecode(raw) as Map<String, dynamic>;
  });

  testWidgets('one set(0.42) on release, zero during the drag', (tester) async {
    final fake = FakeDictaEngine(
      const DictaStatus(connected: true, state: DaemonState.idle),
      fetchSettingsReply: settingsFixture,
    );
    await tester.pumpWidget(buildFakeApp(fake));
    await tick(tester);
    await tester.ensureVisible(find.byKey(const Key('calibrationSurface')));
    await tester.pump();

    final rect = tester.getRect(find.byKey(const Key('calibrationSurface')));
    final gesture = await tester.startGesture(
      rect.topLeft + Offset(rect.width * 0.1, rect.height / 2),
    );
    await gesture.moveBy(const Offset(40, 0));
    await gesture.moveBy(Offset(rect.width * 0.32 - 40, 0));
    await tester.pump();
    expect(fake.setCalls, isEmpty);
    await gesture.up();
    await tester.pump();
    await tester.pump();

    expect(fake.setCalls.length, 1);
    expect(fake.setCalls.single.key, 'min_level');
    expect(fake.setCalls.single.value, '0.42');

    await tester.pumpWidget(const SizedBox.shrink());
  });

  testWidgets('marker starts at the settings min_level', (tester) async {
    final fake = FakeDictaEngine(
      const DictaStatus(connected: true, state: DaemonState.idle),
      fetchSettingsReply: settingsFixture,
    );
    await tester.pumpWidget(buildFakeApp(fake));
    await tick(tester);

    final painter = tester
            .widget<CustomPaint>(find.byKey(const Key('calibrationPainter')))
            .painter as LevelHistogramPainter;
    expect(painter.marker, closeTo(0.35, 1e-6));

    await tester.pumpWidget(const SizedBox.shrink());
  });

  testWidgets('marker follows the canonical snapshot AFTER a successful set '
      '(CR-01: no snap-back to the stale threshold)', (tester) async {
    late final FakeDictaEngine fake;
    fake = FakeDictaEngine(
      const DictaStatus(connected: true, state: DaemonState.idle),
      fetchSettingsReply: settingsFixture,
      // The daemon accepted 0.42; the set reply must carry the NEW snapshot,
      // otherwise the form would echo the pre-drag 0.35.
      onSet: (_, value) {
        fake.fetchSettingsReply = <String, dynamic>{
          'settings': <String, dynamic>{'min_level': double.parse(value)},
        };
      },
    );
    await tester.pumpWidget(buildFakeApp(fake));
    await tick(tester);

    LevelHistogramPainter painter() => tester
            .widget<CustomPaint>(find.byKey(const Key('calibrationPainter')))
            .painter as LevelHistogramPainter;
    expect(painter().marker, closeTo(0.35, 1e-6));

    await dragMarkerTo(tester, 0.42);
    // set -> SetReply snapshot over microtasks; no refetch.
    await tester.pump(const Duration(milliseconds: 100));
    await tester.pump();

    expect(fake.setCalls.single.value, '0.42');
    expect(painter().marker, closeTo(0.42, 1e-6));

    await tester.pumpWidget(const SizedBox.shrink());
  });

  testWidgets('calibration is not blocked while recording', (tester) async {
    final fake = FakeDictaEngine(
      const DictaStatus(
        connected: true,
        state: DaemonState.recording,
        level: 0.5,
      ),
      fetchSettingsReply: settingsFixture,
    );
    await tester.pumpWidget(buildFakeApp(fake));
    await tick(tester);
    await dragMarkerTo(tester, 0.42);

    expect(fake.setCalls.length, 1);
    expect(fake.setCalls.single.value, '0.42');

    await tester.pumpWidget(const SizedBox.shrink());
  });

  testWidgets('empty history hint gives way to live bars', (tester) async {
    final fake = FakeDictaEngine(
      const DictaStatus(connected: true, state: DaemonState.idle),
      fetchSettingsReply: settingsFixture,
    );
    await tester.pumpWidget(buildFakeApp(fake));
    await tick(tester);
    expect(
      find.text('начните запись, чтобы увидеть уровень'),
      findsOneWidget,
    );

    fake.statusValue = const DictaStatus(
      connected: true,
      state: DaemonState.recording,
      level: 0.7,
    );
    await tick(tester);
    expect(
      find.text('начните запись, чтобы увидеть уровень'),
      findsNothing,
    );
    final painter = tester
            .widget<CustomPaint>(find.byKey(const Key('calibrationPainter')))
            .painter as LevelHistogramPainter;
    expect(painter.samples, isNotEmpty);
    expect(painter.samples.last, 0.7);

    await tester.pumpWidget(const SizedBox.shrink());
  });

  testWidgets('accessible slider previews locally, then commits exactly one '
      'set on release', (tester) async {
    late final FakeDictaEngine fake;
    fake = FakeDictaEngine(
      const DictaStatus(connected: true, state: DaemonState.idle),
      fetchSettingsReply: settingsFixture,
      onSet: (_, value) {
        fake.fetchSettingsReply = <String, dynamic>{
          'settings': <String, dynamic>{'min_level': double.parse(value)},
        };
      },
    );
    await tester.pumpWidget(buildFakeApp(fake));
    await tick(tester);

    final slider = tester
        .widget<Slider>(find.byKey(const Key('calibrationSlider')));
    expect(slider.value, closeTo(0.35, 1e-6));

    // onChanged is a local preview: no I/O, but the numeric readout follows.
    slider.onChanged!(0.42);
    await tester.pump();
    expect(fake.setCalls, isEmpty);
    expect(find.text('порог 0.42'), findsOneWidget);

    // onChangeEnd is the single commit, with the panel's {:.2} format.
    final fetchCountBeforeSet = fake.fetchCount;
    slider.onChangeEnd!(0.42);
    await tester.pump();
    await tester.pump(const Duration(milliseconds: 100));
    await tester.pump();

    expect(fake.setCalls.length, 1);
    expect(fake.setCalls.single.key, 'min_level');
    expect(fake.setCalls.single.value, '0.42');
    // The readout settles on the canonical value from the set reply snapshot.
    expect(find.text('порог 0.42'), findsOneWidget);
    // Anti-refetch gate: the set reply already carried the snapshot.
    expect(fake.fetchCount, fetchCountBeforeSet);

    await tester.pumpWidget(const SizedBox.shrink());
  });

  testWidgets('threshold readout is always visible; the histogram is a '
      'read-only labelled visualisation', (tester) async {
    final fake = FakeDictaEngine(
      const DictaStatus(connected: true, state: DaemonState.idle),
      fetchSettingsReply: settingsFixture,
    );
    await tester.pumpWidget(buildFakeApp(fake));
    await tick(tester);

    expect(find.text('порог 0.35'), findsOneWidget);
    expect(find.byKey(const Key('calibrationSlider')), findsOneWidget);
    expect(find.bySemanticsLabel('гистограмма уровня звука'), findsOneWidget);

    await tester.pumpWidget(const SizedBox.shrink());
  });

  testWidgets('auto_stop hint and verbatim set error', (tester) async {
    final settings = settingsFixture['settings'] as Map<String, dynamic>;
    final autoStopFixture = <String, dynamic>{
      ...settingsFixture,
      'settings': <String, dynamic>{...settings, 'auto_stop': true},
    };
    final fake = FakeDictaEngine(
      const DictaStatus(connected: true, state: DaemonState.recording),
      fetchSettingsReply: autoStopFixture,
      setReplyMap: const {
        'error': 'min_level: 1.5 вне диапазона 0..1',
      },
    );
    await tester.pumpWidget(buildFakeApp(fake));
    await tick(tester);
    expect(
      find.text('включён авто-стоп — для калибровки нажимай отмена'),
      findsOneWidget,
    );

    await dragMarkerTo(tester, 0.42);
    expect(find.text('min_level: 1.5 вне диапазона 0..1'), findsOneWidget);

    await tester.pumpWidget(const SizedBox.shrink());
  });
}

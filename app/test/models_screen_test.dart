// MODEL-02 widget gates: installed list from settings.models, active marker,
// ENGINE confirm on select, destructive confirm + guards + refetch on delete.
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

Future<void> openModels(WidgetTester tester) async {
  await tester.tap(find.text('Модели'));
  await tester.pump();
}

Map<String, dynamic> readFixture() =>
    jsonDecode(File('test/fixtures/settings_reply.json').readAsStringSync())
        as Map<String, dynamic>;

void main() {
  testWidgets('installed list shows friendly names and marks the active model',
      (tester) async {
    final fake = FakeDictaEngine(
      const DictaStatus(connected: true, state: DaemonState.idle),
      fetchSettingsReply: readFixture(),
    );
    await tester.pumpWidget(buildFakeApp(fake));
    await tick(tester);
    await openModels(tester);

    expect(find.text('large-v3-turbo-q5_0'), findsOneWidget);
    expect(find.text('medium-q5_0'), findsOneWidget);
    expect(find.text('small-q5_1'), findsOneWidget);
    expect(find.text('активна'), findsOneWidget);
    expect(
      tester
          .widget<OutlinedButton>(
            find.byKey(const Key('modelDeleteButton-medium-q5_0')),
          )
          .onPressed,
      isNull,
    );

    await tester.pumpWidget(const SizedBox.shrink());
  });

  testWidgets('select confirms, sets the path, refetches and moves the active marker',
      (tester) async {
    final fixture = readFixture();
    final baseSettings = Map<String, dynamic>.from(fixture['settings'] as Map);
    final models = List<String>.from(baseSettings['models'] as List);
    final target = models.first; // ggml-large-v3-turbo-q5_0.bin
    late final FakeDictaEngine fake;
    fake = FakeDictaEngine(
      const DictaStatus(connected: true, state: DaemonState.idle),
      fetchSettingsReply: fixture,
      // Real daemon reply: non-null note on success, error stays null.
      setReplyMap: <String, dynamic>{
        'text': 'model = $target — сохранено в /home/user/.config/dicta/config.toml',
        'settings': fixture['settings'],
      },
      onSet: (key, value) {
        // Simulate the daemon switching the active model before the refetch.
        final switched = Map<String, dynamic>.from(baseSettings)
          ..['model'] = value;
        fake.fetchSettingsReply = <String, dynamic>{...fixture, 'settings': switched};
      },
    );
    await tester.pumpWidget(buildFakeApp(fake));
    await tick(tester);
    await openModels(tester);
    final before = fake.fetchCount;

    await tester.tap(find.byKey(const Key('modelSelectButton-large-v3-turbo-q5_0')));
    await tester.pump();
    expect(find.byType(AlertDialog), findsOneWidget);
    expect(fake.setCalls, isEmpty);

    await tester.tap(find.text('продолжить'));
    await tester.pump();
    await tester.pump(const Duration(milliseconds: 100));

    expect(fake.setCalls.length, 1);
    expect(fake.setCalls.single.key, 'model');
    expect(fake.setCalls.single.value, target);
    // The successful set triggered invalidate + await future -> refetch.
    expect(fake.fetchCount, greaterThan(before));
    expect(find.text('активна'), findsOneWidget);
    // The newly active model lost its select button / delete is disabled.
    expect(
      tester
          .widget<OutlinedButton>(
            find.byKey(const Key('modelDeleteButton-large-v3-turbo-q5_0')),
          )
          .onPressed,
      isNull,
    );

    await tester.pumpWidget(const SizedBox.shrink());
  });

  testWidgets('delete confirms, removes the temp file and refetches',
      (tester) async {
    final dir = Directory.systemTemp.createTempSync('dicta-models-');
    final activePath = '${dir.path}/ggml-active.bin';
    final victimPath = '${dir.path}/ggml-victim.bin';
    File(activePath).writeAsStringSync('active');
    File(victimPath).writeAsStringSync('victim');

    final base = readFixture();
    final settings = Map<String, dynamic>.from(base['settings'] as Map);
    settings['model'] = activePath;
    settings['models'] = [activePath, victimPath];
    final initial = <String, dynamic>{...base, 'settings': settings};

    late final FakeDictaEngine fake;
    fake = FakeDictaEngine(
      const DictaStatus(connected: true, state: DaemonState.idle),
      fetchSettingsReply: initial,
    );
    await tester.pumpWidget(buildFakeApp(fake));
    await tick(tester);
    await openModels(tester);
    final before = fake.fetchCount;

    // The test scripts the post-delete snapshot BEFORE the destructive tap.
    final shrunk = Map<String, dynamic>.from(settings);
    shrunk['models'] = [activePath];
    fake.fetchSettingsReply = <String, dynamic>{...base, 'settings': shrunk};

    await tester.tap(find.byKey(const Key('modelDeleteButton-victim')));
    await tester.pump();
    expect(find.byType(AlertDialog), findsOneWidget);
    expect(File(victimPath).existsSync(), isTrue);

    await tester.tap(
      find.descendant(of: find.byType(AlertDialog), matching: find.text('удалить')),
    );
    await tester.pump();
    // File.delete() completes on the real I/O event loop, which pump() alone
    // does not flush - let it settle, then rebuild the widget tree.
    await tester.runAsync(
      () => Future<void>.delayed(const Duration(milliseconds: 10)),
    );
    await tester.pump();
    await tester.pump(const Duration(milliseconds: 100));
    await tester.pump();

    expect(File(victimPath).existsSync(), isFalse);
    expect(find.text('удалено'), findsOneWidget);
    expect(fake.fetchCount, greaterThan(before));
    expect(find.text('victim'), findsNothing);

    dir.deleteSync(recursive: true);
    await tester.pumpWidget(const SizedBox.shrink());
  });

  testWidgets('empty model list states the honest empty state', (tester) async {
    final base = readFixture();
    final settings = Map<String, dynamic>.from(base['settings'] as Map);
    settings['models'] = <String>[];
    final fake = FakeDictaEngine(
      const DictaStatus(connected: true, state: DaemonState.idle),
      fetchSettingsReply: <String, dynamic>{...base, 'settings': settings},
    );
    await tester.pumpWidget(buildFakeApp(fake));
    await tick(tester);
    await openModels(tester);

    expect(
      find.text('моделей нет — скачай из каталога ниже'),
      findsOneWidget,
    );

    await tester.pumpWidget(const SizedBox.shrink());
  });
}

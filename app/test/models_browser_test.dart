// HF browser widget gates: catalog/VRAM, explicit search/tree fetches, direct
// link validation, one-at-a-time download panel and honest error surfaces.
// All network goes through FakeHfClient; all downloads through FakeDownloadEngine.
import 'dart:async';
import 'dart:convert';
import 'dart:io';

import 'package:dicta_app/core/engine.dart';
import 'package:dicta_app/core/hf/hf_urls.dart';
import 'package:dicta_app/core/providers.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';

import 'fake_dicta_engine.dart';
import 'fake_download_engine.dart';
import 'fake_hf_client.dart';

Future<void> tick(WidgetTester tester) async {
  await tester.pump(const Duration(milliseconds: 100));
  await tester.pump();
}

Future<void> openModels(WidgetTester tester) async {
  await tester.tap(find.text('Модели'));
  await tester.pump();
}

Future<void> reveal(WidgetTester tester, Finder finder) async {
  await tester.ensureVisible(finder);
  await tester.pump();
}

String fixture(String name) =>
    File('test/fixtures/$name.json').readAsStringSync();

Map<String, dynamic> readSettings() => jsonDecode(
      File('test/fixtures/settings_reply.json').readAsStringSync(),
    ) as Map<String, dynamic>;

Map<String, dynamic> tempFixture(Directory dir) {
  final base = readSettings();
  final settings = Map<String, dynamic>.from(base['settings'] as Map);
  final model = '${dir.path}/ggml-active.bin';
  settings['model'] = model;
  settings['models'] = <String>[model];
  return <String, dynamic>{...base, 'settings': settings};
}

void main() {
  testWidgets('catalog shows six entries, notes and installed markers',
      (tester) async {
    final fake = FakeDictaEngine(
      const DictaStatus(connected: true, state: DaemonState.idle),
      fetchSettingsReply: readSettings(),
    );
    await tester.pumpWidget(buildFakeApp(
      fake,
      hfClient: FakeHfClient(),
      downloadEngine: FakeDownloadEngine(
        scenario: FakeDownloadScenario.ok200,
      ),
    ));
    await tick(tester);
    await openModels(tester);
    await reveal(tester, find.text('ggml-tiny-q5_1.bin'));

    for (final name in [
      'ggml-tiny-q5_1.bin',
      'ggml-base-q5_1.bin',
      'ggml-small-q5_1.bin',
      'ggml-medium-q5_0.bin',
      'ggml-large-v3-turbo-q5_1.bin',
      'ggml-large-v3-q5_0.bin',
    ]) {
      expect(find.text(name), findsOneWidget, reason: name);
    }
    expect(find.textContaining('сверхбыстрая, для теста'), findsOneWidget);
    expect(find.text('уже есть'), findsNWidgets(2));

    await tester.pumpWidget(const SizedBox.shrink());
  });

  testWidgets('VRAM slider is UI-only and blocks oversized files', (tester) async {
    final fake = FakeDictaEngine(
      const DictaStatus(connected: true, state: DaemonState.idle),
      fetchSettingsReply: readSettings(),
    );
    final bigTree = jsonEncode([
      {
        'type': 'file',
        'path': 'ggml-huge.bin',
        'size': 5000000000,
      },
    ]);
    final repoUrl = treeUrlAt('org/huge', '');
    final hf = FakeHfClient(urlBody: {repoUrl: bigTree});
    late final ProviderContainer container;
    await tester.pumpWidget(buildFakeApp(
      fake,
      hfClient: hf,
      downloadEngine: FakeDownloadEngine(scenario: FakeDownloadScenario.ok200),
      captureContainer: (c) => container = c,
    ));
    await tick(tester);
    await openModels(tester);

    await reveal(tester, find.byKey(const Key('modelVramSlider')));
    final vram = tester.widget<Slider>(find.byKey(const Key('modelVramSlider')));
    expect(vram.value, 24);
    vram.onChanged!(2);
    await tester.pump();
    expect(fake.setCalls, isEmpty);
    expect(container.read(vramLimitGbProvider), 2);

    // Search then open a repo whose file exceeds even the max VRAM limit.
    final searchUrl = browseUrl('huge', 'downloads', false, 0);
    hf.urlBody[searchUrl] = jsonEncode([
      {'id': 'org/huge', 'downloads': 1, 'likes': 1},
    ]);
    await tester.enterText(find.byKey(const Key('modelSearchField')), 'huge');
    await reveal(tester, find.byKey(const Key('modelSearchButton')));
    await tester.tap(find.byKey(const Key('modelSearchButton')));
    await tester.pump();
    await reveal(tester, find.byKey(const Key('modelRepo-org/huge')));
    await tester.tap(find.byKey(const Key('modelRepo-org/huge')));
    await tester.pump();
    await reveal(tester, find.text('ggml-huge.bin'));

    expect(find.textContaining('больше лимита vram'), findsOneWidget);
    final download = tester.widget<FilledButton>(
      find.byKey(const Key('modelDownloadButton-ggml-huge.bin')),
    );
    expect(download.onPressed, isNull);

    await tester.pumpWidget(const SizedBox.shrink());
  });

  testWidgets('search fetches exactly browseUrl and filters junk', (tester) async {
    final url = browseUrl('whisper', 'downloads', false, 0);
    final hf = FakeHfClient(urlBody: {url: fixture('hf_search')});
    final fake = FakeDictaEngine(
      const DictaStatus(connected: true, state: DaemonState.idle),
      fetchSettingsReply: readSettings(),
    );
    expect(hf.fetchedUrls, isEmpty); // no automatic fetch at open

    await tester.pumpWidget(buildFakeApp(
      fake,
      hfClient: hf,
      downloadEngine: FakeDownloadEngine(scenario: FakeDownloadScenario.ok200),
    ));
    await tick(tester);
    await openModels(tester);
    expect(hf.fetchedUrls, isEmpty);

    await tester.enterText(find.byKey(const Key('modelSearchField')), 'whisper');
    await reveal(tester, find.byKey(const Key('modelSearchButton')));
    await tester.tap(find.byKey(const Key('modelSearchButton')));
    await tester.pump();

    expect(hf.fetchedUrls, <String>[url]);
    expect(find.text('ggerganov/whisper.cpp'), findsOneWidget);
    expect(find.text('old/style/repo'), findsOneWidget);
    await tester.pumpWidget(const SizedBox.shrink());
  });

  testWidgets('sort, ASR filter and pagination change the next URL only',
      (tester) async {
    final first = browseUrl('', 'downloads', false, 0);
    final second = browseUrl('', 'likes', true, 0);
    final third = browseUrl('', 'likes', true, 20);
    final hf = FakeHfClient(urlBody: {
      first: fixture('hf_search'),
      second: fixture('hf_search'),
      third: fixture('hf_search'),
    });
    final fake = FakeDictaEngine(
      const DictaStatus(connected: true, state: DaemonState.idle),
      fetchSettingsReply: readSettings(),
    );
    await tester.pumpWidget(buildFakeApp(
      fake,
      hfClient: hf,
      downloadEngine: FakeDownloadEngine(scenario: FakeDownloadScenario.ok200),
    ));
    await tick(tester);
    await openModels(tester);

    await reveal(tester, find.byKey(const Key('modelSortSegments')));
    await tester.tap(find.text('лайки'));
    await tester.pump();
    await tester.tap(find.byKey(const Key('modelAsrSwitch')));
    await tester.pump();
    expect(hf.fetchedUrls, isEmpty); // settings changes do NOT fetch

    await reveal(tester, find.byKey(const Key('modelSearchButton')));
    await tester.tap(find.byKey(const Key('modelSearchButton')));
    await tester.pump();
    await reveal(tester, find.byKey(const Key('modelShowMoreButton')));
    await tester.tap(find.byKey(const Key('modelShowMoreButton')));
    await tester.pump();

    expect(hf.fetchedUrls, <String>[second, third]);
    await tester.pumpWidget(const SizedBox.shrink());
  });

  testWidgets('repo tree navigation uses treeUrlAt for root and subdirs',
      (tester) async {
    final searchUrl = browseUrl('tree', 'downloads', false, 0);
    final rootUrl = treeUrlAt('org/repo', '');
    final subUrl = treeUrlAt('org/repo', 'sub');
    final hf = FakeHfClient(urlBody: {
      searchUrl: jsonEncode([
        {'id': 'org/repo', 'downloads': 1, 'likes': 1},
      ]),
      rootUrl: fixture('hf_tree'),
      subUrl: jsonEncode([
        {'type': 'file', 'path': 'sub/ggml-nested.bin', 'size': 40000000},
      ]),
    });
    final fake = FakeDictaEngine(
      const DictaStatus(connected: true, state: DaemonState.idle),
      fetchSettingsReply: readSettings(),
    );
    await tester.pumpWidget(buildFakeApp(
      fake,
      hfClient: hf,
      downloadEngine: FakeDownloadEngine(scenario: FakeDownloadScenario.ok200),
    ));
    await tick(tester);
    await openModels(tester);

    await tester.enterText(find.byKey(const Key('modelSearchField')), 'tree');
    await reveal(tester, find.byKey(const Key('modelSearchButton')));
    await tester.tap(find.byKey(const Key('modelSearchButton')));
    await tester.pump();
    await reveal(tester, find.byKey(const Key('modelRepo-org/repo')));
    await tester.tap(find.byKey(const Key('modelRepo-org/repo')));
    await tester.pump();

    expect(hf.fetchedUrls, contains(rootUrl));
    expect(find.text('ggml-tiny.bin'), findsOneWidget);
    expect(find.byKey(const Key('modelSubdir-sub')), findsOneWidget);

    await reveal(tester, find.byKey(const Key('modelSubdir-sub')));
    await tester.tap(find.byKey(const Key('modelSubdir-sub')));
    await tester.pump();
    expect(hf.fetchedUrls.last, subUrl);

    await reveal(tester, find.byKey(const Key('modelTreeBackButton')));
    final back = tester.widget<TextButton>(
      find.byKey(const Key('modelTreeBackButton')),
    );
    expect(back.onPressed, isNotNull);
    await tester.tap(find.byKey(const Key('modelTreeBackButton')));
    await tester.pump();
    expect(hf.fetchedUrls.last, rootUrl);

    await tester.pumpWidget(const SizedBox.shrink());
  });

  testWidgets('direct link validates scheme and shows the file name',
      (tester) async {
    final fake = FakeDictaEngine(
      const DictaStatus(connected: true, state: DaemonState.idle),
      fetchSettingsReply: readSettings(),
    );
    final download = FakeDownloadEngine(scenario: FakeDownloadScenario.ok200);
    await tester.pumpWidget(buildFakeApp(
      fake,
      hfClient: FakeHfClient(),
      downloadEngine: download,
    ));
    await tick(tester);
    await openModels(tester);

    await tester.enterText(find.byKey(const Key('modelDirectLinkField')), 'ftp://x');
    await reveal(tester, find.byKey(const Key('modelDirectDownloadButton')));
    expect(
      tester
          .widget<FilledButton>(find.byKey(const Key('modelDirectDownloadButton')))
          .onPressed,
      isNull,
    );

    await tester.enterText(
      find.byKey(const Key('modelDirectLinkField')),
      'https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-x.bin',
    );
    await tester.pump();
    expect(find.text('ggml-x.bin'), findsOneWidget);
    expect(
      tester
          .widget<FilledButton>(find.byKey(const Key('modelDirectDownloadButton')))
          .onPressed,
      isNotNull,
    );

    await tester.pumpWidget(const SizedBox.shrink());
  });

  testWidgets('one active download disables all download buttons', (tester) async {
    final dir = Directory.systemTemp.createTempSync('dicta-browser-');
    addTearDown(() => dir.deleteSync(recursive: true));
    final fake = FakeDictaEngine(
      const DictaStatus(connected: true, state: DaemonState.idle),
      fetchSettingsReply: tempFixture(dir),
    );
    final download = FakeDownloadEngine(
      scenario: FakeDownloadScenario.cancelMid,
    )..downloadGate = Completer<void>();
    await tester.pumpWidget(buildFakeApp(
      fake,
      hfClient: FakeHfClient(),
      downloadEngine: download,
    ));
    await tick(tester);
    await openModels(tester);

    await reveal(tester, find.byKey(const Key('modelDownloadButton-ggml-tiny-q5_1.bin')));
    await tester.tap(
      find.byKey(const Key('modelDownloadButton-ggml-tiny-q5_1.bin')),
    );
    await tester.pump();

    expect(find.byKey(const Key('modelDownloadPanel')), findsOneWidget);
    expect(find.byKey(const Key('modelCancelDownloadButton')), findsOneWidget);
    expect(
      tester
          .widget<FilledButton>(
            find.byKey(const Key('modelDownloadButton-ggml-base-q5_1.bin')),
          )
          .onPressed,
      isNull,
    );

    await tester.pumpWidget(const SizedBox.shrink());
  });

  testWidgets('unknown size shows indeterminate progress and MB counter',
      (tester) async {
    final dir = Directory.systemTemp.createTempSync('dicta-browser-');
    addTearDown(() => dir.deleteSync(recursive: true));
    final fake = FakeDictaEngine(
      const DictaStatus(connected: true, state: DaemonState.idle),
      fetchSettingsReply: tempFixture(dir),
    );
    final download = FakeDownloadEngine(
      scenario: FakeDownloadScenario.cancelMid,
      totalBytes: 2000000,
      chunkBytes: 1000000,
    )..downloadGate = Completer<void>();
    await tester.pumpWidget(buildFakeApp(
      fake,
      hfClient: FakeHfClient(),
      downloadEngine: download,
    ));
    await tick(tester);
    await openModels(tester);

    final directUrl = 'https://huggingface.co/org/repo/resolve/main/ggml-u.bin';
    await tester.enterText(find.byKey(const Key('modelDirectLinkField')), directUrl);
    await reveal(tester, find.byKey(const Key('modelDirectDownloadButton')));
    await tester.tap(find.byKey(const Key('modelDirectDownloadButton')));
    await tester.pump();

    final progress = tester.widget<LinearProgressIndicator>(
      find.descendant(
        of: find.byKey(const Key('modelDownloadPanel')),
        matching: find.byType(LinearProgressIndicator),
      ),
    );
    expect(progress.value, isNull); // indeterminate
    expect(find.text('1 МБ'), findsOneWidget);

    await tester.pumpWidget(const SizedBox.shrink());
  });

  testWidgets('done, cancel and HTTP errors are honest snackbars', (tester) async {
    for (final entry in {
      FakeDownloadScenario.ok200: 'модель скачана — выбери её в списке',
      FakeDownloadScenario.cancelMid: 'загрузка прервана',
      FakeDownloadScenario.http404: 'загрузка не удалась: download failed: 404',
    }.entries) {
      final dir = Directory.systemTemp.createTempSync('dicta-browser-');
      final fake = FakeDictaEngine(
        const DictaStatus(connected: true, state: DaemonState.idle),
        fetchSettingsReply: tempFixture(dir),
      );
      final download = FakeDownloadEngine(scenario: entry.key)
        ..downloadGate = entry.key == FakeDownloadScenario.cancelMid
            ? Completer<void>()
            : null;
      await tester.pumpWidget(buildFakeApp(
        fake,
        hfClient: FakeHfClient(),
        downloadEngine: download,
      ));
      await tick(tester);
      await openModels(tester);

      await tester.enterText(
        find.byKey(const Key('modelDirectLinkField')),
        'https://huggingface.co/org/repo/resolve/main/ggml-z.bin',
      );
      await reveal(tester, find.byKey(const Key('modelDirectDownloadButton')));
      await tester.tap(find.byKey(const Key('modelDirectDownloadButton')));
      await tester.pump();
      if (entry.key == FakeDownloadScenario.cancelMid) {
        tester
            .widget<OutlinedButton>(
              find.byKey(const Key('modelCancelDownloadButton')),
            )
            .onPressed!();
      }
      await tester.runAsync(
        () => Future<void>.delayed(const Duration(milliseconds: 150)),
      );
      await tester.pump();
      await tester.pump(const Duration(milliseconds: 100));

      expect(find.text(entry.value), findsOneWidget, reason: '${entry.key}');
      dir.deleteSync(recursive: true);
      await tester.pumpWidget(const SizedBox.shrink());
    }
  });

  testWidgets('HF 401 and 404 become honest text, never an exception',
      (tester) async {
    for (final code in [401, 404]) {
      // The single FakeHfClient covers both statuses: 404 via a missing URL,
      // 401 via the status hook (no second HfClient implementation).
      final client = FakeHfClient(statusOverride: code == 401 ? 401 : null);
      final fake = FakeDictaEngine(
        const DictaStatus(connected: true, state: DaemonState.idle),
        fetchSettingsReply: readSettings(),
      );
      await tester.pumpWidget(buildFakeApp(
        fake,
        hfClient: client,
        downloadEngine: FakeDownloadEngine(scenario: FakeDownloadScenario.ok200),
      ));
      await tick(tester);
      await openModels(tester);

      await tester.enterText(find.byKey(const Key('modelSearchField')), 'x');
      await reveal(tester, find.byKey(const Key('modelSearchButton')));
      await tester.tap(find.byKey(const Key('modelSearchButton')));
      await tester.pump();

      expect(
        find.text(code == 401 ? 'репозиторий закрыт или не найден' : 'HTTP 404'),
        findsOneWidget,
      );
      await tester.pumpWidget(const SizedBox.shrink());
    }
  });
}

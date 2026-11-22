// DownloadManager unit gates on the fake seams: one at a time, .part resume,
// cancel cleanup, preserved .part on failure, rename + snapshot refetch on done.
// Temp directories via Directory.systemTemp; no network anywhere.
import 'dart:async';
import 'dart:io';

import 'package:dicta_app/core/downloads.dart';
import 'package:dicta_app/core/engine.dart';
import 'package:dicta_app/core/hf/download_engine.dart';
import 'package:dicta_app/core/providers.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';

import 'fake_dicta_engine.dart';
import 'fake_download_engine.dart';

void main() {
  late Directory dir;
  late Map<String, dynamic> settingsReply;

  setUp(() {
    dir = Directory.systemTemp.createTempSync('dicta-downloads-');
    final active = '${dir.path}/ggml-active.bin';
    settingsReply = <String, dynamic>{
      'settings': <String, dynamic>{
        'model': active,
        'models': <String>[active],
      },
    };
  });

  tearDown(() => dir.deleteSync(recursive: true));

  ProviderContainer containerFor(
    DownloadEngine engine, {
    FakeDictaEngine? daemon,
  }) =>
      ProviderContainer(
        overrides: [
          downloadEngineProvider.overrideWithValue(engine),
          engineProvider.overrideWithValue(
            daemon ??
                FakeDictaEngine(
                  const DictaStatus(connected: true, state: DaemonState.idle),
                  fetchSettingsReply: settingsReply,
                ),
          ),
        ],
      );

  Future<DownloadState> settle(ProviderContainer container) async {
    for (var i = 0; i < 200; i++) {
      await Future<void>.delayed(const Duration(milliseconds: 5));
      final state = container.read(downloadManagerProvider);
      if (state is! DownloadActive) return state;
    }
    throw StateError('download never settled');
  }

  test('ok200 streams progress, renames .part and asks for no Range offset',
      () async {
    final fake = FakeDownloadEngine(
      scenario: FakeDownloadScenario.ok200,
      totalBytes: 100,
      chunkBytes: 25,
    );
    final container = containerFor(fake);
    addTearDown(container.dispose);
    final received = <int>[];
    container.listen(
      downloadManagerProvider,
      (_, next) {
        if (next is DownloadActive) received.add(next.received);
      },
    );

    await container.read(downloadManagerProvider.notifier).start(
          url: 'https://hf/ggml-x.bin',
          destName: 'ggml-x.bin',
          expectedBytes: 100,
        );
    final state = await settle(container);

    expect(state, isA<DownloadDone>());
    expect(fake.calls.length, 1);
    expect(fake.calls.single.fromByte, 0);
    expect(received, contains(25));
    expect(received, contains(100));
    expect(File('${dir.path}/ggml-x.bin').existsSync(), isTrue);
    expect(File('${dir.path}/ggml-x.bin.part').existsSync(), isFalse);
  });

  test('resume asks for the .part length and starts progress from it', () async {
    final part = File('${dir.path}/ggml-x.bin.part');
    part.writeAsBytesSync(List<int>.filled(40, 1));
    final fake = FakeDownloadEngine(
      scenario: FakeDownloadScenario.resume206,
      totalBytes: 100,
      chunkBytes: 25,
    );
    final container = containerFor(fake);
    addTearDown(container.dispose);
    final received = <int>[];
    container.listen(
      downloadManagerProvider,
      (_, next) {
        if (next is DownloadActive) received.add(next.received);
      },
    );

    await container.read(downloadManagerProvider.notifier).start(
          url: 'https://hf/ggml-x.bin',
          destName: 'ggml-x.bin',
          expectedBytes: 100,
        );
    final state = await settle(container);

    expect(state, isA<DownloadDone>());
    expect(fake.calls.single.fromByte, 40);
    // The manager seeds progress with the .part length before the first chunk.
    expect(received.first, 40);
    expect(received, contains(65));
    expect(received, contains(100));
    expect(File('${dir.path}/ggml-x.bin').existsSync(), isTrue);
    expect(part.existsSync(), isFalse);
  });

  test('deliberate cancel erases .part and records cancelled', () async {
    final fake = FakeDownloadEngine(
      scenario: FakeDownloadScenario.cancelMid,
    )..downloadGate = Completer<void>();
    final container = containerFor(fake);
    addTearDown(container.dispose);
    final manager = container.read(downloadManagerProvider.notifier);

    await manager.start(
      url: 'https://hf/ggml-x.bin',
      destName: 'ggml-x.bin',
      expectedBytes: 100,
    );
    await Future<void>.delayed(const Duration(milliseconds: 10));
    manager.cancel();
    final state = await settle(container);

    expect(state, isA<DownloadCancelled>());
    expect(File('${dir.path}/ggml-x.bin.part').existsSync(), isFalse);
  });

  for (final entry in {
    401: FakeDownloadScenario.http401,
    404: FakeDownloadScenario.http404,
  }.entries) {
    test('HTTP ${entry.key} fails honestly and keeps .part for retry', () async {
      final part = File('${dir.path}/ggml-${entry.key}.bin.part');
      part.writeAsBytesSync(List<int>.filled(10, 1));
      final fake = FakeDownloadEngine(scenario: entry.value);
      final container = containerFor(fake);
      addTearDown(container.dispose);

      await container.read(downloadManagerProvider.notifier).start(
            url: 'https://hf/ggml-${entry.key}.bin',
            destName: 'ggml-${entry.key}.bin',
          );
      final state = await settle(container);

      expect(state, isA<DownloadFailed>());
      expect((state as DownloadFailed).message, contains('${entry.key}'));
      expect(part.existsSync(), isTrue);
    });
  }

  test('a second start while active is ignored', () async {
    final fake = FakeDownloadEngine(
      scenario: FakeDownloadScenario.cancelMid,
    )..downloadGate = Completer<void>();
    final container = containerFor(fake);
    addTearDown(container.dispose);
    final manager = container.read(downloadManagerProvider.notifier);

    await manager.start(url: 'https://hf/first.bin', destName: 'first.bin');
    await manager.start(url: 'https://hf/second.bin', destName: 'second.bin');

    expect(fake.calls.length, 1);
    expect(fake.calls.single.url, 'https://hf/first.bin');
    manager.cancel();
    await settle(container);
  });

  test('an already installed dest is never overwritten', () async {
    File('${dir.path}/ggml-x.bin').writeAsStringSync('installed');
    final fake = FakeDownloadEngine(scenario: FakeDownloadScenario.ok200);
    final container = containerFor(fake);
    addTearDown(container.dispose);

    await container.read(downloadManagerProvider.notifier).start(
          url: 'https://hf/ggml-x.bin',
          destName: 'ggml-x.bin',
        );
    final state = container.read(downloadManagerProvider);

    expect(state, isA<DownloadDone>());
    expect((state as DownloadDone).alreadyExisted, isTrue);
    expect(fake.calls, isEmpty);
    expect(File('${dir.path}/ggml-x.bin').readAsStringSync(), 'installed');
  });

  test('success invalidates and awaits the settings snapshot refetch', () async {
    final daemon = FakeDictaEngine(
      const DictaStatus(connected: true, state: DaemonState.idle),
      fetchSettingsReply: settingsReply,
    );
    final container = containerFor(
      FakeDownloadEngine(scenario: FakeDownloadScenario.ok200),
      daemon: daemon,
    );
    addTearDown(container.dispose);
    final before = daemon.fetchCount;

    await container.read(downloadManagerProvider.notifier).start(
          url: 'https://hf/ggml-x.bin',
          destName: 'ggml-x.bin',
          expectedBytes: 100,
        );
    await settle(container);

    expect(daemon.fetchCount, greaterThan(before));
  });
}

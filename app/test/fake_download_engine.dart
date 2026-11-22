// The single fake DownloadEngine seam. Scenarios mirror the daemon's honest
// outcomes; `fromByte` records the Range offset the manager asked for (the
// engine derives it from the .part file on disk, exactly like the real one).
import 'dart:async';
import 'dart:io';

import 'package:dicta_app/core/hf/download_engine.dart';

enum FakeDownloadScenario { ok200, resume206, http401, http404, cancelMid }

class FakeDownloadEngine implements DownloadEngine {
  FakeDownloadEngine({
    required this.scenario,
    this.totalBytes = 100,
    this.chunkBytes = 25,
  });

  FakeDownloadScenario scenario;
  int totalBytes;
  int chunkBytes;

  /// Every call records the URL and the offset derived from the .part length.
  final List<({String url, int fromByte})> calls = [];

  /// When set, a mid-flight download stays parked here until the test releases
  /// it (the stopGate pattern from fake_dicta_engine.dart:34-38).
  Completer<void>? downloadGate;

  bool _cancelled = false;

  @override
  void cancel() {
    _cancelled = true;
    final gate = downloadGate;
    if (gate != null && !gate.isCompleted) gate.complete();
  }

  @override
  Stream<DownloadProgress> download(
    Uri url,
    String partPath, {
    int? expectedBytes,
  }) async* {
    final part = File(partPath);
    final fromByte = part.existsSync() ? part.lengthSync() : 0;
    calls.add((url: url.toString(), fromByte: fromByte));

    if (scenario == FakeDownloadScenario.http401) {
      throw const DownloadException(401);
    }
    if (scenario == FakeDownloadScenario.http404) {
      throw const DownloadException(404);
    }

    final total = expectedBytes ?? totalBytes;
    // A 206 reply appends to .part; a 200 restarts from zero. Like the real
    // engine, the fake writes the bytes so the manager can rename .part.
    final resume = scenario == FakeDownloadScenario.resume206;
    var received = resume ? fromByte : 0;
    // Sync writes keep the whole fake stream on the microtask queue, so widget
    // tests (fake-async zone) can observe completion with pump() alone.
    final file = part.openSync(mode: resume ? FileMode.append : FileMode.write);
    try {
      while (received < totalBytes) {
        final next = received + chunkBytes > totalBytes
            ? totalBytes
            : received + chunkBytes;
        file.writeFromSync(List<int>.filled(next - received, 0));
        yield DownloadProgress(received: next, total: total);
        received = next;
        if (scenario == FakeDownloadScenario.cancelMid) {
          final gate = downloadGate;
          if (gate != null) await gate.future;
          if (_cancelled) throw const DownloadCancelledException();
        }
      }
      file.flushSync();
    } finally {
      // Leave whatever was written for resume/cancel inspection.
      file.closeSync();
    }
  }
}

/// Download manager: one download at a time, .part resume, deliberate cancel,
/// rename on success and honest errors. Speaks ONLY to the downloadEngineProvider
/// seam - all dart:io transport details live in core/hf/download_engine.dart.
library;

import 'dart:async';
import 'dart:io';

import 'package:flutter_riverpod/flutter_riverpod.dart';

import 'hf/download_engine.dart';
import 'hf/hf_catalog.dart';
import 'providers.dart';

sealed class DownloadState {
  const DownloadState();
}

class DownloadIdle extends DownloadState {
  const DownloadIdle();
}

class DownloadActive extends DownloadState {
  const DownloadActive({
    required this.file,
    required this.url,
    required this.expectedBytes,
    required this.received,
  });

  final String file;
  final String url;
  final int? expectedBytes;
  final int received;
}

class DownloadDone extends DownloadState {
  const DownloadDone({required this.file, this.alreadyExisted = false});

  final String file;
  final bool alreadyExisted;
}

class DownloadFailed extends DownloadState {
  const DownloadFailed({required this.message});

  final String message;
}

class DownloadCancelled extends DownloadState {
  const DownloadCancelled();
}

final downloadManagerProvider =
    NotifierProvider<DownloadManager, DownloadState>(DownloadManager.new);

class DownloadManager extends Notifier<DownloadState> {
  StreamSubscription<DownloadProgress>? _subscription;

  @override
  DownloadState build() {
    ref.onDispose(() => _subscription?.cancel());
    return const DownloadIdle();
  }

  Future<void> start({
    required String url,
    required String destName,
    int? expectedBytes,
  }) async {
    // One download at a time: a second start is silently ignored (main.rs:225).
    if (state is DownloadActive) return;

    final settings = await ref.read(settingsSnapshotProvider.future);
    final destDirPath = destDir(settings.model);
    final destPath = '$destDirPath${Platform.pathSeparator}$destName';
    final dest = File(destPath);
    if (dest.existsSync()) {
      // Never overwrite an installed model (main.rs:230-233).
      state = DownloadDone(file: destName, alreadyExisted: true);
      return;
    }

    final partPath = partOf(destPath);
    final part = File(partPath);
    final offset = part.existsSync() ? part.lengthSync() : 0;
    final dir = Directory(destDirPath);
    if (!dir.existsSync()) dir.createSync(recursive: true);

    state = DownloadActive(
      file: destName,
      url: url,
      expectedBytes: expectedBytes,
      received: offset,
    );

    try {
      _subscription = ref
          .read(downloadEngineProvider)
          .download(Uri.parse(url), partPath, expectedBytes: expectedBytes)
          .listen(
            (progress) {
              final current = state;
              if (current is DownloadActive) {
                state = DownloadActive(
                  file: current.file,
                  url: current.url,
                  expectedBytes: current.expectedBytes,
                  received: progress.received,
                );
              }
            },
            onError: (Object error, StackTrace _) => _handleError(error, part),
            onDone: () => _finish(dest, part, destName),
            cancelOnError: true,
          );
    } catch (error) {
      _handleError(error, part);
    }
  }

  void _handleError(Object error, File part) {
    _subscription?.cancel();
    _subscription = null;
    if (error is DownloadCancelledException) {
      // A deliberate cancel erases .part so a fresh start begins at zero.
      if (part.existsSync()) part.deleteSync();
      state = const DownloadCancelled();
      return;
    }
    // Code/type only - never a body or a set payload (payload privacy).
    final message = error is DownloadException
        ? 'download failed: ${error.statusCode}'
        : 'download failed: ${error.runtimeType}';
    state = DownloadFailed(message: message);
    // On any other failure .part STAYS so a manual retry resumes (main.rs:408).
  }

  Future<void> _finish(File dest, File part, String destName) async {
    await _subscription?.cancel();
    _subscription = null;
    if (!part.existsSync()) {
      state = const DownloadFailed(message: 'download failed: missing part');
      return;
    }
    try {
      // A rename is an atomic metadata operation - sync keeps the completion
      // observable in the widget test's fake-async zone as well.
      part.renameSync(dest.path);
    } catch (error) {
      state = DownloadFailed(message: 'download failed: ${error.runtimeType}');
      return;
    }
    state = DownloadDone(file: destName);
    // The daemon discovers the new model on its own; refresh the snapshot so
    // it appears in settings.models (parity with main.rs:377-416, CR-01).
    ref.invalidate(settingsSnapshotProvider);
    await ref.read(settingsSnapshotProvider.future);
  }

  /// Explicit user cancellation: routed through the seam; the engine's
  /// cancelled event is what erases .part and records the state.
  void cancel() {
    if (state is! DownloadActive) return;
    ref.read(downloadEngineProvider).cancel();
  }
}

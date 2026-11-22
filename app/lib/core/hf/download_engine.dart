/// Streaming download seam plus the pure 200/206 resume decisions. The real
/// transport is dart:io HttpClient only (no shell-out for network); the manager in
/// core/downloads.dart talks exclusively to this interface (APP-03).
library;

import 'dart:io';

class DownloadProgress {
  const DownloadProgress({required this.received, required this.total});

  final int received;
  final int total;
}

abstract interface class DownloadEngine {
  /// Streams bytes into `partPath`. The engine derives the Range offset from
  /// the existing `.part` length; signed resolve URLs are never cached.
  Stream<DownloadProgress> download(
    Uri url,
    String partPath, {
    int? expectedBytes,
  });

  void cancel();
}

/// HTTP failure at the seam: code only, never a response body (payload privacy).
class DownloadException implements Exception {
  const DownloadException(this.statusCode);

  final int statusCode;

  @override
  String toString() => 'DownloadException($statusCode)';
}

class DownloadCancelledException implements Exception {
  const DownloadCancelledException();

  @override
  String toString() => 'DownloadCancelledException()';
}

/// 206 -> append to `.part` from `offset`; 200 (server ignored Range) -> write
/// from zero so the file is not glued together (R1 Pitfall 3).
({FileMode mode, int offset}) resumePlan(int statusCode, int offset) =>
    statusCode == 206
        ? (mode: FileMode.append, offset: offset)
        : (mode: FileMode.write, offset: 0);

/// Full size from `content-range: bytes start-end/TOTAL`; malformed -> 0.
/// A 206 reply's content-length is only the remaining chunk (R1 finding 4).
int totalFromContentRange(String? header) {
  if (header == null) return 0;
  final match = RegExp(r'bytes\s+\d+-\d+/(\d+)').firstMatch(header);
  if (match == null) return 0;
  return int.tryParse(match.group(1) ?? '') ?? 0;
}

class HttpDownloadEngine implements DownloadEngine {
  HttpClientRequest? _request;

  @override
  void cancel() {
    // abort() tears the connection down: the response stream completes with
    // the exception below, which the manager turns into a cancelled state.
    _request?.abort(const DownloadCancelledException());
    _request = null;
  }

  @override
  Stream<DownloadProgress> download(
    Uri url,
    String partPath, {
    int? expectedBytes,
  }) async* {
    final part = File(partPath);
    final offset = part.existsSync() ? part.lengthSync() : 0;
    final client = HttpClient();
    try {
      final request = await client.getUrl(url);
      _request = request;
      if (offset > 0) {
        request.headers.set(HttpHeaders.rangeHeader, 'bytes=$offset-');
      }
      final response = await request.close(); // followRedirects carries Range.
      if (response.statusCode >= 400) {
        throw DownloadException(response.statusCode);
      }
      // The server may ignore Range and answer 200 - then overwrite from zero.
      final plan = resumePlan(response.statusCode, offset);
      final rangeTotal = totalFromContentRange(
        response.headers.value(HttpHeaders.contentRangeHeader),
      );
      final total = (expectedBytes != null && expectedBytes > 0)
          ? expectedBytes
          : rangeTotal;
      var received = plan.offset;
      final sink = part.openWrite(mode: plan.mode);
      try {
        await for (final chunk in response) {
          sink.add(chunk);
          received += chunk.length;
          yield DownloadProgress(received: received, total: total);
        }
        await sink.flush();
        await sink.close();
      } catch (_) {
        // Network failure or explicit abort: leave `.part` for resume (or for
        // the manager to delete on a deliberate cancel, main.rs:408-413).
        await sink.close();
        rethrow;
      }
    } finally {
      client.close();
      _request = null;
    }
  }
}

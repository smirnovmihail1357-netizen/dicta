/// The HF HTTP seam. Transport lives only in core/; features/ talk to the
/// abstract interface so tests can inject FakeHfClient and never hit network.
library;

import 'dart:convert';
import 'dart:io';

abstract interface class HfClient {
  Future<String> fetchText(String url);
}

/// Error at the seam: carries the status CODE only, never the response body
/// (payload privacy, daemon_engine.dart:57-58 canon).
class HfHttpException implements Exception {
  const HfHttpException(this.statusCode);

  final int statusCode;

  @override
  String toString() => 'HfHttpException($statusCode)';
}

class HttpHfClient implements HfClient {
  static const _timeout = Duration(seconds: 30);

  @override
  Future<String> fetchText(String url) async {
    // A fresh client per request; signed Location URLs are never cached (A2).
    final client = HttpClient();
    try {
      final request = await client.getUrl(Uri.parse(url)).timeout(_timeout);
      final response = await request.close().timeout(_timeout);
      if (response.statusCode >= 400) {
        throw HfHttpException(response.statusCode);
      }
      return await response.transform(utf8.decoder).join().timeout(_timeout);
    } finally {
      client.close();
    }
  }
}

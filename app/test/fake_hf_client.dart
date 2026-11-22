// The single fake HfClient seam: all scenarios live here so browser tests
// never clone the fake or touch the network. Missing URLs fail like 404s.
import 'package:dicta_app/core/hf/hf_client.dart';

class FakeHfClient implements HfClient {
  FakeHfClient({Map<String, String>? urlBody, this.statusOverride})
      : urlBody = urlBody ?? <String, String>{};

  /// Exact URL -> body map; parity assertions use fetchedUrls after a click.
  final Map<String, String> urlBody;

  /// When set, every fetch fails with this status (e.g. 401 for a closed repo).
  final int? statusOverride;

  final List<String> fetchedUrls = [];

  @override
  Future<String> fetchText(String url) async {
    fetchedUrls.add(url);
    if (statusOverride != null) throw HfHttpException(statusOverride!);
    final body = urlBody[url];
    if (body == null) throw const HfHttpException(404);
    return body;
  }
}

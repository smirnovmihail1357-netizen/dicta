// The single fake HfClient seam: all scenarios live here so browser tests
// never clone the fake or touch the network. Missing URLs fail like 404s.
import 'package:dicta_app/core/hf/hf_client.dart';

class FakeHfClient implements HfClient {
  FakeHfClient({
    Map<String, String>? urlBody,
    this.statusOverride,
    this.errorOverride,
    Map<String, Object>? urlErrors,
  })  : urlBody = urlBody ?? <String, String>{},
        urlErrors = urlErrors ?? <String, Object>{};

  /// Exact URL -> body map; parity assertions use fetchedUrls after a click.
  final Map<String, String> urlBody;

  /// Exact URL -> exception map: lets one URL fail while another succeeds
  /// (e.g. a healthy search followed by a tree fetch that times out).
  final Map<String, Object> urlErrors;

  /// When set, every fetch fails with this status (e.g. 401 for a closed repo).
  final int? statusOverride;

  /// When set, every fetch throws this non-HTTP exception (TimeoutException,
  /// SocketException, ...) so the browser's catch-all can be exercised.
  final Object? errorOverride;

  final List<String> fetchedUrls = [];

  @override
  Future<String> fetchText(String url) async {
    fetchedUrls.add(url);
    final urlError = urlErrors[url];
    if (urlError != null) throw urlError;
    if (errorOverride != null) throw errorOverride!;
    if (statusOverride != null) throw HfHttpException(statusOverride!);
    final body = urlBody[url];
    if (body == null) throw const HfHttpException(404);
    return body;
  }
}

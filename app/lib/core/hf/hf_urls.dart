/// Pure Hugging Face URL formulas, ported from the frozen Rust reference
/// `crates/dicta-settings/src/hf.rs:5-49` (READ-ONLY). No network here.
library;

/// Percent-encoding of everything outside the unreserved set.
///
/// `Uri.encodeComponent` (NOT `encodeQueryComponent`) gives `%20` for a space,
/// matching hf.rs:5-15; `encodeQueryComponent` would give `+` instead (A3).
String urlEncode(String s) => Uri.encodeComponent(s);

/// Search/browse URL for models: query, task filter, sort and page offset.
/// `sort` is one of `downloads`, `likes`, `lastModified` (hf.rs:21-35).
String browseUrl(String query, String sort, bool asrOnly, int offset) {
  var url = 'https://huggingface.co/api/models?';
  if (query.isNotEmpty) url += 'search=${urlEncode(query)}&';
  if (asrOnly) url += 'filter=automatic-speech-recognition&';
  url += 'sort=$sort&direction=-1&limit=20&offset=$offset';
  return url;
}

/// Repository tree at the root or at a concrete subpath on branch `main`.
String treeUrlAt(String repo, String subpath) => subpath.isEmpty
    ? 'https://huggingface.co/api/models/$repo/tree/main?recursive=false'
    : 'https://huggingface.co/api/models/$repo/tree/main/$subpath';

/// Direct resolve link for a file inside a repository (hf.rs:47-49).
String resolveUrl(String repo, String path) =>
    'https://huggingface.co/$repo/resolve/main/$path';

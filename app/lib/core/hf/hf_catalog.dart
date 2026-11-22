/// Catalog of verified ggml models and pure destination-path helpers, ported
/// from the frozen Rust reference `crates/dicta-settings/src/catalog.rs`
/// (READ-ONLY). Russian notes live in .arb, not here (cyrillic gate).
library;

import 'dart:io';

class CatalogEntry {
  const CatalogEntry({required this.file, required this.approxBytes});

  final String file;
  final int approxBytes;
}

/// Same six models, same order, same sizes as catalog.rs:15-46.
const List<CatalogEntry> kCatalog = <CatalogEntry>[
  CatalogEntry(file: 'ggml-tiny-q5_1.bin', approxBytes: 32000000),
  CatalogEntry(file: 'ggml-base-q5_1.bin', approxBytes: 60000000),
  CatalogEntry(file: 'ggml-small-q5_1.bin', approxBytes: 180000000),
  CatalogEntry(file: 'ggml-medium-q5_0.bin', approxBytes: 500000000),
  CatalogEntry(file: 'ggml-large-v3-turbo-q5_1.bin', approxBytes: 570000000),
  CatalogEntry(file: 'ggml-large-v3-q5_0.bin', approxBytes: 1100000000),
];

String urlFor(String file) =>
    'https://huggingface.co/ggerganov/whisper.cpp/resolve/main/$file';

/// Unfinished download: the same path with `.part` appended (catalog.rs:54-58).
String partOf(String dest) => '$dest.part';

/// Pure version of catalog.rs:70-88 for tests: no process environment reads.
/// [localAppData] is the Windows WIN-05 contract: models are local data, not
/// roaming settings (phase 7 must switch data_dir() to data_local_dir()).
String destDirFrom(String model, String? xdgDataHome, String? home,
    [String? localAppData]) {
  // Path::new(model).parent() - '/x' has the root '/' as its parent, while a
  // bare name ('ggml.bin') has an empty parent and takes the fallback.
  // Windows custom-model paths use backslashes, so consider both separators.
  final slash = model.lastIndexOf('/');
  final backslash = model.lastIndexOf('\\');
  final index = backslash > slash ? backslash : slash;
  if (index >= 0) {
    final parent = index == 0 ? model[index] : model.substring(0, index);
    if (parent.isNotEmpty) return parent;
  }
  // Windows: local app data + whisper-cpp — BEFORE the Unix data-home
  // fallback (an explicit custom-model parent above still wins). No Unix path
  // hardcodes here.
  if (localAppData != null && localAppData.isNotEmpty) {
    return '$localAppData\\whisper-cpp';
  }
  final xdg = xdgDataHome != null && xdgDataHome.isNotEmpty ? xdgDataHome : null;
  final base = xdg ??
      ((home != null && home.isNotEmpty) ? '$home/.local/share' : '.local/share');
  return '$base/whisper-cpp';
}

/// Production wrapper: Windows local app data / the Unix data-home variable /
/// HOME -> whisper-cpp. On Linux that Windows variable is absent, so the
/// existing Unix resolution is byte-for-byte unchanged.
String destDir(String model) => destDirFrom(
      model,
      Platform.environment['XDG_DATA_HOME'],
      Platform.environment['HOME'],
      Platform.environment['LOCALAPPDATA'],
    );

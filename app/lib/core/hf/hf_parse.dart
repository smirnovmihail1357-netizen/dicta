/// Pure parsers for Hugging Face API replies, ported from the frozen Rust
/// reference `crates/dicta-settings/src/hf.rs:51-165`. Broken JSON and
/// non-arrays yield empty lists, never exceptions (hf.rs:60-83, :93-147).
library;

import 'dart:convert';

class HfRepo {
  const HfRepo({required this.id, this.downloads = 0, this.likes = 0});

  final String id;
  final int downloads;
  final int likes;

  @override
  bool operator ==(Object other) =>
      other is HfRepo &&
      other.id == id &&
      other.downloads == downloads &&
      other.likes == likes;

  @override
  int get hashCode => Object.hash(id, downloads, likes);

  @override
  String toString() => 'HfRepo($id, $downloads, $likes)';
}

class HfFile {
  const HfFile({required this.path, this.size = 0});

  final String path;
  final int size;

  @override
  bool operator ==(Object other) =>
      other is HfFile && other.path == path && other.size == size;

  @override
  int get hashCode => Object.hash(path, size);

  @override
  String toString() => 'HfFile($path, $size)';
}

/// Tolerant array read: any parse error or non-array becomes an empty list.
List<dynamic> _array(String json) {
  try {
    final value = jsonDecode(json);
    return value is List ? value : const <dynamic>[];
  } catch (_) {
    return const <dynamic>[];
  }
}

List<HfRepo> parseSearch(String json) => _array(json)
    .whereType<Map<String, dynamic>>()
    .map((item) {
      final id = item['id'] ?? item['modelId'];
      if (id is! String) return null;
      final downloads = item['downloads'];
      final likes = item['likes'];
      return HfRepo(
        id: id,
        // A String/null value must fall back to 0, not throw the cast (the
        // Rust reference's as_u64() returns None for those).
        downloads: downloads is num ? downloads.toInt() : 0,
        likes: likes is num ? likes.toInt() : 0,
      );
    })
    .whereType<HfRepo>()
    .toList(growable: false);

List<HfFile> parseTree(String json) => _array(json)
    .whereType<Map<String, dynamic>>()
    .map((item) {
      if (item['type'] != 'file') return null;
      final path = item['path'];
      if (path is! String || !path.endsWith('.bin')) return null;
      final rawSize = item['size'];
      var size = rawSize is num && rawSize > 0 ? rawSize.toInt() : 0;
      if (size == 0) {
        // LFS files may carry the real size in `lfs.size` (hf.rs:110-120).
        final lfs = item['lfs'];
        if (lfs is Map<String, dynamic>) {
          final lfsSize = lfs['size'];
          if (lfsSize is num) size = lfsSize.toInt();
        }
      }
      return HfFile(path: path, size: size);
    })
    .whereType<HfFile>()
    .toList(growable: false);

List<String> parseTreeDirs(String json) => _array(json)
    .whereType<Map<String, dynamic>>()
    .where((item) => item['type'] == 'directory')
    .map((item) => item['path'])
    .whereType<String>()
    .toList(growable: false);

/// Flat name of a model file: the last `/`-separated path segment.
String destName(String path) {
  final parts = path.split('/');
  return parts.isEmpty ? path : parts.last;
}

/// File name from a direct link: last segment, query part removed.
/// Returns null when there is no `/` or the name is empty (hf.rs:155-165).
String? filenameOf(String url) {
  final path = url.split('?').first;
  if (!path.contains('/')) return null;
  final name = path.split('/').last;
  if (name.isEmpty) return null;
  return name;
}

/// Exact VRAM comparison from dictad main.rs:683-684 (UI-only, no daemon key).
bool vramBlocked(int size, double vramGb) =>
    size > 0 && size > vramGb * 1e9;

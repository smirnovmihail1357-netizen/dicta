/// Pure helpers for the installed-models list and guarded local deletion.
/// The list itself comes ONLY from settings.models (the daemon scans the
/// directory itself); this file never walks the filesystem to discover models.
library;

import 'dart:io';

/// Port of dicta-settings main.rs:932-936: last segment, minus `.bin`, minus
/// the `ggml-` prefix.
String modelName(String path) {
  var name = path.split('/').last;
  if (name.endsWith('.bin')) name = name.substring(0, name.length - 4);
  if (name.startsWith('ggml-')) name = name.substring(5);
  return name;
}

enum DeleteDecision { allowed, active, outsideDestDir }

/// Triple guard: never the active model, never a path outside the model's
/// destination directory. Paths are normalized through File.parent before the
/// comparison ('..' and double slashes collapse correctly).
DeleteDecision canDelete(String path, String activeModel, String destDir) {
  if (path == activeModel) return DeleteDecision.active;
  final parent = File(path).parent.path;
  if (parent != destDir) return DeleteDecision.outsideDestDir;
  return DeleteDecision.allowed;
}

/// Deletes a model file. Failures are data (an honest string), never an
/// exception escaping into the UI. Payload privacy: no file contents logged.
Future<String?> deleteModel(String path) async {
  try {
    await File(path).delete();
    return null;
  } on FileSystemException catch (error) {
    return 'delete failed: ${error.message}';
  }
}

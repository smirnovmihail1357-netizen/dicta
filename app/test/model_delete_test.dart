// MODEL-02 unit gates: name port, triple guard, honest file deletion.
// Temp directories via Directory.systemTemp; no network anywhere.
import 'dart:io';

import 'package:dicta_app/features/models/model_store.dart';
import 'package:flutter_test/flutter_test.dart';

void main() {
  test('modelName strips path, .bin and ggml- prefix', () {
    expect(modelName('/a/ggml-base-q5_1.bin'), 'base-q5_1');
    expect(modelName('/a/b/ggml-large-v3-q5_0.bin'), 'large-v3-q5_0');
    expect(modelName('ggml-tiny-q5_1.bin'), 'tiny-q5_1');
    expect(modelName('/a/plain'), 'plain');
  });

  test('canDelete blocks the active model and any path outside destDir', () {
    final dir = Directory.systemTemp.createTempSync('dicta-models-');
    try {
      final active = '${dir.path}/ggml-medium.bin';
      final other = '${dir.path}/ggml-base.bin';
      expect(canDelete(active, active, dir.path), DeleteDecision.active);
      expect(
        canDelete('../x.bin', active, dir.path),
        DeleteDecision.outsideDestDir,
      );
      expect(
        canDelete('/tmp/foreign/ggml-x.bin', active, dir.path),
        DeleteDecision.outsideDestDir,
      );
      expect(canDelete(other, active, dir.path), DeleteDecision.allowed);
    } finally {
      dir.deleteSync(recursive: true);
    }
  });

  test('deleteModel deletes inside destDir and reports missing as data',
      () async {
    final dir = Directory.systemTemp.createTempSync('dicta-models-');
    try {
      final file = File('${dir.path}/ggml-x.bin');
      file.writeAsStringSync('binary');
      expect(await deleteModel(file.path), isNull);
      expect(file.existsSync(), isFalse);

      final failure = await deleteModel('${dir.path}/no-such.bin');
      expect(failure, isA<String>());
      expect(failure, isNotEmpty);
    } finally {
      dir.deleteSync(recursive: true);
    }
  });
}

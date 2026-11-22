// Parity tests: strings copied verbatim from the frozen Rust reference
// crates/dicta-settings/src/hf.rs:229-341 and catalog.rs:94-152 (READ-ONLY).
import 'dart:io';

import 'package:dicta_app/core/hf/hf_catalog.dart';
import 'package:dicta_app/core/hf/hf_parse.dart';
import 'package:flutter_test/flutter_test.dart';

void main() {
  test('parseSearch reads id (or modelId), downloads and likes', () {
    const json = r'''[
      {"id":"ggerganov/whisper.cpp","downloads":12345,"likes":678,"pipeline_tag":"automatic-speech-recognition"},
      {"modelId":"old/style/repo","likes":null},
      {"id":"no/downloads","downloads":null,"likes":"12"},
      {"id":17,"downloads":900,"likes":5},
      {"downloads":5}
    ]''';
    expect(
      parseSearch(json),
      <HfRepo>[
        HfRepo(id: 'ggerganov/whisper.cpp', downloads: 12345, likes: 678),
        HfRepo(id: 'old/style/repo', downloads: 0, likes: 0),
        HfRepo(id: 'no/downloads', downloads: 0, likes: 0),
      ],
    );
  });

  test('parseTree keeps only .bin files and uses lfs.size', () {
    const json = r'''[
      {"type":"directory","path":"sub","size":0},
      {"type":"file","path":"README.md","size":12},
      {"type":"file","path":"ggml-tiny.bin","size":32000000},
      {"type":"file","path":"ggml-large.bin","size":0,"lfs":{"size":1100000000,"sha256":"abc"}},
      {"type":"file","path":"sub/ggml-nested.bin","size":40000000},
      {"type":"file","path":"ggml-nolfs.bin"},
      {"path":"ggml-notyped.bin","size":77},
      "мусор"
    ]''';
    expect(
      parseTree(json),
      <HfFile>[
        HfFile(path: 'ggml-tiny.bin', size: 32000000),
        HfFile(path: 'ggml-large.bin', size: 1100000000),
        HfFile(path: 'sub/ggml-nested.bin', size: 40000000),
        HfFile(path: 'ggml-nolfs.bin', size: 0),
      ],
    );
  });

  test('parseTreeDirs selects only directories', () {
    const json = r'''[
      {"type":"directory","path":"sub","size":0},
      {"type":"file","path":"ggml-tiny.bin","size":32},
      {"type":"directory","path":"sub/deeper","size":0},
      {"type":"directory"},
      {"type":17,"path":"bad"},
      "мусор"
    ]''';
    expect(parseTreeDirs(json), <String>['sub', 'sub/deeper']);
  });

  test('broken JSON and non-arrays give empty lists', () {
    for (final bad in ['{', 'null', '{}', '"text"']) {
      expect(parseSearch(bad), isEmpty);
      expect(parseTree(bad), isEmpty);
      expect(parseTreeDirs(bad), isEmpty);
    }
  });

  test('destName takes the last path segment', () {
    expect(destName('sub/ggml.bin'), 'ggml.bin');
    expect(destName('a/b/c/ggml.bin'), 'ggml.bin');
    expect(destName('ggml.bin'), 'ggml.bin');
    expect(destName(''), '');
  });

  test('filenameOf strips the query and rejects nameless paths', () {
    expect(
      filenameOf(
        'https://huggingface.co/a/b/resolve/main/ggml-x.bin?download=true',
      ),
      'ggml-x.bin',
    );
    expect(filenameOf('https://host/dir/model.bin'), 'model.bin');
    expect(filenameOf('https://host/dir/'), isNull);
    expect(filenameOf('https://host/dir'), 'dir');
    expect(filenameOf('example.com'), isNull);
    expect(filenameOf(''), isNull);
  });

  test('vramBlocked uses the exact daemon comparison', () {
    expect(vramBlocked(0, 24), isFalse);
    expect(vramBlocked(24 * 1000000000, 24), isFalse);
    expect(vramBlocked(24 * 1000000000 + 1, 24), isTrue);
    expect(vramBlocked(1100000000, 1), isTrue);
  });

  test('catalog is complete, sized and ordered', () {
    expect(kCatalog.length, 6);
    expect(kCatalog.first.file, 'ggml-tiny-q5_1.bin');
    expect(kCatalog.first.approxBytes, 32000000);
    expect(kCatalog.last.file, 'ggml-large-v3-q5_0.bin');
    expect(kCatalog.last.approxBytes, 1100000000);
    for (final entry in kCatalog) {
      expect(entry.file, startsWith('ggml-'));
      expect(entry.file, endsWith('.bin'));
      expect(entry.approxBytes, greaterThan(0));
    }
  });

  test('urlFor and partOf match catalog.rs', () {
    expect(
      urlFor('ggml-tiny-q5_1.bin'),
      'https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-tiny-q5_1.bin',
    );
    expect(partOf('/a/b.bin'), '/a/b.bin.part');
    expect(partOf('ggml-x.bin'), 'ggml-x.bin.part');
  });

  test('destDirFrom uses parent, XDG or HOME fallback', () {
    expect(destDirFrom('/a/b/ggml-x.bin', null, '/home/u'), '/a/b');
    const fallback = '/home/u/.local/share/whisper-cpp';
    expect(destDirFrom('', null, '/home/u'), fallback);
    expect(destDirFrom('ggml-x.bin', null, '/home/u'), fallback);
    expect(destDirFrom('', '/data', '/home/u'), '/data/whisper-cpp');
  });

  test('fixtures parse through the same pure functions', () {
    final search = File('test/fixtures/hf_search.json').readAsStringSync();
    final tree = File('test/fixtures/hf_tree.json').readAsStringSync();
    expect(parseSearch(search).length, 3);
    expect(parseTree(tree).length, 4);
    expect(parseTreeDirs(tree), <String>['sub']);
  });
}

// Parity tests: strings copied verbatim from the frozen Rust reference
// crates/dicta-settings/src/hf.rs:172-227 (READ-ONLY) + resume helpers.
import 'dart:io';

import 'package:dicta_app/core/hf/download_engine.dart';
import 'package:dicta_app/core/hf/hf_urls.dart';
import 'package:flutter_test/flutter_test.dart';

void main() {
  test('urlEncode escapes everything outside the unreserved set', () {
    expect(urlEncode('whisper ggml'), 'whisper%20ggml');
    expect(urlEncode('aa/bb'), 'aa%2Fbb');
    expect(urlEncode('A-z0._~'), 'A-z0._~');
    expect(urlEncode('ключ'), '%D0%BA%D0%BB%D1%8E%D1%87');
  });

  test('browseUrl combines query, filter, sort and offset', () {
    const base = 'https://huggingface.co/api/models?';
    expect(
      browseUrl('', 'downloads', false, 0),
      '${base}sort=downloads&direction=-1&limit=20&offset=0',
    );
    expect(
      browseUrl('whisper ggml', 'downloads', false, 20),
      '${base}search=whisper%20ggml&sort=downloads&direction=-1&limit=20&offset=20',
    );
    expect(
      browseUrl('', 'likes', true, 0),
      '${base}filter=automatic-speech-recognition&sort=likes&direction=-1&limit=20&offset=0',
    );
    expect(
      browseUrl('wav2vec', 'lastModified', true, 40),
      '${base}search=wav2vec&filter=automatic-speech-recognition&sort=lastModified&direction=-1&limit=20&offset=40',
    );
    expect(
      browseUrl(' a/b ', 'downloads', false, 0),
      '${base}search=%20a%2Fb%20&sort=downloads&direction=-1&limit=20&offset=0',
    );
  });

  test('treeUrlAt distinguishes root and subpath', () {
    expect(
      treeUrlAt('ggerganov/whisper.cpp', ''),
      'https://huggingface.co/api/models/ggerganov/whisper.cpp/tree/main?recursive=false',
    );
    expect(
      treeUrlAt('ggerganov/whisper.cpp', 'sub/dir'),
      'https://huggingface.co/api/models/ggerganov/whisper.cpp/tree/main/sub/dir',
    );
  });

  test('resolveUrl is exact', () {
    expect(
      resolveUrl('ggerganov/whisper.cpp', 'ggml-tiny.bin'),
      'https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-tiny.bin',
    );
  });

  test('resumePlan chooses append on 206 and overwrite on 200', () {
    final resume = resumePlan(206, 1000);
    expect(resume.mode, FileMode.append);
    expect(resume.offset, 1000);
    final restart = resumePlan(200, 1000);
    expect(restart.mode, FileMode.write);
    expect(restart.offset, 0);
    final fresh = resumePlan(200, 0);
    expect(fresh.mode, FileMode.write);
    expect(fresh.offset, 0);
  });

  test('totalFromContentRange reads the total and tolerates garbage', () {
    expect(totalFromContentRange('bytes 0-0/77691713'), 77691713);
    expect(totalFromContentRange('bytes 100-199/77691713'), 77691713);
    expect(totalFromContentRange('nonsense'), 0);
    expect(totalFromContentRange(null), 0);
  });
}

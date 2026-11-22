// Deterministic tray icon generator. Pure dart:io + dart:typed_data, no
// package imports, no system tools and no network: run from the repo root with
//   dart tools/gen_tray_icons.dart
// Re-running is byte-for-byte identical, so the committed PNGs are reproducible
// (final gate: `git diff --quiet -- app/assets/tray/`).

import 'dart:io';
import 'dart:typed_data';

// CRC-32 (IEEE) lookup table, built once at startup.
final List<int> _crcTable = List<int>.generate(256, (n) {
  var c = n;
  for (var k = 0; k < 8; k++) {
    c = (c & 1) != 0 ? (0xEDB88320 ^ (c >> 1)) : (c >> 1);
  }
  return c;
});

int _crc32(List<int> bytes) {
  var c = 0xFFFFFFFF;
  for (final b in bytes) {
    c = _crcTable[(c ^ b) & 0xFF] ^ (c >> 8);
  }
  return (c ^ 0xFFFFFFFF) & 0xFFFFFFFF;
}

// 4-byte big-endian unsigned integer (PNG is big-endian).
List<int> _u32(int value) => <int>[
      (value >> 24) & 0xFF,
      (value >> 16) & 0xFF,
      (value >> 8) & 0xFF,
      value & 0xFF,
    ];

List<int> _chunk(String type, List<int> data) => <int>[
      ..._u32(data.length),
      ...type.codeUnits,
      ...data,
      ..._u32(_crc32(<int>[...type.codeUnits, ...data])),
    ];

/// Builds one minimal valid PNG: 22x22 RGBA 8-bit, filled circle r=8.
List<int> _png(List<int> rgba) {
  const size = 22;
  const radius = 8;
  const center = (size - 1) / 2.0;

  final raw = BytesBuilder(copy: false);
  for (var y = 0; y < size; y++) {
    raw.add(<int>[0]); // filter type 0 (None) for every scanline
    for (var x = 0; x < size; x++) {
      final dx = x - center;
      final dy = y - center;
      final inside = dx * dx + dy * dy <= radius * radius;
      raw.add(inside ? rgba : <int>[0, 0, 0, 0]);
    }
  }

  final ihdr = <int>[
    ..._u32(size),
    ..._u32(size),
    8, // bit depth
    6, // color type: truecolor + alpha
    0, // compression
    0, // filter
    0, // interlace
  ];

  return <int>[
    0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, // PNG signature
    ..._chunk('IHDR', ihdr),
    ..._chunk('IDAT', ZLibEncoder().convert(raw.takeBytes())),
    ..._chunk('IEND', <int>[]),
  ];
}

void main() {
  const icons = <({String name, List<int> rgba})>[
    // idle: brand fallback seed from main.dart:37 (#35689F)
    (name: 'idle', rgba: <int>[53, 104, 159, 255]),
    // recording: also used for transcribing (orchestrator decision 2026-11-22)
    (name: 'recording', rgba: <int>[229, 57, 53, 255]), // #E53935
  ];

  final dir = Directory('app/assets/tray');
  if (!dir.existsSync()) dir.createSync(recursive: true);
  for (final icon in icons) {
    File('app/assets/tray/${icon.name}.png').writeAsBytesSync(_png(icon.rgba));
  }
}

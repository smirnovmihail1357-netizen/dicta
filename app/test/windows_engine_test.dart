// WIN-02 contract tests — no live Windows pipe. dart_ipc.bind over a Linux
// UDS path exercises the exact transport seam used on Windows, without FFI.
import 'dart:async';
import 'dart:convert';
import 'dart:io';

import 'package:dart_ipc/dart_ipc.dart';
import 'package:dicta_app/core/engine.dart';
import 'package:dicta_app/core/windows_engine.dart';
import 'package:flutter_test/flutter_test.dart';

void main() {
  late ServerSocket server;
  late StreamSubscription<Socket> clients;
  late String fakePath;

  setUp(() async {
    fakePath =
        '${Directory.systemTemp.path}/dicta_test_${pid}_${DateTime.now().microsecondsSinceEpoch}.sock';
    server = await bind(fakePath);
    clients = server.listen((client) {
      client
          .cast<List<int>>()
          .transform(utf8.decoder)
          .transform(const LineSplitter())
          .listen((line) {
        final req = jsonDecode(line) as Map<String, dynamic>;
        if (req['cmd'] == 'status') {
          client.write('${jsonEncode({
                'state': 'idle',
                'level': 0.0,
                'seconds': 0.0,
                'text': null,
                'error': null,
                'panel': false,
              })}\n');
        }
      });
    });
  });

  tearDown(() async {
    await clients.cancel();
    await server.close();
  });

  test('status round-trips over an injected dart_ipc UDS path', () async {
    final status = await WindowsPipeEngine(fakePath).status();
    expect(status.connected, isTrue);
    expect(status.state, DaemonState.idle);
  });

  test('transport error becomes a disconnected status snapshot', () async {
    final missing =
        '${Directory.systemTemp.path}/dicta_missing_${pid}_${DateTime.now().microsecondsSinceEpoch}.sock';
    final status = await WindowsPipeEngine(missing).stop();
    expect(status.connected, isFalse);
    expect(status.error, isNotNull);
  });

  test('transport error becomes a failed DictaReply', () async {
    final missing =
        '${Directory.systemTemp.path}/dicta_missing_${pid}_${DateTime.now().microsecondsSinceEpoch}.sock';
    final reply = await WindowsPipeEngine(missing).start();
    expect(reply.ok, isFalse);
    expect(reply.error, isNotNull);
  });

  test('silent fake yields the timeout hint in a snapshot', () async {
    final silentPath =
        '${Directory.systemTemp.path}/dicta_silent_${pid}_${DateTime.now().microsecondsSinceEpoch}.sock';
    final silentServer = await bind(silentPath);
    final silentClients = silentServer.listen((_) {
      // Accept the connection but never answer — exercise the 2 s budget.
    });
    try {
      final status = await WindowsPipeEngine(silentPath).status();
      expect(status.connected, isFalse);
      expect(status.error, contains('\u0442\u0430\u0439\u043c\u0430\u0443\u0442'));
    } finally {
      await silentClients.cancel();
      await silentServer.close();
    }
  });
}

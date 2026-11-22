// WIN-02 contract tests — no live Windows pipe. dart_ipc.bind over a Linux
// UDS path exercises the exact transport seam used on Windows, without FFI.
import 'dart:async';
import 'dart:convert';
import 'dart:io';

import 'package:dart_ipc/dart_ipc.dart';
// dart_ipc.connect delegates to this settable platform instance. Faking it is
// the only way to drive a connect that hangs past the 2 s budget: on Unix,
// Socket.connect to a missing path fails at once instead of retrying like the
// Windows pipe implementation does.
import 'package:dart_ipc/src/dart_ipc_platform_interface.dart';
import 'package:dicta_app/core/engine.dart';
import 'package:dicta_app/core/windows_engine.dart';
import 'package:flutter_test/flutter_test.dart';

class _FakeConnectPlatform extends DartIpcPlatform {
  _FakeConnectPlatform(this.connector);

  final Future<Socket> Function(String path) connector;

  @override
  Future<ServerSocket> bind(String path) => throw UnimplementedError();

  @override
  Future<Socket> connect(String path) => connector(path);
}

class _CountingSocket implements Socket {
  int destroyCount = 0;

  @override
  void destroy() {
    destroyCount++;
  }

  @override
  dynamic noSuchMethod(Invocation invocation) => super.noSuchMethod(invocation);
}

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

  test('connect timeout reports its own budget, not the command budget',
      () async {
    final never = Completer<Socket>();
    final previous = DartIpcPlatform.instance;
    DartIpcPlatform.instance = _FakeConnectPlatform((_) => never.future);
    try {
      final engine = WindowsPipeEngine('fake-pipe');
      final startReply = await engine.start();
      expect(startReply.ok, isFalse);
      expect(startReply.error, contains('\u0442\u0430\u0439\u043c\u0430\u0443\u0442 2 \u0441'));
      expect(startReply.error, isNot(contains('120')));

      final setReply = await engine.set('key', 'value');
      expect(setReply.error, contains('\u0442\u0430\u0439\u043c\u0430\u0443\u0442 2 \u0441'));
      expect(setReply.error, isNot(contains('30')));
    } finally {
      DartIpcPlatform.instance = previous;
    }
  });

  test('late connect success is destroyed instead of leaking a handle',
      () async {
    final completer = Completer<Socket>();
    final lateSocket = _CountingSocket();
    final previous = DartIpcPlatform.instance;
    DartIpcPlatform.instance = _FakeConnectPlatform((_) => completer.future);
    try {
      final reply = await WindowsPipeEngine('fake-pipe').start();
      expect(reply.ok, isFalse);
      // The wrapper timed out, but dart_ipc's retry can still succeed later.
      completer.complete(lateSocket);
      await Future<void>.delayed(const Duration(milliseconds: 10));
      expect(lateSocket.destroyCount, 1);
    } finally {
      DartIpcPlatform.instance = previous;
    }
  });
}

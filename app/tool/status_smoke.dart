// Live end-to-end tracer: real socket -> DaemonEngine -> one status line.
import 'dart:io';

import 'package:dicta_app/core/daemon_engine.dart';

Future<void> main() async {
  try {
    final engine = DaemonEngine(resolveSocketPath());
    final status = await engine.status();
    stdout.writeln('SMOKE OK state=${status.state.name}');
  } catch (error) {
    // Type only: never print the error object (03-SECURITY residual).
    stderr.writeln('SMOKE FAIL ${error.runtimeType}');
    exitCode = 1;
  }
}

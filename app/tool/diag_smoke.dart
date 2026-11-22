// Live diagnostics smoke: one real status tick plus a harmless empty inject.
// The daemon rejects empty text before any insertion, so there are no side
// effects and no dictated payload is ever generated or printed here.
import 'dart:io';

import 'package:dicta_app/core/daemon_engine.dart';

Future<void> main() async {
  try {
    final engine = DaemonEngine(resolveSocketPath());
    final status = await engine.status();
    stdout.writeln(
      'DIAG SMOKE state=${status.state.name} connected=${status.connected}',
    );

    final injected = await engine.inject(text: '');
    if (!status.connected ||
        injected.text != null ||
        injected.error == null ||
        injected.error!.isEmpty) {
      stderr.writeln('DIAG SMOKE FAIL unexpected inject reply');
      exitCode = 1;
      return;
    }

    stdout.writeln('INJECT-REJECTED-AS-EXPECTED');
  } catch (error) {
    stderr.writeln('DIAG SMOKE FAIL $error');
    exitCode = 1;
  }
}

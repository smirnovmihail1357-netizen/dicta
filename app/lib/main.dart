import 'package:dicta_app/core/engine.dart';
import 'package:dicta_app/core/providers.dart';
import 'package:dicta_app/l10n/app_localizations.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

void main() {
  runApp(const ProviderScope(child: DictaApp()));
}

class DictaApp extends StatelessWidget {
  const DictaApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      localizationsDelegates: AppLocalizations.localizationsDelegates,
      supportedLocales: AppLocalizations.supportedLocales,
      locale: const Locale('ru'),
      home: const StatusHome(),
    );
  }
}

class StatusHome extends ConsumerWidget {
  const StatusHome({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final l10n = AppLocalizations.of(context)!;
    final status = ref.watch(dictaStatusProvider);
    final label = switch (status.state) {
      DaemonState.idle => l10n.daemonStateIdle,
      DaemonState.recording => l10n.daemonStateRecording,
      DaemonState.transcribing => l10n.daemonStateTranscribing,
    };

    return Scaffold(
      body: Center(
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            if (!status.connected) Text(l10n.daemonUnavailable),
            Text(label),
            Text('level ${status.level.toStringAsFixed(2)} · ${status.seconds.toStringAsFixed(1)}s'),
          ],
        ),
      ),
    );
  }
}

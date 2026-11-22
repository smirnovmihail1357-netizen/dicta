/// Onboarding diagnostics from existing commands only: two automatic rows
/// from the polled status, one from settings, two honest manual hints.
/// No new daemon commands and no config.toml reading (DIAG-01).
library;

import 'package:dicta_app/core/providers.dart';
import 'package:dicta_app/l10n/generated/app_localizations.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

class DiagnosticsCard extends ConsumerWidget {
  const DiagnosticsCard({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final l10n = AppLocalizations.of(context)!;
    final colors = Theme.of(context).colorScheme;
    final weak = TextStyle(color: colors.onSurfaceVariant);
    final status = ref.watch(dictaStatusProvider);
    final settings = ref.watch(settingsSnapshotProvider).value;
    final connected = status.connected;

    Widget row(IconData icon, Color color, String label) => Padding(
          padding: const EdgeInsets.symmetric(vertical: 4),
          child: Row(
            children: [
              Icon(icon, color: color, size: 20),
              const SizedBox(width: 12),
              Expanded(child: Text(label)),
            ],
          ),
        );

    final daemonIcon = connected ? Icons.check_circle : Icons.error;
    final daemonColor = connected ? colors.primary : colors.error;

    final IconData modelIcon;
    final Color modelColor;
    final String modelLabel;
    if (!connected || settings == null || settings.model.isEmpty) {
      modelIcon = Icons.error;
      modelColor = colors.error;
      modelLabel = l10n.diagModelUnavailable;
    } else if (settings.reloading) {
      modelIcon = Icons.hourglass_top;
      modelColor = colors.onSurfaceVariant;
      modelLabel = l10n.diagModelLoading;
    } else {
      modelIcon = Icons.check_circle;
      modelColor = colors.primary;
      modelLabel = l10n.diagModelReady;
    }

    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            Text(
              l10n.diagTitle,
              style: Theme.of(context).textTheme.titleMedium,
            ),
            const SizedBox(height: 12),
            row(daemonIcon, daemonColor, l10n.diagDaemonAlive),
            row(daemonIcon, daemonColor, l10n.diagSocketAnswers),
            Padding(
              padding: const EdgeInsets.only(left: 32, bottom: 4),
              child: Text(l10n.diagStatusNote, style: weak),
            ),
            row(modelIcon, modelColor, modelLabel),
            Padding(
              padding: const EdgeInsets.only(left: 32, bottom: 4),
              child: Text(l10n.diagModelNote, style: weak),
            ),
            row(Icons.info_outline, colors.onSurfaceVariant,
                l10n.diagHotkeyManual),
            row(Icons.info_outline, colors.onSurfaceVariant,
                l10n.diagInjectManual),
            if (!connected) ...[
              const SizedBox(height: 4),
              Text(l10n.diagStartHint, style: TextStyle(color: colors.error)),
            ],
            const SizedBox(height: 12),
            Align(
              alignment: Alignment.centerLeft,
              child: ElevatedButton(
                key: const Key('diagRecheckButton'),
                onPressed: () {
                  ref.read(dictaStatusProvider.notifier).poll();
                  // Invalidate refetches once; the card watches the provider.
                  ref.invalidate(settingsSnapshotProvider);
                },
                child: Text(l10n.diagRecheck),
              ),
            ),
          ],
        ),
      ),
    );
  }
}

/// Live control surface over the frozen daemon protocol: state, live level,
/// elapsed seconds and the calibration history feeder (DASH-01).
/// Only providers are read here - no transport code under features/.
library;

import 'package:dicta_app/core/engine.dart';
import 'package:dicta_app/core/providers.dart';
import 'package:dicta_app/features/dashboard/calibration_card.dart';
import 'package:dicta_app/features/dashboard/diagnostics_card.dart';
import 'package:dicta_app/features/dashboard/dicta_control.dart';
import 'package:dicta_app/features/dashboard/last_text_card.dart';
import 'package:dicta_app/features/dashboard/level_history.dart';
import 'package:dicta_app/l10n/generated/app_localizations.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

class DashboardScreen extends ConsumerStatefulWidget {
  const DashboardScreen({super.key});

  @override
  ConsumerState<DashboardScreen> createState() => _DashboardScreenState();
}

class _DashboardScreenState extends ConsumerState<DashboardScreen> {
  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context)!;
    final status = ref.watch(dictaStatusProvider);
    final control = ref.watch(dictaControlProvider);
    final colors = Theme.of(context).colorScheme;
    final busy = control.commandInFlight;
    final canStart = status.connected && status.state == DaemonState.idle && !busy;
    final canStop =
        status.connected && status.state == DaemonState.recording && !busy;
    final canCancel = canStop; // cancel during transcribing would lie

    // Feed the histogram only from recording ticks: the daemon reports level
    // 0.0 in every other state and those zeros would drag the view to the floor.
    ref.listen(dictaStatusProvider, (previous, next) {
      if (next.state != DaemonState.recording) return;
      final history = ref.read(levelHistoryProvider.notifier);
      if (previous?.state != DaemonState.recording) history.clear();
      history.push(next.level);
    });

    final label = switch (status.state) {
      DaemonState.idle => l10n.daemonStateIdle,
      DaemonState.recording => l10n.daemonStateRecording,
      DaemonState.transcribing => l10n.daemonStateTranscribing,
    };

    return SingleChildScrollView(
      padding: const EdgeInsets.all(16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Card(
            child: Padding(
              padding: const EdgeInsets.all(16),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text(
                    l10n.stateLabel(label),
                    style: Theme.of(context).textTheme.titleMedium,
                  ),
                  const SizedBox(height: 12),
                  AnimatedContainer(
                    duration: const Duration(milliseconds: 120),
                    curve: Curves.easeOut,
                    child: LinearProgressIndicator(
                      value: status.level.clamp(0.0, 1.0).toDouble(),
                      minHeight: 8,
                    ),
                  ),
                  const SizedBox(height: 12),
                  Text(
                    l10n.dashRecordingTime(_formatSeconds(status.seconds)),
                    style: TextStyle(color: colors.onSurfaceVariant),
                  ),
                  const SizedBox(height: 4),
                  Text(
                    l10n.levelSeconds(
                      status.level.toStringAsFixed(2),
                      status.seconds.toStringAsFixed(1),
                    ),
                    style: TextStyle(color: colors.onSurfaceVariant),
                  ),
                ],
              ),
            ),
          ),
          const SizedBox(height: 12),
          Card(
            child: Padding(
              padding: const EdgeInsets.all(16),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.stretch,
                children: [
                  Row(
                    children: [
                      Expanded(
                        child: ElevatedButton(
                          key: const Key('dashStartButton'),
                          onPressed: canStart
                              ? () => ref
                                  .read(dictaControlProvider.notifier)
                                  .start()
                              : null,
                          child: Text(l10n.dashStartButton),
                        ),
                      ),
                      const SizedBox(width: 8),
                      Expanded(
                        child: ElevatedButton(
                          key: const Key('dashStopButton'),
                          onPressed: canStop
                              ? () => ref
                                  .read(dictaControlProvider.notifier)
                                  .stop()
                              : null,
                          child: Text(l10n.dashStopButton),
                        ),
                      ),
                      const SizedBox(width: 8),
                      Expanded(
                        child: ElevatedButton(
                          key: const Key('dashCancelButton'),
                          onPressed: canCancel
                              ? () => ref
                                  .read(dictaControlProvider.notifier)
                                  .cancel()
                              : null,
                          child: Text(l10n.dashCancelButton),
                        ),
                      ),
                    ],
                  ),
                  // Bound to the polled state, not to the stop Future: a stop
                  // fired by the daemon's hotkey is visible here too.
                  if (status.state == DaemonState.transcribing)
                    Padding(
                      padding: const EdgeInsets.only(top: 12),
                      child: Row(
                        children: [
                          const SizedBox(
                            width: 16,
                            height: 16,
                            child: CircularProgressIndicator(strokeWidth: 2),
                          ),
                          const SizedBox(width: 8),
                          Text(l10n.dashTranscribing),
                        ],
                      ),
                    ),
                ],
              ),
            ),
          ),
          const SizedBox(height: 12),
          const LastTextCard(),
          const SizedBox(height: 12),
          const CalibrationCard(),
          const SizedBox(height: 12),
          const DiagnosticsCard(),
        ],
      ),
    );
  }

  // Seconds come from the daemon snapshot; no local timer is involved.
  String _formatSeconds(double seconds) {
    final duration = Duration(milliseconds: (seconds * 1000).round());
    final minutes = duration.inMinutes.toString().padLeft(2, '0');
    final rest = (duration.inSeconds % 60).toString().padLeft(2, '0');
    return '$minutes:$rest';
  }
}

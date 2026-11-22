/// Live level histogram and graphical min_level calibration (DASH-04).
/// One set command per drag gesture; calibration is legal in every state.
/// Drawn with standard widgets only - no chart packages.
library;

import 'package:dicta_app/core/providers.dart';
import 'package:dicta_app/features/dashboard/level_history.dart';
import 'package:dicta_app/l10n/generated/app_localizations.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

class LevelHistogramPainter extends CustomPainter {
  LevelHistogramPainter({
    required this.samples,
    required this.marker,
    required this.colors,
  });

  final List<double> samples;
  final double marker;
  final ColorScheme colors;

  @override
  void paint(Canvas canvas, Size size) {
    final barWidth = size.width / LevelHistoryNotifier.capacity;
    for (var i = 0; i < samples.length; i++) {
      final level = samples[i].clamp(0.0, 1.0);
      final height =
          (level * size.height).clamp(2.0, size.height).toDouble();
      final left = size.width - (samples.length - i) * barWidth;
      final color = level > 0.6
          ? colors.primary
          : level > 0.3
              ? colors.tertiary
              : colors.outline;
      canvas.drawRect(
        Rect.fromLTWH(left, size.height - height, barWidth, height),
        Paint()..color = color,
      );
    }
    final x = marker.clamp(0.0, 1.0) * size.width;
    canvas.drawLine(
      Offset(x, 0),
      Offset(x, size.height),
      Paint()
        ..color = colors.error
        ..strokeWidth = 2,
    );
  }

  @override
  bool shouldRepaint(covariant LevelHistogramPainter oldDelegate) =>
      oldDelegate.samples != samples || oldDelegate.marker != marker;
}

class CalibrationCard extends ConsumerStatefulWidget {
  const CalibrationCard({super.key});

  @override
  ConsumerState<CalibrationCard> createState() => _CalibrationCardState();
}

class _CalibrationCardState extends ConsumerState<CalibrationCard> {
  double? _marker;
  String? _error;

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context)!;
    final colors = Theme.of(context).colorScheme;
    final samples = ref.watch(levelHistoryProvider);
    final settings = ref.watch(settingsSnapshotProvider).value;
    final marker = _marker ?? settings?.minLevel ?? 0.0;

    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            Text(
              l10n.calibrationTitle,
              style: Theme.of(context).textTheme.titleMedium,
            ),
            const SizedBox(height: 12),
            LayoutBuilder(
              builder: (context, constraints) {
                final width = constraints.maxWidth;
                return GestureDetector(
                  key: const Key('calibrationSurface'),
                  onHorizontalDragUpdate: (details) {
                    final next =
                        (details.localPosition.dx / width).clamp(0.0, 1.0);
                    // Local pending copy: no I/O until the gesture ends.
                    setState(() => _marker = next.toDouble());
                  },
                  onHorizontalDragEnd: (_) async {
                    final value = _marker ?? marker;
                    // Panel format {:.2}; one command per completed gesture.
                    final reply = await ref
                        .read(engineProvider)
                        .set('min_level', value.toStringAsFixed(2));
                    // No ref use after await until the element is known alive.
                    if (!mounted) return;
                    if (reply.error != null) {
                      // Keep the locally dragged marker on failure: no stale
                      // snapshot read must overwrite the value we just sent.
                      setState(() => _error = reply.error);
                      return;
                    }
                    ref.invalidate(settingsSnapshotProvider);
                    // A FutureProvider's snapshot is the PREVIOUS AsyncData
                    // right after refresh/invalidate in Riverpod 3.5 - await
                    // the refetch instead of reading .value synchronously.
                    final fresh = await ref.read(settingsSnapshotProvider.future);
                    if (!mounted) return;
                    setState(() {
                      _error = null;
                      _marker = fresh.minLevel; // daemon's canonical {:.2} value
                    });
                  },
                  child: SizedBox(
                    height: 120,
                    child: CustomPaint(
                      key: const Key('calibrationPainter'),
                      painter: LevelHistogramPainter(
                        samples: samples,
                        marker: marker,
                        colors: colors,
                      ),
                      child: const SizedBox.expand(),
                    ),
                  ),
                );
              },
            ),
            const SizedBox(height: 8),
            Text(
              l10n.calibrationApplyHint,
              style: TextStyle(color: colors.onSurfaceVariant),
            ),
            if (samples.isEmpty) ...[
              const SizedBox(height: 4),
              Text(
                l10n.calibrationEmptyHint,
                style: TextStyle(color: colors.onSurfaceVariant),
              ),
            ],
            if (settings?.autoStop == true) ...[
              const SizedBox(height: 4),
              Text(
                l10n.calibrationAutoStopHint,
                style: TextStyle(color: colors.onSurfaceVariant),
              ),
            ],
            if (_error != null && _error!.isNotEmpty) ...[
              const SizedBox(height: 4),
              Text(
                _error!,
                style: TextStyle(color: colors.error),
              ),
            ],
          ],
        ),
      ),
    );
  }
}

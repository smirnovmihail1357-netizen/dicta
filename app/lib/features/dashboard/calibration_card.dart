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
      // One data colour with alpha by level: no arbitrary tier roles
      // (outline/tertiary) and no false role semantics.
      final color = colors.primary.withValues(alpha: 0.35 + level * 0.65);
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
        ..color = colors.primary
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

  /// Local-only move: no I/O until the gesture or slider drag is released.
  void _previewMarker(double value) {
    setState(() => _marker = value.clamp(0.0, 1.0).toDouble());
  }

  /// One set command per completed interaction. Shared by the histogram drag
  /// and the accessible Slider so both paths stay on the same pending value.
  Future<void> _commitMarker(double value) async {
    final next = value.clamp(0.0, 1.0).toDouble();
    setState(() => _marker = next);
    // Panel format {:.2}; one command per completed gesture/slider release.
    final reply = await ref
        .read(engineProvider)
        .set('min_level', next.toStringAsFixed(2));
    // No ref use after await until the element is known alive.
    if (!mounted) return;
    if (reply.error != null) {
      // Keep the locally chosen marker on failure: no stale snapshot read
      // must overwrite the value we just sent.
      setState(() => _error = reply.error);
      return;
    }
    ref.invalidate(settingsSnapshotProvider);
    // A FutureProvider's snapshot is the PREVIOUS AsyncData right after
    // refresh/invalidate in Riverpod 3.5 - await the refetch (CR-01) instead
    // of reading .value synchronously.
    final fresh = await ref.read(settingsSnapshotProvider.future);
    if (!mounted) return;
    setState(() {
      _error = null;
      _marker = fresh.minLevel; // daemon's canonical {:.2} value
    });
  }

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
                  // Mouse shortcut only; the Slider below is the keyboard/AT
                  // path. Both feed the same pending value and commit helper.
                  onHorizontalDragUpdate: (details) {
                    _previewMarker(details.localPosition.dx / width);
                  },
                  onHorizontalDragEnd: (_) => _commitMarker(_marker ?? marker),
                  child: Semantics(
                    container: true,
                    label: l10n.calibrationHistogramLabel,
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
                  ),
                );
              },
            ),
            const SizedBox(height: 8),
            Text(
              l10n.calibrationThreshold(marker.toStringAsFixed(2)),
              style: TextStyle(
                color: colors.onSurfaceVariant,
                fontFeatures: const [FontFeature.tabularFigures()],
              ),
            ),
            Slider(
              key: const Key('calibrationSlider'),
              // Panel format is {:.2}, so 100 steps match one commit value.
              value: marker.clamp(0.0, 1.0).toDouble(),
              divisions: 100,
              label: marker.toStringAsFixed(2),
              onChanged: _previewMarker,
              onChangeEnd: _commitMarker,
            ),
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

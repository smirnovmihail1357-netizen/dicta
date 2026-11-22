/// Settings form. Tracer slice: one key (paste_delay_ms) travels the full
/// seam - slider commit -> set -> SetReply snapshot -> form, with no refetch.
/// Task 4 expands this skeleton to all 17 editable keys.
library;

import 'package:dicta_app/core/providers.dart';
import 'package:dicta_app/l10n/generated/app_localizations.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

class SettingsScreen extends ConsumerStatefulWidget {
  const SettingsScreen({super.key});

  @override
  ConsumerState<SettingsScreen> createState() => _SettingsScreenState();
}

class _SettingsScreenState extends ConsumerState<SettingsScreen> {
  int? _pendingPasteDelay;
  String? _error;

  /// One commit per completed gesture; the form then follows the reply's
  /// snapshot (Pattern 3) instead of issuing a second fetchSettings.
  Future<void> _commitPasteDelay(int value) async {
    final reply = await ref.read(engineProvider).set('paste_delay_ms', value.toString());
    if (!mounted) return;
    final notice = reply.error ?? reply.note;
    if (notice != null) {
      // Refusals stay verbatim; keep the locally chosen value on failure.
      setState(() => _error = notice);
      return;
    }
    setState(() {
      _error = null;
      _pendingPasteDelay = reply.settings.pasteDelayMs;
    });
  }

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context)!;
    final colors = Theme.of(context).colorScheme;
    final settings = ref.watch(settingsSnapshotProvider).value;
    final pasteDelay = _pendingPasteDelay ?? settings?.pasteDelayMs ?? 150;

    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        Padding(
          padding: const EdgeInsets.all(16),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text(
                l10n.confGroupInsert,
                style: Theme.of(context).textTheme.titleMedium,
              ),
              const SizedBox(height: 12),
              Text(l10n.confPasteDelayLabel),
              Slider(
                key: const Key('confPasteDelaySlider'),
                min: 0,
                max: 3000,
                divisions: 3000,
                value: pasteDelay.clamp(0, 3000).toDouble(),
                onChanged: (value) =>
                    setState(() => _pendingPasteDelay = value.toInt()),
                onChangeEnd: (value) => _commitPasteDelay(value.toInt()),
              ),
              Text(
                '$pasteDelay',
                key: const Key('confPasteDelayReadout'),
              ),
              Semantics(
                liveRegion: true,
                child: Text(
                  _error ?? '',
                  style: TextStyle(color: colors.error),
                ),
              ),
            ],
          ),
        ),
      ],
    );
  }
}

/// Honest, read-only hotkey card inside the existing Settings tab. It shows
/// what the DAEMON reads from config.toml; the app never writes config and
/// never registers a global hotkey (must-NOT 1/3, HK-02).
library;

import 'dart:io';

import 'package:dicta_app/core/config_path.dart';
import 'package:dicta_app/core/providers.dart';
import 'package:dicta_app/core/settings.dart';
import 'package:dicta_app/l10n/generated/app_localizations.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

/// Fetch-on-demand, never polled (settingsSnapshotProvider precedent). The
/// daemon's own config_path is primary; the pure resolver is only a fallback
/// for a dead daemon or a snapshot without a path. Reads the same settings
/// reply the shared snapshot parses — directly, so there is no nested-future
/// disposal race — and surfaces a failed file read as an AsyncError.
final hotkeyConfigTextProvider = FutureProvider<String>((ref) async {
  var path = resolveConfigPath();
  try {
    final reply = await ref.read(engineProvider).fetchSettings();
    final snapshot = SettingsReply.fromReply(reply).settings;
    if (snapshot.configPath.isNotEmpty) path = snapshot.configPath;
  } catch (_) {
    // Dead daemon: fall back to the known config path. A file read failure is
    // still surfaced as an AsyncError by this provider (errors-as-data).
  }
  return File(path).readAsString();
});

class HotkeyCard extends ConsumerWidget {
  const HotkeyCard({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final l10n = AppLocalizations.of(context)!;
    final colors = Theme.of(context).colorScheme;
    final config = ref.watch(hotkeyConfigTextProvider);

    return Card(
      key: const Key('hotkeyCard'),
      child: Padding(
        padding: const EdgeInsets.all(12),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              l10n.hkTitle,
              style: Theme.of(context).textTheme.titleMedium,
            ),
            const SizedBox(height: 8),
            // Riverpod 3 keeps the previous error attached to AsyncLoading
            // while refreshing, so `hasError` is the honest signal here —
            // `when()` would pick the loading branch and hide a read failure.
            if (config.hasError)
              _ConfigError(
                onRetry: () => ref.invalidate(hotkeyConfigTextProvider),
              )
            else
              config.when(
                data: (text) => _CurrentHotkey(text: text),
                loading: () =>
                    const Center(child: CircularProgressIndicator()),
                error: (_, _) => _ConfigError(
                  onRetry: () => ref.invalidate(hotkeyConfigTextProvider),
                ),
              ),
            const SizedBox(height: 12),
            // Static reference only: these are adapter combinations known in
            // advance. The commands are TEXT, never executed (Anti-Pattern).
            _deLine(l10n.hkDeKwin),
            _deLine(l10n.hkDeHyprland),
            _deLine(l10n.hkDeSway),
            _deLine(l10n.hkDeGnome),
            const SizedBox(height: 8),
            Text(l10n.hkInputGroupHint),
            const SizedBox(height: 8),
            Container(
              width: double.infinity,
              color: colors.errorContainer,
              padding: const EdgeInsets.all(8),
              child: Text(
                l10n.hkRestartWarning,
                style: TextStyle(color: colors.onErrorContainer),
              ),
            ),
            const SizedBox(height: 8),
            Text(l10n.hkEditHint),
          ],
        ),
      ),
    );
  }

  Widget _deLine(String text) =>
      Padding(padding: const EdgeInsets.only(bottom: 4), child: Text(text));
}

class _CurrentHotkey extends StatelessWidget {
  const _CurrentHotkey({required this.text});

  final String text;

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context)!;
    final value = extractHotkey(text);
    // "off" and a missing hotkey line are the daemon's honest default
    // (config.rs:90), not a read failure.
    final shown = (value == null || value.isEmpty || value == 'off')
        ? l10n.hkNotSet
        : value;

    return MergeSemantics(
      child: Semantics(
        label: '${l10n.hkCurrentLabel}: $shown',
        child: Row(
          children: [
            Text(l10n.hkCurrentLabel),
            const SizedBox(width: 8),
            Text(shown, key: const Key('hkCurrentValueField')),
          ],
        ),
      ),
    );
  }
}

class _ConfigError extends StatelessWidget {
  const _ConfigError({required this.onRetry});

  final VoidCallback onRetry;

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context)!;
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(
          l10n.hkUnavailable,
          style: TextStyle(color: Theme.of(context).colorScheme.error),
        ),
        OutlinedButton(
          onPressed: onRetry,
          child: Text(l10n.modelFetchRetry),
        ),
      ],
    );
  }
}

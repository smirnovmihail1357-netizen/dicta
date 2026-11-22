/// Installed models: the list comes from settings.models only, the active one
/// is marked, deletion is guarded and confirmed (MODEL-02). The HF browser is
/// attached below this section in Task 7.
library;

import 'package:dicta_app/core/hf/hf_catalog.dart';
import 'package:dicta_app/core/providers.dart';
import 'package:dicta_app/core/theme_ext.dart';
import 'package:dicta_app/features/models/hf_browser.dart';
import 'package:dicta_app/features/models/model_store.dart';
import 'package:dicta_app/l10n/generated/app_localizations.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

class ModelsScreen extends ConsumerWidget {
  const ModelsScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final l10n = AppLocalizations.of(context)!;
    final colors = Theme.of(context).colorScheme;
    final settingsAsync = ref.watch(settingsSnapshotProvider);
    final settings = settingsAsync.value;
    final models = settings?.models ?? const <String>[];
    final active = settings?.model ?? '';

    Future<void> selectModel(String path) async {
      final confirmed = await showDialog<bool>(
        context: context,
        builder: (context) => AlertDialog(
          title: Text(l10n.confEngineConfirmTitle),
          content: Text(l10n.confEngineConfirmBody),
          actions: [
            TextButton(
              onPressed: () => Navigator.of(context).pop(false),
              child: Text(l10n.confDecline),
            ),
            FilledButton(
              onPressed: () => Navigator.of(context).pop(true),
              child: Text(l10n.confConfirm),
            ),
          ],
        ),
      );
      if (confirmed != true) return;
      final reply = await ref.read(engineProvider).set('model', path);
      // The daemon's success note is data (main.rs:737), not a refusal. Only a
      // real error stops the mandatory done -> invalidate + await refetch that
      // makes the active-model marker follow the switched model. A refusal is
      // never silent: show it verbatim (phase-3 inject-failure snackbar canon:
      // .arb title + daemon body, both in the error tone).
      if (reply.error != null) {
        if (context.mounted) {
          ScaffoldMessenger.of(context).showSnackBar(
            SnackBar(
              content: Column(
                mainAxisSize: MainAxisSize.min,
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text(
                    l10n.modelSelectFailed,
                    style: TextStyle(color: colors.error),
                  ),
                  Text(
                    reply.error!,
                    style: TextStyle(color: colors.error),
                  ),
                ],
              ),
            ),
          );
        }
        return;
      }
      try {
        ref.invalidate(settingsSnapshotProvider);
        await ref.read(settingsSnapshotProvider.future);
      } catch (_) {
        // The set already succeeded; only the marker refresh failed. Never
        // let a transport error escape the tap handler as an unhandled zone
        // error after the confirm dialog closed.
        if (context.mounted) {
          ScaffoldMessenger.of(context).showSnackBar(
            SnackBar(content: Text(l10n.daemonUnavailable)),
          );
        }
      }
    }

    Future<void> deleteModelWithConfirm(String path) async {
      final confirmed = await showDialog<bool>(
        context: context,
        builder: (context) => AlertDialog(
          title: Text(l10n.modelDeleteConfirmTitle),
          content: Text(l10n.modelDeleteConfirmBody),
          actions: [
            TextButton(
              onPressed: () => Navigator.of(context).pop(false),
              child: Text(l10n.modelDecline),
            ),
            FilledButton(
              // Destructive canon: error background + onError foreground.
              style: FilledButton.styleFrom(
                backgroundColor: colors.error,
                foregroundColor: colors.onError,
              ),
              onPressed: () => Navigator.of(context).pop(true),
              child: Text(l10n.modelDeleteConfirm),
            ),
          ],
        ),
      );
      if (confirmed != true) return;
      // Re-read the live snapshot so a stale build cannot delete the active
      // model after it changed between render and confirmation.
      final live = await ref.read(settingsSnapshotProvider.future);
      final decision = canDelete(path, live.model, destDir(live.model));
      if (decision != DeleteDecision.allowed) {
        if (context.mounted) {
          ScaffoldMessenger.of(context).showSnackBar(
            SnackBar(content: Text(l10n.modelDeleteBlocked)),
          );
        }
        return;
      }
      final failure = await deleteModel(path);
      if (!context.mounted) return;
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text(failure ?? l10n.modelDeleted)),
      );
      // No daemon reply carries the new list: a refetch is mandatory here.
      try {
        ref.invalidate(settingsSnapshotProvider);
        await ref.read(settingsSnapshotProvider.future);
      } catch (_) {
        // The file is already deleted; a failed refresh must not throw out of
        // this async handler. The honest "удалено" feedback above still holds.
        if (context.mounted) {
          ScaffoldMessenger.of(context).showSnackBar(
            SnackBar(content: Text(l10n.daemonUnavailable)),
          );
        }
      }
    }

    return SingleChildScrollView(
      padding: const EdgeInsets.all(16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Text(
            l10n.modelInstalledTitle,
            style: Theme.of(context).textTheme.titleMedium,
          ),
          const SizedBox(height: 8),
          if (settingsAsync is AsyncError)
            Card(
              child: Padding(
                padding: const EdgeInsets.all(16),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.stretch,
                  children: [
                    Text(
                      l10n.modelFetchFailed,
                      key: const Key('modelFetchError'),
                      style: TextStyle(color: colors.error),
                    ),
                    const SizedBox(height: 8),
                    Align(
                      alignment: Alignment.centerLeft,
                      child: OutlinedButton(
                        key: const Key('modelFetchRetryButton'),
                        onPressed: () => ref.invalidate(settingsSnapshotProvider),
                        child: Text(l10n.modelFetchRetry),
                      ),
                    ),
                  ],
                ),
              ),
            )
          else if (models.isEmpty && settingsAsync is AsyncData)
            Card(
              child: Padding(
                padding: const EdgeInsets.all(16),
                child: Text(
                  l10n.modelEmpty,
                  style: AppTextStyles.mutedOf(context),
                ),
              ),
            )
          else
            ...models.map((path) {
              final name = modelName(path);
              final isActive = path == active;
              return Card(
                child: Padding(
                  padding: const EdgeInsets.all(12),
                  child: Row(
                    children: [
                      Expanded(
                        child: Column(
                          crossAxisAlignment: CrossAxisAlignment.start,
                          children: [
                            Text(name),
                            if (isActive)
                              Text(
                                l10n.modelActive,
                                style: AppTextStyles.mutedOf(context),
                              ),
                          ],
                        ),
                      ),
                      if (!isActive) ...[
                        FilledButton.tonal(
                          key: Key('modelSelectButton-$name'),
                          onPressed: () => selectModel(path),
                          child: Text(l10n.modelSelectButton),
                        ),
                        const SizedBox(width: 8),
                      ],
                      OutlinedButton(
                        key: Key('modelDeleteButton-$name'),
                        // Destructive canon: error foreground + error side.
                        style: OutlinedButton.styleFrom(
                          foregroundColor: colors.error,
                          side: BorderSide(color: colors.error),
                        ),
                        onPressed:
                            isActive ? null : () => deleteModelWithConfirm(path),
                        child: Text(l10n.modelDeleteButton),
                      ),
                    ],
                  ),
                ),
              );
            }),
          const HfBrowser(),
        ],
      ),
    );
  }
}

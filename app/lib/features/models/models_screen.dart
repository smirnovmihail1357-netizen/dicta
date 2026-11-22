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
    final settings = ref.watch(settingsSnapshotProvider).value;
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
      if (reply.error != null || reply.note != null) return;
      ref.invalidate(settingsSnapshotProvider);
      await ref.read(settingsSnapshotProvider.future);
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
              style: FilledButton.styleFrom(backgroundColor: colors.error),
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
      ref.invalidate(settingsSnapshotProvider);
      await ref.read(settingsSnapshotProvider.future);
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
          if (models.isEmpty)
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

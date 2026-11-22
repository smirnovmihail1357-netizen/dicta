/// Browsable in-memory journal of dictations made through the app (HIST-02/03).
/// Entries arrive only from the stop seam; search is a pure build-time filter.
/// No persistence, no daemon commands of its own, no sixth muted-style copy.
library;

import 'package:dicta_app/core/theme_ext.dart';
import 'package:dicta_app/features/dashboard/dicta_control.dart';
import 'package:dicta_app/features/history/history.dart';
import 'package:dicta_app/l10n/generated/app_localizations.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

class HistoryScreen extends ConsumerWidget {
  const HistoryScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final l10n = AppLocalizations.of(context)!;
    final colors = Theme.of(context).colorScheme;
    final entries = ref.watch(historyProvider);
    final query = ref.watch(historyQueryProvider);
    final busy = ref.watch(dictaControlProvider).commandInFlight;
    final filtered = entries
        .where((e) => e.text.toLowerCase().contains(query.toLowerCase()))
        .toList();

    return Column(
      children: [
        Padding(
          padding: const EdgeInsets.all(16),
          // Merge the mandated label into the editable node: a bare Semantics
          // wrapper would create a second, non-editable focus stop (WR-01).
          child: MergeSemantics(
            child: Semantics(
              label: l10n.histSearchSemantics,
              child: TextField(
                key: const Key('historySearchField'),
                decoration: InputDecoration(
                  hintText: l10n.histSearchHint,
                  prefixIcon: const Icon(Icons.search),
                  // Clear suffix only for a non-empty query: one tap removes
                  // the filter without a second focus stop when it is empty.
                  suffixIcon: query.isEmpty
                      ? null
                      : IconButton(
                          key: const Key('historyClearSearchButton'),
                          icon: const Icon(Icons.close),
                          onPressed: () =>
                              ref.read(historyQueryProvider.notifier).set(''),
                        ),
                  border: const OutlineInputBorder(),
                ),
                // No debounce and no TextEditingController double state: the
                // provider IS the query and the filter is a pure build function.
                onChanged: (value) =>
                    ref.read(historyQueryProvider.notifier).set(value),
              ),
            ),
          ),
        ),
        Expanded(
          child: entries.isEmpty
              ? Card(
                  margin: const EdgeInsets.symmetric(horizontal: 16),
                  child: Padding(
                    padding: const EdgeInsets.all(16),
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      mainAxisSize: MainAxisSize.min,
                      children: [
                        Text(
                          l10n.histEmpty,
                          style: Theme.of(context).textTheme.titleMedium,
                        ),
                        const SizedBox(height: 8),
                        // Honest A4 limit: hotkey dictations are invisible.
                        Text(
                          l10n.histEmptyHint,
                          style: AppTextStyles.mutedOf(context),
                        ),
                      ],
                    ),
                  ),
                )
              : filtered.isEmpty
                  ? Card(
                      margin: const EdgeInsets.symmetric(horizontal: 16),
                      child: Padding(
                        padding: const EdgeInsets.all(16),
                        child: Text(
                          l10n.histSearchNoMatches,
                          style: Theme.of(context).textTheme.titleMedium,
                        ),
                      ),
                    )
                  : ListView.builder(
                      // Bottom padding keeps the last row off the screen edge.
                      padding: const EdgeInsets.fromLTRB(16, 0, 16, 16),
                      itemCount: filtered.length,
                      // State is already newest-first; no reverse needed.
                      itemBuilder: (context, index) {
                        final entry = filtered[index];
                        return Card(
                          child: Padding(
                            padding: const EdgeInsets.all(16),
                            child: Column(
                              crossAxisAlignment: CrossAxisAlignment.stretch,
                              children: [
                                Text(
                                  formatEntryTime(entry.at, l10n.localeName),
                                  style: AppTextStyles.mutedOf(context)
                                      .copyWith(
                                        fontFeatures: const [
                                          FontFeature.tabularFigures(),
                                        ],
                                      ),
                                ),
                                const SizedBox(height: 8),
                                SelectableText(entry.text),
                                const SizedBox(height: 12),
                                Row(
                                  children: [
                                    Expanded(
                                      child: ElevatedButton(
                                        key: Key('histCopyButton-${entry.id}'),
                                        onPressed: () {
                                          // Explicit click only.
                                          ref.read(clipboardWriterProvider)(
                                              entry.text);
                                          ScaffoldMessenger.of(context)
                                              .showSnackBar(
                                            SnackBar(
                                              content: Text(l10n.dashCopied),
                                            ),
                                          );
                                        },
                                        child: Text(l10n.dashCopyButton),
                                      ),
                                    ),
                                    const SizedBox(width: 8),
                                    Expanded(
                                      // Re-insert is the priority action on the
                                      // row: tonal sits above the plain copy
                                      // button, matching the dashboard canon.
                                      child: FilledButton.tonal(
                                        key: Key('histInjectButton-${entry.id}'),
                                        // The single insertion path stays
                                        // behind the commandInFlight guard.
                                        onPressed: busy
                                            ? null
                                            : () async {
                                                await ref
                                                    .read(
                                                      dictaControlProvider
                                                          .notifier,
                                                    )
                                                    .inject(entry.text);
                                                if (!context.mounted) return;
                                                final result = ref.read(
                                                    dictaControlProvider);
                                                if (result.lastError == null &&
                                                    !result.genericFailure) {
                                                  ScaffoldMessenger.of(context)
                                                      .showSnackBar(
                                                    SnackBar(
                                                      content: Text(
                                                          l10n.dashInjected),
                                                    ),
                                                  );
                                                } else {
                                                  // Mirror the last-text card's
                                                  // inject-failure display: a
                                                  // verbatim daemon body, or the
                                                  // localised generic one.
                                                  ScaffoldMessenger.of(context)
                                                      .showSnackBar(
                                                    SnackBar(
                                                      content: Column(
                                                        mainAxisSize:
                                                            MainAxisSize.min,
                                                        crossAxisAlignment:
                                                            CrossAxisAlignment
                                                                .start,
                                                        children: [
                                                          Text(
                                                            l10n.dashInjectFailed,
                                                            style: TextStyle(
                                                                color: colors
                                                                    .error),
                                                          ),
                                                          Text(
                                                            result.genericFailure
                                                                ? l10n.dashCommandFailedBody
                                                                : result.lastError!,
                                                            style: TextStyle(
                                                                color: colors
                                                                    .error),
                                                          ),
                                                        ],
                                                      ),
                                                    ),
                                                  );
                                                }
                                              },
                                        child: Text(l10n.dashInjectButton),
                                      ),
                                    ),
                                  ],
                                ),
                                const SizedBox(height: 4),
                                Align(
                                  alignment: Alignment.centerLeft,
                                  // Local removal only: no daemon command.
                                  child: OutlinedButton(
                                    key: Key('histDeleteButton-${entry.id}'),
                                    // Destructive intent: error foreground on
                                    // an outlined shell (cancel-button canon).
                                    style: OutlinedButton.styleFrom(
                                      foregroundColor: colors.error,
                                      side: BorderSide(color: colors.error),
                                    ),
                                    onPressed: () {
                                      // Capture the position in the full
                                      // journal (not the filtered view) before
                                      // removal so undo restores the order.
                                      final removalIndex = entries.indexWhere(
                                          (e) => e.id == entry.id);
                                      ref
                                          .read(historyProvider.notifier)
                                          .remove(entry.id);
                                      ScaffoldMessenger.of(context)
                                          .showSnackBar(
                                        SnackBar(
                                          content: Text(l10n.histDeleted),
                                          action: SnackBarAction(
                                            label: l10n.histUndo,
                                            onPressed: () => ref
                                                .read(historyProvider.notifier)
                                                .restoreAt(
                                                    removalIndex, entry),
                                          ),
                                        ),
                                      );
                                    },
                                    child: Text(l10n.histDeleteButton),
                                  ),
                                ),
                              ],
                            ),
                          ),
                        );
                      },
                    ),
        ),
      ],
    );
  }
}

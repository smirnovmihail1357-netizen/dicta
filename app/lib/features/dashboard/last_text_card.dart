/// Last recognized text: only text from our own action replies is shown.
/// Hotkey dictations are invisible to the protocol, so the honest empty state
/// says so instead of inventing text (DASH-03).
library;

import 'package:dicta_app/features/dashboard/dicta_control.dart';
import 'package:dicta_app/l10n/generated/app_localizations.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

class LastTextCard extends ConsumerWidget {
  const LastTextCard({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final l10n = AppLocalizations.of(context)!;
    final control = ref.watch(dictaControlProvider);
    final text = control.lastText;
    final hasText = text != null && text.isNotEmpty;
    final busy = control.commandInFlight;
    final colors = Theme.of(context).colorScheme;
    final weak = TextStyle(color: colors.onSurfaceVariant);

    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            Text(
              l10n.dashLastTextTitle,
              style: Theme.of(context).textTheme.titleMedium,
            ),
            const SizedBox(height: 12),
            if (hasText)
              SelectableText(text)
            else
              Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text(l10n.dashNoText),
                  const SizedBox(height: 4),
                  Text(l10n.dashNoTextHint, style: weak),
                ],
              ),
            const SizedBox(height: 12),
            Row(
              children: [
                Expanded(
                  child: ElevatedButton(
                    key: const Key('dashCopyButton'),
                    onPressed: hasText && !busy
                        ? () {
                            // Explicit click only; no background clipboard writes.
                            ref.read(clipboardWriterProvider)(text);
                            ScaffoldMessenger.of(context).showSnackBar(
                              SnackBar(content: Text(l10n.dashCopied)),
                            );
                          }
                        : null,
                    child: Text(l10n.dashCopyButton),
                  ),
                ),
                const SizedBox(width: 8),
                Expanded(
                  child: ElevatedButton(
                    key: const Key('dashInjectButton'),
                    onPressed: hasText && !busy
                        ? () async {
                            await ref
                                .read(dictaControlProvider.notifier)
                                .inject(text);
                            // No BuildContext/ref use after await while dead.
                            if (!context.mounted) return;
                            final result = ref.read(dictaControlProvider);
                            if (result.lastError == null &&
                                !result.genericFailure) {
                              ScaffoldMessenger.of(context).showSnackBar(
                                SnackBar(content: Text(l10n.dashInjected)),
                              );
                            }
                          }
                        : null,
                    child: Text(l10n.dashInjectButton),
                  ),
                ),
              ],
            ),
            const SizedBox(height: 8),
            Text(l10n.dashInjectHint, style: weak),
            if ((control.lastError != null && control.lastError!.isNotEmpty) ||
                control.genericFailure) ...[
              const SizedBox(height: 8),
              Text(
                // The heading is shared by all commands; only inject gets
                // the inject-specific one (phase 2 WR-01).
                control.lastErrorCommand == DictaCommand.inject
                    ? l10n.dashInjectFailed
                    : l10n.dashCommandFailed,
                style: TextStyle(color: colors.error),
              ),
              // Daemon error text is data: shown verbatim. A command that
              // threw before a daemon reply uses the localised body instead —
              // no user-visible Russian text is stored in Dart.
              Text(
                control.genericFailure
                    ? l10n.dashCommandFailedBody
                    : control.lastError!,
                style: TextStyle(color: colors.error),
              ),
            ],
          ],
        ),
      ),
    );
  }
}

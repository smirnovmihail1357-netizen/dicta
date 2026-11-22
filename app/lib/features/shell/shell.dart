/// Material You shell: four destinations, placeholder bodies and a persistent
/// daemon-unavailable banner. Widgets read only providers - no transport code
/// lives under features/ (APP-03 seam).
library;

import 'package:dicta_app/core/providers.dart';
import 'package:dicta_app/features/dashboard/dashboard_screen.dart';
import 'package:dicta_app/l10n/generated/app_localizations.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

class ShellWidget extends ConsumerStatefulWidget {
  const ShellWidget({super.key});

  @override
  ConsumerState<ShellWidget> createState() => _ShellWidgetState();
}

class _ShellWidgetState extends ConsumerState<ShellWidget> {
  int _selected = 0;

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context)!;
    final status = ref.watch(dictaStatusProvider);
    final colors = Theme.of(context).colorScheme;

    return Scaffold(
      body: Column(
        children: [
          if (!status.connected)
            Semantics(
              liveRegion: true,
              child: Container(
                width: double.infinity,
                color: colors.errorContainer,
                padding:
                    const EdgeInsets.symmetric(vertical: 8, horizontal: 16),
                child: Text(
                  l10n.daemonUnavailable,
                  textAlign: TextAlign.center,
                  style: TextStyle(color: colors.onErrorContainer),
                ),
              ),
            ),
          Expanded(
            child: Row(
              children: [
                NavigationRail(
                  selectedIndex: _selected,
                  onDestinationSelected: (index) =>
                      setState(() => _selected = index),
                  labelType: NavigationRailLabelType.all,
                  destinations: [
                    NavigationRailDestination(
                      icon: const Icon(Icons.space_dashboard_outlined),
                      selectedIcon: const Icon(Icons.space_dashboard),
                      label: Text(l10n.dashboardTitle),
                    ),
                    NavigationRailDestination(
                      icon: const Icon(Icons.history_outlined),
                      selectedIcon: const Icon(Icons.history),
                      label: Text(l10n.historyTitle),
                    ),
                    NavigationRailDestination(
                      icon: const Icon(Icons.memory_outlined),
                      selectedIcon: const Icon(Icons.memory),
                      label: Text(l10n.modelsTitle),
                    ),
                    NavigationRailDestination(
                      icon: const Icon(Icons.settings_outlined),
                      selectedIcon: const Icon(Icons.settings),
                      label: Text(l10n.settingsTitle),
                    ),
                  ],
                ),
                const VerticalDivider(width: 1),
                Expanded(
                  child: IndexedStack(
                    index: _selected,
                    children: const [
                      DashboardScreen(),
                      _SectionPlaceholder(),
                      _SectionPlaceholder(),
                      _SectionPlaceholder(),
                    ],
                  ),
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }
}

class _SectionPlaceholder extends StatelessWidget {
  const _SectionPlaceholder();

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context)!;
    return Center(
      child: Text(
        l10n.placeholderScreen,
        style: TextStyle(
          color: Theme.of(context).colorScheme.onSurfaceVariant,
        ),
      ),
    );
  }
}

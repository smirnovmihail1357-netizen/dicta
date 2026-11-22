import 'package:dicta_app/features/shell/shell.dart';
import 'package:dicta_app/l10n/app_localizations.dart';
import 'package:dynamic_color/dynamic_color.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:window_manager/legacy.dart';

Future<void> main() async {
  WidgetsFlutterBinding.ensureInitialized();
  await windowManager.ensureInitialized();

  final options = WindowOptions(
    title: lookupAppLocalizations(const Locale('ru')).appTitle,
    minimumSize: const Size(800, 600),
    center: true,
  );
  windowManager.waitUntilReadyToShow(options, () async {
    await windowManager.show();
    await windowManager.focus();
  });

  runApp(const ProviderScope(child: DictaApp()));
}

class DictaApp extends StatelessWidget {
  const DictaApp({super.key});

  @override
  Widget build(BuildContext context) {
    return DynamicColorBuilder(
      builder: (lightDynamic, darkDynamic) {
        // Flutter 3.50 splits dynamic_color's ColorScheme (material_ui) from
        // the deprecated flutter/material.dart shim. Feed the portal accent
        // into this app's ColorScheme.fromSeed instead of harmonizing across
        // two incompatible type hierarchies.
        const fallback = 0xFF35689F;
        final light = ColorScheme.fromSeed(
          seedColor: Color(lightDynamic?.primary.toARGB32() ?? fallback),
        );
        final dark = ColorScheme.fromSeed(
          seedColor: Color(darkDynamic?.primary.toARGB32() ?? fallback),
          brightness: Brightness.dark,
        );
        return MaterialApp(
          theme: ThemeData(colorScheme: light, useMaterial3: true),
          darkTheme: ThemeData(colorScheme: dark, useMaterial3: true),
          localizationsDelegates: AppLocalizations.localizationsDelegates,
          supportedLocales: AppLocalizations.supportedLocales,
          locale: const Locale('ru'),
          home: const ShellWidget(),
        );
      },
    );
  }
}

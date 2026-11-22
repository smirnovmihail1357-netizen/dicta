import 'package:flutter/material.dart';

/// Entry point. Task 3 replaces this with ProviderScope + DynamicColorBuilder
/// + MaterialApp carrying l10n and live daemon state.
void main() {
  runApp(const PlaceholderApp());
}

class PlaceholderApp extends StatelessWidget {
  const PlaceholderApp({super.key});

  @override
  Widget build(BuildContext context) {
    return const MaterialApp(home: SizedBox.shrink());
  }
}

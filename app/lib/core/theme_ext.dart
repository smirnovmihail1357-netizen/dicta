/// Cross-feature text styles that Material 3 tokens do not cover.
/// Only the muted caption style lives here: everything else stays on tokens.
/// This extension must be registered in EVERY ThemeData: main.dart (both
/// themes), buildFakeApp in fake_dicta_engine.dart, and any suite-local
/// scaffold (see buildTestApp in shell_test.dart) — otherwise
/// [AppTextStyles.mutedOf] asserts. Check with `grep -rn "AppTextStyles.from" lib test`.
library;

import 'package:flutter/material.dart';

class AppTextStyles extends ThemeExtension<AppTextStyles> {
  const AppTextStyles({required this.muted});

  final TextStyle muted;

  factory AppTextStyles.from(ColorScheme scheme) =>
      AppTextStyles(muted: TextStyle(color: scheme.onSurfaceVariant));

  @override
  AppTextStyles copyWith({TextStyle? muted}) =>
      AppTextStyles(muted: muted ?? this.muted);

  @override
  AppTextStyles lerp(covariant AppTextStyles? other, double t) {
    if (other == null) return this;
    return AppTextStyles(muted: TextStyle.lerp(muted, other.muted, t)!);
  }

  /// Single source of the muted text style. The bang is deliberate: the
  /// extension is mandatory in both app and test themes.
  static TextStyle mutedOf(BuildContext context) {
    final styles = Theme.of(context).extension<AppTextStyles>();
    assert(
      styles != null,
      'AppTextStyles must be registered in every ThemeData',
    );
    return styles!.muted;
  }
}

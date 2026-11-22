/// Read-only helpers for the daemon's flat config.toml. The file is written
/// line-by-line by dictad itself (config.rs:180-192), so no TOML package is
/// needed — and no new dependency is allowed (must-NOT 7).
library;

import 'dart:io';

/// Mirrors dicta-core's `config_path()` (config.rs:173-178):
/// DICTA_CONFIG → XDG_CONFIG_HOME (or ~/.config) → dicta/config.toml, with
/// "dicta-config.toml" as the honest last resort when no home can be found.
///
/// Parameters are injectable because Platform.environment is immutable in
/// widget tests (the DaemonEngine([socketPath]) precedent).
String resolveConfigPath({
  String? configOverride,
  String? xdgConfigHome,
  String? home,
  String? appData,
}) {
  final override = configOverride ?? Platform.environment['DICTA_CONFIG'];
  // An empty override is treated as unset: the zero-length path that the
  // daemon's var_os("") edge would produce is never a usable config file.
  if (override != null && override.isNotEmpty) return override;

  // Windows: %APPDATA% (roaming). This is a CONTRACT for phase 7, not a mirror
  // of config.rs — no Rust parity reference exists for Windows yet. This branch
  // must stay BEFORE the Unix XDG resolution: on Windows those variables are
  // absent and the last-resort path would otherwise win.
  if (Platform.isWindows) return windowsConfigPath(appData: appData);

  final xdg = xdgConfigHome ?? Platform.environment['XDG_CONFIG_HOME'];
  // dirs::config_dir() (config.rs:176) accepts ONLY an absolute XDG_CONFIG_HOME;
  // a relative value is discarded and $HOME/.config is used instead. Keep that
  // edge in step so the card cannot read a different file than the daemon.
  if (xdg != null && xdg.isNotEmpty && xdg.startsWith('/')) {
    return '$xdg/dicta/config.toml';
  }

  final userHome = home ?? Platform.environment['HOME'];
  if (userHome == null || userHome.isEmpty) return 'dicta-config.toml';
  return '$userHome/.config/dicta/config.toml';
}

/// Windows contract (phase 7): `%APPDATA%\dicta\config.toml`. No Rust parity
/// reference exists yet — this records the CONTRACT, not a mirror of config.rs.
/// The env read is injectable so widget tests never mutate Platform.environment.
String windowsConfigPath({String? appData}) {
  final roaming = appData ?? Platform.environment['APPDATA'];
  if (roaming == null || roaming.isEmpty) return r'dicta\config.toml';
  return '$roaming\\dicta\\config.toml';
}

// TOML allows both basic ("...") and literal ('...') strings. The daemon's
// real parser accepts either, so the honest card must too (WR-04): this key is
// written by the user's hand, not by set_key. Quote pairing is not enforced.
final RegExp _hotkeyRe = RegExp(r'''^\s*hotkey\s*=\s*["']([^"']*)["']''');

/// Extracts the evdev hotkey value from flat TOML text. Line based, so a
/// trailing `#` comment is ignored and a stray section header changes nothing.
/// No match → null → the card shows the daemon's default "off" honestly.
String? extractHotkey(String configText) {
  for (final line in configText.split('\n')) {
    final match = _hotkeyRe.firstMatch(line);
    if (match != null) return match.group(1);
  }
  return null;
}

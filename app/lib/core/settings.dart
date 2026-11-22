/// Dart mirror of the daemon's settings snapshot (the deliberate 4th copy).
///
/// The daemon is the source of truth: this model only deserializes its reply.
/// The golden fixture test is the drift tripwire. Unknown keys are tolerated
/// and missing or mistyped fields fall back to documented defaults, the Dart
/// equivalent of the panel's `#[serde(default)]`. A `reloading: true` on a
/// freshly booted daemon is normal, not an error.
library;

const Set<String> kEditableKeys = {
  'min_level',
  'auto_stop',
  'auto_stop_level',
  'auto_stop_after_ms',
  'auto_stop_grace_ms',
  'max_record_ms',
  'inject_mode',
  'paste_keys',
  'clipboard_restore_ms',
  'paste_delay_ms',
  'atspi_allow_any_role',
  'language',
  'model',
  'use_gpu',
  'gpu_device',
  'n_threads',
  'beam_size',
};

const Set<String> kEngineKeys = {
  'model',
  'language',
  'use_gpu',
  'gpu_device',
  'n_threads',
  'beam_size',
};

String _string(Map<String, dynamic> json, String key, String fallback) {
  final value = json[key];
  return value is String ? value : fallback;
}

double _double(Map<String, dynamic> json, String key, double fallback) {
  final value = json[key];
  return value is num ? value.toDouble() : fallback;
}

int _int(Map<String, dynamic> json, String key, int fallback) {
  final value = json[key];
  return value is num ? value.toInt() : fallback;
}

bool _bool(Map<String, dynamic> json, String key, bool fallback) {
  final value = json[key];
  return value is bool ? value : fallback;
}

List<String> _strings(Map<String, dynamic> json, String key) {
  final value = json[key];
  if (value is List) return value.whereType<String>().toList(growable: false);
  return const <String>[];
}

class SettingsReply {
  const SettingsReply({
    this.language = 'ru',
    this.injectMode = 'paste',
    this.minLevel = 0.0,
    this.autoStop = false,
    this.autoStopLevel = 0.22,
    this.autoStopAfterMs = 1500,
    this.autoStopGraceMs = 1200,
    this.maxRecordMs = 60000,
    this.useGpu = true,
    this.gpuDevice = 0,
    this.nThreads = 4,
    this.beamSize = 5,
    this.pasteKeys = 'ctrl+v',
    this.clipboardRestoreMs = 2500,
    this.pasteDelayMs = 150,
    this.atspiAllowAnyRole = false,
    this.model = '',
    this.models = const <String>[],
    this.reloading = false,
    this.configPath = '',
  });

  /// Hand-rolled equivalent of `#[serde(default)]` on the 20-field struct.
  factory SettingsReply.fromJson(Map<String, dynamic> json) => SettingsReply(
        language: _string(json, 'language', 'ru'),
        injectMode: _string(json, 'inject_mode', 'paste'),
        minLevel: _double(json, 'min_level', 0.0),
        autoStop: _bool(json, 'auto_stop', false),
        autoStopLevel: _double(json, 'auto_stop_level', 0.22),
        autoStopAfterMs: _int(json, 'auto_stop_after_ms', 1500),
        autoStopGraceMs: _int(json, 'auto_stop_grace_ms', 1200),
        maxRecordMs: _int(json, 'max_record_ms', 60000),
        useGpu: _bool(json, 'use_gpu', true),
        gpuDevice: _int(json, 'gpu_device', 0),
        nThreads: _int(json, 'n_threads', 4),
        beamSize: _int(json, 'beam_size', 5),
        pasteKeys: _string(json, 'paste_keys', 'ctrl+v'),
        clipboardRestoreMs: _int(json, 'clipboard_restore_ms', 2500),
        pasteDelayMs: _int(json, 'paste_delay_ms', 150),
        atspiAllowAnyRole: _bool(json, 'atspi_allow_any_role', false),
        model: _string(json, 'model', ''),
        models: _strings(json, 'models'),
        reloading: _bool(json, 'reloading', false),
        configPath: _string(json, 'config_path', ''),
      );

  /// Poisoned replies never throw: broken `settings` yields all defaults and
  /// the error is taken from the top-level field (legacy `base.error` too).
  static ({SettingsReply settings, String? error}) fromReply(
    Map<String, dynamic> reply,
  ) {
    final raw = reply['settings'];
    final settings = raw is Map<String, dynamic>
        ? SettingsReply.fromJson(raw)
        : const SettingsReply();

    final topLevel = reply['error'];
    final base = reply['base'];
    final legacy = base is Map<String, dynamic> ? base['error'] : null;
    final error = topLevel is String
        ? topLevel
        : legacy is String
            ? legacy
            : null;
    return (settings: settings, error: error);
  }

  final String language;
  final String injectMode;
  final double minLevel;
  final bool autoStop;
  final double autoStopLevel;
  final int autoStopAfterMs;
  final int autoStopGraceMs;
  final int maxRecordMs;
  final bool useGpu;
  final int gpuDevice;
  final int nThreads;
  final int beamSize;
  final String pasteKeys;
  final int clipboardRestoreMs;
  final int pasteDelayMs;
  final bool atspiAllowAnyRole;
  final String model;
  final List<String> models;
  final bool reloading;
  final String configPath;
}

/// The set reply always carries a fresh settings snapshot (dictad
/// main.rs:730-739). `note` is the reply's text and may hold the engine-reload
/// refusal while `error` stays null; `error` is the top-level/transport error.
class SetReply {
  const SetReply({this.note, this.error, required this.settings});

  /// Reuses the single snapshot parser in SettingsReply.fromReply - there is
  /// deliberately no second settings parser for the set reply.
  factory SetReply.fromReply(Map<String, dynamic> reply) {
    final parsed = SettingsReply.fromReply(reply);
    return SetReply(
      note: reply['text'] as String?,
      error: parsed.error,
      settings: parsed.settings,
    );
  }

  final String? note;
  final String? error;
  final SettingsReply settings;
}

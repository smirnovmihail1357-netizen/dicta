/// Dart mirror of the daemon's settings snapshot (the deliberate 4th copy).
/// The golden fixture test is the drift tripwire; daemon stays the source of truth.
library;

const Set<String> kEditableKeys = {
  'min_level', 'auto_stop', 'auto_stop_level', 'auto_stop_after_ms',
  'auto_stop_grace_ms', 'max_record_ms', 'inject_mode', 'paste_keys',
  'clipboard_restore_ms', 'paste_delay_ms', 'atspi_allow_any_role',
  'language', 'model', 'use_gpu', 'gpu_device', 'n_threads', 'beam_size',
};

const Set<String> kEngineKeys = {
  'model', 'language', 'use_gpu', 'gpu_device', 'n_threads', 'beam_size',
};

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

  factory SettingsReply.fromJson(Map<String, dynamic> json) =>
      const SettingsReply();

  static ({SettingsReply settings, String? error}) fromReply(
          Map<String, dynamic> reply) =>
      (settings: const SettingsReply(), error: null);

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

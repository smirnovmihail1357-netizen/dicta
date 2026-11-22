/// Settings form: all 17 editable keys in four groups, one set per completed
/// gesture, confirm before every ENGINE set, and honest reload states. The
/// form follows SetReply snapshots - never a refetch after a successful set.
library;

import 'dart:async';

import 'package:dicta_app/core/providers.dart';
import 'package:dicta_app/core/settings.dart';
import 'package:dicta_app/core/theme_ext.dart';
import 'package:dicta_app/l10n/generated/app_localizations.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

class SettingsScreen extends ConsumerStatefulWidget {
  const SettingsScreen({super.key});

  @override
  ConsumerState<SettingsScreen> createState() => _SettingsScreenState();
}

class _SettingsScreenState extends ConsumerState<SettingsScreen> {
  /// Unsent local edits, keyed by daemon setting key. onChanged only writes
  /// here; onChangeEnd sends exactly one set command.
  final Map<String, String> _pending = {};

  /// Last successful set snapshot. It overrides the provider value so the form
  /// never needs a second fetchSettings (Pattern 3).
  SettingsReply? _replySnapshot;

  String? _error;

  /// The liveRegion status line carries EITHER a transport/validation error
  /// (red) OR the daemon's success note (neutral, like l10n.diagModelLoading).
  /// A note is DATA, never an error - main.rs:737 sends `text: Some(note)` on
  /// every successful set with `error: None`.
  bool _statusIsError = false;
  Timer? _reloadTimer;

  final _pasteKeysController = TextEditingController();
  final _configPathController = TextEditingController();
  final _pasteKeysFocus = FocusNode();

  @override
  void dispose() {
    _reloadTimer?.cancel();
    _pasteKeysController.dispose();
    _configPathController.dispose();
    _pasteKeysFocus.dispose();
    super.dispose();
  }

  double _sliderValue(String key, num fallback) {
    final raw = _pending[key];
    if (raw == null) return fallback.toDouble();
    // Pending values are raw wire strings ('1500'). Tolerate a legacy
    // unit-suffixed value ('1500 мс') too so a readout can never freeze.
    final match = RegExp(r'^-?\d+(?:\.\d+)?').firstMatch(raw.trim());
    final parsed = match == null ? null : num.tryParse(match.group(0)!);
    return (parsed ?? fallback).toDouble();
  }

  bool _flag(String key, bool fallback) {
    final value = _pending[key];
    return value == null ? fallback : value == 'on';
  }

  String _text(String key, String fallback) => _pending[key] ?? fallback;

  void _preview(String key, String value) =>
      setState(() => _pending[key] = value);

  void _stopReloadPolling() {
    _reloadTimer?.cancel();
    _reloadTimer = null;
  }

  void _startReloadPolling() {
    if (_reloadTimer != null) return;
    // A reloading snapshot is transient: re-fetch once a second until the
    // daemon says it finished (CONTEXT:20 permits this timer).
    _reloadTimer = Timer.periodic(const Duration(seconds: 1), (_) async {
      ref.invalidate(settingsSnapshotProvider);
      try {
        final fresh = await ref.read(settingsSnapshotProvider.future);
        if (!mounted) return;
        setState(() => _replySnapshot = fresh);
        if (!fresh.reloading) _stopReloadPolling();
      } catch (_) {
        // A daemon that died mid-reload must not turn this into a 1 Hz
        // unhandled-error loop: stop polling and show the honest state.
        _stopReloadPolling();
        if (mounted) {
          setState(() {
            _error = AppLocalizations.of(context)!.daemonUnavailable;
            _statusIsError = true;
          });
        }
      }
    });
  }

  Future<void> _commit(String key, String value) async {
    if (kEngineKeys.contains(key)) {
      final l10n = AppLocalizations.of(context)!;
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
      if (!mounted || confirmed != true) return;
    }

    final reply = await ref.read(engineProvider).set(key, value);
    if (!mounted) return;
    if (reply.error != null) {
      // A real error keeps the local edit untouched so the user can retry.
      setState(() {
        _error = reply.error;
        _statusIsError = true;
      });
    } else {
      // Every successful daemon set carries a non-null note (main.rs:737).
      // It is a status message, not a failure: apply the reply snapshot and
      // clear pending here, exactly like CalibrationCard._commitMarker.
      setState(() {
        _error = reply.note;
        _statusIsError = false;
        _pending.clear();
        _replySnapshot = reply.settings;
      });
    }
    if (reply.settings.reloading) {
      _startReloadPolling();
    } else {
      _stopReloadPolling();
    }
  }

  Widget _group(String title, List<Widget> children) => Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          const SizedBox(height: 12),
          Text(title, style: Theme.of(context).textTheme.titleMedium),
          const SizedBox(height: 8),
          ...children,
        ],
      );

  Widget _slider({
    required String label,
    required String key,
    required String settingKey,
    required double value,
    required double min,
    required double max,
    required int divisions,
    required String Function(double) format,
    // Wire format defaults to the display format. Keys whose readout carries
    // a unit override it so the daemon receives a bare u64, never '1500 мс'.
    String Function(double)? wireFormat,
    String? readoutKey,
    bool disabled = false,
  }) {
    final wire = wireFormat ?? format;
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(label),
        Slider(
          key: Key(key),
          min: min,
          max: max,
          divisions: divisions,
          value: value.clamp(min, max),
          label: format(value),
          // One set per release, carrying the RAW wire value. Units live only
          // in the label/readout (the daemon parses these keys as u64).
          onChanged: disabled ? null : (v) => _preview(settingKey, wire(v)),
          onChangeEnd: disabled ? null : (v) => _commit(settingKey, wire(v)),
        ),
        Text(
          format(value),
          key: Key(readoutKey ?? '${key}Readout'),
          style: AppTextStyles.mutedOf(context).copyWith(
            fontFeatures: const [FontFeature.tabularFigures()],
          ),
        ),
      ],
    );
  }

  Widget _switch({
    required String label,
    required String key,
    required String settingKey,
    required bool value,
    bool disabled = false,
  }) =>
      SwitchListTile(
        key: Key(key),
        title: Text(label),
        value: value,
        onChanged: disabled
            ? null
            : (next) => _commit(settingKey, next ? 'on' : 'off'),
      );

  Widget _segmented<T>({
    required String label,
    required String key,
    required String settingKey,
    required Set<T> values,
    required Set<T> selected,
    required String Function(T) format,
    bool disabled = false,
  }) =>
      Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(label),
          SegmentedButton<T>(
            key: Key(key),
            segments: values
                .map((v) => ButtonSegment<T>(value: v, label: Text(format(v))))
                .toList(),
            selected: selected,
            onSelectionChanged: disabled
                ? null
                : (next) {
                    if (next.isNotEmpty) _commit(settingKey, format(next.first));
                  },
          ),
        ],
      );

  Widget _field({
    required String label,
    required String key,
    required TextEditingController controller,
    String? hint,
    bool readOnly = false,
    Function(String)? onSubmitted,
    FocusNode? focusNode,
  }) =>
      Padding(
        padding: const EdgeInsets.symmetric(vertical: 4),
        child: MergeSemantics(
          child: Semantics(
            label: label,
            child: TextField(
              key: Key(key),
              controller: controller,
              focusNode: focusNode,
              readOnly: readOnly,
              decoration: InputDecoration(
                hintText: hint,
                border: const OutlineInputBorder(),
              ),
              onSubmitted: onSubmitted,
            ),
          ),
        ),
      );

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context)!;
    final colors = Theme.of(context).colorScheme;
    final providerSettings = ref.watch(settingsSnapshotProvider).value;
    final s = _replySnapshot ?? providerSettings ?? const SettingsReply();
    final reloading = s.reloading;
    if (!_pasteKeysFocus.hasFocus && _pasteKeysController.text != s.pasteKeys) {
      _pasteKeysController.text = s.pasteKeys;
    }
    if (_configPathController.text != s.configPath) {
      _configPathController.text = s.configPath;
    }
    const presets = <String>{'ctrl+v', 'ctrl+shift+v', 'shift+insert'};
    final pasteKeys = _text('paste_keys', s.pasteKeys);
    final language = _text('language', s.language);

    return SingleChildScrollView(
      padding: const EdgeInsets.all(16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          _group(l10n.confGroupRecord, [
            _slider(
              label: l10n.confMinLevelLabel,
              key: 'confMinLevelSlider',
              settingKey: 'min_level',
              value: _sliderValue('min_level', s.minLevel),
              min: 0,
              max: 1,
              divisions: 100,
              format: (v) => v.toStringAsFixed(2),
            ),
            _switch(
              label: l10n.confAutoStopLabel,
              key: 'confAutoStopSwitch',
              settingKey: 'auto_stop',
              value: _flag('auto_stop', s.autoStop),
            ),
            _slider(
              label: l10n.confAutoStopLevelLabel,
              key: 'confAutoStopLevelSlider',
              settingKey: 'auto_stop_level',
              value: _sliderValue('auto_stop_level', s.autoStopLevel),
              min: 0,
              max: 1,
              divisions: 100,
              format: (v) => v.toStringAsFixed(2),
            ),
            _slider(
              label: l10n.confAutoStopAfterLabel,
              key: 'confAutoStopAfterSlider',
              settingKey: 'auto_stop_after_ms',
              value: _sliderValue('auto_stop_after_ms', s.autoStopAfterMs),
              min: 200,
              max: 60000,
              divisions: 598,
              format: (v) => '${v.toInt()} ${l10n.confMilliseconds}',
              wireFormat: (v) => v.toInt().toString(),
            ),
            _slider(
              label: l10n.confAutoStopGraceLabel,
              key: 'confAutoStopGraceSlider',
              settingKey: 'auto_stop_grace_ms',
              value: _sliderValue('auto_stop_grace_ms', s.autoStopGraceMs),
              min: 0,
              max: 20000,
              divisions: 200,
              format: (v) => '${v.toInt()} ${l10n.confMilliseconds}',
              wireFormat: (v) => v.toInt().toString(),
            ),
            _slider(
              label: l10n.confMaxRecordLabel,
              key: 'confMaxRecordSlider',
              settingKey: 'max_record_ms',
              value: _sliderValue('max_record_ms', s.maxRecordMs),
              min: 1000,
              max: 600000,
              divisions: 599,
              format: (v) => '${(v / 1000).toStringAsFixed(0)} ${l10n.confSeconds}',
              wireFormat: (v) => v.toInt().toString(),
            ),
          ]),
          _group(l10n.confGroupInsert, [
            _segmented<String>(
              label: l10n.confInjectModeLabel,
              key: 'confInjectModeSegments',
              settingKey: 'inject_mode',
              values: const {'paste', 'clipboard', 'stdout', 'atspi'},
              selected: {_text('inject_mode', s.injectMode)},
              format: (v) => v,
            ),
            _segmented<String>(
              label: l10n.confPasteKeysLabel,
              key: 'confPasteKeysPresets',
              settingKey: 'paste_keys',
              values: presets,
              selected: presets.contains(pasteKeys) ? {pasteKeys} : const {},
              format: (v) => v,
            ),
            _field(
              label: l10n.confPasteKeysLabel,
              key: 'confPasteKeysField',
              controller: _pasteKeysController,
              hint: l10n.confPasteKeysHint,
              focusNode: _pasteKeysFocus,
              onSubmitted: (value) {
                if (value.isNotEmpty) _commit('paste_keys', value);
              },
            ),
            _slider(
              label: l10n.confClipboardRestoreLabel,
              key: 'confClipboardRestoreSlider',
              settingKey: 'clipboard_restore_ms',
              value: _sliderValue('clipboard_restore_ms', s.clipboardRestoreMs),
              min: 200,
              max: 60000,
              divisions: 598,
              format: (v) => '${v.toInt()} ${l10n.confMilliseconds}',
              wireFormat: (v) => v.toInt().toString(),
            ),
            _slider(
              label: l10n.confPasteDelayLabel,
              key: 'confPasteDelaySlider',
              readoutKey: 'confPasteDelayReadout',
              settingKey: 'paste_delay_ms',
              value: _sliderValue('paste_delay_ms', s.pasteDelayMs),
              min: 0,
              max: 3000,
              divisions: 3000,
              format: (v) => v.toInt().toString(),
            ),
            _switch(
              label: l10n.confAtspiLabel,
              key: 'confAtspiSwitch',
              settingKey: 'atspi_allow_any_role',
              value: _flag('atspi_allow_any_role', s.atspiAllowAnyRole),
            ),
          ]),
          _group(l10n.confGroupEngine, [
            Text(l10n.confEngineHint, style: AppTextStyles.mutedOf(context)),
            if (reloading) ...[
              const SizedBox(height: 8),
              Row(
                key: const Key('confReloadSpinner'),
                children: [
                  const SizedBox(
                    width: 16,
                    height: 16,
                    child: CircularProgressIndicator(strokeWidth: 2),
                  ),
                  const SizedBox(width: 8),
                  Text(l10n.diagModelLoading),
                ],
              ),
            ],
            DropdownButtonFormField<String>(
              key: const Key('confModelDropdown'),
              decoration: InputDecoration(labelText: l10n.confModelLabel),
              initialValue: s.models.contains(s.model) ? s.model : null,
              items: s.models
                  .map((path) => DropdownMenuItem<String>(
                        value: path,
                        child: Text(path.split('/').last),
                      ))
                  .toList(),
              onChanged: reloading
                  ? null
                  : (path) {
                      if (path != null) _commit('model', path);
                    },
            ),
            _switch(
              label: l10n.confUseGpuLabel,
              key: 'confUseGpuSwitch',
              settingKey: 'use_gpu',
              value: _flag('use_gpu', s.useGpu),
              disabled: reloading,
            ),
            DropdownButtonFormField<int>(
              key: const Key('confGpuDeviceDropdown'),
              decoration: InputDecoration(labelText: l10n.confGpuDeviceLabel),
              initialValue: s.gpuDevice,
              items: [
                for (var i = 0; i <= 8; i++)
                  DropdownMenuItem<int>(value: i, child: Text('$i')),
              ],
              onChanged: reloading
                  ? null
                  : (device) {
                      if (device != null) _commit('gpu_device', '$device');
                    },
            ),
            _slider(
              label: l10n.confThreadsLabel,
              key: 'confThreadsSlider',
              settingKey: 'n_threads',
              value: _sliderValue('n_threads', s.nThreads),
              min: 1,
              max: 64,
              divisions: 63,
              format: (v) => v.toInt().toString(),
              disabled: reloading,
            ),
            _slider(
              label: l10n.confBeamLabel,
              key: 'confBeamSlider',
              settingKey: 'beam_size',
              value: _sliderValue('beam_size', s.beamSize),
              min: 1,
              max: 10,
              divisions: 9,
              format: (v) => v.toInt().toString(),
              disabled: reloading,
            ),
          ]),
          _group(l10n.confGroupSystem, [
            _segmented<String>(
              label: l10n.confLanguageLabel,
              key: 'confLanguageSegments',
              settingKey: 'language',
              values: const {'ru', 'en', 'auto'},
              selected: {'ru', 'en', 'auto'}.contains(language)
                  ? {language}
                  : const <String>{},
              format: (v) => v,
              disabled: reloading,
            ),
            _field(
              label: l10n.confConfigPathLabel,
              key: 'confConfigPathField',
              controller: _configPathController,
              readOnly: true,
            ),
          ]),
          const SizedBox(height: 12),
          Semantics(
            liveRegion: true,
            child: Text(
              _error ?? '',
              key: const Key('confStatus'),
              style: _statusIsError
                  ? TextStyle(color: colors.error)
                  : AppTextStyles.mutedOf(context),
            ),
          ),
        ],
      ),
    );
  }
}

// Single fake implementation of the DictaEngine seam for widget tests.
// Declaring every scenario hook here keeps the Task 2 interface extension
// (stop -> DictaStatus, +inject) to one copy instead of two inline fakes.
// No dart:io transport is reachable through this class.
import 'dart:async';

import 'package:dicta_app/core/engine.dart';
import 'package:dicta_app/core/providers.dart';
import 'package:dicta_app/features/dashboard/dicta_control.dart';
import 'package:dicta_app/features/shell/shell.dart';
import 'package:dicta_app/l10n/generated/app_localizations.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

class FakeDictaEngine implements DictaEngine {
  FakeDictaEngine(
    this.statusValue, {
    this.stopGate,
    this.injectGate,
    this.fetchSettingsReply,
    this.stopReply,
    this.injectReply,
    this.setReply,
    this.cancelReply,
    this.statusError,
    this.onSet,
  });

  /// Mutable snapshot: tests rewrite it between real poller ticks to script
  /// state transitions (recording -> transcribing -> idle).
  DictaStatus statusValue;

  /// When set, stop() stays in flight until this completer is resolved.
  Completer<void>? stopGate;

  /// When set, inject() stays in flight until this completer is resolved.
  Completer<void>? injectGate;

  /// Reply returned by fetchSettings(); null means an empty map.
  Map<String, dynamic>? fetchSettingsReply;

  /// Scripted action replies (Task 2): default snapshots carry no text.
  DictaStatus? stopReply;
  DictaStatus? injectReply;

  /// Scripted set reply (Task 5): defaults to a silent success.
  DictaReply? setReply;

  /// Scripted cancel reply (WR-04): lets tests pin the daemon's cancellation
  /// notice exactly, including the "suppress honest noise" branch.
  DictaReply? cancelReply;

  /// When set, status() throws instead of returning a snapshot (WR-04 backoff).
  Object? statusError;

  /// Called after a set() is recorded: lets a test change fetchSettingsReply
  /// so the calibration marker can be asserted against a CHANGED snapshot
  /// (CR-01) rather than the pre-drag value.
  void Function(String key, String value)? onSet;

  /// Every set(key, value) call is recorded here in order.
  final List<({String key, String value})> setCalls = [];

  /// Ready for Task 2: inject() appends its exact text argument here.
  final List<String> injectTexts = [];

  /// Number of fetchSettings() calls (recheck asserts one refetch).
  int fetchCount = 0;

  /// Number of status() calls (recheck asserts one extra poll).
  int statusCount = 0;

  @override
  Future<DictaStatus> status() async {
    statusCount++;
    final error = statusError;
    if (error != null) throw error;
    return statusValue;
  }

  @override
  Future<DictaReply> start({String? app}) async => const DictaReply(ok: true);

  @override
  Future<DictaStatus> stop({String? app}) async {
    final gate = stopGate;
    if (gate != null) await gate.future;
    return stopReply ??
        const DictaStatus(connected: true, state: DaemonState.idle);
  }

  @override
  Future<DictaStatus> inject({required String text, String? app}) async {
    injectTexts.add(text);
    final gate = injectGate;
    if (gate != null) await gate.future;
    return injectReply ??
        DictaStatus(
          connected: true,
          state: DaemonState.idle,
          text: text,
        );
  }

  @override
  Future<DictaReply> toggle({String? app}) async => const DictaReply(ok: true);

  @override
  Future<DictaReply> cancel() async =>
      cancelReply ?? const DictaReply(ok: true);

  @override
  Future<Map<String, dynamic>> fetchSettings() async {
    fetchCount++;
    return fetchSettingsReply ?? const <String, dynamic>{};
  }

  @override
  Future<DictaReply> set(String key, String value) async {
    setCalls.add((key: key, value: value));
    onSet?.call(key, value);
    return setReply ?? const DictaReply(ok: true);
  }
}

/// Single parameterized app scaffold for every widget suite. Scenario suites
/// override only what they need (container capture, clipboard seam, ...);
/// wiring the ProviderScope/MaterialApp/ru-locale once keeps interface changes
/// from drifting across six copies (phase 2 IN-06).
Widget buildFakeApp(
  FakeDictaEngine fake, {
  void Function(String text)? onCopy,
  void Function(ProviderContainer container)? captureContainer,
}) {
  return ProviderScope(
    // The list type is inferred from ProviderScope.overrides; Riverpod 3.5
    // does not export the `Override` type name for explicit annotation.
    overrides: [
      engineProvider.overrideWithValue(fake),
      if (onCopy != null) clipboardWriterProvider.overrideWithValue(onCopy),
    ],
    child: Builder(
      builder: (context) {
        captureContainer?.call(ProviderScope.containerOf(context));
        return MaterialApp(
          localizationsDelegates: AppLocalizations.localizationsDelegates,
          supportedLocales: AppLocalizations.supportedLocales,
          locale: const Locale('ru'),
          home: const ShellWidget(),
        );
      },
    ),
  );
}

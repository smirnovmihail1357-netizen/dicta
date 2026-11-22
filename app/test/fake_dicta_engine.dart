// Single fake implementation of the DictaEngine seam for widget tests.
// Declaring every scenario hook here keeps the Task 2 interface extension
// (stop -> DictaStatus, +inject) to one copy instead of two inline fakes.
// No dart:io transport is reachable through this class.
import 'dart:async';

import 'package:dicta_app/core/engine.dart';

class FakeDictaEngine implements DictaEngine {
  FakeDictaEngine(
    this.statusValue, {
    this.stopGate,
    this.injectGate,
    this.fetchSettingsReply,
    this.stopReply,
    this.injectReply,
    this.setReply,
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
  Future<DictaReply> cancel() async => const DictaReply(ok: true);

  @override
  Future<Map<String, dynamic>> fetchSettings() async {
    fetchCount++;
    return fetchSettingsReply ?? const <String, dynamic>{};
  }

  @override
  Future<DictaReply> set(String key, String value) async {
    setCalls.add((key: key, value: value));
    return setReply ?? const DictaReply(ok: true);
  }

  @override
  Stream<DictaStatus> get statusStream => Stream<DictaStatus>.empty();
}

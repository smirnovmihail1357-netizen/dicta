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
    this.fetchSettingsReply,
  });

  /// Mutable snapshot: tests rewrite it between real poller ticks to script
  /// state transitions (recording -> transcribing -> idle).
  DictaStatus statusValue;

  /// When set, stop() stays in flight until this completer is resolved.
  Completer<void>? stopGate;

  /// Reply returned by fetchSettings(); null means an empty map.
  Map<String, dynamic>? fetchSettingsReply;

  /// Every set(key, value) call is recorded here in order.
  final List<({String key, String value})> setCalls = [];

  /// Ready for Task 2: inject() appends its exact text argument here.
  final List<String> injectTexts = [];

  /// Number of fetchSettings() calls (recheck asserts one refetch).
  int fetchCount = 0;

  @override
  Future<DictaStatus> status() async => statusValue;

  @override
  Future<DictaReply> start({String? app}) async => const DictaReply(ok: true);

  @override
  Future<DictaReply> stop({String? app}) async {
    final gate = stopGate;
    if (gate != null) await gate.future;
    return const DictaReply(ok: true);
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
    return const DictaReply(ok: true);
  }

  @override
  Stream<DictaStatus> get statusStream => Stream<DictaStatus>.empty();
}

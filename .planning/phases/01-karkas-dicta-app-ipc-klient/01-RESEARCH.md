# Phase 1: Каркас dicta-app + IPC-клиент - Research

**Researched:** 2026-11-22
**Domain:** Flutter Linux desktop app skeleton + Dart UDS IPC client over the frozen dictad line-JSON protocol
**Confidence:** HIGH (protocol facts read from source this session; stack versions verified on pub.dev; live daemon fixture captured)

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
1. **Stack:** Flutter (Material You, like FlClash) — NOT an egui redesign. Flutter 3.50 / Dart 3.14.
2. **Everything in one window eventually** (dashboard, history, settings+models, tray+hotkeys) — Phase 1 builds only the shell + IPC foundation.
3. **IPC, not FFI:** the app talks to the existing dictad daemon over its frozen line-delimited JSON protocol (unix socket `/run/user/1000/dicta.sock`). No protocol changes, no FRB, no embedding whisper into the app.
4. **FlClash is pattern-inspiration only** (GPL-3.0): copy architectural ideas, never code.

### Phase Scope (APP-01..APP-06)
- **APP-01 Shell:** `app/` Flutter project (repo root, NOT a cargo workspace member). Sidebar navigation: Дашборд / История / Модели / Настройки — placeholder screens for phases 2–5. Dark + light theme, Material You via `dynamic_color`. Window chrome via `window_manager`. l10n ru-first (`gen-l10n`), Russian UI strings.
- **APP-02 IPC client:** poll `status` every ~100 ms; on socket failure show «демон недоступен» and auto-reconnect when the daemon returns (no app restart). One command — one line JSON, same framing as dictactl/dicta-settings.
- **APP-03 DictaEngine seam:** all UI talks to an abstract `DictaEngine` interface (Dart), never to sockets directly. The Windows bridge (Phase 6) plugs in behind the same seam. Linux implementation = UDS client. Default `dart:io` AF_UNIX; the dart_ipc-vs-dart:io mini-spike only runs if plain dart:io proves insufficient.
- **APP-04 Settings golden test:** Dart model of the daemon `settings` reply, deserializing a captured real fixture one-to-one (17 keys). Golden test green.
- **APP-05 l10n:** ru default, all UI strings through gen-l10n.
- **APP-06 set discipline:** `set` payloads built with `jsonEncode`, no manual escaping.

### Constraints & Non-Goals
- Daemon `dictad`, protocol, overlay `dicta-ui`, `dicta-settings`, Rust workspace: untouched. No Makefile changes to existing targets (adding new install targets for the app is allowed if needed for later phases, keep minimal in Phase 1).
- No tray, no global hotkeys, no Windows code (phases 5–7).
- No history capture yet (phase 3) — but the engine seam should expose state transitions so passive history can hook in later.
- Flutter app must run from source on cachyos (Linux desktop) — `flutter run -d linux`.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| APP-01 | Flutter 3.50 app builds/runs on Linux, sidebar shell (Дашборд/История/Модели/Настройки), dark/light Material You | Standard Stack (verified pub.dev versions), Recommended Project Structure, dynamic_color Linux findings, Environment Availability (all Linux deps present except Flutter SDK — install step required) |
| APP-02 | UDS client polls `status` at 100 ms, shows «демон недоступен», auto-reconnects without restart | Protocol facts (status closes connection after reply — verified dictad:1007-1009), Riverpod 3 Timer.periodic Notifier pattern (riverpod.dev), reconnect strategy section |
| APP-03 | Platform fork hidden behind `DictaEngine`; UI never touches sockets | dart:io AF_UNIX verification (api.dart.dev InternetAddress/InternetAddressType.unix), seam code example, Riverpod provider-overrides for widget tests |
| APP-04 | Dart settings model deserializes daemon reply one-to-one (17 keys), golden test on captured fixture | 17 EDITABLE_KEYS quoted verbatim (config.rs:142-160), live fixture captured this session (see Code Examples), golden-test-as-plain-unit-test finding |
| APP-05 | ru-first gen-l10n, no hardcoded strings | gen-l10n config table verified (docs.flutter.dev/ui/internationalization), minimal l10n.yaml example |
| APP-06 | All writes via `set` with `jsonEncode`, no manual escaping | Manual-escaping bug class verified in dictactl:138-144,154-156 (strips `"` and `\`), set-value-is-always-string verified (dictad:40-42, dicta-settings/net.rs:48-53), jsonEncode test case |
</phase_requirements>

## Summary

Phase 1 builds a Flutter 3.50 desktop shell over the existing dictad daemon, connected by a pure-Dart UDS client speaking the daemon's frozen line-delimited JSON protocol. The protocol is fully specified by the existing Rust sources read this session: request is one JSON line (`{"cmd":"status"}\n`), reply is one JSON line (`{"state","level","seconds","text","error","panel","settings"?}`), and — critically — **the daemon closes the connection after every `status` reply** (`if req.cmd == "status" { break; }` [VERIFIED: crates/dictad/src/main.rs:1007-1009]), so the poll loop is connect→ask→read→EOF each tick, exactly as dicta-ui already does. `dart:io` natively supports AF_UNIX via `InternetAddress(path, type: InternetAddressType.unix)` + `Socket.connect(..., 0)` [VERIFIED: api.dart.dev InternetAddress constructor docs], which resolves the STACK-vs-PITFALLS contradiction in favor of STACK: **dart:io is sufficient on Linux; dart_ipc 2.0.2 is Windows-named-pipe territory (Phase 6)**. The mini-spike's success criterion is one line of Dart connecting to the live socket — which this session's environment audit makes easy, since the daemon is running right now at `/run/user/1000/dicta.sock`.

The state layer is Riverpod 3.5.0 with a `Notifier` whose `build()` starts a `Timer.periodic(100 ms)` and cancels it in `ref.onDispose` [VERIFIED: riverpod.dev docs concepts2/refs + migration/from_state_notifier] — the timer lives in the provider, never in widgets. Auto-reconnect is nearly free under polling: a failed connect flips the UI to «демон недоступен» and the next tick retries; add a capped exponential backoff only for the dead-daemon case so the client doesn't hammer a nonexistent socket path 10×/s. The one protocol behavior that will bite if missed: **`stop` blocks until text injection completes** (serve_conn holds the connection until `done.recv()` [VERIFIED: crates/dictad/src/main.rs:965-969]), so the client needs a generous timeout on stop (dictactl uses 120 s [VERIFIED: crates/dictactl/src/main.rs:119]).

The settings golden test (APP-04) needs no Flutter goldens infrastructure: it is a plain `flutter test` unit test that decodes a captured fixture JSON into the Dart model and asserts field-by-field. A **live fixture was captured this session** via `dictactl settings --json` (see Code Examples) — commit it as `test/fixtures/settings_reply.json`. The 17 editable keys are quoted verbatim below from `EDITABLE_KEYS` [VERIFIED: crates/dicta-core/src/config.rs:142-160]; the Dart model must additionally carry `models`, `reloading`, `config_path` from the same reply object. One environment blocker: **Flutter/Dart SDK is not installed on this machine** — ninja, clang, cmake, pkg-config, gtk3-dev, libx11-dev are all present, so a single SDK install (fvm 4.3.2 in cachyos repos, or AUR `flutter-bin` via paru) unblocks everything.

**Primary recommendation:** Standalone `app/` Flutter project at repo root; `lib/core/engine.dart` defines the `DictaEngine` interface, `lib/core/daemon_engine.dart` implements it with `dart:io` AF_UNIX + line-split JSON; Riverpod 3 `Notifier` owns a 100 ms poll timer; fixture-based golden unit test for settings; `gen-l10n` with `app_ru.arb` template; no dart_ipc, no FRB in Phase 1.

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Status polling (100 ms) | Dart service layer (Riverpod Notifier + DictaEngine impl) | — | Sockets must never be called from widgets [PITFALLS P6]; the provider owns the Timer and exposes immutable snapshots |
| Daemon connection / UDS transport | `daemon_engine.dart` behind `DictaEngine` seam | — | UI must not know the transport (APP-03); Phase 6 Windows impl replaces only this file |
| Settings schema (17 keys) | Dart model in `lib/core/models.dart` + golden fixture test | daemon (source of truth, frozen) | 4th copy of Settings must be pinned by a test against the daemon's real reply [PITFALLS P7] |
| Validation of setting values | Daemon (unchanged) | — | Locked: `set` validates server-side; app surfaces the reply's error, never pre-validates [CONTEXT.md Known Facts] |
| Theme (Material You dark/light) | Flutter app (dynamic_color + ColorScheme.fromSeed fallback) | OS (XDG portal / GTK) | dynamic_color 2.1.0 reads the OS accent; app owns the fallback when null |
| l10n (ru-first) | Flutter app (gen-l10n ARB) | — | All UI strings through AppLocalizations; no hardcoded strings in widgets |
| Window chrome | window_manager 0.7.0 plugin | — | Basic show/hide/size; Wayland caveats acceptable for Phase 1 scope |
| History state transitions (future hook) | DictaEngine stream surface | Phase 3 store | Engine exposes status snapshots as a Stream so Transcribing→Idle transitions can be observed later without protocol changes |

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| Flutter SDK (stable) | 3.50.0 / Dart 3.14.0 | App framework, Linux desktop target | Locked decision (CONTEXT.md); leanflutter plugin generation aligned to 3.50 [STACK.md, versions re-verified on pub.dev this session] |
| `flutter_riverpod` | 3.5.0 | State management | Latest is exactly 3.5.0, published 2026-11-06 [VERIFIED: pub.dev API]. Riverpod 3 `Notifier` + `Timer.periodic` + `ref.onDispose` is the documented polling pattern [CITED: riverpod.dev/docs/concepts2/refs, /docs/migration/from_state_notifier] |
| `dynamic_color` | 2.1.0 | Material You dynamic color | Latest 2.1.0, published 2026-08-20, verified publisher material.io [VERIFIED: pub.dev API + pub.dev changelog]. Changelog 2.1.0: "Add support for the XDG Desktop Portal Settings for accent-color on Linux" — exactly the KDE Plasma 6 path [CITED: pub.dev/packages/dynamic_color/changelog] |
| `window_manager` | 0.7.0 | Window chrome | Latest 0.7.0, published 2026-11-12 [VERIFIED: pub.dev API]; leanflutter generation with nativeapi 0.6.0 [STACK.md] |
| `flutter_localizations` + `intl` | SDK-bundled | gen-l10n ru-first | Official tooling, no third-party l10n packages [STACK.md; CITED: docs.flutter.dev/ui/internationalization] |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `dart_ipc` | 2.0.2 | Named-pipe IPC transport | **NOT in Phase 1.** Windows named pipe territory (Phase 6). Only pull in if the Phase 6 bridge decision picks it [STACK.md] |
| `dart:io` (SDK) | — | AF_UNIX socket client | The Linux transport. Zero new dependencies [VERIFIED: api.dart.dev InternetAddress supports `type: InternetAddressType.unix` with a file path] |
| `dio` | 5.11.1 | HF model browser HTTP | **NOT in Phase 1** — Phase 4 territory; listed so pubspec stays minimal now [STACK.md] |
| `flutter_test` | SDK-bundled | Unit + widget tests | Golden fixture test, seam fake-engine widget test |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| `dart:io` AF_UNIX | `dart_ipc` on Linux | Unneeded duplication: dart:io covers UDS natively [VERIFIED: api.dart.dev]; dart_ipc adds value only where dart:io can't go (Windows named pipes) |
| Riverpod 3 manual `Notifier` | `riverpod_generator` 4.0.9 codegen | STACK.md: official recommendation is codegen only for projects already codegening; dicta-app isn't (gen-l10n is a separate tool, no build_runner) |
| Polling `status` 100 ms | subscribe/event push | **Forbidden** — protocol frozen for the whole milestone [REQUIREMENTS.md Out of Scope] |
| Timer in Riverpod Notifier | Timer in a StatefulWidget | Widget-scoped timers rebuild with navigation and die with the shell route; the provider survives and stays testable via ProviderScope overrides |

**Installation (in `app/`, after SDK install):**
```bash
flutter create --platforms=linux,windows dicta-app   # run from repo root → app/ dir; windows platform files harmless, Phase 6 reuses them
cd app
flutter pub add flutter_riverpod dynamic_color window_manager
flutter pub add flutter_localizations --sdk=flutter
flutter pub add intl
```

**Version verification (done this session, pub.dev API):** flutter_riverpod 3.5.0 (latest, 2026-11-06) ✓; dynamic_color 2.1.0 (latest, 2026-08-20) ✓; window_manager 0.7.0 (latest, 2026-11-12) ✓.

## Package Legitimacy Audit

> The seam's `package-legitimacy check` only covers npm/pypi/crates — pub.dev packages return a false `SLOP/does-not-exist` there (cross-ecosystem confusion, the exact vector the gate warns about). Legitimacy was therefore established on the correct registry (pub.dev API + publisher verification), per the protocol's Step 2.

| Package | Registry | Age | Downloads | Source Repo | Verdict | Disposition |
|---------|----------|-----|-----------|-------------|---------|-------------|
| flutter_riverpod | pub.dev | ~5 yrs (3.x line active; 3.5.0 on 2026-11-06) | 263k/wk-class (riverpod ecosystem) | github.com/rrousselgit/riverpod | OK (verified on pub.dev) | Approved |
| dynamic_color | pub.dev | ~5 yrs (2.1.0 on 2026-08-20) | high | github.com/material-foundation/flutter-packages | OK — verified publisher `material.io` | Approved |
| window_manager | pub.dev | ~4 yrs (0.7.0 on 2026-11-12) | high | github.com/leanflutter/window_manager (leanflutter org) | OK | Approved |
| dart_ipc | pub.dev | 2.0.2 (2026-09) | moderate | github.com/monkeyWie/dart_ipc | OK | NOT installed in Phase 1 (Windows fallback only) |

**Packages removed due to [SLOP] verdict:** none (the npm-gate SLOP verdicts above are registry-mismatch artifacts, not real findings).
**Packages flagged as suspicious [SUS]:** none.
No packages discovered via websearch alone; all four were already version-verified in STACK.md and re-confirmed against pub.dev this session.

## Architecture Patterns

### System Architecture Diagram

```
 [dictad daemon (FROZEN — untouched)]
   UnixListener /run/user/1000/dicta.sock  (0o600, same-user only)
        ▲  one JSON request line in, one JSON reply line out
        │  * status  → reply, then daemon CLOSES the socket
        │  * set/settings/start/stop/toggle/cancel/inject → reply (stop blocks until inject done)
        │
        │  AF_UNIX  (dart:io Socket.connect(InternetAddress(path, type: .unix), 0))
 ┌──────┴───────────────────────────────────────────┐
 │  app/  (standalone Flutter project, no cargo)    │
 │                                                  │
 │  lib/core/daemon_engine.dart  ── implements ──  │
 │  lib/core/engine.dart         ◄── DictaEngine seam (abstract)
 │        │              ▲                          │
 │        │ status()     │ future: windows_engine.dart (Phase 6,
 │        ▼              │   named pipe, plugs in here — UI unchanged)
 │  Riverpod Notifier (owns Timer.periodic 100ms))  │
 │   state: DictaStatus{connected, state, level,   │
 │           seconds, text, error}                  │
 │        │ ProviderScope                          │
 │        ▼                                         │
 │  Shell UI: sidebar (Дашборд/История/Модели/      │
 │  Настройки placeholders)  ── reads only providers │
 │  «демон недоступен» banner when !connected       │
 └──────────────────────────────────────────────────┘
   theming: DynamicColorBuilder → light/dark ColorScheme (portal accent or fromSeed fallback)
   l10n: gen-l10n (app_ru.arb template) → AppLocalizations
```

Trace of the primary use case: app start → Notifier's timer fires → engine.status() → socket connect + `{"cmd":"status"}\n` → one reply line → snapshot into provider state → sidebar/dashboard placeholders render; connect failure → `connected=false` → banner; next tick retries (backoff when daemon dead).

### Recommended Project Structure
```
app/                          # repo root sibling of crates/, NOT a cargo member
├── lib/
│   ├── core/
│   │   ├── engine.dart       # abstract DictaEngine interface (APP-03 seam)
│   │   ├── daemon_engine.dart# dart:io AF_UNIX implementation
│   │   ├── models.dart       # DictaStatus, DictaReply, SettingsReply (17 keys + models/reloading/config_path)
│   │   └── socket_path.dart  # XDG_RUNTIME_DIR/dicta.sock resolution (mirror of dicta-core default_socket_path)
│   ├── features/
│   │   ├── shell/            # sidebar navigation shell, daemon-unavailable banner
│   │   ├── dashboard/        # placeholder screen (Phase 2 fills)
│   │   ├── history/          # placeholder (Phase 3)
│   │   ├── models/           # placeholder (Phase 4)
│   │   └── settings/         # placeholder (Phase 4)
│   ├── l10n/
│   │   └── app_ru.arb        # TEMPLATE arb — Russian is the source language
│   └── main.dart             # ProviderScope, DynamicColorBuilder, MaterialApp with l10n delegates
└── test/
    ├── fixtures/
    │   └── settings_reply.json   # captured from live daemon 2026-11-22 (see Code Examples)
    ├── settings_model_test.dart # APP-04 golden test
    ├── daemon_engine_framing_test.dart # APP-02/APP-06 framing + jsonEncode round-trip
    └── shell_test.dart           # APP-03 widget test with a FakeDictaEngine via provider override
```

### Pattern 1: DictaEngine seam (APP-03)
**What:** One abstract interface owns every daemon interaction; UI and providers depend on it, never on `Socket`.
**When to use:** Always in this codebase — Phase 6's Windows bridge must plug in without touching UI code.
**Example:**
```dart
// lib/core/engine.dart — the seam. Status snapshots are also exposed as a
// Stream so Phase 3 history can observe Transcribing→Idle transitions.
abstract interface class DictaEngine {
  Future<DictaStatus> status();
  Future<DictaReply> start({String? app});
  Future<DictaReply> stop({String? app});     // blocks until inject done — generous timeout
  Future<DictaReply> toggle({String? app});
  Future<DictaReply> cancel();
  Future<SettingsReply> fetchSettings();
  Future<DictaReply> set(String key, String value); // validation stays in the daemon (APP-06)
  Stream<DictaStatus> get statusStream;       // fed by the 100 ms poll loop, for future history
}
```

### Pattern 2: Polled provider — Riverpod 3 Notifier with a timer
**What:** The poll loop lives in a `Notifier.build()`; disposal cancels the timer. Adapted from the official Riverpod docs tick-provider pattern [CITED: riverpod.dev/docs/concepts2/refs].
**Example:**
```dart
// Source pattern: riverpod.dev/docs/concepts2/refs (Timer.periodic + ref.onDispose)
final dictaStatusProvider =
    NotifierProvider<DictaStatusNotifier, DictaStatus>(DictaStatusNotifier.new);

class DictaStatusNotifier extends Notifier<DictaStatus> {
  @override
  DictaStatus build() {
    final timer = Timer.periodic(const Duration(milliseconds: 100), (_) => poll());
    ref.onDispose(timer.cancel);
    return const DictaStatus.disconnected();
  }

  Future<void> poll() async {
    if (_inFlight) return;                 // never overlap ticks
    _inFlight = true;
    try {
      final s = await ref.read(engineProvider).status();
      state = s;                           // connected snapshot
      _backoff = const Duration(milliseconds: 200);
    } catch (_) {
      state = state.asDisconnected();      // UI shows «демон недоступен»
      await Future.delayed(_backoff);      // backoff while daemon dead: 200ms→…→2s cap
      _backoff = (_backoff * 2) < const Duration(seconds: 2)
          ? _backoff * 2 : const Duration(seconds: 2);
    } finally { _inFlight = false; }
  }
}
```
Note: Riverpod 3 has no `StateNotifier`/`mounted`; guard reentrancy with a plain bool [CITED: riverpod.dev/docs/migration/from_state_notifier]. The `_backoff`-when-disconnected behavior is a design recommendation of this research, not a doc citation — simple capped doubling, testable as a pure function.

### Pattern 3: UDS request/reply over dart:io (APP-02)
**What:** One command = one connection = one line out + one line in. The daemon closes status connections itself, so per-tick connect is the protocol-faithful shape.
**Example:**
```dart
// Source: api.dart.dev — InternetAddress('/path', type: InternetAddressType.unix)
//         + HttpClient.connectionFactory example (uses /var/run/docker.sock this way)
Future<String> _call(Map<String, Object?> cmd, {Duration timeout = const Duration(seconds: 30)}) async {
  final socket = await Socket.connect(
    InternetAddress(socketPath, type: InternetAddressType.unix),
    0,
    timeout: const Duration(seconds: 2),
  );
  try {
    socket.write(jsonEncode(cmd));        // APP-06: jsonEncode, never manual escaping
    socket.write('\n');
    await socket.flush();
    final line = await socket
        .transform(utf8.decoder)
        .transform(const LineSplitter())
        .first
        .timeout(timeout);                // stop needs a long timeout — daemon holds the conn
    return line;
  } finally {
    await socket.destroy();
  }
}
```

### Anti-Patterns to Avoid
- **Reading FlClash source code** — GPL-3.0 contamination [PITFALLS P1]. Screenshots/README/architecture descriptions only; do not mirror its file names or structure. Guardrail line belongs in the PLAN.md.
- **Manual JSON string building** — dictactl's `app.replace(['"', '\\'], "")` silently *deletes* quotes and backslashes from user text [VERIFIED: crates/dictactl/src/main.rs:138-144] and `AtspiInsert`/`Inject` strip characters likewise [VERIFIED: crates/dictactl/src/main.rs:265-266,275]. `jsonEncode` makes the class of bug impossible.
- **Holding one connection open for the poll loop** — the daemon breaks the loop after a status reply [VERIFIED: crates/dictad/src/main.rs:1007-1009]. A "persistent" status connection silently becomes EOF-on-every-second-tick.
- **Pre-validating setting values in the app** — validation belongs to the daemon; the app surfaces the reply's `error` field [CONTEXT.md locked: "surface, don't pre-validate"].
- **Calling sockets from widgets / setState on the whole shell per tick** — PITFALLS Performance Traps: the service/provider owns the timer; leaf widgets select narrow provider slices.
- **Riverpod 2.x tutorial API** — `StateNotifier`, `mounted`, `ConsumerWidget`-era snippets won't compile on 3.5.0 [CITED: riverpod.dev/docs/migration/from_state_notifier].

## Protocol Reference (frozen — do not change, cite when implementing)

All facts below were read from source this session:

- **Reply envelope:** `{"state","level","seconds","text","error","panel","settings"?}` — `Res` is `{"state","level","seconds","text","error"}` [VERIFIED: crates/dictad/src/main.rs:48-55] flattened into `Reply` with `panel` and optional `settings` [VERIFIED: crates/dictad/src/main.rs:352-359 — `#[serde(flatten)] base: Res`, `panel: bool`, `settings: Option<Settings>` with `skip_serializing_if = "Option::is_none"`].
- **States (verbatim strings):** `"idle" | "recording" | "transcribing"` [VERIFIED: crates/dictad/src/main.rs:91-98 — `State::Idle => "idle", State::Recording => "recording", State::Transcribing => "transcribing"`].
- **The 17 editable keys (verbatim, in EDITABLE_KEYS order):** `"min_level", "auto_stop", "auto_stop_level", "auto_stop_after_ms", "auto_stop_grace_ms", "max_record_ms", "inject_mode", "paste_keys", "clipboard_restore_ms", "paste_delay_ms", "atspi_allow_any_role", "language", "model", "use_gpu", "gpu_device", "n_threads", "beam_size"` [VERIFIED: crates/dicta-core/src/config.rs:142-160].
- **ENGINE_KEYS (reload-triggering, for later warning UI):** `"model", "language", "use_gpu", "gpu_device", "n_threads", "beam_size"` [VERIFIED: crates/dicta-core/src/config.rs:163-170].
- **`settings` object carries 20 fields:** the 17 above plus `models: Vec<String>` (up to 12 `.bin` files beside the current model), `reloading: bool`, `config_path: String` [VERIFIED: crates/dictad/src/main.rs:57-82].
- **`set` semantics:** `{"cmd":"set","key":"...","value":"..."}` — value is **always a JSON string** (daemon parses then validates; rejection returns the same reply shape with `error` set and, on success, the full `settings` object attached) [VERIFIED: crates/dictad/src/main.rs:40-42 — `Req.value: Option<String>` "новое значение (текстом; демон проверяет и котирует сам)"; dicta-settings/net.rs:48-53 — "значение всегда строкой (так ждёт демон)"].
- **Bool tolerance on the daemon side:** `set` accepts "1|true|yes|y|on|вкл|включено|да" / "0|false|no|n|off|выкл|выключено|нет" for flags [VERIFIED: crates/dictad/src/main.rs:505-513].
- **Connection lifecycle:** status → reply → daemon closes [VERIFIED: crates/dictad/src/main.rs:1005-1009]; stop → reply only after injection finished [VERIFIED: crates/dictad/src/main.rs:965-969 — "Соединение со \"stop\" держим до конца вставки"]; other commands leave the loop open, but per-command connections (dictactl/dicta-settings style) are the simplest correct client shape.
- **`reloading` starts `true`** at daemon boot (model warms up in a thread) [VERIFIED: crates/dictad/src/main.rs:179] — the app must not treat `reloading:true` on the first settings fetch as an error.
- **Socket path resolution (mirror this in Dart):** `XDG_RUNTIME_DIR/dicta.sock`, else `dirs::runtime_dir()` (i.e. `/run/user/<uid>`), else temp dir [VERIFIED: crates/dicta-core/src/config.rs:253-261 — `default_socket_path()`]. Config override `socket = "..."` in config.toml also possible — Phase 1 may honor `DICTA_CONFIG`/config.toml or hardcode the default resolution; recommend mirroring `default_socket_path` only, with the full config.toml read deferred (TOML parsing in Dart is out of Phase 1 scope — see Open Questions).
- **Client-side precedent:** dicta-settings keeps one connection for many commands with a 30 s read timeout [VERIFIED: crates/dicta-settings/src/net.rs:10-40]; dictactl opens a fresh connection per command with 120 s read timeout [VERIFIED: crates/dictactl/src/main.rs:111-127].
- **Live fixture (captured this session, 2026-11-22, daemon running):** see Code Examples — all 20 settings fields present, real shapes confirmed (integers unquoted, floats unquoted, booleans, string arrays).

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| JSON encoding of commands | string `.replace` escaping | `dart:convert` `jsonEncode` | The replace-based approach is a *documented live bug* in dictactl (drops `"` and `\` from user text) [VERIFIED: crates/dictactl/src/main.rs:138-144]. stdlib, zero deps |
| AF_UNIX socket client | FFI wrapper / dart_ipc on Linux | `dart:io` `Socket.connect` | Native support confirmed in API docs [VERIFIED: api.dart.dev InternetAddress with InternetAddressType.unix] |
| Line framing | custom buffer parsing | `utf8.decoder` + `LineSplitter` transforms | stdlib; handles partial chunks |
| State management / polling loop | setState + Timer in widgets | Riverpod 3 `Notifier` + provider | Testable via ProviderScope overrides; timer disposal wired by `ref.onDispose` [CITED: riverpod.dev] |
| Material You palette | hand-mixed ColorScheme | `dynamic_color` 2.1.0 + `ColorScheme.fromSeed` fallback | Only official dynamic-color package; Linux portal support landed exactly in 2.1.0 [CITED: pub.dev changelog] |
| Localization | string maps / easy_localization | `gen-l10n` + ARB | Official, ru-template supported via `template-arb-file: app_ru.arb` [CITED: docs.flutter.dev/ui/internationalization] |

**Key insight:** every hand-roll candidate in this phase already has a stdlib or verified-pub.dev answer; the only genuinely new code is the `DictaEngine` interface + its UDS implementation and the Dart settings model — and the settings model is *pinned by a fixture test*, not trusted.

## Common Pitfalls

### Pitfall 1: Assuming a persistent status connection
**What goes wrong:** Client connects once and loops `status` on one socket; the daemon closed it after the first reply [VERIFIED: crates/dictad/src/main.rs:1007-1009] — every second read is EOF.
**Why it happens:** Reading dicta-settings' persistent `Client` (net.rs) and assuming all commands behave alike; only status breaks the loop.
**How to avoid:** Per-tick (or per-command) connections, dictactl-style. UDS connect cost is microseconds; dicta-ui already does ~10/s.
**Warning signs:** UI updates once then freezes; client log shows repeated EOF on a "live" connection.

### Pitfall 2: Short timeout on `stop`
**What goes wrong:** `stop` doesn't reply until injection completes (daemon holds the connection until `done.recv()`) [VERIFIED: crates/dictad/src/main.rs:965-969]; a 2 s client timeout reports failure while the daemon is mid-inject.
**Why it happens:** Treating all commands as uniformly fast; inference of a large model takes many seconds.
**How to avoid:** Per-command timeouts: status 1–2 s; stop/inject 120 s (dictactl's value [VERIFIED: crates/dictactl/src/main.rs:119]); set/settings 30 s (dicta-settings' value [VERIFIED: crates/dicta-settings/src/net.rs:25]).
**Warning signs:** «демон недоступен» flashes right after pressing stop on a long dictation.

### Pitfall 3: Missing reconnect path (the "Looks Done But Isn't" item)
**What goes wrong:** App works, daemon restarts (systemd update, crash) → app silently dead forever.
**Why it happens:** Happy-path development against an always-running daemon.
**How to avoid:** The poll loop *is* the reconnect probe: catch connect errors → disconnected state → next tick retries. Manual verification step in the plan: with the app running, `systemctl --user restart dictad` (or kill dictad), confirm banner appears and live state returns without app restart.
**Warning signs:** No `catch` around `Socket.connect`; no disconnected state class in the model.

### Pitfall 4: The 4th copy of Settings drifts
**What goes wrong:** Dart model silently diverges from the daemon's 20-field settings object; new daemon fields vanish, app writes keys the daemon ignores.
**Why it happens:** Hand-transcribed without a pin [PITFALLS P7].
**How to avoid:** Golden test on the captured fixture (APP-04) + unknown-key tolerance (extra JSON keys ignored) + default-on-missing (mirror of panel.rs `#[serde(default)]` [VERIFIED: crates/dicta-ui/src/panel.rs:15-38]). The fixture committed this phase is the regression tripwire for the whole milestone.
**Warning signs:** Settings model edited without the fixture/test changing in the same commit.

### Pitfall 5: GPL contamination from FlClash
**What goes wrong:** "Inspiration" becomes copied Dart code → dicta must go GPL-3.0.
**Why it happens:** FlClash is the declared design reference; its repo is one click away [PITFALLS P1].
**How to avoid:** Screenshots, README, architecture descriptions only. Never open FlClash source in any UI phase session. Do not mirror its directory/file names.
**Warning signs:** Commits whose Dart style suddenly diverges; identifiers matching FlClash's naming.

### Pitfall 6: Riverpod 2-era snippets won't compile
**What goes wrong:** Executor pastes a StateNotifier/`mounted` example → build errors on riverpod 3.5.0.
**Why it happens:** Most tutorials predate Riverpod 3.
**How to avoid:** Use `Notifier`/`NotifierProvider` + `ref.onDispose` only [CITED: riverpod.dev/docs/migration/from_state_notifier].
**Warning signs:** `StateNotifier` or `state_notifier` imports in diffs.

## Code Examples

### Captured golden fixture (live daemon, this session — 2026-11-22)
Real `{"cmd":"settings"}` reply, byte-accurate as printed by `dictactl settings --json`. Commit as `test/fixtures/settings_reply.json` (single line):
```json
{"state":"idle","level":0.0,"seconds":0.0,"text":null,"error":null,"panel":false,"settings":{"language":"ru","inject_mode":"paste","min_level":0.35,"auto_stop":false,"auto_stop_level":0.2,"auto_stop_after_ms":1500,"auto_stop_grace_ms":1200,"max_record_ms":60000,"use_gpu":true,"gpu_device":0,"n_threads":8,"beam_size":1,"paste_keys":"ctrl+v","clipboard_restore_ms":2500,"paste_delay_ms":150,"atspi_allow_any_role":false,"model":"/home/wardreg/.local/share/whisper-cpp/ggml-medium-q5_0.bin","models":["/home/wardreg/.local/share/whisper-cpp/ggml-large-v3-turbo-q5_0.bin","/home/wardreg/.local/share/whisper-cpp/ggml-medium-q5_0.bin","/home/wardreg/.local/share/whisper-cpp/ggml-small-q5_1.bin"],"reloading":false,"config_path":"/home/wardreg/.config/dicta/config.toml"}}
```
Types confirmed: floats/ints unquoted, bools bare, `text`/`error` nullable at top level, `models` a string array. Anonymize the home path if desired — the *shape*, not the values, is what the golden test pins (assert field-by-field against expected parsed values, plus unknown-key tolerance).

### Golden test shape (APP-04) — plain unit test, no Flutter goldens
```dart
// test/settings_model_test.dart
// Source mechanism: docs.flutter.dev/cookbook/testing/unit/introduction (test/ dir, _test.dart)
import 'dart:convert';
import 'dart:io';
import 'package:flutter_test/flutter_test.dart';
import 'package:dicta_app/core/models.dart';

void main() {
  test('settings fixture deserializes one-to-one (17 keys + extras)', () {
    final raw = File('test/fixtures/settings_reply.json').readAsStringSync();
    final reply = SettingsReply.fromDaemonJson(jsonDecode(raw) as Map<String, dynamic>);
    expect(reply.language, 'ru');
    expect(reply.minLevel, closeTo(0.35, 1e-6));
    expect(reply.autoStop, isFalse);
    expect(reply.maxRecordMs, 60000);
    expect(reply.useGpu, isTrue);
    expect(reply.beamSize, 1);
    expect(reply.models.length, 3);
    // ... all 17 editable keys + models/reloading/config_path
  });

  test('unknown keys are tolerated, missing keys default', () {
    final s = SettingsReply.fromDaemonJson({'min_level': 0.5, 'future_key': 1});
    expect(s.minLevel, 0.5);
    expect(s.language, 'ru'); // default, mirrors #[serde(default)] in panel.rs
  });
}
```

### jsonEncode discipline (APP-06) — the regression test for the known bug class
```dart
// test/daemon_engine_framing_test.dart
test('set payload survives quotes, backslashes and newlines', () {
  final line = buildSetLine('paste_keys', 'он сказал "да\\n"'); // hypothetical value
  expect(line, '{"cmd":"set","key":"paste_keys","value":"он сказал \\"да\\\\n\\""}\n');
  // decodes back to the exact original value:
  expect((jsonDecode(line) as Map)['value'], 'он сказал "да\\n"');
});
```

### Minimal ru-first l10n config
```yaml
# l10n.yaml — options verified against docs.flutter.dev/ui/internationalization
arb-dir: lib/l10n
template-arb-file: app_ru.arb        # Russian IS the template/source language
output-localization-file: app_localizations.dart
preferred-supported-locales: [ru]
untranslated-messages-file: .l10n-untranslated.json   # controls leakage of untranslated keys
```
```dart
// main.dart
MaterialApp(
  localizationsDelegates: AppLocalizations.localizationsDelegates,
  supportedLocales: AppLocalizations.supportedLocales,
  // strings: context.l10n.dashboardTitle etc. — never hardcoded literals in widgets
)
```

### Material You with fallback (dynamic_color 2.1.0)
```dart
// Source: pub.dev/packages/dynamic_color README (DynamicColorBuilder + harmonized fallback)
DynamicColorBuilder(
  builder: (ColorScheme? lightDynamic, ColorScheme? darkDynamic) {
    final light = lightDynamic?.harmonized() ??
        ColorScheme.fromSeed(seedColor: const Color(0xFF35689F)); // fallback when portal says null
    final dark = darkDynamic?.harmonized() ??
        ColorScheme.fromSeed(seedColor: const Color(0xFF35689F), brightness: Brightness.dark);
    return MaterialApp(theme: ThemeData(colorScheme: light, useMaterial3: true),
                        darkTheme: ThemeData(colorScheme: dark, useMaterial3: true), ...);
  },
);
```
On KDE Plasma 6, 2.1.0 reads the accent color via the XDG Desktop Portal Settings interface [CITED: pub.dev changelog 2.1.0]; on bare WMs the builder hands you `null` and the seed fallback engages — that is the designed behavior, not a bug.

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| StateNotifier + mounted checks (Riverpod 2) | `Notifier`/`AsyncNotifier` + `ref.onDispose` (Riverpod 3) | Riverpod 3.0 stable 2025-09; 3.5.0 current | Polling providers are 10 lines; old snippets fail to compile |
| dynamic_color Linux = GTK `@theme_selected_bg_color` only | + XDG Desktop Portal Settings accent-color | 2.1.0, 2026-08-20 | KDE Plasma 6 users get real accent-driven Material You |
| (n/a) | Dart `InternetAddressType.unix` stable in `dart:io` | long-standing; Windows AF_UNIX only since Dart 3.11 | Linux UDS client needs zero packages |

**Deprecated/outdated to refuse:** `system_tray` (stale, appindicator), easy_localization (bypasses gen-l10n), Riverpod codegen for a non-codegen project [STACK.md What NOT to Use].

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | Backoff-when-disconnected (200 ms → ×2 → 2 s cap) is a good reconnect cadence; plain next-tick retry is also acceptable | Pattern 2 | Low — worst case 10 failed connects/s on a dead socket path (cheap, but noisy) |
| A2 | Dart model should mirror all 20 settings fields, not just the 17 editable ones | Protocol Reference / APP-04 | Low — `models`/`reloading`/`config_path` are needed by Phase 4 UI anyway; fixture pins them for free |
| A3 | Phase 1 resolves the socket path from `XDG_RUNTIME_DIR` only (mirror of `default_socket_path`), deferring config.toml `socket=` override parsing (no TOML parser in Dart yet) | Protocol Reference | Low-Medium — a user with a custom `socket` in config.toml won't be reached until the config-reading task lands (flag to planner: either accept for Phase 1 or hand-parse the single `socket = "..."` line like dicta-ui does today) |
| A4 | `flutter create --platforms=linux,windows` is harmless now and useful for Phase 6 CI skeleton | Standard Stack | Low — unused windows/ dir sits idle; alternatively create linux-only and regenerate later |
| A5 | fvm (cachyos repo) or AUR `flutter-bin` (paru) installs a usable Flutter 3.50 SDK on cachyos | Environment Availability | Medium — needs verification at execution time; version pinning method differs per install path |
| A6 | `AppLocalizations.localizationsDelegates` bundle includes the Global* delegates once `flutter_localizations` is a dependency (standard gen-l10n behavior) | Code Examples (l10n) | Low — if not, add the three Global delegates manually |

## Open Questions (RESOLVED)

1. **Flutter SDK install path — needs a human decision before execution.**
   - What we know: SDK absent; fvm 4.3.2 in cachyos repos; `paru` present (AUR `flutter-bin` available); manual tarball possible. Build deps all present.
   - What's unclear: which install path the user prefers (system package vs version manager vs ~/ tarball).
   - Recommendation: plan a Wave 0 task `checkpoint:human-verify` "install Flutter SDK 3.50.0" with `flutter --version && flutter doctor` as its verification.
   - **RESOLVED:** Task 1 of 01-01-PLAN.md is exactly that `checkpoint:human-verify` install gate (fvm 4.3.2 / AUR flutter-bin / official tarball, verified by `flutter --version` + `flutter doctor`); the human picks the install path at the checkpoint, everything after it is autonomous.
2. **Config.toml `socket=` override support in Phase 1.**
   - What we know: Rust clients read full Config; Dart has no TOML parser dependency yet.
   - What's unclear: whether Phase 1 must honor a custom socket path from config.toml.
   - Recommendation: mirror `default_socket_path()` (XDG_RUNTIME_DIR) for Phase 1; note the limitation in code comment; revisit in Phase 4 config work. (Or one-line scan for `^socket = "…"` — the dicta-ui precedent, ugly but tested.)
   - **RESOLVED:** deferred per assumption A3 — Phase 1 mirrors `default_socket_path()` only (XDG_RUNTIME_DIR → /run/user/<uid>) plus a `DICTA_SOCK` env override for tests; implemented in 01-01-PLAN.md Task 3 (`resolveSocketPath()` with a code comment citing the deferral) and carried in the plan's flagged_assumptions. TOML parsing is revisited in Phase 4 config work.
3. **Per-tick connect vs held connection for non-status commands.**
   - What we know: daemon keeps non-status connections open; both client styles exist in the repo (dictactl per-command, dicta-settings persistent).
   - Recommendation: per-command connections everywhere — uniform, stateless, matches the poll cycle; no connection-state machine to get wrong.
   - **RESOLVED:** per-command fresh connections everywhere — pinned as a plan truth in 01-01-PLAN.md must_haves («Command ordering needs no guarantee and none is implemented: every command opens its own fresh UDS connection…») and stated in the engine.dart library doc (01-01-PLAN.md Task 3, APP-06 ordering answer).

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Flutter SDK + Dart 3.14 | everything | ✗ | — | Install: fvm 4.3.2 (cachyos repo) / AUR `flutter-bin` via paru / manual tarball; **blocking until installed** |
| clang | Linux toolchain | ✓ | 23.1.5 | — |
| cmake | Linux toolchain | ✓ | (present via toolchain) | — |
| ninja | Linux toolchain | ✓ | system | — |
| pkg-config | Linux toolchain | ✓ | system | — |
| libgtk-3-dev | Linux embedder | ✓ | pkg-config gtk+-3.0 OK | — |
| libx11-dev | Linux embedder | ✓ | pkg-config x11 OK | — |
| libstdc++ ≥12 | Linux embedder | ✓ | 16.2.1 | — |
| dictad daemon (live) | manual smoke, fixture capture | ✓ | running, socket `/run/user/1000/dicta.sock` (mode 0600) | — |
| dictactl | fixture capture, smoke checklist | ✓ | ~/.local/bin/dictactl | — |

**Missing dependencies with no fallback:**
- Flutter/Dart SDK — **blocks all `flutter` commands.** First plan task must install it (human-verify checkpoint).

**Missing dependencies with fallback:**
- None besides the SDK.

## Validation Architecture

> `workflow.nyquist_validation` absent in `.planning/config.json` → treated as enabled. There is no existing test infrastructure in `app/` (project does not exist yet) — Wave 0 creates it.

### Test Framework
| Property | Value |
|----------|-------|
| Framework | `flutter_test` (bundled with Flutter SDK; package:test semantics for pure Dart files) |
| Config file | none — see Wave 0 (`flutter create` scaffold provides `test/` + implicit config) |
| Quick run command | `cd app && flutter test` |
| Full suite command | `cd app && flutter test && flutter analyze` |

### Phase Requirements → Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| APP-01 | Shell builds & runs, sidebar, themes | smoke (manual) + static | `flutter analyze`; manual `flutter run -d linux` checklist | ❌ Wave 0 (project) |
| APP-02 | 100 ms poll, «демон недоступен», auto-reconnect | unit (backoff/poll loop as pure logic) + manual kill-dictad smoke | `flutter test test/daemon_engine_framing_test.dart` + manual: `systemctl --user restart dictad` while app runs | ❌ Wave 0 |
| APP-03 | UI behind DictaEngine seam, transport-agnostic | widget test with FakeDictaEngine via Riverpod provider override | `flutter test test/shell_test.dart` | ❌ Wave 0 |
| APP-04 | Settings model ↔ fixture one-to-one (17 keys) | unit (golden fixture) | `flutter test test/settings_model_test.dart` | ❌ Wave 0 (fixture already captured this session — paste into repo) |
| APP-05 | ru-first gen-l10n, no hardcoded strings | static + smoke | `flutter analyze` (l10n generation runs in build); manual check of rendered strings | ❌ Wave 0 |
| APP-06 | set built with jsonEncode only | unit (regression: quotes/backslashes/newlines round-trip) | `flutter test test/daemon_engine_framing_test.dart` | ❌ Wave 0 |

### Sampling Rate
- **Per task commit:** `cd app && flutter test` (fast: pure Dart tests, milliseconds each)
- **Per wave merge:** `cd app && flutter test && flutter analyze`
- **Phase gate:** full suite green + manual smoke checklist against the live daemon (below) before `/gsd-verify-work`

### Manual smoke checklist (phase gate, against live daemon)
1. `cd app && flutter run -d linux` — window opens, sidebar switches between 4 placeholder screens.
2. Dark/light toggle (or system theme switch) — Material You colors follow the KDE accent (or graceful seed fallback).
3. Dashboard placeholder shows live state (idle) — proves the poll loop and JSON framing.
4. Kill/restart dictad while app runs — «демон недоступен» appears; state returns within ~2 s of daemon restart; **no app restart**.
5. All visible strings Russian; no hardcoded literals found by grepping widgets for `Text('` with Cyrillic outside l10n.

### Wave 0 Gaps
- [ ] `app/` project does not exist — `flutter create --platforms=linux,windows dicta-app`, `flutter pub add …`, `l10n.yaml`, first `app_ru.arb`
- [ ] `test/fixtures/settings_reply.json` — paste the fixture captured this session (Code Examples above)
- [ ] `test/settings_model_test.dart` — APP-04 golden
- [ ] `test/daemon_engine_framing_test.dart` — APP-02/APP-06 framing + jsonEncode regression
- [ ] `test/shell_test.dart` — APP-03 fake-engine widget test
- [ ] Framework install: Flutter SDK 3.50.0 (`checkpoint:human-verify`) — nothing else runs without it

## Security Domain

> `security_enforcement` absent in config → enabled. Scope note: Phase 1 is a local-only client over an existing socket; no network surface, no secrets, no auth. The daemon's own hardening is frozen and not this phase's code.

### Applicable ASVS Categories
| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no | Local UDS, same-user only; socket mode 0o600 set by the daemon [VERIFIED: crates/dictad/src/main.rs:161] |
| V3 Session Management | no | No sessions; per-command connections |
| V4 Access Control | no (daemon-side, frozen) | Client inherits the socket's user-scoped access; app adds nothing |
| V5 Input Validation | yes | `jsonEncode` for all outgoing payloads (APP-06); incoming JSON parsed leniently with unknown-key tolerance; **value validation deliberately left in the daemon** (locked decision) |
| V6 Cryptography | no | No crypto in scope |

### Known Threat Patterns for Flutter-over-UDS
| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Malformed JSON to daemon (injection via `set` values) | Tampering | `jsonEncode` only — the mitigation *is* APP-06; regression test with quotes/backslashes/newlines |
| Untrusted reply fields rendered in UI | Tampering | Model parsing with typed fields + defaults; never `jsonDecode` results fed straight into widgets |
| App writing config.toml directly (race with daemon, `set_key` quirks) | Tampering | Locked decision: all writes through `set`; app never opens config.toml for writing |
| Fixture file with real paths committed | Info disclosure | Fixture contains only user-local paths/settings — inspect before commit; optionally anonymize the home path |

## Runtime State Inventory

> Phase 1 is greenfield (new `app/` project) — not a rename/refactor/migration phase. Inventory not applicable except one note: nothing in the daemon, socket, or existing clients is renamed or touched; zero runtime state migration. **Verified by reading the phase scope (CONTEXT.md) — no existing files modified.**

## Sources

### Primary (HIGH confidence)
- `crates/dictad/src/main.rs` — protocol: Req/Res/Reply/Settings structs (25-82), serve_conn lifecycle (810-1012), set_setting (641-740), states (91-98), socket perms (161), reloading init (179) — read this session
- `crates/dicta-core/src/config.rs` — EDITABLE_KEYS (142-160), ENGINE_KEYS (163-170), default_socket_path (253-261) — read this session
- `crates/dicta-settings/src/net.rs` — persistent-client precedent, 30 s timeout, set_json escaping tests — read this session
- `crates/dictactl/src/main.rs` — per-command client precedent, 120 s timeout, manual-escaping bug (138-144) — read this session
- `crates/dicta-ui/src/panel.rs` — third Settings copy with `#[serde(default)]` (15-66), defaults to mirror — read this session
- Live daemon: socket present (0600), `dictactl status` and `dictactl settings --json` executed this session; fixture captured
- pub.dev API: flutter_riverpod 3.5.0, dynamic_color 2.1.0, window_manager 0.7.0 — latest versions + publish dates confirmed

### Secondary (MEDIUM confidence)
- api.dart.dev (via Context7 `/websites/api_dart_dev`) — `InternetAddress(path, type: InternetAddressType.unix)`, `InternetAddressType.unix` requires a valid file path; HttpClient.connectionFactory UDS example
- riverpod.dev (via Context7 `/websites/riverpod_dev`) — Timer.periodic Notifier tick provider, ref.onDispose, from_state_notifier migration (no StateNotifier/mounted)
- docs.flutter.dev (via Context7 `/websites/flutter_dev`) — l10n.yaml option table (arb-dir, template-arb-file, preferred-supported-locales, untranslated-messages-file), Linux setup deps, flutter test conventions
- pub.dev dynamic_color README/changelog — Linux = XDG portal or GTK `@theme_selected_bg_color`; 2.1.0 portal accent-color addition; DynamicColorBuilder null-fallback pattern

### Tertiary (LOW confidence)
- Third-party Material 3 guide (sagnikbhattacharya.com, 2026-06) — corroborates `lightDynamic?.harmonized() ?? ColorScheme.fromSeed` fallback idiom only

## Metadata

**Confidence breakdown:**
- Protocol facts: HIGH — every value quoted verbatim from source files read this session; live fixture captured from the running daemon
- Standard stack: HIGH — versions re-verified on pub.dev this session (correct registry; npm-gate false negative documented)
- dart:io AF_UNIX: MEDIUM-HIGH — API docs confirm the constructor semantics; the one-line live-connect spike remains the executor's first commit (STATE.md blocker resolved *in favor of dart:io*, pending the actual connect)
- Pitfalls: HIGH — five of six grounded in repo source; GPL/tray pitfalls carried from milestone research (PITFALLS.md, MEDIUM)

**Research date:** 2026-11-22
**Valid until:** 2027-01-22 (stable desktop stack; re-check leanflutter plugin versions if Flutter 3.51+ ships before execution)

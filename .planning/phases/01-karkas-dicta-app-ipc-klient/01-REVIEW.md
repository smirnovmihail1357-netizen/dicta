---
phase: 01-karkas-dicta-app-ipc-klient
plan: 01-01
reviewed: 2026-11-22T14:05:00Z
reviewed_commits: f00426a..HEAD
depth: deep
blockers: 0
warnings: 3
info: 7
necessity: 1
status: issues_found
files_reviewed: 14
files_reviewed_list:
  - app/lib/core/engine.dart
  - app/lib/core/daemon_engine.dart
  - app/lib/core/providers.dart
  - app/lib/core/settings.dart
  - app/lib/features/shell/shell.dart
  - app/lib/main.dart
  - app/lib/l10n/app_ru.arb
  - app/l10n.yaml
  - app/pubspec.yaml
  - app/test/daemon_engine_framing_test.dart
  - app/test/settings_model_test.dart
  - app/test/shell_test.dart
  - app/test/widget_test.dart
  - app/test/fixtures/settings_reply.json
  - app/tool/status_smoke.dart
---

# Phase 1: Code Review Report

**Reviewed:** 2026-11-22T14:05:00Z (commits f00426a..HEAD, scope `app/`)
**Depth:** deep (per-file + cross-file against `crates/dictad/src/main.rs` serve_conn / Reply)
**Files Reviewed:** 15 source files under `app/`
**Status:** issues_found — **0 blockers, 3 warnings, 7 info**

## Summary

The delivered skeleton is solid on the contract's critical points, each verified directly against the daemon source, not just the plan prose:

- **Framing matches the server.** Client writes one `jsonEncode` line + `\n` (engine.dart:88-91, daemon_engine.dart:64-71); the daemon reads one line (`read_line` in serve_conn), replies one line + `\n` (main.rs:995-1002). Client `LineSplitter().first` consumes exactly that. `status` gets a fresh connection per call and the socket is `destroy()`ed in a `finally` on every path after connect (daemon_engine.dart:63-74) — no leak path exists: connect failure creates no socket; write/flush/read/timeout failures all hit the `finally`. The daemon closes status connections itself (main.rs:1007-1009), and the client never assumes persistence.
- **Timeouts match protocol reality:** status 2 s, settings/set 30 s (net.rs precedent), stop/inject 120 s (dictactl precedent, matching the `done.recv()` hold at main.rs:965-969). `_command` start/toggle/cancel also use the 120 s inject timeout — a safe superset.
- **Prohibitions all hold**, verified by grep and by reading: no string-built JSON anywhere (`jsonEncode` only, including the `?app` null-aware element so `app` is omitted, not `null` — the daemon's `#[serde(default)]` accepts both); no dictated-text logging (nothing logs payloads; the smoke prints only `state.name`); no client-side `set` validation; no `dart:io` under `lib/features/`; no hotkey/keyboard code; no changes to the Rust workspace — the only non-app/ files in the range are `.planning/` artifacts and a `.gitignore` line ignoring `.planning/research/.cache/`.
- **Golden + framing + shell suites green:** `flutter analyze` → "No issues found!"; `flutter test` → 13/13 passed (re-run during this review); live smoke re-run during this review: `SMOKE OK state=idle`. The fixture is byte-identical to the RESEARCH.md capture and the test asserts all 20 fields plus the 17-key tripwire (`kEditableKeys` ≡ fixture keys − {models, reloading, config_path}). Dart fallback defaults mirror dicta-ui panel.rs `Default` exactly (min_level 0.0 = «неизвестно», n_threads 4, beam_size 5, auto_stop_level 0.22 — verified against config.rs:78-98 and panel.rs:40-65).

No blockers. The three warnings are robustness/consistency defects that should be fixed before Phase 2 builds UI on top of this seam.

## Narrative Findings (AI reviewer)

### WR-01: Committed symlink points into the gitignored build-artifact tree `.dart_tool/`

**Severity:** WARNING
**File:** `app/lib/l10n/generated` (symlink, tracked by git) + `app/lib/l10n/app_localizations.dart:1-3` + `app/l10n.yaml:6`
**Issue:** `app/lib/l10n/generated -> ../../.dart_tool/flutter_gen/gen_l10n` is committed to the repo, while its target lives in `.dart_tool` (gitignored, machine- and build-state-local). On a fresh clone, `package:dicta_app/l10n/app_localizations.dart` is an export of a dangling path: `flutter analyze` fails with an unresolved-URI error until some build step regenerates gen_l10n (the comment in l10n.yaml admits the arrangement is a Cyrillic-gate dodge — the generated ru literals are reachable under `app/lib` only through a symlink that `grep -r` deliberately does not follow). Phase 6 CI will hit exactly this on a clean runner.
**Fix:** Import the synthetic package gen-l10n already emits by default instead of a lib/-side export chain:

```yaml
# l10n.yaml — drop output-dir; default synthetic location is
# .dart_tool/flutter_gen/gen_l10n, addressable as a package:
# (no output-dir line needed)
```

```dart
// everywhere (main.dart, shell.dart, shell_test.dart):
import 'package:flutter_gen/gen_l10n/app_localizations.dart';
```
then delete the symlink and the hand-written `lib/l10n/app_localizations.dart` stub. The Cyrillic gate already excludes generated code by construction; if the team wants belt-and-braces, narrow the gate to `app/lib --exclude` generated globs rather than relying on symlink semantics.

### WR-02: `_commandLine` masks the Russian connect-failure hint for 6 of 8 commands

**Severity:** WARNING
**File:** `app/lib/core/daemon_engine.dart:77-86` (catch-all), `:57-61` (the hint being discarded), `:90-91` (its callers)
**Issue:** `_call` carefully wraps `SocketException` as «демон не отвечает <path> — запусти systemctl --user start dictad…» (:57-61), but every reply-carrying command routes through `_commandLine`, whose `catch (_)` at :83 replaces **all** exceptions — connect failure, timeout, EOF — with the generic `DictaReply(ok: false, error: «некорректный ответ демона»)`. Only `status()` and `fetchSettings()` ever surface the actionable hint. When Phase 2+ wires stop/toggle/set buttons, a dead daemon will present as "bad reply from daemon" with no fix hint, and a 120 s `stop` timeout will read identically to a syntax error. (No current user impact — no UI calls these yet — hence WARNING, not BLOCKER.)
**Fix:** Distinguish transport errors from parse errors inside `_commandLine`:

```dart
Future<DictaReply> _commandLine(String line,
    {required Duration timeout}) async {
  try {
    final raw = await _call(line, timeout: timeout);
    try {
      final map = decodeReplyMap(raw);
      final error = map['error'] as String?;
      return DictaReply(ok: error == null, error: error);
    } on FormatException {
      return DictaReply(ok: false, error: _badReply);
    }
  } on SocketException catch (error) {
    return DictaReply(ok: false, error: error.message); // the Russian hint
  } on TimeoutException {
    return DictaReply(ok: false, error: _timeoutHint);
  }
}
```

### WR-03: In-flight poll can write `state` after the notifier is disposed

**Severity:** WARNING
**File:** `app/lib/core/providers.dart:24-27` (timer cancel only), `:33` (`state =` after `await`), `:37` (backoff sleep inside the guard)
**Issue:** `ref.onDispose` cancels the `Timer`, but a `poll()` already awaiting `engine.status()` (or the backoff `Future.delayed`) is not cancelled; when it completes it executes `state = …` and `ref.read(engineProvider)` on a disposed notifier/ref. In Riverpod 3 there is no `mounted` to check (the plan itself says so), and touching a disposed notifier's `state` is undefined behavior that surfaces as a `StateError` in the async error zone — reachable via widget-test teardown, hot restart, or any future provider rebuild/invalidate. The current tests pass only because no poll is in flight at teardown (the fake resolves in a microtask inside the last `pump`).
**Fix:** Add a dispose flag, mirroring the plain-bool reentrancy guard the plan already mandates:

```dart
bool _disposed = false;

@override
DictaStatus build() {
  _disposed = false;
  _timer = Timer.periodic(const Duration(milliseconds: 100), (_) => poll());
  ref.onDispose(() {
    _disposed = true;
    _timer?.cancel();
  });
  return const DictaStatus.disconnected();
}

Future<void> poll() async {
  if (_inFlight || _disposed) return;
  _inFlight = true;
  try {
    final snapshot = await ref.read(engineProvider).status();
    if (_disposed) return;
    state = snapshot;
    ...
```

## Info

### IN-01: Unreachable legacy `base.error` branch (dead defensive code) — **Category: necessity**

**Severity:** INFO
**File:** `app/lib/core/settings.dart:124-125`
**Issue:** `fromReply` extracts an error from a `reply['base']` map "legacy `base.error` too" — but the daemon's `Reply` serializes `Res` with `#[serde(flatten)]` (crates/dictad/src/main.rs:353-359, verified this review), so `base` never appears on the wire and the branch cannot execute against any daemon this milestone allows (the protocol is frozen). It is generality without instantiation.
**Evidence:** main.rs:354 `#[serde(flatten)] base: Res<'a>` → serde emits `state/level/seconds/text/error` at the top level; no client in the repo produces a `base`-wrapped reply.
**Fix:** Delete the `base` lookup (keep the top-level `error` extraction); if defensive tolerance is wanted, a comment citing the flatten fact is cheaper than dead code.

### IN-02: Golden fixture commits real user-local paths (known, accepted)

**Severity:** INFO
**File:** `app/test/fixtures/settings_reply.json:1`
**Issue:** The fixture contains `/home/wardreg/.local/share/whisper-cpp/*.bin` model paths and `/home/wardreg/.config/dicta/config.toml`. Per the threat model (T-1-04) this was inspected and accepted before commit: no dictated text (`"text":null`), no secrets, user-local paths only. Recorded here as the review focus requests; no action required. RESEARCH.md notes optional home-path anonymization if the repo ever becomes public.
**Fix:** none (accepted disposition); optionally anonymize before any public push.

### IN-03: Russian runtime strings as `\uXXXX` escapes — outside l10n, invisible to the Cyrillic gate

**Severity:** INFO
**File:** `app/lib/core/daemon_engine.dart:24-31`
**Issue:** The three daemon-error strings are written as Unicode escapes to keep the Cyrillic gate green. Honest (commented at :24-25) and technically compliant — they are runtime exception/reply texts, not rendered widgets — but `_badReply` and the connect hint travel inside `DictaReply.error`, which Phase 4 will render; at that point they are Russian UI strings living outside the arb/l10n system, and the gate cannot detect violations of its own spirit because the escape trick is now the established pattern.
**Fix:** When Phase 4 renders `DictaReply.error`, either move these strings into `app_ru.arb` (error-message keys) or document a separate runtime-strings convention; avoid extending the `\uXXXX` trick.

### IN-04: `DaemonEngine`'s broadcast `StreamController` is never closed

**Severity:** INFO
**File:** `app/lib/core/daemon_engine.dart:38-39`, `app/lib/core/providers.dart:8-10`
**Issue:** `engineProvider` constructs `DaemonEngine` without `ref.onDispose`; the broadcast controller (and its per-instance state) is never closed. Harmless for the single app-lifetime instance (broadcast, no buffering), but every `ProviderScope` in tests creates another unclosed controller.
**Fix:** `final engineProvider = Provider<DictaEngine>((ref) { final e = DaemonEngine(resolveSocketPath()); ref.onDispose(e.dispose); return e; });` with a small `dispose()` that closes the controller; nothing in Phase 1 depends on the omission.

### IN-05: `fetchSettings` throws on a malformed reply while every sibling is exception-free

**Severity:** INFO
**File:** `app/lib/core/daemon_engine.dart:119-122` (via `engine.dart:82-86`)
**Issue:** `status()` maps garbage to a disconnected snapshot; `set/stop/...` map it to `DictaReply(ok:false)`; but `fetchSettings()` propagates raw `FormatException` (and `TimeoutException`/`SocketException`) to the caller. Phase 4 will be the first caller and must know to catch — the interface doc doesn't say so. Consistency defect, not a current bug (no caller exists yet).
**Fix:** Document the throwing contract on `DictaEngine.fetchSettings`, or return `(Map<String, dynamic>?, String?)`-style result; decide before Phase 4 wires it.

### IN-06: `resolveSocketPath` can crash at startup if `id` is not resolvable

**Severity:** INFO
**File:** `app/lib/core/daemon_engine.dart:20-21`
**Issue:** The `/run/user/<uid>` fallback runs `Process.runSync('id', ['-u'])` via PATH lookup and uses the raw stdout; a missing binary throws `ProcessException` inside `engineProvider`'s construction (app startup crash), and empty/garbage stdout silently yields `/run/user/dicta.sock`. Only reachable when `XDG_RUNTIME_DIR` is unset — vanishingly rare on a systemd desktop, and mirror-of-dicta-core keeps it honest — hence INFO.
**Fix:** Wrap in a defensive `try` with a documented fallback, or parse `stdout` with a digit check: `if (!RegExp(r'^\d+$').hasMatch(uid)) …`.

### IN-07: Placeholder test asserting `1 + 1 == 2` ships in the final suite

**Severity:** INFO
**File:** `app/test/widget_test.dart:6-8`
**Issue:** The Task-2 scaffold placeholder survived into the phase's committed state ("real suites land in tasks 3-5" — they did, as daemon_engine_framing/settings_model/shell tests). It asserts nothing about the app and inflates the green count (13 tests, one of which is `expect(1 + 1, 2)`).
**Fix:** Delete `app/test/widget_test.dart`; the phase's real suites fully supersede it.

## Verification performed (this review)

- `cd app && flutter analyze` → "No issues found!"
- `cd app && flutter test` → 13/13 passed
- `cd app && dart run tool/status_smoke.dart` → `SMOKE OK state=idle` (live daemon)
- Framing/timeout/destroy paths traced line-by-line against `crates/dictad/src/main.rs` serve_conn (:940-1012) and `Reply` (:353-359); defaults traced against `crates/dicta-core/src/config.rs:78-98` and `crates/dicta-ui/src/panel.rs:40-65`
- Prohibition greps: `dart:io` under features/ = 0; hotkey/keyboard identifiers = 0; `StateNotifier` = 0; non-comment Cyrillic in `app/lib` (following only tracked files) = 0; changes outside `app/` limited to `.planning/` artifacts and one `.gitignore` line

---

_Reviewed: 2026-11-22T14:05:00Z_
_Reviewer: gsd-code-reviewer_
_Depth: deep_

## REVIEW COMPLETE

**0 blockers · 3 warnings · 7 info (1 necessity)**

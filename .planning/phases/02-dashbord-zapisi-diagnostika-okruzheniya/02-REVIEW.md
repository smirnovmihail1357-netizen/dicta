---
phase: 02-dashbord-zapisi-diagnostika-okruzheniya
plan: 02-01
reviewed: 2026-11-22T11:32:34Z
reviewed_commits: 2714e27..39f8880
depth: deep
critical: 1
warning: 4
info: 7
necessity: 1
total: 12
status: issues_found
files_reviewed: 25
files_reviewed_list:
  - app/lib/core/daemon_engine.dart
  - app/lib/core/engine.dart
  - app/lib/core/providers.dart
  - app/lib/features/dashboard/calibration_card.dart
  - app/lib/features/dashboard/dashboard_screen.dart
  - app/lib/features/dashboard/diagnostics_card.dart
  - app/lib/features/dashboard/dicta_control.dart
  - app/lib/features/dashboard/last_text_card.dart
  - app/lib/features/dashboard/level_history.dart
  - app/lib/features/shell/shell.dart
  - app/lib/l10n/app_ru.arb
  - app/lib/l10n/generated/app_localizations.dart
  - app/lib/l10n/generated/app_localizations_ru.dart
  - app/test/calibration_test.dart
  - app/test/daemon_engine_framing_test.dart
  - app/test/dashboard_buttons_test.dart
  - app/test/dashboard_state_test.dart
  - app/test/diagnostics_test.dart
  - app/test/fake_dicta_engine.dart
  - app/test/fake_dicta_engine_test.dart
  - app/test/last_text_test.dart
  - app/test/level_history_test.dart
  - app/test/shell_test.dart
  - app/test/widget_test.dart
  - app/tool/diag_smoke.dart
---

# Phase 2: Code Review Report

**Reviewed:** 2026-11-22T11:32:34Z (commits 2714e27..39f8880, scope `app/`)
**Depth:** deep (per-file + cross-file against `crates/dictad/src/main.rs` serve_conn/Res, plus a live Riverpod 3.5.0 semantics probe)
**Files Reviewed:** 25 source files under `app/`
**Status:** issues_found — **1 blocker, 4 warnings, 7 info (1 necessity)**

## Summary

The phase's honesty contract holds on every point that was pinned by tests, each re-verified directly against the daemon source, not just the plan prose:

- **Seam extension is correct.** `stop`/`inject` return `DictaStatus` via `_snapshotCommand` with the 120 s `_injectTimeout` (daemon_engine.dart:107-147); the stop reply's `text` is parsed (framing test :47-53) and is the only text source — status never carries text (main.rs:828-833 re-read). `buildInjectLine` serializes only through `jsonEncode`, `'app': ?app` omits null (daemon's `#[serde(default)]` accepts), round-trip with quotes/newlines pinned (framing test :62-83). No manual JSON anywhere.
- **Poller is now dispose-safe.** Phase-1 WR-03 fixed: `_disposed` flag guards every post-`await` write (providers.dart:30, 45, 49, 53-56); backoff caps at 2 s and holds `_inFlight` through the sleep so ticks can't stack. Phase-1 IN-04 also fixed (`engineProvider` disposes `DaemonEngine`, providers.dart:9-13; `dispose()` closes the broadcast controller). The phase-1 placeholder test (IN-07) was replaced by a real DictaApp boot smoke.
- **Button matrix is honest** (dashboard_screen.dart:32-35): start only idle+connected, stop/cancel only recording+connected, cancel disabled in transcribing, all disabled when disconnected — all four states widget-tested; spinner bound to the polled state, dies on the next idle tick, no UX timeout after stop.
- **Threats T-2-01/02/03/SC hold.** No payload logging (grep: zero `print`/`debugPrint`/`log` in lib/ and tool/; diag_smoke sends only `''`, which the daemon rejects before insertion — main.rs:881-893 re-verified, live smoke re-run during this review); `Clipboard.setData` exists only behind the click-only `clipboardWriterProvider` (single grep hit, dicta_control.dart:129); >5000 chars is not client-validated and daemon errors render verbatim; `git diff 2714e27..HEAD -- app/pubspec.yaml app/pubspec.lock` and `-- crates/` are both empty (re-run, byte-identical).
- **Battery re-run during this review:** `flutter test` 45/45 passed; `flutter analyze` "No issues found!"; `bash tool/cyrillic_gate.sh` → 0; `dart run tool/diag_smoke.dart` → `DIAG SMOKE state=idle connected=true` + `INJECT-REJECTED-AS-EXPECTED`, exit 0. One fake factory (`implements DictaEngine` count = 1). The l10n symlink problem from phase-1 WR-01 is gone — `lib/l10n/generated/` is a real tracked directory with committed sources (mode 100644 at 2714e27 already).

But the adversarial pass found one shipped defect the green battery cannot see, plus robustness and honesty-labeling gaps. The blocker is in DASH-04 itself: **after every successful calibration gesture the marker snaps back to the stale pre-drag threshold and never re-syncs**, because the code reads `.value` off a just-refreshed `FutureProvider` synchronously — which in Riverpod 3.5 returns the *previous* data (verified with a live probe, see CR-01). The current test suite cannot catch it because no test asserts the marker position after a release.

## Structural Findings (fallow)

None provided for this phase (no structural pre-pass block in the task prompt).

## Narrative Findings (AI reviewer)

### CR-01: Calibration marker regresses to the stale threshold after every successful set — the "marker updates after set" contract is broken

**Severity:** CRITICAL (BLOCKER)
**File:** `app/lib/features/dashboard/calibration_card.dart:103-111`
**Issue:** `onHorizontalDragEnd` does:

```dart
final refreshed = ref.refresh(settingsSnapshotProvider);
...
final freshMin = reply.error == null ? refreshed.value?.minLevel : null;
setState(() {
  _error = reply.error;
  if (freshMin != null) _marker = freshMin;
});
```

`settingsSnapshotProvider` is a `FutureProvider` over an async socket fetch. In Riverpod 3.5, `ref.refresh` on a FutureProvider returns the *previous* `AsyncData` in loading state — the new value cannot be observed synchronously. Verified with a live probe against the repo's locked riverpod 3.5.0 (`dart run` with the app's package config):

```
before: AsyncData<int>(value: 42)
right-after-refresh: AsyncData<int>(isLoading: true, value: 42)   <-- old value
settled: AsyncData<int>(value: 99)
```

Consequences, per gesture with a successful set: (a) `freshMin` is the **pre-drag** `minLevel` (e.g. 0.35), so `_marker` is overwritten with 0.35 right after the daemon accepted 0.42 — the visible marker **jumps back to where it was before the gesture**; (b) `_marker` stays non-null forever, so the later-arriving refetched settings are shadowed in `build()` (`marker = _marker ?? settings?.minLevel ?? 0.0`, :73) — neither the post-set refetch nor the diagnostics "перепроверить" can ever move the marker again; (c) the intended re-sync branch (`if (freshMin != null) _marker = freshMin;`) can never receive the *new* value — it is dead-as-designed. The plan's acceptance criterion "Отметка инициализируется из settings-ответа … обновляется после set" is violated with a visible effect: after calibrating, the UI lies about where the threshold is (off by one gesture, including the daemon's `{:.2}` rounding, e.g. dragged 0.4249 → sent `0.42` → shown 0.35). The suite misses it because `calibration_test.dart` asserts `setCalls` after release (:76-78) but never the painter marker.
**Fix:** Await the refetch's future instead of reading the snapshot synchronously:

```dart
onHorizontalDragEnd: (_) async {
  final value = _marker ?? marker;
  final reply =
      await ref.read(engineProvider).set('min_level', value.toStringAsFixed(2));
  if (!mounted) return; // before any ref use
  if (reply.error != null) {
    setState(() => _error = reply.error);
    return;
  }
  ref.invalidate(settingsSnapshotProvider);
  final fresh = await ref.read(settingsSnapshotProvider.future);
  if (!mounted) return;
  setState(() {
    _error = null;
    _marker = fresh.minLevel; // canonical daemon value, incl. {:.2} rounding
  });
}
```

and add a regression test asserting the painter marker after a release with a scripted changed `fetchSettingsReply`.

## Warnings

### WR-01: Every command's error is rendered under the inject-specific heading «ошибка вставки»

**Severity:** WARNING
**File:** `app/lib/features/dashboard/last_text_card.dart:80-91` (with `app/lib/features/dashboard/dicta_control.dart:16-25`)
**Issue:** `lastError` is shared across start/stop/cancel/inject in `DictaControlState`, but the only place it renders is the last-text card, under the hardcoded `dashInjectFailed` label «ошибка вставки». Reachable mislabel: daemon dies between the last poll and a Stop click (a 100 ms window) → `stop()` → `_snapshotCommand` wraps the `SocketException` → `lastError` = «демон не отвечает … запусти systemctl --user start dictad…» → the card shows «ошибка вставки» + that hint. Same for a failed start/cancel. The phase's whole theme is honest UI; labeling a stop failure as an inject error is a dishonest label.
**Fix:** Either store the failing command in `DictaControlState` (e.g. `lastErrorCommand`) and choose the heading per command, or change `dashInjectFailed` to a neutral «ошибка команды» and keep the daemon text verbatim below it.

### WR-02: Use of `ref` after `await` without a mounted guard in the calibration drag handler

**Severity:** WARNING
**File:** `app/lib/features/dashboard/calibration_card.dart:100-103`
**Issue:** `onHorizontalDragEnd` awaits `set(...)` (up to the 30 s `_settingsTimeout`), then calls `ref.refresh(...)` **before** the `if (!mounted) return;` check at :104. `WidgetRef` after the element is unmounted throws; nothing catches it (the closure is fire-and-forget from the gesture recognizer), producing an unhandled async-zone error at app teardown/hot restart with a set in flight. In the running app the card is permanently mounted (IndexedStack), so the practical trigger is teardown — the same class of defect as phase-1 WR-03, which this phase fixed in the poller but reintroduced in a new callback.
**Fix:** Move the guard up, before any `ref` use:

```dart
final reply = await ref.read(engineProvider).set(...);
if (!mounted) return;
final refreshed = ...;
```

### WR-03: `TimeoutException` is still masked as «некорректный ответ демона» — phase-1 WR-02 only half-fixed

**Severity:** WARNING
**File:** `app/lib/core/daemon_engine.dart:93-96` (`_commandLine`), `:117-123` (`_snapshotCommand`)
**Issue:** Both wrappers now distinguish `SocketException` (surfaces the Russian connect hint — good, that part of phase-1 WR-02 is fixed), but `TimeoutException` still falls into the generic `catch (_)` → `_badReply`. A 120 s `stop` held by a legal paste (`clipboard_restore_ms` up to 60000, main.rs:548 — the reason the timeout exists), a 120 s `inject`, or a 30 s `set` that times out is reported to the user as "bad reply from daemon" — indistinguishable from a protocol syntax error. For the flagship stop path this is the most likely real-world failure after a wedged paste.
**Fix:** Add explicit branches in both wrappers:

```dart
} on TimeoutException {
  return DictaStatus(
    connected: false,
    state: DaemonState.idle,
    error: _timeoutHint, // "daemon did not answer in time - is a paste stuck?"
  );
}
```

### WR-04: Coverage holes the blocker slipped through — cancel() is never pressed by any test, and the factory cannot even script its reply

**Severity:** WARNING
**File:** `app/test/dashboard_buttons_test.dart` (matrix only), `app/test/fake_dicta_engine.dart:85-86`, `app/lib/features/dashboard/dicta_control.dart:88-94`
**Issue:** Three untested branches, two of them the most brittle code in the phase:
1. `DictaControlNotifier.cancel()` is **never invoked by any test** — dashboard_buttons_test only checks enabled-state. The `_cancelNotice` exact-string suppression (`reply.error == '\u0437\u0430\u043f\u0438\u0441\u044c \u043e\u0442\u043c\u0435\u043d\u0435\u043d\u0430'`, dicta_control.dart:11-12, 88-94) compares a hardcoded copy of the daemon's runtime string (verified byte-equal to main.rs:947 today) — zero tests pin it, and the shared fake has **no `cancelReply` hook** (`FakeDictaEngine.cancel` always returns `ok: true`), so the suppression path is currently *untestable without editing the factory*. A daemon wording change would silently turn an honest notice into a displayed error, and no suite would notice.
2. No test asserts the calibration marker position after a drag release — exactly where CR-01 hides.
3. The poller's failure path (backoff doubling, disconnect state, recovery) in `providers.dart:52-60` is untested; the fake never throws from `status()`.
**Fix:** Add a `cancelReply` hook to the factory; add tests: (a) cancel with `error: 'запись отменена'` leaves `lastError` null and shows no error row; (b) cancel with any other error shows it verbatim; (c) marker equals the refetched `minLevel` after a successful release; (d) a throwing `status()` (scriptable via a status hook or a small wrapper fake) drives the disconnected state and backoff.

## Info

### IN-01: Diagnostics model row goes red «модель: данных нет» while settings are merely loading or the fetch failed

**Severity:** INFO
**File:** `app/lib/features/dashboard/diagnostics_card.dart:40-43`
**Issue:** `ref.watch(settingsSnapshotProvider).value` collapses `AsyncLoading` and `AsyncError` into `settings == null` → red error row. At app start with a healthy daemon the model row flashes red until the fetch lands; on a settings-fetch hiccup (socket busy) it stays red and reads as "no model" even when the model is loaded and fine. The plan's matrix ("иначе красная") is literally satisfied, but the label is dishonest for the not-yet-data case, against the phase's own "по данным демона" caveat spirit.
**Fix:** Split the null case: `if (async.isLoading) → neutral 'данные модели загружаются…'`, keep red only for settled empty/missing model, and surface AsyncError distinctly.

### IN-02: `settingsSnapshotProvider` silently discards the parser's error

**Severity:** INFO
**File:** `app/lib/core/providers.dart:20-23` (with `app/lib/core/settings.dart:115-131`)
**Issue:** `SettingsReply.fromReply(reply).settings` drops the tuple's `error` field. A poisoned settings reply (top-level error) yields an all-defaults `SettingsReply` — model `''` (red diag row), `minLevel 0.0` (marker at floor) — with the error string vanishing. Violates the phase's errors-as-data doctrine ("ответы демона НЕ доверены … показываются как данные"); also `fetchSettings()` itself still propagates raw `SocketException`/`TimeoutException` (phase-1 IN-05, still undocumented), which lands in the FutureProvider as an opaque AsyncError.
**Fix:** Include the error in the provider value (e.g. a small record type `({SettingsReply settings, String? error})`), or rethrow the reply error so `AsyncError` is at least distinguishable, and document the throwing contract of `fetchSettings`.

### IN-03: Unreachable `catch (_)` blocks in DictaControlNotifier — **Category: necessity**

**Severity:** INFO
**File:** `app/lib/features/dashboard/dicta_control.dart:57-61, 76-80, 95-99, 114-118`
**Issue:** Generality without instantiation: all four `catch (_) → _commandFailed` blocks are unreachable in this repo. `DaemonEngine.start/cancel` route through `_commandLine`, which catches everything into `DictaReply` (daemon_engine.dart:83-97); `stop/inject` route through `_snapshotCommand`, which catches everything into `DictaStatus` (:107-124); `FakeDictaEngine` never throws either. The file's own header (:1-3) documents the contract: "errors arrive as data, never as exceptions". Not a Warning because the `DictaEngine` interface does not formally promise non-throwing per method (`fetchSettings` does throw — phase-1 IN-05), so this rests on judgment: cheap guards against an undocumented seam contract, and `_commandFailed` is their only cost.
**Fix:** Either document "never throws" on the four methods in `engine.dart` and drop the catches, or keep one shared guard; decide before Phase 3 builds on the control notifier.

### IN-04: `statusStream` is dead interface surface

**Severity:** INFO
**File:** `app/lib/core/engine.dart:86`, `app/lib/core/daemon_engine.dart:38-42, :131`
**Issue:** The interface requires `statusStream`; `DaemonEngine` maintains a broadcast controller and pushes into it on every `status()` (:131); grep across the repo shows exactly three hits — declaration, implementation, and the fake's empty stream. No consumer exists (the poller calls `status()` directly). Phase-1 leftover that the phase-2 interface extension faithfully preserved.
**Fix:** Remove `statusStream` (and the controller + `dispose` simplifies to nothing) in a later seam-touching task, or land its intended consumer; don't carry it into Phase 3 untouched.

### IN-05: Russian user-visible runtime strings escaped outside the l10n system have spread from core/ into features/

**Severity:** INFO
**File:** `app/lib/features/dashboard/dicta_control.dart:11-14`
**Issue:** `_cancelNotice` and `_commandFailed` are `\uXXXX`-escaped Russian strings living outside `app_ru.arb`, invisible to the Cyrillic gate by construction. `_commandFailed` would render to the user via `lastError` (WR-01 path); `_cancelNotice` participates in a string equality against daemon output. This extends the phase-1 IN-03 pattern (daemon_engine.dart:24-31) from core into feature code — the escape trick is now the established convention in two layers.
**Fix:** Move rendered strings into `app_ru.arb` (error-message keys); keep `_cancelNotice` as a documented protocol-data constant (it is compared, not displayed).

### IN-06: Six near-identical test app builders cloned across the new suites

**Severity:** INFO
**File:** `app/test/dashboard_state_test.dart:15-33`, `app/test/dashboard_buttons_test.dart:16-34`, `app/test/last_text_test.dart:16-32`, `app/test/calibration_test.dart:17-27`, `app/test/diagnostics_test.dart:16-26`, `app/test/fake_dicta_engine_test.dart:15-25`
**Issue:** The plan's rule was "тест-специфичный хелпер дописывается в фабрику, не клонируется" — but each new suite carries its own `build…App` clone of the same ProviderScope+MaterialApp+ru-locale scaffold, differing only in overrides (`captureContainer`, `onCopy`) and none hosted in `fake_dicta_engine.dart`. Six copies of the same wiring is a drift risk for the next interface change.
**Fix:** One parameterized helper in `fake_dicta_engine.dart`:

```dart
Widget buildFakeApp(FakeDictaEngine fake, {List<Override> overrides = const []}) =>
    ProviderScope(
      overrides: [engineProvider.overrideWithValue(fake), ...overrides],
      child: MaterialApp(...locale: const Locale('ru'), home: const ShellWidget()),
    );
```

### IN-07: Shared `commandInFlight` disables *all* controls during any in-flight command — undocumented superset of the plan

**Severity:** INFO
**File:** `app/lib/features/dashboard/dicta_control.dart:16-25` (single flag), `app/lib/features/dashboard/dashboard_screen.dart:31-35`, `app/lib/features/dashboard/last_text_card.dart:52, 68`
**Issue:** The plan pinned "команда «в полёте» дизейблит **свою** кнопку"; the implementation disables start/stop/cancel **and copy** for the whole in-flight window. For a 120 s stop/inject that is up to two minutes of a fully frozen dashboard even though the daemon takes fresh connections per command and would accept a concurrent start. Safe (a superset can't send an invalid sequence), so INFO — but it's a behavioral deviation worth a conscious decision rather than an accident.
**Fix:** Either accept and document it, or track which command is in flight (`commandInFlight` → enum) and disable only the affected controls.

## Test-coverage notes (as requested, beyond the findings)

Untested branches observed while tracing: the entire `DaemonEngine` error mapping (`_snapshotCommand`'s SocketException → `connected:false` snapshot; `_commandLine`'s hint path) is exercised only by phase-1 pure framing tests, never against a scripted socket; `LevelHistogramPainter.paint` (thresholds 0.6/0.3, marker line) has no direct paint assertions — tests assert the sample list only; `_formatSeconds` is pinned only at 65.3 s (hour rollover untested, though the math is trivially correct); the poller's backoff/disconnect path (see WR-04.3); `cancel()` end-to-end (WR-04.1); marker-after-set (WR-04.2 / CR-01).

## Phase-1 findings disposition (re-checked in this review)

- WR-02 (masked connect hint): **partially fixed** — `SocketException` now surfaces the hint; `TimeoutException` still masked (WR-03 above).
- WR-03 (dispose race in poller): **fixed** — `_disposed` flag on every post-await path.
- WR-01 (l10n symlink into `.dart_tool`): **resolved** — real committed generated sources (already regular files at 2714e27).
- IN-04 (unclosed engine controller): **fixed** — `engineProvider` disposes the engine.
- IN-05 (`fetchSettings` throwing contract): **still open**, now with a real consumer — see IN-02.
- IN-07 (`1+1==2` placeholder): **fixed** — replaced by a real DictaApp boot smoke.

## Verification performed (this review)

- `cd app && flutter test` → 45/45 passed (re-run)
- `cd app && flutter analyze` → "No issues found!" (re-run)
- `cd app && bash tool/cyrillic_gate.sh` → 0 (re-run)
- `cd app && dart run tool/diag_smoke.dart` → `DIAG SMOKE state=idle connected=true` + `INJECT-REJECTED-AS-EXPECTED`, exit 0 (live daemon, re-run)
- `git diff --quiet 2714e27 HEAD -- crates/` → empty; `git diff --quiet 2714e27 HEAD -- app/pubspec.yaml app/pubspec.lock` → empty (protocol + dependency freeze intact)
- `grep -rn "implements DictaEngine" app/test/*.dart | wc -l` → 1
- Prohibition greps: `dart:io` under `lib/features/` = 0; `print|debugPrint|log(` in lib/+tool/ = 0; `Clipboard` hits = 1 (the seam); TODO/FIXME = 0
- Daemon cross-checks re-read in `crates/dictad/src/main.rs`: status never carries text (:828-833); inject empty/>5000 rejection before insertion (:881-893); action replies carry `last_text`/`last_error` (:978-984); cancel sets exactly «запись отменена» (:947) — byte-matches the client constant; `run_start` clears `last_error`/`last_text` (:1018-1019), so no stale-error leak into stop replies
- Riverpod semantics probe (repo-locked riverpod 3.5.0, live run): `ref.refresh(futureProvider)` immediately after refresh returns the **previous** `AsyncData` in loading state — the basis of CR-01

---

_Reviewed: 2026-11-22T11:32:34Z_
_Reviewer: gsd-code-reviewer_
_Depth: deep_

## REVIEW COMPLETE

**1 blocker · 4 warnings · 7 info (1 necessity)**

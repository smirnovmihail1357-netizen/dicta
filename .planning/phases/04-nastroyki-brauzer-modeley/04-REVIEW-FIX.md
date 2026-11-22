---
phase: 04-nastroyki-brauzer-modeley
fixed_at: 2026-11-22T17:43:40+03:00
review_path: .planning/phases/04-nastroyki-brauzer-modeley/04-REVIEW.md
iteration: 1
findings_in_scope: 15
fixed: 13
skipped: 2
status: partial
---

# Phase 4: Code Review Fix Report

**Fixed at:** 2026-11-22T17:43:40+03:00
**Source review:** `.planning/phases/04-nastroyki-brauzer-modeley/04-REVIEW.md`
**Iteration:** 1
**Verification location:** main checkout (`workflow.use_worktrees = false`; no worktree created, no recovery sentinel)

**Summary:**
- Findings in scope: 15
- Fixed: 13 (3 critical, 6 warnings, 4 info)
- Skipped (WONTFIX with justification): 2 info
- Battery: 131/131 before → **138/138 after** (7 new regression tests)

## Fixed Issues

### CR-01: Set-success branch dead in production — success note treated as error, snapshot never applied

**Files modified:** `app/lib/features/settings/settings_screen.dart`, `app/test/fake_dicta_engine.dart`, `app/test/settings_screen_test.dart`
**Commit:** `d867622`
**Applied fix:** `_commit` now branches on `reply.error` only. On `error == null` it stores `reply.note` as a neutral liveRegion status line (`_statusIsError = false`), clears `_pending` and applies `reply.settings` — the same canon as `CalibrationCard._commitMarker`. Red styling is reserved for `error != null`. `FakeDictaEngine.set` now synthesizes the daemon-true non-null success note when no scripted error/note exists; regression test asserts a success note is NOT error-colored AND the snapshot applies with no refetch.

### CR-02: Successful model selection skipped the mandatory refetch

**Files modified:** `app/lib/features/models/models_screen.dart`, `app/test/models_screen_test.dart` (same commit as CR-01)
**Commit:** `d867622`
**Applied fix:** Removed `|| reply.note != null` from the early return; only a real error short-circuits. Successful select now does invalidate + await refetch. Test scripts a daemon-true success note, mutates the snapshot via `onSet`, and asserts `fetchCount` growth plus the active marker moving to the newly selected model.

### CR-03: Four sliders sent unit-suffixed values the daemon rejects

**Files modified:** `app/lib/features/settings/settings_screen.dart`, `app/test/settings_screen_test.dart`
**Commit:** `ecee344`
**Applied fix:** `_slider` gained a `wireFormat` callback (defaults to display `format`); `onChanged`/`onChangeEnd` commit the raw wire value while labels/readouts keep units. `_sliderValue` now tolerates a leading numeric token so a readout cannot snap back. All four affected keys (`auto_stop_after_ms`, `auto_stop_grace_ms`, `max_record_ms`, `clipboard_restore_ms`) pass raw `v.toInt().toString()`. New drag test covers all four keys: one set per release, `^\d+$` wire values, `int.tryParse` success, readout follows the drag.

### WR-01: HF search pagination leaked offset and replaced instead of appending

**Files modified:** `app/lib/features/models/hf_browser.dart`, `app/test/models_browser_test.dart`
**Commit:** `8fd9e9d`
**Applied fix:** `_search({bool append = false})` resets `_offset = 0` on every explicit find, and the show-more handler uses `append: true`. Success merges by repo id (unique widget keys) instead of replacing the list. New test verifies page-2 append, offset reset on a new query, and replacement of the old result list.

### WR-02: One-at-a-time download guard had an await gap

**Files modified:** `app/lib/core/downloads.dart`, `app/test/fake_dicta_engine.dart`, `app/test/download_state_test.dart`
**Commit:** `851eea2`
**Applied fix:** `DownloadManager._starting` claims the slot synchronously before the settings await; errors go through the existing `_handleError` (now accepts `File?`). Added `fetchSettingsGate` fake hook and a test that parks two starts at the same await and asserts only one engine call, for the first URL.

### WR-03: Cancel during connection establishment was silently dropped

**Files modified:** `app/lib/core/hf/download_engine.dart`, `app/test/download_state_test.dart`
**Commit:** `b659c7a`
**Applied fix:** Added `DownloadCancelGate` — `cancel()` records the intent even with `_request == null`; `download()` resets it and calls `throwIfCancelled()` after `getUrl` and after `close()`. Unit test covers cancel-before-request, reset, and re-cancel.

### WR-04: Unguarded snapshot awaits; reload timer error-looped

**Files modified:** `app/lib/features/settings/settings_screen.dart`, `app/lib/features/models/models_screen.dart`, `app/lib/core/downloads.dart`, `app/test/fake_dicta_engine.dart`, `app/test/settings_screen_test.dart`
**Commit:** `3b002c8`
**Applied fix:** All four refetch sites wrapped in try/catch. The reload timer stops on transport failure and surfaces `l10n.daemonUnavailable` in the neutral status line instead of refetching every second. Models select/delete and `DownloadManager._finish` no longer leak unhandled zone errors. New test kills the daemon mid-reload and asserts the honest hint plus a stopped timer (no further fetches across 3 s).

### WR-05: `filenameOf` accepted `..`, escaping dest_dir

**Files modified:** `app/lib/core/hf/hf_parse.dart`, `app/lib/core/downloads.dart`, `app/test/hf_parse_test.dart`, `app/test/download_state_test.dart`
**Commit:** `01460fe`
**Applied fix:** `filenameOf` rejects `.`/`..`/backslash names; `DownloadManager.start` adds a second guard against traversal-like `destName`. Tests assert `https://x/..` → null, percent-encoded `..%2F..bin` stays a literal name, and the manager rejects `..` before any engine/filesystem call.

### WR-06: Models screen showed the empty state on fetch failure

**Files modified:** `app/lib/l10n/app_ru.arb`, both generated l10n files, `app/lib/features/models/models_screen.dart`, `app/test/models_screen_test.dart`
**Commit:** `cc4ef19`
**Applied fix:** `settingsSnapshotProvider` is now watched as an `AsyncValue`: `AsyncError` → honest error row (`modelFetchFailed`) with a retry button that invalidates the provider; `AsyncData` with an empty `models` list → the original empty card; loading → no dishonest message. Two new .arb keys regenerated via `flutter gen-l10n`; Cyrillic gate stays 0.

### IN-02: Dead `isEmpty` arms in split helpers

**Files modified:** `app/lib/core/hf/hf_parse.dart`, `app/lib/features/models/model_store.dart` (commit `a0c9750`)
**Applied fix:** Both helpers now use `path.split('/').last` directly (split is never empty).

### IN-03: Intolerant `text` cast in `SetReply.fromReply`

**Files modified:** `app/lib/core/settings.dart` (commit `a0c9750`)
**Applied fix:** `note: reply['text'] is String ? reply['text'] as String : null` — poisoned replies never throw.

### IN-05: VRAM slider wrote per drag frame

**Files modified:** `app/lib/features/models/hf_browser.dart`, `app/test/models_browser_test.dart` (commit `a0c9750`)
**Applied fix:** Notifier write moved to `onChangeEnd`; `onChanged` is a required no-op in this Flutter version. Test updated to drive `onChangeEnd`.

### IN-06: `language` control lacked the ENGINE hint

**Files modified:** `app/lib/features/settings/settings_screen.dart` (commit `a0c9750`)
**Applied fix:** Added `l10n.confEngineHint` above the language segment in the «система» group.

## Skipped Issues

### IN-01: Mixed-language user-facing strings

**File:** `app/lib/core/downloads.dart:133-134`, `app/lib/features/models/hf_browser.dart:107-110`
**Reason (WONTFIX):** Fixing it properly requires new .arb keys plus regenerating l10n and rewriting the existing snackbar expectation (`models_browser_test.dart` asserts the exact `'загрузка не удалась: download failed: 404'` string). Also, adding `ГБ`/`МБ` literals in `lib/` would violate the Cyrillic gate unless routed through .arb. Not cheap/safe; the gate already passes and no user-visible contract is broken.

### IN-04: 416 Range Not Satisfiable has no recovery path

**File:** `app/lib/core/hf/download_engine.dart:87-89`
**Reason (WONTFIX):** Explicitly noted as parity with the frozen reference (`curl -C -` fails the same way) and "not demanded" in the review. Adding .part deletion/restart logic would change download semantics beyond a safe review fix; the review itself marks the fix optional.

---

_Fixed: 2026-11-22T17:43:40+03:00_
_Fixer: the agent (gsd-code-fixer)_
_Iteration: 1_

## Gate log (final, main checkout)

- `flutter test` → **All tests passed! (138)** — 131 baseline + 7 new regression tests
- `flutter analyze` → No issues found!
- `bash tool/cyrillic_gate.sh` → 0
- `git diff 9aee17e HEAD -- crates/ app/pubspec.yaml app/pubspec.lock` → empty (freeze intact)
- `implements DictaEngine/HfClient/DownloadEngine` in `test/` → 1/1/1
- No worktree created (`workflow.use_worktrees = false`); no recovery sentinel; all 9 fix commits advanced `main` directly.

> Note: all applied fixes are behavior/logic changes (branch conditions, state handling, parsing, ordering). Tier-1 re-read + Tier-2 `flutter analyze`/`flutter test` verification passed for each group, but human confirmation of the intended semantics before the phase advances to verifier is advisable.

---
phase: 04-nastroyki-brauzer-modeley
reviewed: 2026-11-22T18:40:00+03:00
depth: deep
files_reviewed: 39
files_reviewed_list:
  - app/lib/core/engine.dart
  - app/lib/core/settings.dart
  - app/lib/core/daemon_engine.dart
  - app/lib/core/providers.dart
  - app/lib/core/downloads.dart
  - app/lib/core/hf/hf_urls.dart
  - app/lib/core/hf/hf_parse.dart
  - app/lib/core/hf/hf_catalog.dart
  - app/lib/core/hf/hf_client.dart
  - app/lib/core/hf/download_engine.dart
  - app/lib/features/settings/settings_screen.dart
  - app/lib/features/models/model_store.dart
  - app/lib/features/models/models_screen.dart
  - app/lib/features/models/hf_browser.dart
  - app/lib/features/shell/shell.dart
  - app/lib/features/dashboard/calibration_card.dart
  - app/lib/features/dashboard/dashboard_screen.dart
  - app/lib/features/history/history_screen.dart
  - app/lib/l10n/app_ru.arb
  - app/lib/l10n/generated/app_localizations.dart
  - app/lib/l10n/generated/app_localizations_ru.dart
  - app/tool/status_smoke.dart
  - app/test/fake_dicta_engine.dart
  - app/test/fake_hf_client.dart
  - app/test/fake_download_engine.dart
  - app/test/calibration_test.dart
  - app/test/settings_model_test.dart
  - app/test/settings_screen_test.dart
  - app/test/models_screen_test.dart
  - app/test/models_browser_test.dart
  - app/test/model_delete_test.dart
  - app/test/download_state_test.dart
  - app/test/hf_urls_test.dart
  - app/test/hf_parse_test.dart
  - app/test/dashboard_state_test.dart
  - app/test/history_screen_test.dart
  - app/test/fixtures/hf_search.json
  - app/test/fixtures/hf_tree.json
findings:
  critical: 3
  warning: 6
  info: 6
  total: 15
  necessity: 0
status: issues_found
---

# Phase 4: Code Review Report

**Reviewed:** 2026-11-22 18:40 (+03:00)
**Depth:** deep (per-commit diffs `8fbfc91..HEAD`, cross-file tracing against frozen `crates/dictad/src/main.rs`, `crates/dicta-settings/src/hf.rs`, `catalog.rs`, `main.rs`)
**Files Reviewed:** 39 (all phase-4 commits d646e3a..229705f)
**Status:** issues_found

## Summary

Gates re-verified live during this review: battery **131/131 green**, `flutter analyze` "No issues found!", cyrillic gate 0, `git diff 9aee17e HEAD -- crates/ app/pubspec.yaml app/pubspec.lock` **empty** (protocol/manifest freeze intact), implements DictaEngine/HfClient/DownloadEngine == 1/1/1 in `test/`, zero curl/Process.run additions, `status_smoke.dart` prints `runtimeType` only.

However, the green battery masks **three production-breaking defects** whose common root cause is that the test fakes encode a set-reply shape the real daemon never produces. Verified against `crates/dictad/src/main.rs:739` (`text: Some(note)` — **unconditionally non-null on every successful set**): the settings screen and the models screen treat any non-null note as a failure/refusal, so the success path of the seam is **dead code in production**. Separately, four of the seventeen sliders commit unit-suffixed strings (`"1500 мс"`) that the daemon's `count()` parser always rejects — those keys can never be changed, and no test drags them. These are precisely the class of bug the phase-2 lessons (fake-mirrors-wrong-shape, seam reply truth) were meant to prevent: the fakes passed, the daemon disagrees.

Security posture is otherwise solid: no payload logging (set values, error bodies, download URLs with tokens all absent from logs and UI), deletion triple-guarded (active model + lexical parent-vs-destDir comparison, `File(path).parent` never resolves `..`), transport confined to `dart:io HttpClient` behind seams, no network in tests. One threat-model mitigation claim (T-4-01: "имя с '/' или '..' невозможно") is factually false — `filenameOf` accepts `..` as the last segment (WARNING WR-05).

Accepted deviations honored (not re-flagged): sync ops in `FakeDownloadEngine`, `renameSync` in `_finish`, extra l10n keys with descriptions, summary double-naming.

## Critical Issues

### CR-01: Set-success branch is dead in production — every successful set renders as a red error and the reply snapshot is never applied

**File:** `app/lib/features/settings/settings_screen.dart:103-113` (root evidence: `crates/dictad/src/main.rs:695-721,739`)
**Issue:**
The frozen daemon returns `Res { text: Some(note), error: None, ... }` on **every successful set** — the note is always a non-empty success string (`"{key} = {toml} — сохранено в {path}"` or `"... — уже так"`, main.rs:695-721), not just an engine-reload refusal. The screen's commit path:

```dart
final notice = reply.error ?? reply.note;
if (notice != null) {
  setState(() => _error = notice);          // <-- taken on EVERY real success
} else {
  setState(() { _error = null; _pending.clear(); _replySnapshot = reply.settings; });
}
```

makes the `else` branch unreachable against the real daemon. In production every successful set therefore: (а) displays the daemon's success note (`"min_level = 0.50 — сохранено в /home/…/config.toml"`) verbatim in **error color** in the liveRegion status line — a success presented as a failure; (б) never clears `_pending` and never stores `_replySnapshot = reply.settings`, so the must-have truth "после каждого set форма обновляется из reply.settings" is only satisfied via the local echo, and the settings shown for every *other* key stay stale from the initial fetch until some unrelated invalidate.

The widget tests don't catch this because the fake success shape is wrong: `app/test/settings_screen_test.dart:94-97` scripts success as `'text': null` — a shape the daemon never sends. `FakeDictaEngine.set` (fake_dicta_engine.dart:125-132) defaults to `fetchSettingsReply` which has no `text` key. Note the correctly-migrated `CalibrationCard._commitMarker` (calibration_card.dart:82-94) applies the snapshot on `error == null` and ignores the note — the settings screen deviates from its own canon.

**Fix:**
Always apply the snapshot when `error == null`, and show the note as a status (refusal notes included — the refusal snapshot carries the *saved* value, main.rs:731):

```dart
if (reply.error != null) {
  setState(() => _error = reply.error);
} else {
  setState(() {
    _error = reply.note;            // verbatim daemon note; style per taste (see below)
    _pending.clear();
    _replySnapshot = reply.settings;
  });
}
```

A refusal is still fully visible (its note contains the main.rs:752 refusal string verbatim); if a non-error styling for pure-success notes is wanted, prefix-match on `« — сохранено в »`/`« — уже так »` or compare `reply.settings` against the sent value. Then fix the fake/test shape: `setReplyMap` for success scenarios must include `'text': 'paste_delay_ms = 900 — сохранено в /path'` (or the fake default must mirror the daemon by synthesizing a success note), and the "form updates from the set snapshot with no refetch" test must assert both the red-line absence and the snapshot application with that shape.

### CR-02: Successful model selection skips the mandatory refetch — active-model marker stays stale

**File:** `app/lib/features/models/models_screen.dart:44-48`
**Issue:**
Same root cause as CR-01:

```dart
final reply = await ref.read(engineProvider).set('model', path);
if (reply.error != null || reply.note != null) return;   // note is ALWAYS non-null on success
ref.invalidate(settingsSnapshotProvider);
await ref.read(settingsSnapshotProvider.future);
```

In production a successful `set('model', …)` always carries a note, so the early return fires and the invalidate+refetch never runs. The daemon has switched and reloaded the model, but the models screen keeps marking the **old** model as active and the settings form keeps the old model until some unrelated action refreshes the provider — an indefinitely stale UI after a successful action, violating the key_link "по done/успеху invalidate+await future". The widget test (`models_screen_test.dart:53-86`) passes because the fake default reply has no `text` key; it also never asserts `fetchCount` growth after select.

**Fix:**
Branch on the reply snapshot instead of the note's nullity:

```dart
final reply = await ref.read(engineProvider).set('model', path);
if (reply.error != null) return;   // refusal/error: note already verbatim-shown by daemon semantics
ref.invalidate(settingsSnapshotProvider);
await ref.read(settingsSnapshotProvider.future);
```

(Refusal detection, if a distinct message is wanted: the refusal note is identifiable by the main.rs:752/757 refusal substrings, or simply surface `reply.note` in a snackbar while still refetching.) Add a test asserting `fake.fetchCount` grows after a confirmed select **and** scripting `setReplyMap` with a daemon-true success note (`'text': 'model = … — сохранено в …'`).

### CR-03: Four sliders send unit-suffixed values the daemon always rejects — 4 of 17 keys are unusable, and their readouts freeze during drag

**File:** `app/lib/features/settings/settings_screen.dart:298, 308, 318, 356` (with `:155-156`, `:47-48`)
**Issue:**
The generic slider commits `format(v)` and stores the same formatted string in `_pending` via `_preview`:

- `:298` `auto_stop_after_ms` → `'${v.toInt()} ${l10n.confMilliseconds}'` → e.g. `"1500 мс"`
- `:308` `auto_stop_grace_ms` → `"1200 мс"`
- `:318` `max_record_ms` → `"60 с"`
- `:356` `clipboard_restore_ms` → `"2500 мс"`

The daemon's parser is `raw.parse::<u64>()` (main.rs:522-529) — `"1500 мс"` fails with `auto_stop_after_ms: "1500 мс" — ожидается целое 200..60000`, so **every commit of these four keys fails** and the red error line shows. Additionally `_sliderValue` (`:47-48`) does `num.tryParse(_pending[key])`, which returns null for the suffixed string, so during a drag the readout below the slider (and the slider position after the gesture) snaps back to the stale snapshot value — the control appears frozen. No test drags any of these four sliders (`settings_screen_test.dart:163-169` only asserts the widgets exist; the drag tests cover `min_level`/`paste_delay_ms`, whose format happens to be the wire format).

**Fix:**
Separate the wire format from the display format. Commit and preview raw values; keep units only in labels/readouts:

```dart
_slider(
  …
  settingKey: 'auto_stop_after_ms',
  format: (v) => v.toInt().toString(),                    // wire format for commit+pending
  readoutSuffix: ' ${l10n.confMilliseconds}',             // display-only
)
```

(or minimal change: keep `format` for the readout but pass `settingKey`-specific `commit: (v) => v.toInt().toString()` and `_preview(settingKey, v.toInt().toString())`). Then add drag tests for at least one suffixed slider asserting `setCalls.single.value` matches `^\d+$` and the readout moved.

## Warnings

### WR-01: HF search pagination leaks offset into new searches and replaces instead of appending — both diverge from the reference

**File:** `app/lib/features/models/hf_browser.dart:29, 57, 313-315, 61-65`
**Issue:**
The reference (dicta-settings `main.rs:270-271, 324-328`) resets `offset = 0` for every non-append search and **appends** the next page (`append=true → offset = hf_repos.len()`, results extended). The port: `_offset += 20` on "показать ещё" (`:313-315`) but `_offset` is **never reset** when a new query/sort/filter search runs, so after one pagination a brand-new search starts at `offset=20/40` and silently skips the top of the results; and `_search()` does `_repos = parseSearch(body)` (`:61-65`), so "показать ещё" **replaces** page 1 with page 2 instead of appending. Tests only assert the URLs (`models_browser_test.dart:200-204`), not list composition or offset reset.
**Fix:** add an `append` flag to `_search()`: `onPressed find → _offset = 0`, `показать ещё → append=true`; on success `if (append) _repos = [..._repos, ...parseSearch(body)] else _repos = parseSearch(body)`.

### WR-02: One-at-a-time download guard has an await gap — two starts can run concurrently

**File:** `app/lib/core/downloads.dart:72-74`
**Issue:**
```dart
if (state is DownloadActive) return;                 // :72 guard
final settings = await ref.read(settingsSnapshotProvider.future);   // :74 async gap
```
`state` is only set to `DownloadActive` at `:90`, **after** the await. When the snapshot needs a real refetch (right after a completed download invalidated it, `:157`, or at app start), the guard window spans a full daemon round-trip: a second tap on another "скачать" button passes the guard, both proceed, `_subscription` (`:98`) is overwritten (first subscription leaks until dispose), and two engine streams can write the same `.part` concurrently if the dest matches — corrupted model bytes that would still be renamed to `dest`. `download_state_test.dart:172-187` only exercises the guard *after* state is Active, so the race is untested.
**Fix:** claim the slot synchronously before the first await — e.g. set `state = DownloadActive(file: destName, url: url, expectedBytes: expectedBytes, received: 0)` (or a `DownloadStarting` substate) immediately after the guard, and re-read the real `.part` offset later; or guard with a plain `bool _starting` reset in `_handleError/_finish`.

### WR-03: Cancel during connection establishment is silently dropped

**File:** `app/lib/core/hf/download_engine.dart:63-69, 81-86`; `app/lib/core/downloads.dart:163-166`
**Issue:**
`DownloadManager.cancel()` forwards to `engine.cancel()`, which does `_request?.abort(...)`. But `_request` is null until `await client.getUrl(url)` completes (`:81-82`) — a cancel issued during DNS/connect (which for a cold HF/Xet connection can be seconds) hits null and is **lost**; the download then proceeds to completion while the UI already reported the cancel intent. The `cancelMid` fake parks inside the byte stream, so the race is invisible to tests.
**Fix:** add a `bool _cancelled` field in `HttpDownloadEngine`: `cancel()` sets it and aborts `_request` if present; in `download()`, check `_cancelled` immediately after each await (after `getUrl`, after `request.close()`) and throw `DownloadCancelledException` / close the client without streaming. Reset the flag at the start of each `download()`.

### WR-04: Unguarded `settingsSnapshotProvider.future` awaits — unhandled async errors, and the reload timer never stops on persistent failure

**File:** `app/lib/features/settings/settings_screen.dart:69-75`; `app/lib/features/models/models_screen.dart:73, 90`; `app/lib/core/downloads.dart:157-158`
**Issue:**
All four sites `await ref.read(settingsSnapshotProvider.future)` without any error handling. `settingsSnapshotProvider` throws `StateError` on a reply error (providers.dart:49-51) or propagates a raw `SocketException` when the daemon is down. Consequences: (а) `models_screen` select/delete flows die silently mid-flow with an unhandled zone error (dialog closed, no feedback); (б) `DownloadManager._finish` completes the rename but its unawaited `_finish` future throws unhandled; (в) worst: `_startReloadPolling`'s `Timer.periodic` callback (`settings_screen.dart:69-75`) — if the daemon dies mid-reload, the await throws **before** the `fresh.reloading` check, `_stopReloadPolling()` is never reached, and the timer refetches-and-throws every second until the screen is disposed.
**Fix:** wrap each refetch in try/catch (show the honest transport hint on failure and, in the timer, stop polling on a transport error rather than looping forever), or catch in the timer specifically:
```dart
try {
  final fresh = await ref.read(settingsSnapshotProvider.future);
  ...
} catch (_) { _stopReloadPolling(); /* surface honest state */ }
```

### WR-05: T-4-01's claimed mitigation is false — `filenameOf`/`destName` accept `..`, and the direct link path then escapes dest_dir lexically

**File:** `app/lib/core/hf/hf_parse.dart:110-116`; `app/lib/features/models/hf_browser.dart:142-146`; `app/lib/core/downloads.dart:76-82`
**Issue:**
The threat register (plan T-4-01) states "имя с '/' или '..' в последнем сегменте невозможно (filenameOf — последний сегмент)". It is not true: `filenameOf('https://evil.example/..')` returns `'..'`, `hf_browser` marks the link valid (`directName != null`), and `DownloadManager.start` builds `destPath = '$destDir/..'` — which lexically resolves to the **parent of dest_dir**. `File(destPath).existsSync()` then returns true (it is a directory) and the manager reports `DownloadDone(alreadyExisted: true)` — a silent lie, no download, no error. The Rust reference shares the hole (hf.rs:155-165), so the port is faithful — but the threat model relies on this mitigation, and the plan's behavior list explicitly promised the impossibility. Nothing outside dest_dir is deletable (delete path is separately guarded), so impact is confined to a confusing silent no-op / rename-failure.
**Fix:** cheap defense-in-depth in `DownloadManager.start` (or in direct-link validation): reject `destName` in `{'.' , '..'}` and any name containing a path separator; add the plan-promised unit gate (`filenameOf('https://x/..')` → blocked, `'https://x/..%2F..bin'` → literal name, no traversal).

### WR-06: Models screen shows «моделей нет» when the settings fetch failed — dishonest state

**File:** `app/lib/features/models/models_screen.dart:22-23, 103-112`
**Issue:**
`ref.watch(settingsSnapshotProvider).value` is null both while loading **and** on `AsyncError`. The empty-state card (`:103-112`, `l10n.modelEmpty` «моделей нет — скачай из каталога ниже») therefore renders when the daemon is unreachable, telling the user their installed models don't exist. The settings form has the same degenerate fallback to `const SettingsReply()` (`settings_screen.dart:246`) but there the values are merely default-looking; here the state is actively false. The phase's own canon is honest error rows (dashboard diagnostics pattern).
**Fix:** switch on the provider's AsyncValue: `AsyncError` → an honest error row (reuse the diagnostics-card pattern/strings), `AsyncLoading` → the existing card or a spinner, only `AsyncData` with an empty `models` list → `modelEmpty`.

## Info

### IN-01: Mixed-language user-facing strings

**File:** `app/lib/core/downloads.dart:133-134`; `app/lib/features/models/hf_browser.dart:107-110, 138`
**Issue:** `DownloadFailed.message` is the English `'download failed: 404'` / `'download failed: FileSystemException'`, surfaced inside a Russian snackbar (`'загрузка не удалась: download failed: 404'`); `_sizeLabel` hardcodes Latin `'GB'`/`'MB'` while the rest of the phase uses `МБ` (`modelDownloadingMb`). The full-Russian-UI convention (cyrillic gate philosophy) is bent, though the gate itself passes.
**Fix:** map the failure to an l10n string with a numeric placeholder (`modelDownloadFailedHttp(code)`); either move size units to .arb or switch `_sizeLabel` to `ГБ`/`МБ` for consistency with `modelDownloadingMb`.

### IN-02: Dead branches in split-based helpers

**File:** `app/lib/core/hf/hf_parse.dart:105`; `app/lib/features/models/model_store.dart:12`
**Issue:** `parts.isEmpty ? path : parts.last` — `String.split` never returns an empty list, so the `isEmpty` arm is unreachable dead code in both `destName` and `modelName`.
**Fix:** drop the conditional; `path.split('/').last` directly (with a comment that split is never empty), or leave as-is consciously.

### IN-03: `SetReply.fromReply` casts `text` intolerantly

**File:** `app/lib/core/settings.dart:167`
**Issue:** `reply['text'] as String?` throws a `TypeError` if `text` is ever a non-string non-null (every other field of this module uses the tolerant `_string`-style check, and the library doc promises poisoned replies never throw). The frozen daemon always sends a string or null, so this is theoretical hardening only.
**Fix:** `note: reply['text'] is String ? reply['text'] as String : null` or a `_string(reply, 'text', '')`-style helper.

### IN-04: 416 (Range Not Satisfiable) is a dead end with no recovery path

**File:** `app/lib/core/hf/download_engine.dart:87-89`
**Issue:** A `.part` that is already exactly complete (rename previously failed) makes every retry re-send `Range: bytes=<len>-`, the server answers 416, and the manager permanently reports failure while preserving the unrenameable `.part`. The app offers no way out (no delete-.part action). Parity with the reference (`curl -C -` fails the same way), so noted, not demanded.
**Fix (optional):** on 416, treat the `.part` as complete — verify against `expectedBytes`/content-range and rename, or delete `.part` and restart from zero.

### IN-05: VRAM slider writes the notifier per drag frame, not on gesture end

**File:** `app/lib/features/models/hf_browser.dart:188-189`
**Issue:** The plan specified `onChangeEnd` for the VRAM slider; the implementation uses `onChanged` (per-frame notifier writes). Harmless — the value is UI-only and emits no daemon command (the `setCalls`-empty truth holds) — but it deviates from the stated plan decision and from the one-commit-per-gesture canon used everywhere else in this phase.
**Fix:** move to `onChangeEnd` for consistency.

### IN-06: `language` control lacks the engine hint marker

**File:** `app/lib/features/settings/settings_screen.dart:376-377 vs 453-464`
**Issue:** The plan explicitly required the confEngineHint marker for `language` too («⚠️ эталон кладёт его в "Система", пометка нужна и ему»). The confirm dialog is present (kEngineKeys), but the hint text («эти настройки перезагружают модель…») renders only inside the «движок» group, so the «система» group's language control looks non-ENGINE until the dialog surprises the user.
**Fix:** add the hint line (or a per-control marker) to the «система» group next to the language segment.

## Verification log (re-run during this review)

- `flutter test` → **All tests passed! (131)** — including the 71 inherited from phases 1-3.
- `flutter analyze` → No issues found!
- `bash tool/cyrillic_gate.sh` → 0
- `git diff --quiet 9aee17e HEAD -- crates/ app/pubspec.yaml app/pubspec.lock` → clean (freeze intact)
- implements DictaEngine/HfClient/DownloadEngine in `test/` → 1/1/1; `grep curl lib/` → 0; `Process.run` in `lib/` → 1 (pre-existing `id -u`, unchanged)
- `grep '\$error' tool/status_smoke.dart` → 0 (03-SECURITY debt paid)

The battery being green while CR-01..03 exist is the review's key caveat: **the fake set-reply shape (`text: null` on success) diverges from the daemon's always-`Some(note)` contract, and no test drags the four unit-suffixed sliders.** All three blockers are invisible to the current suite by construction.

---

_Reviewed: 2026-11-22T18:40+03:00_
_Reviewer: the agent (gsd-code-reviewer)_
_Depth: deep_

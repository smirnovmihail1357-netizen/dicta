---
phase: "3"
plan: "01"
title: "История диктовок — сводка executions"
subsystem: app
tags: [flutter, riverpod, history, in-memory, l10n, theme-extension, intl, tdd]
requires:
  - phase: "2"
    provides: "single FakeDictaEngine factory, stop/inject seams, dashboard widget stack, snackbar and tabular-figure conventions"
provides: [HIST-01, HIST-02, HIST-03, HIST-04]
affects: [phase-4, phase-5]
tech-stack:
  added: []
  patterns: [Riverpod Notifier store, ThemeExtension AppTextStyles.mutedOf, DateFormat.Hm, gen-l10n, TDD RED->GREEN]
key-files:
  created:
    - app/lib/features/history/history.dart
    - app/lib/features/history/history_screen.dart
    - app/lib/core/theme_ext.dart
    - app/test/history_store_test.dart
    - app/test/history_capture_test.dart
    - app/test/history_screen_test.dart
    - .planning/phases/03-istoriya-diktovok/phase-base.txt
  modified:
    - app/lib/features/dashboard/dicta_control.dart
    - app/lib/features/shell/shell.dart
    - app/lib/main.dart
    - app/lib/l10n/app_ru.arb + lib/l10n/generated/
    - app/lib/features/dashboard/{last_text,diagnostics,calibration}_card.dart
    - app/lib/features/dashboard/dashboard_screen.dart
    - app/test/fake_dicta_engine.dart
    - app/test/shell_test.dart
    - app/tool/diag_smoke.dart
    - README.md + README.ru.md
decisions:
  - "HIST-01 is implemented at the stop reply seam only (A4 correction): status never carries text; inject echoes never journal"
  - "Store stays in memory with ids derived from the head of the list; no persistence API and no capacity trimming"
  - "One muted text style via ThemeExtension, registered in main.dart (both themes) and in every test scaffold"
  - "Seven hist* l10n keys; copy/re-insert/snackbars reuse the existing dash* canon"
  - "Insertion from history goes only through dictaControlProvider.inject() under commandInFlight"
requirements-completed: [HIST-01, HIST-02, HIST-03, HIST-04]
actuals:
  tokens: 11995          # chars/4 over the realized diff (47983 chars)
  tasks: 4
  commits: 4             # 4 task commits measured base..HEAD at summary write; +1 metadata commit after
  plan_head_before: 305ed18c956afa5c33d5a7ad8c15414465988bba
  plan_head_after: 9ce9071cd21e4989e4180535484ef3894a7ce043
metrics:
  tasks: 4
  commits: 5             # 4 task commits + 1 metadata (docs) commit
  tests_total: 65
  tests_new: 14          # 4 store unit + 4 capture + 6 screen widget
  new_l10n_keys: 7
  duration: "15 min, 4 atomic task commits + 1 metadata commit"
status: complete
---

# Phase 3 Plan 01: История диктовок — Summary

**In-memory dictation journal fed only by the app's stop seam, with case-insensitive search, copy/re-insert/delete rows, one shared muted text style, and both MVP limits fixed in the English and Russian READMEs**

## Performance

- **Duration:** 15 min
- **Started:** 2026-11-22T12:24:32Z
- **Completed:** 2026-11-22T12:39:34Z
- **Tasks:** 4 of 4
- **Files modified:** 22 (864 insertions, 18 deletions)

## Tasks, commits and verify results

| Task | Requirement | Commit | Verify |
|------|-------------|--------|--------|
| 1 — Wave 0: store, ThemeExtension, hist* keys, diag_smoke debt | HIST-01/04 foundation | `de9e94c` | TDD RED (store missing) → GREEN; full battery 55/55; analyze clean; cyrillic 0; weak copies 0, `mutedOf` call sites 9, `historyTitle` 1 |
| 2 — Wave 1 TRACER: stop-seam capture | HIST-01 | `37c1f3b` | TDD RED 3/4 failing → GREEN 4/4; full 59/59; analyze clean; `historyProvider` referenced once, inside `stop()`; tracer feedback gate re-run green |
| 3 — Wave 2: history screen + actions | HIST-02, HIST-03 | `7a4fb90` | TDD RED 6/6 failing → GREEN 6/6; full 65/65; analyze clean; cyrillic 0; `mutedOf` in history 3 |
| 4 — final gate + README limits | HIST-04 | `9ce9071` | full battery 65/65; analyze; cyrillic; `implements DictaEngine` 1; contracts byte-identical; persistence 0; `status.text` 0; diag_smoke exit 0; 4 README anchors 1 |

Per-task `<verify>` commands ran before each commit. Task 2 was a tracer: its end-to-end verify was re-run after the commit (all green) before expansion to Task 3 — `⚡ Tracer verified end-to-end — expanding`. No checkpoints were needed; the phase is fully autonomous.

Plan metadata: `docs(03): execution summary` (separate metadata commit, not one of the four task commits).

## Full gate (Task 4)

- `cd app && flutter test` — **65/65 passed** (51 inherited + 4 store unit + 4 capture + 6 screen widget).
- `cd app && flutter analyze` — **No issues found!**
- `cd app && bash tool/cyrillic_gate.sh` — **0**.
- `cd app && grep -rn "implements DictaEngine" test/*.dart | wc -l` — **1** (single factory still).
- `git diff --quiet <phase-base> HEAD -- crates/ app/pubspec.yaml app/pubspec.lock` — **empty** (exit 0, both HEAD and working tree); phase base is `305ed18c956afa5c33d5a7ad8c15414465988bba`.
- `cd app && grep -rn "SharedPreferences\|sqflite\|writeAsString" lib --include='*.dart' | wc -l` — **0**.
- `cd app && grep -rn "status\.text" lib --include='*.dart' | wc -l` — **0**.
- `cd app && dart run tool/diag_smoke.dart` — `DIAG SMOKE state=idle connected=true` + `INJECT-REJECTED-AS-EXPECTED`, exit 0 (live daemon untouched).
- README anchors: `cleared on app restart` **1**, `never enter the journal` **1**, `очищаются при перезапуске` **1**, `глобальным хоткеем демона` **1**.

## Requirements delivered

- **HIST-01** — `history.dart` store (immutable entries, newest-first ids, local remove, `formatEntryTime`); `dicta_control.dart` journals the stop reply under `error == null && text != null && trim().isNotEmpty`. Inject echoes, failed stops and whitespace-only replies are proven not to journal.
- **HIST-02** — `HistoryScreen` on shell slot 1: `ListView.builder` with HH:mm tabular timestamps, case-insensitive substring search through `historyQueryProvider` (no debounce, empty query = all), honest empty states.
- **HIST-03** — copy via `clipboardWriterProvider` on explicit click; re-insert only via `dictaControlProvider.inject()` under `commandInFlight`; delete via local `remove(id)` with no daemon command.
- **HIST-04** — both READMEs state that the journal lives only in memory and that daemon global-hotkey dictations never enter it (A4), with the exact anchor phrases; no persistence API in `app/lib`.

## Deviations from plan

1. **[Rule 1 - Bug] Third test scaffold needed ThemeExtension registration.** Found during Task 1: after registering `AppTextStyles` in `main.dart` and `buildFakeApp` as specified, two `shell_test.dart` tests were red — that suite builds its own `MaterialApp` in `buildTestApp` (not mentioned in the plan's two registration points), so `mutedOf(context)` asserted. Fix: registered the same extension in that local test scaffold using the default `ThemeData().colorScheme` (test-only, 6 lines). Verified by the full battery returning to 55/55 before the Task 1 commit. Files: `app/test/shell_test.dart`; commit `de9e94c`. This was required by the plan's own `fails_when` ("any test red, including all 51 existing").
2. **Summary filename follows the parent instruction, not the plan's `<output>`.** The plan names `03-01-SUMMARY.md`; the orchestrating session explicitly required `03-SUMMARY.md` "как 02-SUMMARY.md по стилу". Wrote `03-SUMMARY.md` in the phase-2 style. No other .planning files were changed.

**Total deviations:** 1 auto-fixed (Rule 1, test-only) + 1 instruction-level output-path override. **Impact:** no production behavior changed by the auto-fix; both MVP limits and all contract gates are intact.

## Issues Encountered

- The first RED run of `history_capture_test.dart` failed to compile because `container` was a nullable local (`ProviderContainer?`) and was read without `!` in one assertion. Fixed the test-side nullability before confirming genuine RED (3 assertion failures); no production impact.
- No authentication gates, no package installs, no skipped tests, no unrun `<verify>` commands, no stubs introduced. The `placeholderScreen` key and `_SectionPlaceholder` class remain intentionally for phases 4–5 (slots 2–3), as the plan requires.
- Threat register T-3-01/02/03/SC mitigations hold: no payload printing (`diag_smoke` prints only `error.runtimeType`), clipboard writes only on explicit click, zero new dependencies, journal strictly in process memory.

## Self-Check: PASSED

All created files exist on disk; all four task commits (`de9e94c` → `9ce9071`) are ancestors of HEAD; `git status --short` shows no untracked files; `crates/` and `app/pubspec.{yaml,lock}` are byte-identical to the phase base; the four README anchors each occur exactly once.

---
*Phase: 03-istoriya-diktovok*
*Completed: 2026-11-22*

---
phase: "2"
plan: "01"
title: "Дашборд записи + диагностика окружения — сводка executions"
subsystem: app
tags: [flutter, riverpod, ipc, dashboard, diagnostics, l10n]
requires: []
provides: [DASH-01, DASH-02, DASH-03, DASH-04, DIAG-01]
affects: [phase-3]
tech-stack:
  added: []
  patterns: [Riverpod Notifier/FutureProvider, CustomPainter, jsonEncode seam, gen-l10n]
key-files:
  created:
    - app/lib/features/dashboard/dashboard_screen.dart
    - app/lib/features/dashboard/level_history.dart
    - app/lib/features/dashboard/dicta_control.dart
    - app/lib/features/dashboard/last_text_card.dart
    - app/lib/features/dashboard/calibration_card.dart
    - app/lib/features/dashboard/diagnostics_card.dart
    - app/tool/diag_smoke.dart
    - app/test/fake_dicta_engine.dart
    - app/test/{fake_dicta_engine,dashboard_state,level_history,dashboard_buttons,last_text,calibration,diagnostics}_test.dart
  modified:
    - app/lib/core/engine.dart
    - app/lib/core/daemon_engine.dart
    - app/lib/core/providers.dart
    - app/lib/features/shell/shell.dart
    - app/lib/l10n/app_ru.arb + lib/l10n/generated/
    - app/test/{shell,widget,daemon_engine_framing}_test.dart
    - .planning/phases/02-dashbord-zapisi-diagnostika-okruzheniya/02-VALIDATION.md
decisions:
  - "stop/inject return DictaStatus — status reply is the only text source; protocol unchanged"
  - "one FakeDictaEngine factory; all scenario hooks (incl. injectGate/setReply/statusCount) declared there"
  - "one set('min_level', {:.2}) per drag release; calibration never blocked by daemon state"
  - "model readiness is a settings-based proxy with an honest 'per daemon data' caveat"
metrics:
  tasks: 7
  commits: 8
  tests_total: 45
  tests_new: 32
  new_l10n_keys: 29
  duration: "single session, 7 atomic task commits + 1 gate docs commit"
status: complete
---

# Phase 2 Plan 01: Дашборд записи + диагностика окружения — Summary

The phase-1 dashboard placeholder became a live, honest control surface over the frozen
dictad socket: state/live level/mm:ss from the 100 ms poller, a button matrix that never
promises what the daemon cannot deliver, last text captured only from action replies,
one-command-per-gesture min_level calibration on a 200-sample histogram, and an
environment checklist built solely from status/settings. Zero protocol, dependency or
transport changes.

## Tasks, commits and verify results

| Task | Requirement | Commit | Verify |
|------|-------------|--------|--------|
| 1 — shared fake factory (Wave 0) | infra | `603d410` | 8 targeted tests green; `implements DictaEngine` count 1 (was 2) |
| 2 — tracer: stop/inject seam + live dashboard | DASH-01 | `0e0b193` | RED→GREEN: 15 focused tests; full suite 28, analyze clean, gate 0 |
| 3 — honest button matrix + spinner | DASH-02 | `c3e5c98` | 4 focused tests; full 32, analyze clean, gate 0 |
| 4 — last text, copy, re-inject | DASH-03 | `4a5d162` | 4 focused tests; full 36, analyze clean, gate 0 |
| 5 — histogram + min_level calibration | DASH-04 | `e5ef722` | 5 focused tests; full 41, analyze clean, gate 0 |
| 6 — diagnostics checklist + live smoke | DIAG-01 | `770403c` | 4 focused tests; full 45, analyze clean, gate 0; live smoke exit 0 |
| 7 — phase gate | all | `a46edd7` | full battery green (see below) |

Per-task `<verify>` commands were run before moving on. Task 2 was a tracer: after its
end-to-end verify the expansion proceeded (live mic visuals are a manual backstop).

## Full gate (Task 7, HEAD `a46edd7`)

- `cd app && flutter test` — **45/45 passed** (13 phase-1 + 32 new).
- `cd app && flutter analyze` — **No issues found!**
- `cd app && bash tool/cyrillic_gate.sh` — **0**.
- `cd app && dart run tool/diag_smoke.dart` — `DIAG SMOKE state=idle connected=true` +
  `INJECT-REJECTED-AS-EXPECTED`, exit 0 (empty-text inject rejected before insertion).
- `git diff --quiet 2714e27 HEAD -- crates/` — empty (protocol frozen).
- `git diff --quiet 2714e27 HEAD -- app/pubspec.yaml app/pubspec.lock` — empty (zero new deps).
- `grep -rn "implements DictaEngine" test/*.dart | wc -l` — **1**.
- No `flutter run -d linux` was started; no app process to kill. Live daemon untouched.

## Requirements delivered

- **DASH-01** — `dashboard_state_test.dart` drives the real `DictaStatusNotifier` with 100 ms
  ticks: three labels via `stateLabel`, `LinearProgressIndicator.value == status.level`,
  `65.3 s → "время 01:05"` from `base.seconds` (no local clock), unknown state → idle fallback.
  Level history pushes only on recording ticks and clears on re-entry.
- **DASH-02** — full 4-state matrix pinned; cancel disabled while transcribing; spinner bound
  to polled state (dies on next idle tick, no UX timeout); in-flight flag disables buttons;
  stop text captured, start clears it, cancel notice not shown as error.
- **DASH-03** — text only from stop/inject replies; honest no-text placeholder for hotkey
  dictations; copy via `clipboardWriterProvider` on explicit click; inject sends exactly the
  captured text via `buildInjectLine` (jsonEncode), 120 s engine budget; daemon errors verbatim.
- **DASH-04** — one `set('min_level','0.42')` on drag release (format `{:.2}`), zero during drag;
  marker initialized from `settingsSnapshotProvider`; legal during recording; auto_stop hint;
  standard `CustomPainter`/`LayoutBuilder`, no chart packages.
- **DIAG-01** — two automatic status rows + settings-based model row (green/neutral/red), two
  neutral manual hints that send no commands, start hint when disconnected; recheck = one poll
  + one settings refetch. Live smoke confirms daemon liveness without side effects.

## Deviations from plan

1. **Fake factory hooks extended (Rule 2, tests only).** `injectGate` (Task 4), `setReply`
   (Task 5) and `statusCount` (Task 6) were added to `app/test/fake_dicta_engine.dart` for
   in-flight/error/refetch assertions. Still exactly one `implements DictaEngine` in tests;
   the factory remains the single source, as the plan directs for test-specific helpers.
2. **Three extra l10n keys** beyond the planned ~25: `diagStatusNote` (honest note that the two
   status rows are one transport test), `diagModelNote` ("per daemon data" caveat), and
   `diagModelUnavailable` (red model row when disconnected/no model). Total 29 new keys.
3. **Successful `set` note not rendered.** The daemon's note travels in the reply's `text`
   field, but the frozen `DictaReply` seam discards `text`; expanding it was outside this
   plan. Errors are shown verbatim and the marker is re-derived from a post-set settings
   refetch, so no success is invented.
4. **Error presentation.** Last-text inject errors are rendered inline as data (persistent and
   verbatim) instead of a SnackBar; copy confirmation still uses a SnackBar.
5. **Provider refresh API.** Diagnostics uses `ref.invalidate(settingsSnapshotProvider)` and
   calibration consumes `ref.refresh(...)`'s returned snapshot to rebase the marker — both
   achieve the planned refetch without tripping the analyzer's `unused_result` lint.
6. **Test ergonomics.** Below-the-fold cards are `ensureVisible`-scrolled before tap/drag;
   calibration uses multi-step `dragFrom` so horizontal-drag recognition is reliable.
7. **Analyzer auto-fixes (Rule 1).** Removed an unused import, redundant null assertions and
   unused refresh results within the owning tasks; each task's analyze is clean.

No architectural changes, no authentication gates, no package installs, no skipped tests,
no unrun verifies, no stubs. Threat register T-2-01/02/03/T-2-SC mitigations are in place
(no payload logging, copy only on click, empty inject disabled, zero new dependencies).

## Manual UAT — [delegated to user]

These four backstop items stay open for `/gsd-verify-work` (visual/acoustic/real-window only):

1. **DASH-01/04 live level:** `cd app && flutter run -d linux`; speak/stay silent — bar and
   histogram bars live at ~10 Hz, silence vs speech visually distinct, threshold shades.
2. **DASH-04 live calibration:** record → silence → drag marker → release; threshold applies to
   the live stream and survives a daemon restart (saved in config.toml).
3. **DASH-03 inject:** button inserts into the active window (= the app itself when clicked);
   the cross-window path is the daemon hotkey; caption is honest.
4. **DIAG-01 manual rows:** verify hotkey and paste paths by hand (`dictactl toggle`, live
   dictation); the app never triggers either.

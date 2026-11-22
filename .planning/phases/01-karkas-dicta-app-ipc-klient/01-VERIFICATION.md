---
phase: 01-karkas-dicta-app-ipc-klient
verified: 2026-11-22T14:05:00Z
status: human_needed
score: 9/11 must-haves verified
covered_files:
  - .planning/phases/01-karkas-dicta-app-ipc-klient/01-01-PLAN.md
  - .planning/phases/01-karkas-dicta-app-ipc-klient/01-01-SUMMARY.md
  - app/.gitignore
  - app/analysis_options.yaml
  - app/l10n.yaml
  - app/lib/core/daemon_engine.dart
  - app/lib/core/engine.dart
  - app/lib/core/providers.dart
  - app/lib/core/settings.dart
  - app/lib/features/shell/shell.dart
  - app/lib/l10n/app_ru.arb
  - app/lib/l10n/generated/app_localizations.dart
  - app/lib/l10n/generated/app_localizations_ru.dart
  - app/lib/main.dart
  - app/pubspec.yaml
  - app/test/daemon_engine_framing_test.dart
  - app/test/fixtures/settings_reply.json
  - app/test/settings_model_test.dart
  - app/test/shell_test.dart
  - app/test/widget_test.dart
  - app/tool/cyrillic_gate.sh
  - app/tool/status_smoke.dart
covered_digest: "v4:sha256:3c74c2cceb72cc1b39dddc32e1ae072dcd9174396f61ded340b47a42f27b80c9"
behavior_unverified: 1
behavior_unverified_items:
  - truth: "With the app running: stopping dictad makes the «демон недоступен» banner appear, and restarting dictad restores live state within ~2 s WITHOUT restarting the app (auto-reconnect via the poll loop)"
    test: "Run the app (flutter run -d linux), then systemctl --user stop dictad, wait, then systemctl --user start dictad"
    expected: "Banner appears while the daemon is dead; live state returns within ~2 s of the daemon restarting, with no app restart"
    why_human: "The reconnect invariant spans a live daemon lifecycle and a real desktop window — the widget test only fakes a disconnected snapshot, and presence checks cannot see the timer actually re-establishing the socket"
human_verification:
  - test: "flutter run -d linux and click through the sidebar (Дашборд/История/Модели/Настройки); switch dark/light or the KDE accent"
    expected: "Window opens; sidebar switches between the 4 placeholder screens; Material You follows the KDE accent color or falls back to the fromSeed scheme; every rendered string is Russian"
    why_human: "Backstop truth (verification: backstop): window chrome, accent-color following and theme switching are visual — no headless check can confirm them"
  - test: "With the app running: systemctl --user stop dictad → watch the banner; systemctl --user start dictad → watch recovery"
    expected: "«демон недоступен» appears on daemon death; live state restores within ~2 s after daemon restart, WITHOUT restarting the app"
    why_human: "Backstop truth (verification: backstop): the reconnect path needs a live daemon kill/restart plus a human watching the window"
  - test: "Human review of the 6 flagged judgment-tier prohibitions from the PLAN (FlClash GPL separation, frozen daemon protocol, no global hotkeys, no dictated-text logging, jsonEncode-only, no client-side set validation)"
    expected: "No violation of any of the 6 must-NOT statements"
    why_human: "Judgment-tier prohibitions are non-authoritatively assessed by the LLM verifier below (all 6 look clean on code evidence) — a human sign-off is required per the escalation-gate contract; none can be silently passed"
---

# Phase 1: Каркас dicta-app + IPC-клиент Verification Report

**Phase Goal:** Приложение dicta-app собирается, запускается и живёт поверх существующего демона — архитектура «IPC, не FFI» зафиксирована до первой строки UI-кода
**Verified:** 2026-11-22T14:05:00Z
**Status:** human_needed
**Re-verification:** No — initial verification

**Method note:** every automated claim below was re-executed by the verifier this session against the live daemon (`/run/user/1000/dicta.sock` present and answering). SUMMARY.md was treated as claims, not evidence.

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence (produced this session) |
|---|-------|--------|----------------------------------|
| 1 | `flutter test` exits 0 (framing + golden + shell suites) and `flutter analyze` reports "No issues found!" | ✓ VERIFIED | Ran both: 13/13 "All tests passed!"; analyze → "No issues found! (ran in 3.3s)" |
| 2 | `dart run tool/status_smoke.dart` prints "SMOKE OK state=" (idle\|recording\|transcribing) against the live daemon | ✓ VERIFIED | Ran it: `SMOKE OK state=idle`, exit 0; socket `/run/user/1000/dicta.sock` live (srw, since 09:59) |
| 3 | Framing test pins APP-06: quotes/backslashes/newlines round-trip byte-exact; empty value emits valid `"value":""` | ✓ VERIFIED | `test/daemon_engine_framing_test.dart:9-23` asserts exact wire string `{"cmd":"set","key":"paste_keys","value":"он сказал \\"да\\n\\""}\n` + jsonDecode round-trip + empty-value case; passed in suite |
| 4 | Ordering needs no guarantee (fresh connection per command), stated in engine.dart library doc | ✓ VERIFIED | `lib/core/engine.dart:1-7` library doc: "Every command opens a fresh connection, so command ordering needs no guarantee and none is implemented"; `set()`/`status()` each call `_call` → `Socket.connect` fresh |
| 5 | Golden test decodes fixture field-by-field (17 editable keys + models/reloading/config_path); unknown keys tolerated; poisoned JSON → defaults, no throw | ✓ VERIFIED | `kEditableKeys` has exactly 17 entries (`settings.dart:10-28`); test asserts 20 named fields + set-equality vs fixture keys minus 3 + unknown-key + poisoned cases (4 test blocks, all passed) |
| 6 | shell_test pumps shell via FakeDictaEngine, imports no dart:io; 4 Russian labels render; connected=false shows «демон недоступен» | ✓ VERIFIED | `shell_test.dart` has zero dart:io/DaemonEngine references (grep); labels come from `l10n.dashboardTitle/historyTitle/modelsTitle/settingsTitle` (`shell.dart:55-70`); both tests passed in suite |
| 7 | Zero non-comment Cyrillic literals in app/lib Dart — Russian lives in app_ru.arb, gate script holds | ✓ VERIFIED | `bash tool/cyrillic_gate.sh` → prints `0`, exit 0; gate excludes committed generated l10n output, filters comments |
| 8 | Per-command timeouts: status 2 s, set/settings 30 s, stop/inject 120 s | ✓ VERIFIED | `daemon_engine.dart:50-53`: `_statusTimeout 2s`, `_settingsTimeout 30s`, `_injectTimeout 120s`; `set()`/`fetchSettings()` → 30 s, `stop()` → 120 s |
| 9 | `flutter run -d linux` opens the window: sidebar switches, Material You follows KDE accent / seed fallback, all strings Russian | ⚠ HUMAN NEEDED (backstop) | Code is present and wired (main.dart: windowManager + DynamicColorBuilder + l10n delegates; widget_test pumps the real DictaApp to «Дашборд»), but the visual truth is a declared `verification: backstop` — not confirmable headless |
| 10 | Kill dictad → «демон недоступен» banner; restart dictad → reconnect within ~2 s without app restart | ⚠ HUMAN NEEDED (backstop) | Reconnect machinery verified at source (Timer 100 ms, `_disposed` guard, backoff 200 ms→×2→2 s cap in `providers.dart:23-51`) and the disconnected→banner transition is widget-tested; the live kill/restart invariant is `verification: backstop` — needs a human at the desktop |

**Score:** 9/11 truths verified (2 backstop truths routed to human verification; 1 of them is also behavior-unverified in the runtime-invariant sense)

### Roadmap Success Criteria

| SC | Criterion | Status | Evidence |
|----|-----------|--------|---------|
| 1 | App runs on Linux, 4-destination sidebar, dark/light Material You switch | ✓ code-verified / ⚠ visual = human (truth 9) | Widget tests + source; visual confirmation pending |
| 2 | Live daemon data; «демон недоступен» on death; self-reconnect without app restart | ✓ code-verified / ⚠ live-restart = human (truth 10) | SMOKE OK against live socket; poll/backoff source; live kill/restart pending |
| 3 | Dart settings model deserializes fixture one-to-one (17 keys), golden green | ✓ VERIFIED | Truth 5 |
| 4 | All UI Russian via gen-l10n; sets go out as `set` with jsonEncode, no manual escaping | ✓ VERIFIED | Truths 3, 7 + `buildSetLine` = `jsonEncode({'cmd':'set',…})`; no `replaceAll`/manual escaping anywhere in lib (grep empty) |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|------------|---------------------|---------|
| APP-01 | 01-01-PLAN.md | App builds/runs on Linux, sidebar shell, Material You dark/light | ✓ SATISFIED (automated) / ⚠ visual backstop | Truths 1, 6, 9 |
| APP-02 | 01-01-PLAN.md | UDS connect + 100 ms status poll; daemon-death banner; reconnect without restart | ✓ SATISFIED (automated) / ⚠ live-restart backstop | Truths 2, 8, 10; providers.dart poll loop |
| APP-03 | 01-01-PLAN.md | DictaEngine seam — UI transport-agnostic, Windows swap = one file | ✓ SATISFIED | engine.dart imports no dart:io; zero dart:io under lib/features (grep empty); only transport file is daemon_engine.dart; shell/widget tests use FakeDictaEngine |
| APP-04 | 01-01-PLAN.md | Dart settings model 1:1 with daemon reply, golden fixture test | ✓ SATISFIED | Truth 5 |
| APP-05 | 01-01-PLAN.md | Russian UI via gen-l10n, no hardcoded strings in widgets | ✓ SATISFIED | Truth 7; all 4 sidebar labels via AppLocalizations (shell.dart:55-70) |
| APP-06 | 01-01-PLAN.md | Sets only via `set` command, jsonEncode, no manual escaping | ✓ SATISFIED | Truths 3, 4; engine doc: "validation stays in the daemon" |

Orphaned requirements: none — PLAN declares exactly APP-01..06, matching REQUIREMENTS.md Phase-1 rows.

### Required Artifacts

| Artifact | Status | Details |
|----------|--------|---------|
| app/ standalone Flutter 3.50 project (linux+windows platforms) | ✓ VERIFIED | linux/ + windows/ dirs present; not a cargo member (Cargo.toml untouched per git) |
| app/lib/core/engine.dart — DictaEngine + models + pure builders | ✓ VERIFIED | Abstract interface, no dart:io/Flutter imports, buildStatusLine/buildSetLine via jsonEncode |
| app/lib/core/daemon_engine.dart — AF_UNIX engine, resolveSocketPath | ✓ VERIFIED | DICTA_SOCK → XDG_RUNTIME_DIR → /run/user/uid fallback; fresh connection per `_call` |
| app/lib/core/providers.dart — engine + status providers, 100 ms notifier | ✓ VERIFIED | Timer.periodic 100 ms, ref.onDispose, `_inFlight` + `_disposed` guards, 200 ms→2 s capped backoff |
| app/lib/core/settings.dart — SettingsReply, kEditableKeys, kEngineKeys | ✓ VERIFIED | 17 editable keys, 6 engine keys, tolerant fromJson |
| app/lib/features/shell/shell.dart — NavigationRail shell + banner | ✓ VERIFIED | 4 destinations, placeholders, daemon-unavailable banner; no transport imports |
| app/lib/main.dart — ProviderScope + DynamicColorBuilder + MaterialApp + windowManager | ✓ VERIFIED | All present; windowManager via legacy.dart bridge (deviation #5, analyze-clean) |
| app/l10n.yaml + app_ru.arb — ru-first gen-l10n | ✓ VERIFIED | Template app_ru.arb; generated output committed as real files under lib/l10n/generated (W1 fix confirmed) |
| app/test/fixtures/settings_reply.json — captured live reply | ✓ VERIFIED | Single line, 20 settings fields, `text:null` |
| app/test/{settings_model,daemon_engine_framing,shell_test}.dart | ✓ VERIFIED | All exist, substantive, all 13 tests pass |
| app/tool/status_smoke.dart — live smoke | ✓ VERIFIED | Ran it: SMOKE OK state=idle |
| app/tool/cyrillic_gate.sh (review fix 675269c) | ✓ VERIFIED | Ran it: prints 0, exit 0 |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|----|--------|---------|
| features/shell/shell.dart | core/daemon_engine.dart | engineProvider/dictaStatusProvider (Riverpod) | ✓ WIRED | Shell reads only providers; zero dart:io under lib/features |
| core/providers.dart | core/daemon_engine.dart | 100 ms poll → fresh UDS per tick | ✓ WIRED | Proven live: SMOKE OK against the running daemon |
| l10n/app_ru.arb | features/shell/shell.dart | gen-l10n → AppLocalizations | ✓ WIRED | shell.dart:55-70 uses all four l10n labels + daemonUnavailable |
| core/settings.dart | test/fixtures/settings_reply.json | golden test pins the 4th Settings copy | ✓ WIRED | 20-field assertion suite passes |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
|---------|--------------|--------|-------------------|--------|
| tool/status_smoke.dart → DaemonEngine.status() | DictaStatus | live UDS `/run/user/1000/dicta.sock` | Yes — observed `SMOKE OK state=idle` | ✓ FLOWING |
| shell.dart dashboard placeholder | dictaStatusProvider | providers poll loop → DaemonEngine | Yes via provider chain (widget smoke pumps real DictaApp to «Дашборд»); full window chain = backstop | ✓ FLOWING |
| settings_model_test | SettingsReply | committed captured fixture | Yes — 20 fields asserted | ✓ FLOWING |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| Full test suite | `cd app && flutter test` | 13/13, "All tests passed!" | ✓ PASS |
| Static analysis | `cd app && flutter analyze` | "No issues found! (ran in 3.3s)" | ✓ PASS |
| Live daemon smoke | `cd app && dart run tool/status_smoke.dart` | `SMOKE OK state=idle`, exit 0 | ✓ PASS |
| Cyrillic gate | `bash tool/cyrillic_gate.sh` | `0`, exit 0 | ✓ PASS |

### Probe Execution

| Probe | Command | Result | Status |
|-------|---------|--------|--------|
| app/tool/status_smoke.dart (live daemon) | `dart run tool/status_smoke.dart` | SMOKE OK state=idle | PASS |
| app/tool/cyrillic_gate.sh | `bash tool/cyrillic_gate.sh` | 0 | PASS |

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| (none) | — | Zero TBD/FIXME/XXX/TODO/HACK markers in app/ Dart sources | — | — |
| lib/features/shell/shell.dart | 1, 79 | "placeholder" word usage | ℹ️ Info | Intentional: placeholder screens ARE the phase-1 deliverable; the `placeholderScreen` l10n key is by design |
| lib/core/daemon_engine.dart | 24 | Russian strings as `\uXXXX` escapes | ℹ️ Info | Review I-5, accepted: gate counts Cyrillic in lib; runtime exception strings are not UI strings |
| .gitignore (repo root) | — | Phase commit touched one file outside app/ (`.planning/research/.cache/` ignore line) | ℹ️ Info | Not a protocol/daemon change; prohibition intact |

### Prohibition Review (judgment-tier — non-authoritative LLM verdict, human sign-off required)

| # | Prohibition | LLM verdict | Evidence |
|---|-------------|-------------|----------|
| 1 | FlClash source never opened/read/copied | clean | Zero FlClash-named files/identifiers in app/ (grep); code style matches plan conventions; mentions exist only in research/pattern docs |
| 2 | Daemon protocol NOT modified (Rust frozen) | clean | `git diff --name-only f00426a..675269c` — no crates/, no Makefile, no Cargo.toml; only app/ + .planning/ + one .gitignore line |
| 3 | No global hotkey registration | clean | grep hotkey/registerHotKey/ShortcutManager in lib/ → empty |
| 4 | Dictated text not logged/persisted | clean | daemon_engine.dart:64 "Error type and socket path only - never log reply payloads"; no print/debugPrint of payload in lib |
| 5 | jsonEncode only, never manual escaping | clean | Builders use jsonEncode; grep replaceAll/manual-escape in lib → empty |
| 6 | No client-side set validation | clean | engine.dart doc + `set()` sends buildSetLine verbatim, surfaces reply error |

All six remain `flagged: true` per the escalation-gate contract — recorded for the end-of-phase human checkpoint, not silently passed.

### Review Disposition Cross-Check

Fix commit `675269c` verified as an ancestor of HEAD and effective: W1 (generated l10n committed as real files, no symlink — `git ls-files app/lib/l10n/generated` shows both files), W2/W3/I-1/I-2 spot-checked in source (SocketException branch, `_disposed` guard after awaits, `engine.dispose` via `ref.onDispose`, real DictaApp widget smoke). Disposition table is consistent with the code.

### Human Verification Required

**1. Window + sidebar + themes (backstop truth 9)**
**Test:** `cd app && flutter run -d linux`; click through Дашборд/История/Модели/Настройки; toggle dark/light or change the KDE accent.
**Expected:** Window opens (min ~800×600, centered, Russian title); sidebar switches between the 4 placeholder screens; Material You follows the KDE accent or falls back to the fromSeed scheme; every visible string is Russian; dashboard shows the live state (idle).
**Why human:** Backstop truth — window chrome, accent following and theme switching are visual; no headless test can confirm them.

**2. Daemon kill/restart reconnect (backstop truth 10)**
**Test:** With the app running: `systemctl --user stop dictad` → watch; then `systemctl --user start dictad` → watch.
**Expected:** «демон недоступен» banner appears while the daemon is dead; live state returns within ~2 s of the daemon restarting; the app is NOT restarted.
**Why human:** Backstop truth — the reconnect invariant spans a live daemon lifecycle plus a real window; the widget test only fakes a disconnected snapshot.

**3. Prohibition sign-off (6 judgment-tier must-NOTs)**
**Test:** Review the table above; confirm no FlClash copying, protocol freeze, no hotkeys, no text logging, jsonEncode-only, no client-side validation.
**Expected:** No violation found (LLM verdict is clean on all six, but the sign-off is yours per the escalation gate).
**Why human:** Judgment-tier prohibitions cannot be authoritatively machine-verified.

### Gaps Summary

No automated gaps. All 9 non-backstop truths verified with evidence produced this session (13/13 tests, clean analyze, live SMOKE OK, gate 0, source constants confirmed). The phase goal is achieved at the code level: the app builds, the IPC-not-FFI architecture is fixed (engine seam proven transport-free), and the full chain to the live daemon works.

Status is `human_needed` because the two `verification: backstop` truths (visual window/theme check; live daemon kill/restart reconnect) plus the six flagged judgment-tier prohibitions require a human at a real desktop session — they were declared as backstop at plan time and remain unobserved. These items belong in the phase UAT.

---

_Verified: 2026-11-22T14:05:00Z_
_Verifier: the agent (gsd-verifier)_

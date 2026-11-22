---
phase: "1"
slug: "karkas-dicta-app-ipc-klient"
# status lifecycle: draft (seeded by plan-phase) → validated (set by validate-phase §6)
# audit-milestone §5.5 distinguishes NOT-VALIDATED (draft) from PARTIAL (validated + nyquist_compliant: false) (#2117)
status: draft
nyquist_compliant: false
wave_0_complete: false
created: "2026-11-22"
---

# Phase 1 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | `flutter_test` (bundled with Flutter SDK; package:test semantics for pure Dart files) |
| **Config file** | none — `flutter create` scaffold provides `test/`; Wave 0 creates the project |
| **Quick run command** | `cd app && flutter test` |
| **Full suite command** | `cd app && flutter test && flutter analyze` |
| **Estimated runtime** | ~30 seconds (after first build; excludes toolchain warm-up) |

---

## Sampling Rate

- **After every task commit:** Run `cd app && flutter test`
- **After every plan wave:** Run `cd app && flutter test && flutter analyze`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 60 seconds (first run after `flutter create` pays toolchain warm-up once)

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 1 | 01 | 0 | APP-01..06 | — | N/A (no secrets, local socket only) | toolchain | `cd app && flutter --version` | ❌ W0 | ⬜ pending |
| 2 | 01 | 0 | APP-01 | — | N/A | project scaffold | `cd app && flutter analyze` | ❌ W0 | ⬜ pending |
| 4 | 01 | 1 | APP-04 | — | N/A | unit (golden fixture) | `cd app && flutter test test/settings_model_test.dart` | ❌ W0 | ⬜ pending |
| 3 | 01 | 1 | APP-02, APP-06 | T-1-01 / — | UDS client only connects to `$XDG_RUNTIME_DIR/dicta.sock`; never logs payload contents | unit (framing + jsonEncode regression) | `cd app && flutter test test/daemon_engine_framing_test.dart` | ❌ W0 | ⬜ pending |
| 5 | 01 | 2 | APP-03 | — | N/A | widget test (FakeDictaEngine via provider override) | `cd app && flutter test test/shell_test.dart` | ❌ W0 | ⬜ pending |
| 5 | 01 | 3 | APP-01, APP-02, APP-05 | — | N/A | manual smoke | see Manual-Only Verifications | n/a | n/a | ⬜ pending |

*Task IDs refer to tasks 1–5 of plan 01-01-PLAN.md: Task 1 = SDK install checkpoint, Task 2 = scaffold, Task 3 = tracer/framing+engine, Task 4 = settings golden, Task 5 = shell + manual smokes.*

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] Flutter SDK 3.50.x installed and on PATH (`checkpoint:human-verify` — fvm 4.3.2 from cachyos repos or AUR `flutter-bin` via paru; nothing else runs without it)
- [ ] `app/` project created via `flutter create --platforms=linux,windows`, deps added (flutter_riverpod 3.5.0, dynamic_color 2.1.0, window_manager 0.7.0), `l10n.yaml` + first `app_ru.arb`
- [ ] `app/test/fixtures/settings_reply.json` — paste the fixture captured live this session (see 01-RESEARCH.md Code Examples)
- [ ] `app/test/settings_model_test.dart` — stubs for APP-04
- [ ] `app/test/daemon_engine_framing_test.dart` — stubs for APP-02/APP-06
- [ ] `app/test/shell_test.dart` — stubs for APP-03

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Shell renders + sidebar navigation | APP-01 | visual window behavior | `cd app && flutter run -d linux` — window opens, sidebar switches between 4 placeholder screens |
| Material You follows KDE accent; light/dark switch | APP-01 | compositor/theme integration | toggle dark/light (or system theme switch) — colors follow KDE accent or graceful `ColorScheme.fromSeed` fallback |
| Kill daemon → «демон недоступен»; restart → auto-reconnect, no app restart | APP-02 | requires killing a live system service | with app running: `systemctl --user stop dictad` → message appears; `systemctl --user start dictad` → state returns within ~2 s |
| Russian strings everywhere, l10n only | APP-05 | rendered-string check | inspect running UI; grep widgets for hardcoded Cyrillic outside l10n |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 60s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending

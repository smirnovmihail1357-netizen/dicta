---
phase: 01-karkas-dicta-app-ipc-klient
date: 2026-11-22
asvs_level: 1
threats_total: 6
threats_mitigated: 5
threats_accepted: 1
threats_open: 0
---

# Phase 01 Security Audit — Каркас dicta-app + IPC-клиент

Scope: `app/` only (commits 8139b87..675269c). All Rust/daemon code frozen, out of scope.
ASVS L1 verification (mitigation present in cited file), with L2 boundary checks where
multiple code paths could bypass the declared mitigation.

## Mitigation Table

| Threat | Severity | Disposition | Status | Evidence |
|--------|----------|-------------|--------|----------|
| T-1-01 payload injection via set values (Tampering) | medium | mitigate | CLOSED | `app/lib/core/engine.dart:90-91` — `buildSetLine` = `jsonEncode({'cmd':'set','key':key,'value':value})` only; grep across `app/` finds zero replace/concat escaping (`replaceAll`, `"value"` concat: no lib hits). All wire builders (`engine.dart:88-91`, `daemon_engine.dart:101,131`) are jsonEncode. Regression test `app/test/daemon_engine_framing_test.dart:9-18` — quotes/backslashes/newlines round-trip byte-exact; empty-value case `:20-24` |
| T-1-02 dictated text leakage (Info disclosure) | medium | mitigate | CLOSED | grep `print(|debugPrint|log(|logger|stdout` over `app/lib/` → single hit: `daemon_engine.dart:20` (`Process.runSync('id')` uid resolution, not reply payload). Error paths (`daemon_engine.dart:63-67,89-96`) log error type + socket path only, comment at `:64` pins "never log reply payloads". `status.text` is held in the model (`engine.dart:46,58`) but rendered/logged nowhere (`shell.dart` renders state/level/seconds only). Smoke tool `app/tool/status_smoke.dart:10` prints state name only |
| T-1-03 env var trust DICTA_SOCK (Tampering) | low | accept | CLOSED (accepted risk) | Override at `daemon_engine.dart:16-17`, commented as "test override" (`:11`). Acceptance rationale documented in PLAN threat register row T-1-03: user-session-local env, no remote reachability. See residual risk below |
| T-1-04 fixture privacy (Info disclosure) | low | mitigate | CLOSED | `app/test/fixtures/settings_reply.json` — `"text":null`, content is only settings defaults + user-local model paths (`~/.local/share/whisper-cpp/*.bin`) + `config_path`. No dictated text, no secrets, no credentials. Committed knowingly as the golden pin (PLAN Task 2 safety check) |
| T-1-05 malformed reply parsing (Tampering) | medium | mitigate | CLOSED | `decodeReplyMap` (`engine.dart:82-86`) throws typed FormatException on non-map; `DictaStatus.fromReplyLine` (`engine.dart:33-52`) catches all → `DictaStatus.disconnected()`, per-field typed defaults. Poison tests: `daemon_engine_framing_test.dart:41-45` (garbage/partial lines) and `settings_model_test.dart:56-67` (poisoned settings: garbage string, wrong types, null, empty map → defaults, no throw). Typed `SettingsReply.fromJson` (`settings.dart:90-111`) — never raw maps into widgets |
| T-1-SC package legitimacy (Tampering) | high | mitigate | CLOSED | `app/pubspec.yaml:33-38` pins exactly the audited list: flutter_riverpod 3.5.0, dynamic_color 2.1.0, window_manager 0.7.0, intl ^0.20.3 + flutter_localizations (SDK) — all five sanctioned by 01-RESEARCH.md Package Legitimacy Audit (riverpod/material.io/leanflutter, all "Approved", no [SLOP]/[SUS]). Dev-only `flutter_lints ^6.0.0` is the official `flutter create` scaffold lint set, not a runtime dependency. `import 'package:window_manager/legacy.dart'` (`main.dart:6`) is a library inside the pinned window_manager 0.7.0, not a new package. No package beyond the audited list |

## Residual Risks

- **T-1-03 (accepted):** a hostile user-session environment can set `DICTA_SOCK` and redirect the
  client to a fake socket (a same-user local impersonation of dictad). The threat model accepts
  this: whoever controls the session env already owns the session (single-user desktop app, the
  real socket is 0o600 same-user). Accepted per PLAN register; do not re-litigate without a
  multi-user threat appearing.
- **T-1-04 (informational):** fixture contains the user's real home paths (`/home/wardreg/...`).
  Non-secret, reviewer-accepted for shape-pinning; if the repo ever goes public, consider
  path anonymization in a later phase.
- **L1 note (informational):** `DictaReply.error` surfaces the daemon's error string to the UI.
  The daemon is inside the trust boundary and its error field is validation output, not dictated
  text — consistent with T-1-02's boundary. Re-check if Phase 3 ever feeds user text into error strings.

## Verdict

All 6 declared threats verified present in implemented code (5 mitigated, 1 documented accepted).
No unregistered threat flags (SUMMARY.md has no `## Threat Flags` section; no new attack surface
found in scope). `threats_open: 0`.

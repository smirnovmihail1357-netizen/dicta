# Phase 1 Context: Каркас dicta-app + IPC-клиент

**Status:** Complete
**Inputs:** REQUIREMENTS.md (APP-01..APP-06), research/ (STACK.md, SUMMARY.md, FEATURES.md, PITFALLS.md), ROADMAP.md Phase 1, PROJECT.md

## User Decisions (from milestone setup, carried into this phase)

1. **Stack:** Flutter (Material You, like FlClash) — NOT an egui redesign. Flutter 3.50 / Dart 3.14.
2. **Everything in one window eventually** (dashboard, history, settings+models, tray+hotkeys) — Phase 1 builds only the shell + IPC foundation.
3. **IPC, not FFI:** the app talks to the existing dictad daemon over its frozen
   line-delimited JSON protocol (unix socket `/run/user/1000/dicta.sock`). No protocol
   changes, no FRB, no embedding whisper into the app.
4. **FlClash is pattern-inspiration only** (GPL-3.0): copy architectural ideas, never code.

## Phase Scope (APP-01..APP-06)

- **APP-01 Shell:** `app/` Flutter project (repo root, NOT a cargo workspace member).
  Sidebar navigation: Дашборд / История / Модели / Настройки — placeholder screens for
  phases 2–5. Dark + light theme, Material You via `dynamic_color`. Window chrome via
  `window_manager`. l10n ru-first (`gen-l10n`), Russian UI strings.
- **APP-02 IPC client:** poll `status` every ~100 ms; on socket failure show
  «демон недоступен» and auto-reconnect when the daemon returns (no app restart).
  One command — one line JSON, same framing as dictactl/dicta-settings.
- **APP-03 DictaEngine seam:** all UI talks to an abstract `DictaEngine` interface
  (Dart), never to sockets directly. The Windows bridge (Phase 6) plugs in behind the
  same seam. Linux implementation = UDS client. Default `dart:io` AF_UNIX; the
  dart_ipc-vs-dart:io mini-spike only runs if plain dart:io proves insufficient.
- **APP-04 Settings golden test:** Dart model of the daemon `settings` reply,
  deserializing a captured real fixture one-to-one (17 keys). Golden test green.
- **APP-05 l10n:** ru default, all UI strings through gen-l10n.
- **APP-06 set discipline:** `set` payloads built with `jsonEncode`, no manual escaping.

## Constraints & Non-Goals

- Daemon `dictad`, protocol, overlay `dicta-ui`, `dicta-settings`, Rust workspace:
  untouched. No Makefile changes to existing targets (adding new install targets for
  the app is allowed if needed for later phases, keep minimal in Phase 1).
- No tray, no global hotkeys, no Windows code (phases 5–7).
- No history capture yet (phase 3) — but the engine seam should expose state
  transitions so passive history can hook in later.
- Flutter app must run from source on cachyos (Linux desktop) — `flutter run -d linux`.

## Known Facts to Carry Into the Plan

- Daemon reply format: `{"base":{"state","level","seconds","text","error"},"panel":bool,"settings":{...}?}`.
- Socket path from dicta-core `Config::load` — reuse the same resolution logic in Dart
  (XDG_RUNTIME_DIR), fallback `/run/user/1000/dicta.sock`.
- Daemon states: Idle/Recording/Transcribing (+error field). `set` validates on the
  daemon side; rejected values come back in the reply — surface, don't pre-validate.
- ENGINE-affecting keys (model, use_gpu, gpu_device, n_threads, beam_size) trigger a
  model reload on the daemon; the reload is announced via `reloading` in settings.
- Conventions: Rust repo uses Russian UI strings and honest, measured comments;
  Flutter side follows suit (ru-first l10n).

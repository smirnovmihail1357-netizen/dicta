# Phase 1: dicta-app shell + IPC client — Pattern Map

**Mapped:** 2026-11-22
**Files analyzed:** 8 (engine, settings model, golden test, shell screens, bootstrap, pubspec, l10n, widget test)
**Analogs found:** 6 / 8 (all analog paths git-tracked, verified via `git ls-files`)

## File Classification

| New file | Analog (path) | Pattern to carry | Deviations for Dart/Flutter |
|---|---|---|---|
| `app/lib/core/engine.dart` (DictaEngine seam + UDS client) | `crates/dicta-settings/src/net.rs` | `Client{writer,reader}` (:11-14); `connect()` maps connect failure to a Russian error with a fix hint (:18-29 «демон не отвечает … — запусти systemctl…»); `call()` = write line + flush + read one `\n`-terminated reply, EOF ⇒ error (:32-40); one command = one line; no client-side validation of set values (:1-4 doc comment). | Dart `Socket.connect(InternetAddress.fromUnixDomain(path))` gives one full-duplex socket — no `try_clone`, no separate BufReader; read via `socket.map(...)`/`readLine`-style line splitting. Read timeout: no per-read timeout, wrap the reply future with `.timeout(...)`. **Protocol fact:** dictad closes the connection after answering `status` (`crates/dictad/src/main.rs:1007-1009`) — poll `status` every 100 ms with a fresh connection (dicta-ui pattern, `CONCERNS.md` «Каждый опрос статуса — новое соединение» documents this as accepted), reconnect on error with backoff, surface «демон недоступен» instead of crashing. Engine is an abstract interface (`DictaEngine`); UDS impl is one class behind it — Windows bridge plugs in later (Phase 6). |
| `app/lib/core/settings.dart` (Dart Settings model) | `crates/dicta-settings/src/state.rs` | `Settings` snapshot with exactly the daemon's 17 fields (:10-31) and per-field defaults (:33-58); `from_reply` (:62-73): missing/broken `settings` ⇒ defaults, error extracted from `base.error` — poisoned JSON never throws. Deliberate schema duplication (client mirrors daemon, not shared type — `ARCHITECTURE.md` «Settings-снимок (3 копии)»). | Hand-rolled `Settings.fromJson(Map<String, dynamic>)` — every field read with `json[key] as T? ?? default`; `fromReply` returns `(Settings, String?)` record (or small result class). No `#[serde(default)]` — the `??` chain is the Dart equivalent. Keep field names snake_case to match the wire JSON exactly; no codegen needed at 17 fields. |
| `app/test/settings_golden_test.dart` (fixture golden test) | `crates/dicta-settings/src/state.rs:75-127` (`mod tests`) | Three test shapes to mirror: full reply with settings + error (`from_reply_reads_settings_and_error` :81-101), reply without `settings` ⇒ defaults (`reply_without_settings_gives_defaults` :103-112), poisoned settings don't panic (:114-127 — wrong type / null / garbage all ⇒ defaults). Fixture = captured real daemon reply. | Dart: golden test package or plain `expect(decodedJson, expectedMap)`; test file co-located under `app/test/` mirroring the Rust "tests in same file" spirit. The captured fixture is the one true source for the 17-key contract — this test is the mitigation for adding a 4th copy of Settings. |
| `app/lib/shell/` (sidebar + screens: Дашборд/История/Модели/Настройки) | `crates/dicta-settings/src/main.rs` | `Tab` enum with Russian titles (:25-42, `Tab::title() -> "Запись"/"Модель"/…`); App state carries `error: Option<String>` + `flash_info: Option<String>` (:102-104) — bottom line renders error red else info green else weak config hint (:905-912: rgb(255,120,120) error, rgb(120,220,120) info, `ui.weak` secondary). Sliders send on release, not per-frame (:159-177) — carry into Phase-2+ settings widgets. | Flutter: `NavigationRail`/`Drawer` instead of egui tabs; error/flash via a persistent `SnackBar`/banner zone at screen bottom; weak text = `Theme.of(context).colorScheme.onSurfaceVariant`. Titles via l10n (`AppLocalizations.dashboard` etc.), not literals. Placeholder screens show weak secondary text («раздел появится в следующих фазах»). |
| `app/lib/main.dart` (bootstrap: window, theme, socket resolution, engine wiring) | `crates/dictactl/src/main.rs:111-152` + `crates/dicta-core/src/config.rs:253-259` | `ask()`/`ask_json()`: one command — one connection, connect failure ⇒ Russian error naming the socket path and the fix (:113-118). Socket path resolution: `XDG_RUNTIME_DIR/dicta.sock`, empty/missing ⇒ `dirs::runtime_dir()` ⇒ `/tmp` fallback (`default_socket_path`, config.rs:253-259). | Dart: `Platform.environment['XDG_RUNTIME_DIR']` → `'$dir/dicta.sock'`, fallback `/run/user/${uid}/dicta.sock` (uid via `Process.run('id', ['-u'])` or hardcode per CONTEXT) — replicate the fallback chain, not just the happy path. Bootstrap: `windowManager.ensureInitialized()`, `dynamic_color` for Material You, ru locale default. **Anti-pattern to avoid:** dictactl builds JSON by string concat with quote-stripping (`main.rs:140` — known bug in CONCERNS); Dart must always `jsonEncode` (APP-06). |
| `app/lib/core/commands.dart` (or engine.dart helpers) — request builders | `crates/dicta-settings/src/net.rs:43-53` + its tests :55-70 | `settings_json()`/`set_json()` builders are pure functions with unit tests asserting exact wire format; value is always sent as a string. | `String buildSet(String key, String value) => jsonEncode({'cmd':'set','key':key,'value':value}) + '\n';` — `jsonEncode` replaces manual escaping entirely (APP-06); still unit-test the exact strings. |
| `app/pubspec.yaml`, `app/lib/l10n/app_ru.arb` (+ `app_en.arb`) | none (Flutter tooling) | n/a | No repo analog — repo is Rust/cargo. Follow RESEARCH.md stack (Flutter 3.50, `dynamic_color`, `window_manager`, `flutter_gen` or built-in `gen-l10n`); ru-first arb. |
| `app/test/widget_test.dart` (smoke) | none | n/a | Standard Flutter counter-test replacement; smoke: app boots, sidebar renders 4 Russian titles, daemon-offline banner appears without crash. |

## Reply Contract (authoritative shapes to copy into engine/settings)

- Daemon reply (`crates/dictad/src/main.rs:9-10, 48-55, 995-1002`): `{"base":{"state":"idle|recording|transcribing","level":0..1,"seconds":f32,"text":null|str,"error":null|str},"panel":bool,"settings":{…}?}` + `\n`. One JSON object, one line, written with serde on the daemon side.
- Protocol doc comment to keep beside the Dart client: `crates/dictad/src/main.rs:1-10`.
- States lowercase `idle/recording/transcribing` (`State::as_str`, dictad :92-98).

## Conventions (from .planning/codebase/CONVENTIONS.md — apply to Flutter side)

- **Russian UI strings, English identifiers** — but Flutter routes them through gen-l10n arb files (APP-05), not inline literals; identifiers/files snake_case→Dart camelCase naturally.
- **Honest measured comments**: module-level doc in the Rust `//!` style → Dart `///` library doc at top of `engine.dart`/`settings.dart` explaining the seam and the protocol («одна команда — одна строка», why no client-side validation). Comments explain why, cite real measurements/limits.
- **No silent failures**: every socket error surfaces to the UI (red line / «демон недоступен»), mirroring `error`-field-always pattern; also logged to stderr-equivalent (`debugPrint`).
- **No secrets**, minimal deps (CONTRIBUTING rules carry over).
- Commit style: lowercase Russian, `<область>: <что и почему>`.

## Anti-patterns to avoid (from .planning/codebase/CONCERNS.md)

- **4th copy of Settings** («Три независимые копии схемы настроек»): the golden fixture test is the mitigation — it pins the Dart model to a captured daemon reply so drift is caught in CI, not at runtime.
- **Hand-built JSON** («Четыре способа собрать JSON-команду вручную», the `dictactl inject` quote-loss bug): always `jsonEncode`, never string concat/quote-stripping (APP-06 exists for this).
- **Don't parse config.toml by string split** («Ручной разбор config.toml в dicta-ui»): Dart resolves only the socket path via env/fallback chain, never parses TOML in Phase 1.
- `hf.rs`/`catalog.rs` (curl subprocess patterns): not relevant to Phase 1 — no downloads in the app shell; revisit in the models phase.

## Metadata

**Analog search scope:** `crates/dicta-settings/src/`, `crates/dictactl/src/`, `crates/dictad/src/`, `crates/dicta-core/src/config.rs`; `.planning/codebase/*.md`. All named analogs verified git-tracked.

## PATTERNS COMPLETE

Phase 1 (dicta-app shell + IPC): 8 new artifacts classified, 6 mapped to tracked Rust analogs, 2 Flutter-tooling files with no analog (use RESEARCH.md).
Key patterns: net.rs framing (one command — one line, EOF/timeout errors in Russian), state.rs defaults-on-poison parsing, dictad-closes-after-status contract (poll with fresh connections, auto-reconnect).
Golden fixture test is the designed mitigation against the known 4-copies-of-Settings debt; jsonEncode-only (APP-06) closes the hand-built-JSON bug class.

---
last_mapped_commit: 20e00408af8921966d7d02f862d064e71658eb28
last_mapped_at: 2026-11-22
---
# Codebase Structure

**Analysis Date:** 2026-11-22

## Directory Layout

```
dicta/
├── crates/
│   ├── dicta-core/          # разделяемая библиотека: pipeline записи→STT→вставки
│   │   └── src/              #   config, capture, audio, stt, filter, inject, focus,
│   │                        #   hotkey, atspi, lib.rs (реэкспорты)
│   ├── dictad/              # демон: сокет, состояние, вставка (один main.rs)
│   ├── dictactl/            # CLI: диагностика + управление демоном (один main.rs)
│   ├── dicta-ui/             # layer-shell оверлей-индикатор
│   │   └── src/              #   main (Wayland), render, drag, edge, panel
│   └── dicta-settings/       # eframe-окно настроек
│       └── src/              #   main (egui), state, net, catalog, hf
├── adapters/                 # скрипты горячей клавиши под DE
│   ├── kwin/dicta-hotkey/    #   KWin-скрипт Plasma 6 (main.js + metadata.json + install.sh)
│   ├── hyprland/dicta.conf   #   bind для Hyprland
│   ├── sway/dicta.conf       #   bindsym для sway
│   ├── gnome/apply.sh        #   GNOME
│   └── quickshell/dicta.qml  #   Quickshell
├── systemd/                  # user-юниты + install.sh
├── tools/                    # fetch-model.sh, insert-test.sh
├── docs/                     # performance.md, RELEASE.md
├── config.example.toml       # полный пример конфига с комментариями
├── config.toml               # рабочий конфиг (не пример!)
├── Makefile                  # build / test / install / fetch-model
├── run.sh                    # dictad + dicta-ui одной командой
└── Cargo.toml                # workspace (members, workspace.dependencies)
```

## Directory Purposes

**`crates/dicta-core/src/` (ядро, 9 модулей + lib.rs):**
- Purpose: весь переиспользуемый код; от сюда зависят dictad и dictactl (и частично dicta-settings)
- Contains: плоские модули-файлы, никаких поддиректорий
- Key files: `config.rs` (Config, `EDITABLE_KEYS`, `ENGINE_KEYS`, `set_key`), `capture.rs` (`Recorder`), `stt.rs` (`Engine`), `inject.rs` (`Combo`, `insert_text`, uinput-`Keyboard`), `focus.rs`, `hotkey.rs`, `atspi.rs` (feature `atspi`), `audio.rs`, `filter.rs`, `lib.rs` (пути модулей + `pub use` главных типов)

**`crates/dictad/src/` (демон):**
- Purpose: единственный владелец состояния и модели
- Key files: `main.rs` — весь демон в одном файле (сокет+роутер `serve_conn`, `run_start`/`run_stop`, `set_setting`/`Change`, `settings_view`, watchdog, hotkey-поток, `spawn_ui`, `install_autostart`, юнит-тесты парсера настроек)

**`crates/dictactl/src/` (CLI):**
- Purpose: диагностика без демона (infer/record/dictate/calibrate/atspi-dump/test-keys) и управление (start/stop/toggle/cancel/status/inject/panel/settings)
- Key files: `main.rs` — clap-структуры `Cli`/`Command`, `ask()`/`ask_json()` — один запрос = одно соединение

**`crates/dicta-ui/src/` (оверлей):**
- Purpose: индикатор поверх всех окон; Wayland-код изолирован в `main.rs`, вся геометрия/логика — в чистых модулях
- Key files: `main.rs` (Wayland: wl_shm, wlr_layer_shell, события мыши, цикл опроса `POLL_MS=100`), `render.rs` (кадр: пилюля/волна/кнопки; `--preview` пишет BMP), `edge.rs` (Edge/Placement, `ui.toml`), `drag.rs` (автомат перетаскивания, тестируем синтетическими событиями), `panel.rs` (Settings-снимок + какие кнопки рисовать)

**`crates/dicta-settings/src/` (окно настроек):**
- Purpose: eframe/egui-окно, вкладки Запись/Модель/Вставка/Система; скачивание моделей
- Key files: `main.rs` (egui-UI), `state.rs` (Settings-снимок из ответа демона), `net.rs` (постоянное соединение `Client`), `catalog.rs` (статический каталог ggml-моделей), `hf.rs` (чистые функции Hugging Face API — URL/парсеры, тестируемые без сети)

**`adapters/`:**
- Purpose: три способа повесить клавишу (systemd-юнит / скрипт DE / встроенный evdev); `README.md` — матрица способов и инструкции
- Key files: `kwin/dicta-hotkey/contents/code/main.js` (KWin 6: `callDBus` → `dicta-toggle@<окно>.service`), `hyprland/dicta.conf`, `sway/dicta.conf`

**`systemd/`:**
- Purpose: user-юниты; `install.sh [--enable]` кладёт их
- Key files: `dictad.service` (`ExecStart=%h/.local/bin/dictad --ui`, `PartOf=graphical-session.target`), `dicta-toggle@.service` / `dicta-cancel@.service` / `dicta-probe@.service` (шаблон `<окно>` → `dictactl toggle --app <окно>`), `dicta-panel.service`

**`tools/`:**
- Purpose: shell-помощники; `fetch-model.sh` (ggml из Hugging Face в `~/.local/share/whisper-cpp/`), `insert-test.sh` (E2E вставки в foot-терминал)

## Key File Locations

**Entry Points:**
- `crates/dictad/src/main.rs`: демон (`main()` :131; флаги `--install-autostart`, `--ui`)
- `crates/dictactl/src/main.rs`: CLI
- `crates/dicta-ui/src/main.rs`: оверлей (`--edge`, `--preview`)
- `crates/dicta-settings/src/main.rs`: окно настроек

**Configuration:**
- `Cargo.toml`: workspace; версии общих зависимостей — только здесь (`anyhow`, `serde`, `serde_json`, `toml`, `dirs`, `whisper-rs`)
- `crates/dicta-core/src/config.rs`: `Config` + дефолты + `config_path()`; `DICTA_CONFIG` env перекрывает `~/.config/dicta/config.toml`
- `config.example.toml`: документированный пример (все ключи с комментариями)
- `crates/dicta-core/Cargo.toml`: features `vulkan` (default) / `atspi`
- `systemd/*.service`, `adapters/*/install.sh`: установка триггеров

**Core Logic (pipeline):**
- `crates/dicta-core/src/capture.rs` → `audio.rs` → `stt.rs` → `filter.rs` → `focus.rs` + `config.rs::paste_keys_for` → `inject.rs` (и `atspi.rs`)

**Точки обмена (интерфейс демона):**
- Сокет: `$XDG_RUNTIME_DIR/dicta.sock` (из `socket` в конфиге)
- Протокол: JSON-строки, команды и схема ответа документированы в док-комментарии `crates/dictad/src/main.rs:1-10`
- Whitelist настроек: `crates/dicta-core/src/config.rs:142-160` (`EDITABLE_KEYS`), `:163-170` (`ENGINE_KEYS`)

**Testing:**
- Юнит-тесты co-located: `#[cfg(test)] mod tests` внизу каждого файла — `crates/dicta-core/src/{config,focus,filter}.rs`, `crates/dictad/src/main.rs`, `crates/dicta-ui/src/{drag,edge,render,panel}.rs`, `crates/dicta-settings/src/{hf,catalog,state}.rs`
- E2E-скрипты: `tools/insert-test.sh`, `dicta-probe@.service`
- Запуск: `make test` = `cargo test --workspace`

## Naming Conventions

**Crates:**
- `dicta-core` (библиотека), короткие бинарники: `dictad` (демон), `dictactl` (CLI), `dicta-ui`, `dicta-settings`

**Files (Rust):**
- dicta-core и слои UI: один файл = один модуль, snake_case: `config.rs`, `focus.rs`; UI-модули короткие: `edge.rs`, `drag.rs`, `render.rs`, `panel.rs`
- Демоны/CLI держат весь код в `main.rs` (без модулей) — до размера ~1300 строк

**Типы:** `Engine`, `Recorder`, `Combo`, `Change` — существительные; enum-варианты PascalCase; методы snake_case (`insert_text`, `paste_keys_for`, `active_app`)

**Конфиг-ключи:** snake_case, ровно поля `Config` (`min_level`, `auto_stop_after_ms`); имена клавиш вставки — строки `ctrl+shift+v`, `shift+insert`

**Доки и язык:** док-комментарии `//!`/`///` на русском, с конкретикой («зачем так», замеры, честные рамки) — смотри `crates/dicta-core/src/atspi.rs:1-40` как эталон

## Where to Add New Code

**Новая команда демона:**
1. Разобрать её в `serve_conn` (`crates/dictad/src/main.rs:827-993` — match по `req.cmd`); если нужны поля — добавь их в `Req` (`:26-46`) с `#[serde(default)]`.
2. Ответ собери через `Res`/`Reply` (serde), не руками.
3. Клиент: подкоманда в `Command` enum `crates/dictactl/src/main.rs:36-109` + вызов `ask()`; для UI — обработка в `panel.rs` / `main.rs` dicta-ui.

**Новая настройка (полный чек-лист, 6 мест):**
1. Поле + дефолт в `Config` и `Default` (`crates/dicta-core/src/config.rs:10-102`)
2. Ключ в `EDITABLE_KEYS` (`:142-160`); в `ENGINE_KEYS`, если меняет модель
3. Рука `match key` в `parse_change` — валидация диапазона/вариантов (`crates/dictad/src/main.rs:542-609`)
4. Рука в `Change::apply` (`:444-483`) и в `current_value` (`:615-637`)
5. Поле в `Settings` демона (`:58-82`) + `settings_view` (`:382-406`)
6. Поле в клиентских снимках: `crates/dicta-ui/src/panel.rs` и `crates/dicta-settings/src/state.rs` (оба с дефолтом)
7. Если ключ редактируемый из GUI — кнопка в `crates/dicta-ui/src/panel.rs` или вкладка в `crates/dicta-settings/src/main.rs`

**Новый режим вставки:**
- Рука в `insert_text` (`crates/dicta-core/src/inject.rs:415-433`, match по `mode`) + разрешённое значение в `parse_change` dictad (`inject_mode`, `crates/dictad/src/main.rs:550-553`) + комментарий в `config.example.toml`

**Новый модуль ядра:**
- Файл в `crates/dicta-core/src/`, `pub mod` в `lib.rs`; для UI — модуль рядом с `main.rs` dicta-ui и `use crate::...`

**Новый адаптер DE:**
- Каталог в `adapters/<de>/` + строка в таблицу `adapters/README.md`; триггер = `dictactl toggle --app <окно>` или systemd-юнит

**Новый юнит systemd:**
- Файл в `systemd/` + строка копирования в `systemd/install.sh`

**Тесты:**
- Юнит-тест — `#[cfg(test)] mod tests` внизу того же файла; tmp-файлы через `std::env::temp_dir().join(format!("dicta-...-{name}"))` (уникальные имена — тесты параллельны, образец `crates/dicta-core/src/config.rs:324-330`); чистые функции без сети/композитора выноси как в `hf.rs`/`drag.rs`/`render.rs`

**Утилиты/скрипты:**
- Shell-помощники — в `tools/`; обновление установки — `Makefile` (install кладёт в `~/.local/bin`)

## Special Directories

**`target/`:** сборка cargo; generated, не коммитится (в .gitignore).
**`.planning/`:** артефакты GSD (планы, карты кодовой базы); коммитится согласно workflow GSD.
**`config.toml` (корень):** рабочий конфиг машины, отличается от `config.example.toml`; при чтении кодом не участвует — демон читает `~/.config/dicta/config.toml` или `$DICTA_CONFIG`.
**Runtime-пути (не в репо, но контракт кода):**
- `~/.config/dicta/config.toml` — конфиг (пишет `set_key`)
- `~/.config/dicta/ui.toml` — Edge-положение оверлея (`crates/dicta-ui/src/edge.rs`)
- `$XDG_RUNTIME_DIR/dicta.sock` — сокет демона, 0600
- `~/.cache/dicta/focus` — кэш фокусного окна (`crates/dicta-core/src/focus.rs:16-26`)
- `~/.local/share/whisper-cpp/*.bin` — ggml-модели (`tools/fetch-model.sh`, дефолт в `config.rs:246-251`)
- `$TMPDIR/dicta-<pid>-<ms>.wav` — temp-записи (удаляются `Drop` рекордера)

---

*Structure analysis: 2026-11-22*

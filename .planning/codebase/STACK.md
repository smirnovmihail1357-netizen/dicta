---
last_mapped_commit: 20e00408af8921966d7d02f862d064e71658eb28
last_mapped_at: 2026-11-22
---
# Technology Stack

**Analysis Date:** 2026-11-22

## Languages

**Primary:**
- Rust (edition 2021) — весь код, 5 крейтов в workspace (`Cargo.toml`, members: `crates/dicta-core`, `crates/dicta-ui`, `crates/dictactl`, `crates/dictad`, `crates/dicta-settings`)

**Secondary:**
- JavaScript — KWin-скрипт глобальной горячей клавиши (`adapters/kwin/dicta-hotkey/contents/code/main.js`)
- QML — адаптер Quickshell (`adapters/quickshell/dicta.qml`)
- Bash — установочные скрипты и инструменты (`Makefile` цели, `adapters/kwin/install.sh`, `adapters/gnome/apply.sh`, `systemd/install.sh`, `tools/fetch-model.sh`, `tools/insert-test.sh`, `run.sh`)
- Shell-конфиги композиторов: Hyprland (`adapters/hyprland/dicta.conf`), sway (`adapters/sway/dicta.conf`)

## Runtime

**Environment:**
- Linux, сессия Wayland (KWin/Plasma 6 — основная цель; Hyprland, sway, GNOME, Quickshell — через адаптеры)
- Rust-тулчейн не закреплён в `rust-toolchain.toml` (файла нет); CI использует контейнер `rust:1-trixie` (`.github/workflows/ci.yml`)
- Пользовательская systemd-сессия (`systemd --user`) для демона и oneshot-юнитов

**Package Manager:**
- Cargo (workspace, `resolver = "2"`)
- Lockfile: `Cargo.lock` — присутствует; сборка идёт с `--locked` (цель `make build`, CI)

## Frameworks

**Core:**
- Без веб-фреймворка. Три «GUI»-подхода, все нативные:
  - `smithay-client-toolkit` 0.21.1 + `wayland-client` 0.31.15 — оверлей-индикатор `dicta-ui` через `zwlr_layer_shell_v1` + `wl_shm` (софт-рендер без GPU) — `crates/dicta-ui/Cargo.toml`, `crates/dicta-ui/src/main.rs`
  - `eframe`/`egui` 0.31 — окно настроек `dicta-settings` — `crates/dicta-settings/Cargo.toml`, `crates/dicta-settings/src/main.rs`
  - Рендер текста в оверлее — `ab_glyph` 0.2.32, шрифт ищется в системных путях (`/usr/share/fonts/TTF/DejaVuSans.ttf`, `/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf`) — `crates/dicta-ui/src/main.rs:153-154`, `crates/dicta-ui/src/render.rs`
- CLI: `clap` 4 (feature `derive`) — `crates/dictactl/src/main.rs`

**Testing:**
- Стандартный `cargo test` — модули `#[cfg(test)]` внутри файлов (например, `crates/dicta-settings/src/hf.rs` — 8 тестов на чистых функциях). Отдельного test-раннера нет.
- `cargo test --workspace` — цель `make test` и шаг CI

**Build/Dev:**
- `cargo build --release --locked`; release-профиль: `lto = "thin"`, `codegen-units = 1` (`Cargo.toml` [profile.release])
- `Makefile`: `build`, `build-cpu`, `test`, `install` (`~/.local/bin`), `uninstall`, `fetch-model`
- `run.sh` — запуск демона + оверлея вместе для отладки
- CI: GitHub Actions (`.github/workflows/ci.yml` — build Vulkan и CPU-only, тесты, `bash -n` для скриптов; `.github/workflows/release.yml` — артефакты по тегам `v*`, матрица vulkan/cpu)

## Key Dependencies

**Critical:**
- `whisper-rs` 0.16 + `whisper-rs-sys` 0.15 — Rust-биндинги к whisper.cpp (STT). Транскрибация в `crates/dicta-core/src/stt.rs` (`WhisperContext::new_with_params`, `whisper_full`, `SamplingStrategy::Greedy/BeamSearch`)
- `hound` 3.5 — чтение/разбор WAV (`crates/dicta-core/src/audio.rs`)
- `anyhow` 1 — ошибки везде (`Result` с контекстом на русском)
- `serde`/`serde_json` — протокол Unix-сокета (JSON-строки) и конфиги
- `toml` 0.8 — `config.toml` с точечным редактированием без потери комментариев (`crates/dicta-core/src/config.rs:180`)
- `libc` 0.2 — сырые ioctl для `/dev/uinput` (`crates/dicta-core/src/inject.rs`), сигналы `SIGINT` для pw-record (`crates/dicta-core/src/capture.rs:53`), чтение `/dev/input/event*` (`crates/dicta-core/src/hotkey.rs`)
- `regex` 1 — нормализация/фильтры (`crates/dicta-core/src/filter.rs`)

**Optional features (`crates/dicta-core/Cargo.toml`):**
- `vulkan` (default): `whisper-rs/vulkan` + `whisper-rs-sys/vulkan` — GPU-инференс через ggml-vulkan (Radeon iGPU/dGPU). Сборка требует `glslangValidator` (пакет `glslang-tools`). CPU-сборка: `cargo build --release --no-default-features`
- `atspi`: `atspi` 0.22 (features `proxies`, `async-std`) + `futures` 0.3 + `zbus` 4.2 — вставка текста напрямую в фокусное поле через `org.a11y.atspi.EditableText` (`crates/dicta-core/src/atspi.rs`). Включается так: `cargo build --release --features dictad/atspi,dictactl/atspi`

**Infrastructure:**
- `dirs` 6 — XDG-пути (конфиг, кэш фокуса)
- `smithay-client-toolkit` 0.21.1 / `wayland-client` 0.31.15 — Wayland-клиент оверлея

## Configuration

**Environment:**
- Основной конфиг: `~/.config/dicta/config.toml` (пример — `config.example.toml`; в корне репо лежит рабочий `config.toml`). Все поля необязательны, дефолты совпадают с примером. Приоритет путей: `--config` CLI → `$DICTA_CONFIG` → `XDG_CONFIG_HOME/dicta/config.toml` (`crates/dicta-core/src/config.rs:264-272`)
- Позиция оверлея: `~/.config/dicta/ui.toml` (пишется перетаскиванием пилюли и `--edge`, `crates/dicta-ui/src/main.rs:184`)
- Ключевые поля `config.toml`: `model` (путь к ggml-файлу), `language`, `beam_size`, `n_threads`, `use_gpu`, `gpu_device`, `flash_attn`, `prompt` (вокабуляр), `mic` (PipeWire source), `socket`, `inject_mode` (`paste` | `clipboard` | `stdout` | `atspi`), `paste_keys`, `paste_keys_by_app`, `min_level`, `auto_stop*`, `max_record_ms`, `hotkey`
- Env-переменные:
  - `DICTA_CONFIG` — альтернативный путь конфига (нужен контейнеру)
  - `DICTA_MODEL` — путь модели для `tools/fetch-model.sh`
  - `DICTA_KEY`, `DICTA_CANCEL_KEY`, `DICTA_PANEL_KEY`, `DICTA_UNIT`, `DICTA_SELFTEST` — параметры `adapters/kwin/install.sh`
  - `BASE_URL` — зеркало Hugging Face в `tools/fetch-model.sh`
  - Отладка `dicta-ui`: `DICTA_DEBUG`, `DICTA_ANIM`, `DICTA_ANIM_MS`, `DICTA_ANIM_OUT_MS`, `DICTA_LAYER`, `DICTA_EXCLUSIVE`, `DICTA_ALWAYS_DRAW`, `DICTA_PANEL` (`crates/dicta-ui/src/main.rs:68-124`)
- Секретов в проекте нет: не нужны ни API-ключи, ни учётные записи

**Build:**
- `Cargo.toml` (workspace) + `Cargo.lock`
- Системные зависимости для сборки с Vulkan (из CI): `build-essential pkg-config cmake clang libclang-dev libvulkan-dev glslang-tools glslc libxkbcommon-dev` — whisper.cpp собирается из исходников через `whisper-rs-sys`/`cc`
- Рантайм-зависимости (внешние бинарники): `pw-record` (PipeWire), `wl-copy`/`wl-paste` (wl-clipboard), `curl` (для dicta-settings и fetch-model), опционально `jq` (адаптеры Hyprland/sway)
- Модель whisper: ggml-файл(ы) в `~/.local/share/whisper-cpp/`, рабочая — `ggml-medium-q5_0.bin` (`config.example.toml`); каталог моделей — `crates/dicta-settings/src/catalog.rs` (6 позиций от tiny-q5_1 до large-v3)

## Platform Requirements

**Development:**
- Linux + Wayland-сессия, Rust ≥ 1.75 (edition 2021), `glslangValidator` для default-сборки
- Для живого тестирования: доступ к `/dev/input/event*` (группа `input` или udev-ACL) для встроенной горячей клавиши; `/dev/uinput` для вставки клавиш
- Диагностика без DE-адаптера: `dictactl infer --timing file.wav`, `dictactl paste-keys`, `tools/insert-test.sh`

**Production:**
- Установка в `~/.local/bin` (цель `make install`); systemd user-юниты через `systemd/install.sh`; KWin-скрипт через `adapters/kwin/install.sh`
- Демон запускается как `systemctl --user start dictad.service` (`PartOf=graphical-session.target`, `Wants=pipewire.service`, `Restart=on-failure`)
- Замеры нагрузки/памяти в `docs/performance.md`: dictad idle 65–67 МБ RSS, инференс medium-q5_0 на iGPU ~3.2 с, оверлей 2–4% одного ядра при перерисовке 100 мс

---

*Stack analysis: 2026-11-22*

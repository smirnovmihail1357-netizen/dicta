---
last_mapped_commit: 20e00408af8921966d7d02f862d064e71658eb28
last_mapped_at: 2026-11-22
---
# External Integrations

**Analysis Date:** 2026-11-22

Проект — локальный системный демон; сетевых сервисов, БД и auth-провайдеров нет. Все «интеграции» — это локальные подсистемы Linux (PipeWire, Wayland, uinput, AT-SPI, systemd) и один внешний HTTP API (Hugging Face, только для скачивания моделей).

## APIs & External Services

**Hugging Face (единственный сетевой вызов, только в dicta-settings и tools/fetch-model.sh):**
- Поиск/обзор ASR-моделей, дерево файлов репозитория и прямые ссылки на ggml-файлы — для загрузки моделей whisper.cpp из GUI настроек
- SDK/Client: без HTTP-библиотеки — спавнится системный `curl` как дочерний процесс (`crates/dicta-settings/src/main.rs:241,281`); докачка через `curl -C -`, `.part`-файлы
- Чистые функции URL/парсинга без сети — `crates/dicta-settings/src/hf.rs`:
  - `browse_url()` → `https://huggingface.co/api/models?search=...&filter=automatic-speech-recognition&sort=...&limit=20&offset=...`
  - `tree_url_at()` → `https://huggingface.co/api/models/{repo}/tree/main/{subpath}`
  - `resolve_url()` → `https://huggingface.co/{repo}/resolve/main/{path}`
  - `parse_search()` / `parse_tree()` / `parse_tree_dirs()` — толерантный разбор JSON (битый JSON → пустой список; LFS-размер из `lfs.size`)
- Auth: не требуется (публичные репозитории; токен не используется)
- Альтернативный CLI-путь: `tools/fetch-model.sh` — `curl -L --fail --retry 3` на `https://huggingface.co/ggerganov/whisper.cpp/resolve/main/<model>` (или `$BASE_URL`), цель в `$DICTA_MODEL` или `~/.local/share/whisper-cpp/`
- Встроенный каталог из 6 моделей без поиска — `crates/dicta-settings/src/catalog.rs` (`CATALOG`)

## Audio & STT

**PipeWire (запись):**
- Запуск: `pw-record --target <mic> --rate 16000 --channels 1 --format s16 <tmp.wav>` — `crates/dicta-core/src/capture.rs:28-33`
- Остановка: `SIGINT` (не SIGTERM — чтобы дописался WAV-заголовок), дедлайн 2 с, затем `kill` — `crates/dicta-core/src/capture.rs:50-66`
- Источник по умолчанию — системный микрофон; переопределение через `mic` в конфиге (список: `pactl list short sources`) или `dictactl --mic`
- systemd-зависимость: `dictad.service` объявляет `Wants=pipewire.service` / `After=pipewire.service`

**whisper.cpp (STT):**
- Биндинги `whisper-rs` 0.16 / `whisper-rs-sys` 0.15 — `crates/dicta-core/src/stt.rs`
- Загрузка ggml-модели из `cfg.model`, параметры: `use_gpu`, `gpu_device`, `flash_attn`, `n_threads`, `beam_size` (1 = greedy, >1 = BeamSearch), `language`, `prompt` (начальный промпт-вокабуляр)
- Логи whisper.cpp глушатся своим callback'ом (`ggml_log_set`/`whisper_log_set` → `quiet_log`), на экран идут только WARN+ с префиксом `dicta/whisper` — `crates/dicta-core/src/stt.rs:13-27`
- Ввод: WAV 16 кГц mono s16 (`hound` — `crates/dicta-core/src/audio.rs::load_wav`), ресемплинг `resample_to_16k` при необходимости

**Vulkan (GPU-бэкенд whisper):**
- Cargo-фича `vulkan` (default) → фичи `whisper-rs/vulkan`, `whisper-rs-sys/vulkan` (ggml-vulkan) — `crates/dicta-core/Cargo.toml`
- Сборка требует `glslangValidator` (`glslang-tools`); CPU-сборка: `make build-cpu` / `--no-default-features` (важно на машинах, где нельзя трогать VRAM)
- Выбор устройства: `gpu_device` в конфиге (0 = быстрое, 1 = медленное iGPU/dGPU — сверять по «распознано за» в journalctl)

## Text Injection (Wayland)

**wl-clipboard (буфер обмена):**
- `wl-copy --type text/plain` для записи текста (именно `text/plain` — Konsole/Firefox не принимают анонсированный MIME `text` — `crates/dicta-core/src/inject.rs:395-398`)
- `wl-paste --no-newline` — чтение прежнего буфера и подтверждение записи (поллинг до `paste_delay_ms`, не слепая пауза — `crates/dicta-core/src/inject.rs:328-400`)
- Восстановление прежнего буфера через `clipboard_restore_ms` (2500 мс) в фоне, т.к. приложения читают буфер асинхронно

**uinput (виртуальная клавиатура, вставка по клавишам):**
- Сырые ioctl `/dev/uinput` (`UI_DEV_CREATE`, `UI_SET_EVBIT/KEYBIT`, события `EV_KEY`+`EV_SYN`) без crate-зависимостей — `crates/dicta-core/src/inject.rs:26-170`
- Комбо `Ctrl+V` / `Shift+Insert` и др. — парсер `Combo::parse()` (`crates/dicta-core/src/inject.rs:77`)
- Пересоздание uinput-устройства при сбое (libinput/KWin нумерует устройство не мгновенно) — `crates/dicta-core/src/inject.rs:115-153`

**AT-SPI2 (альтернативная вставка, фича `atspi`):**
- Пишет текст прямо в фокусный редактируемый объект через `org.a11y.atspi.EditableText` — без буфера обмена и без синтезированных клавиш — `crates/dicta-core/src/atspi.rs`
- Транспорт: `zbus` 4.2 (D-Bus-сессия), прокси `atspi` 0.22 (`AccessibleProxy`, `EditableTextProxy`, `TextProxy`), корень реестра `/org/a11y/atspi/accessible/root`, `org.a11y.atspi.Registry`
- Ограничения: Firefox/GTK отдают EditableText; часть Qt/Electron — нет; фильтр ролей через `atspi_allow_any_role` в конфиге
- Диагностика: `dictactl atspi-dump` (дерево доступности), `dictactl atspi-test`

**wtype/ydotool:**
- Не используются — вставка идёт через uinput (клавиши) и wl-clipboard (текст), что композитор-независимо (KWin, Hyprland, sway, XWayland)

## Desktop Environment Adapters (горячие клавиши)

Все адаптеры сводятся к запуску `dictactl toggle --app <окно>` и передают имя фокусного окна, чтобы демон подобрал клавиши вставки (`paste_keys_by_app`; нормализация `firefox.desktop` → `firefox` — `crates/dicta-core/src/focus.rs::normalize`). Обзор — `adapters/README.md`.

**KWin / Plasma 6:**
- KWin-скрипт `dicta-hotkey` (`adapters/kwin/dicta-hotkey/`) — KWin 6 не даёт `spawn()`, поэтому клавиша зовёт `callDBus("org.freedesktop.systemd1", ..., "StartUnit", "dicta-toggle@<app>.service")` — `adapters/kwin/dicta-hotkey/contents/code/main.js`
- Имя фокусного окна: `workspace.activeWindow.appId || resourceClass` (Wayland/XWayland), санитизация под имена systemd-инстансов `[A-Za-z0-9._-]`
- Установка: `adapters/kwin/install.sh` — sed-подстановка ключей в `main.js` (`DICTA_KEY`, по умолчанию `Alt+V`, т.к. Meta+V занят историей клипборда), копия в `~/.local/share/kwin/scripts/dicta-hotkey`, затем `loadScript`/`start` через `kwriteconfig5/6` + DBus
- Самопроверка: `DICTA_SELFTEST=1` дёргает юнит при загрузке скрипта; `dicta-probe@.service` пишет лог цепочки в `/tmp/dicta-probe.log`
- Изменение клавиши требует перелогина (KWin не перечитывает kglobalshortcutsrc на лету)

**GNOME:**
- `adapters/gnome/apply.sh` — custom keybinding через `gsettings` (`org.gnome.settings-daemon.plugins.media-keys custom-keybindings/custom0..7`), команда `dictactl toggle`; снятие — `apply.sh --remove`

**Hyprland:**
- `adapters/hyprland/dicta.conf` — `bind = SUPER, V, exec, dictactl toggle --app "$(hyprctl activewindow -j | jq -r '.class // .initialClass // empty')"`; требует `jq`

**sway:**
- `adapters/sway/dicta.conf` — `bindsym $mod+v exec ... swaymsg -t get_tree | jq -r 'recurse(.nodes[]) | select(.focused == true) | (.app_id // .window_properties.class)'`; требует `jq`

**Quickshell:**
- `adapters/quickshell/dicta.qml` — `ShellShortcut` + `Focus.window.active?.appId` → `Process.exec(["dictactl","toggle","--app",app])`

**Встроенная горячая клавиша (без DE-адаптера):**
- evdev-чтение `/dev/input/event*` в потоке демона — `crates/dicta-core/src/hotkey.rs` (модификаторы ctrl/alt/shift/meta по Linux keycodes; конфиг `hotkey = "ctrl+alt+v"`, `"off"` — отключить)
- Требует группу `input` или udev-ACL; недоступна — используются юниты-адаптеры

**Фокус-кэш:**
- Любой сторонний скрипт слежения за фокусом пишет `~/.cache/dicta/focus` (или вызывает `dictactl focus-report <app>`) — `crates/dicta-core/src/focus.rs`

## Daemon IPC

**Unix-сокет:**
- Путь из конфига (`socket`, по умолчанию `/run/user/1000/dicta.sock`)
- Протокол: одна JSON-строка на команду, одна строка JSON в ответ (`crates/dictad/src/main.rs:4-10`): `{"cmd":"start|stop|toggle|cancel|status|panel|ui|set","key":...,"value":...}`, опционально `"app": "<окно>"`
- Ответ: `{"state":"idle|recording|transcribing","level":0..1,"seconds":..,"text":..,"error":..,"panel":bool,"settings":{...}}`
- Особенность: соединение с `stop` демон держит открытым до конца вставки текста; статусные закрываются сразу — `crates/dictad/src/main.rs:1004-1029`
- Клиенты: `dictactl` (CLI), `dicta-ui` (поллинг статуса каждые 100 мс), `dicta-settings` (`crates/dicta-settings/src/net.rs::Client::call` — отправка `{"cmd":"set"}` для живого изменения настроек с записью в config.toml)

## Process Spawning (внутренние интеграции)

- `dictad --ui` поднимает и держит живым `dicta-ui` (ищет бинарник рядом с собой, затем в `~/.local/bin`) — `crates/dictad/src/main.rs:1243`
- `dictactl settings` спавнит `dicta-settings` — `crates/dictactl/src/main.rs:158-178`
- `dictad` с `{"cmd":"panel"}` открывает панель настроек внутри оверлея (`dicta-panel.service` → `dictactl panel`)

## systemd (user units)

- `systemd/dictad.service` — демон: `PartOf=graphical-session.target`, `After=`/`Wants=pipewire.service`, `Restart=on-failure`, `ExecStart=%h/.local/bin/dictad --ui`, `WantedBy=graphical-session.target`
- `systemd/dicta-toggle@.service`, `systemd/dicta-cancel.service` (+`@`) — oneshot-триггеры от клавиш DE; `%i` = имя фокусного окна от адаптера; `StartLimitIntervalSec=0` (клавишу жмут часто)
- `systemd/dicta-panel.service` — открытие панели настроек
- `systemd/dicta-probe@.service` — диагностика цепочки «клавиша → systemd → демон» с логом в `/tmp/dicta-probe.log`
- Установка юнитов в `~/.config/systemd/user` — `systemd/install.sh`; KWin-скрипт зависит от их наличия

## Data Storage

**Databases:**
- Нет. Всё состояние — файлы и сокет.

**File Storage:**
- Конфиг: `~/.config/dicta/config.toml` (точечная правка значений демоном с сохранением комментариев — `crates/dicta-core/src/config.rs:180`)
- Позиция оверлея: `~/.config/dicta/ui.toml`
- Кэш фокуса: `~/.cache/dicta/focus`
- Модели whisper: `~/.local/share/whisper-cpp/*.bin`
- Запись: временный WAV в `$TMPDIR` (`dicta-<pid>-<ms>.wav`)

**Caching:**
- Нет.

## Authentication & Identity

**Auth Provider:**
- Не применимо. Локальное приложение; Hugging Face API используется анонимно.

## Monitoring & Observability

**Error Tracking:**
- Нет (внешних сервисов). Ошибки идут в JSON-ответ сокета (`"error"`) и `eprintln!` в journalctl.

**Logs:**
- `journalctl --user` (systemd user-юниты); логи KWin-скрипта — `journalctl --user -b 0 | grep dicta-hotkey`; диагностика вставки — `/tmp/dicta-probe.log`; отладка оверлея — переменные `DICTA_*`

## CI/CD & Deployment

**Hosting:**
- Локальная установка (не сервис): `make install` → `~/.local/bin`, юниты → `~/.config/systemd/user`, KWin-скрипт → `~/.local/share/kwin/scripts`

**CI Pipeline:**
- GitHub Actions: `.github/workflows/ci.yml` (push в main + PR: сборка Vulkan и CPU-only, `cargo test --workspace`, `bash -n` скриптов; контейнер `rust:1-trixie`, системные пакеты whisper/vulkan/glslang/pipewire/wl-clipboard)
- `.github/workflows/release.yml` (теги `v*`: release-архивы, матрица vulkan/cpu)
- `dependabot.yml` — обновления зависимостей

## Environment Configuration

**Required env vars:**
- Нет обязательных. Все опциональны (см. STACK.md «Configuration»).

**Secrets location:**
- Не применимо — секретов в проекте нет.

## Webhooks & Callbacks

**Incoming:**
- Нет. Входящие интеграции только локальные: Unix-сокет (`dictactl`/`dicta-ui`/`dicta-settings`), systemd StartUnit из KWin-скрипта, evdev-устройства, D-Bus (AT-SPI registry).

**Outgoing:**
- Нет. Исходящие вызовы — только HTTPS-чтение из huggingface.co (поиск и скачивание моделей, curl из `dicta-settings`/`tools/fetch-model.sh`).

---

*Integration audit: 2026-11-22*

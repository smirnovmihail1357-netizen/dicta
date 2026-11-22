---
last_mapped_commit: 20e00408af8921966d7d02f862d064e71658eb28
last_mapped_at: 2026-11-22
---
<!-- refreshed: 2026-11-22 -->
# Architecture

**Analysis Date:** 2026-11-22

## System Overview

dicta — системный голосовой диктант для Linux/Wayland. Паттерн: **hub-and-spoke демон с разделяемой библиотекой**. Вся логика и состояние живут в демоне `dictad`; все прочие бинарники — тонкие клиенты Unix-сокета. Общего кода между клиентами и демоном минимум: всё переиспользуемое лежит в `dicta-core`.

```text
┌────────────────────────────────────────────────────────────────────────────┐
│                            Клиенты (тонкие)                                 │
├──────────────────────┬──────────────────────┬──────────────────────────────┤
│   dictactl (CLI)     │   dicta-ui (оверлей)  │   dicta-settings (окно)      │
│  `crates/dictactl/`  │  `crates/dicta-ui/`   │  `crates/dicta-settings/`    │
│  clap-подкоманды,    │  layer-shell, wl_shm  │  eframe/egui, вкладки,       │
│  разовое соединение  │  опрос статуса 100 мс │  постоянное соединение        │
└──────────┬───────────┴──────────┬───────────┴──────────────┬───────────────┘
           │  JSON-строка →        │                          │
           │  Unix-сокет           │                          │
           ▼                       ▼                          ▼
┌────────────────────────────────────────────────────────────────────────────┐
│                    dictad (демон, хаб)                                      │
│                     `crates/dictad/src/main.rs`                             │
│  UnixListener → поток на соединение · воркер распознавания · watchdog      │
│  тишины 80 мс · evdev-хоткей · прогрев модели · spawn dicta-ui             │
└──────────┬─────────────────────┬──────────────────┬─────────────────────────┘
           │                     │                  │
           ▼                     ▼                  ▼
┌────────────────────────────────────────────────────────────────────────────┐
│                    dicta-core (разделяемая библиотека)                      │
│                     `crates/dicta-core/src/`                                │
│  config · capture · audio · stt · filter · inject · focus · hotkey ·        │
│  atspi (feature-gated)                                                     │
└──────────┬─────────────────────┬──────────────────┬─────────────────────────┘
           │                     │                  │
           ▼                     ▼                  ▼
┌──────────────────┐  ┌───────────────────┐  ┌───────────────────────────────┐
│ PipeWire         │  │ whisper.cpp       │  │ Ввод/вывод ОС                 │
│ `pw-record`      │  │ (whisper-rs,      │  │ uinput (/dev/uinput),         │
│ → WAV 16k mono   │  │ Vulkan/CPU)       │  │ evdev (/dev/input/event*),    │
│                  │  │                   │  │ wl-copy, AT-SPI2 (D-Bus)      │
└──────────────────┘  └───────────────────┘  └───────────────────────────────┘
```

## Component Responsibilities

| Component | Responsibility | File |
|-----------|----------------|------|
| dictad | Демон: сокет, состояние записи, watchdog, вставка, живой конфиг, перезагрузка модели | `crates/dictad/src/main.rs` |
| dictactl | CLI-диагностика и управление: infer/record/dictate без демона, start/stop/toggle/cancel/status/settings через сокет | `crates/dictactl/src/main.rs` |
| dicta-ui | Layer-shell оверлей-индикатор: волна уровня, клик = stop, drag между краями, панель настроек в оверлее | `crates/dicta-ui/src/main.rs` |
| dicta-settings | Окно настроек (eframe): вкладки Запись/Модель/Вставка/Система, браузер Hugging Face, скачивание моделей | `crates/dicta-settings/src/main.rs` |
| dicta-core::config | TOML-конфиг, `EDITABLE_KEYS`/`ENGINE_KEYS`, перезапись одного ключа с сохранением комментариев | `crates/dicta-core/src/config.rs` |
| dicta-core::capture | `Recorder`: запись через subprocess `pw-record` в temp-WAV | `crates/dicta-core/src/capture.rs` |
| dicta-core::audio | Чтение WAV, ресэмпл в 16k, RMS/уровень речи, тишина | `crates/dicta-core/src/audio.rs` |
| dicta-core::stt | `Engine`: обёртка whisper-rs, один экземпляр модели в памяти | `crates/dicta-core/src/stt.rs` |
| dicta-core::filter | Вычищение галлюцинаций whisper («спасибо за просмотр», повторы) | `crates/dicta-core/src/filter.rs` |
| dicta-core::inject | Вставка текста: uinput-клавиатура + wl-copy + AT-SPI | `crates/dicta-core/src/inject.rs` |
| dicta-core::focus | Какое окно фокусное: подсказка → `$DICTA_APP` → кэш `~/.cache/dicta/focus` | `crates/dicta-core/src/focus.rs` |
| dicta-core::hotkey | Глобальная клавиша через evdev, без DE-скриптов | `crates/dicta-core/src/hotkey.rs` |
| dicta-core::atspi | Вставка через AT-SPI2/D-Bus (feature `atspi`) | `crates/dicta-core/src/atspi.rs` |
| adapters | Скрипты клавиш DE: KWin (JS), Hyprland, sway, GNOME, Quickshell | `adapters/` |
| systemd | User-юниты: демон + oneshot-триггеры клавиш | `systemd/` |

## Pattern Overview

**Overall:** Hub-and-spoke + разделяемая библиотека. Никакого async-рантайма: только `std::thread`, `Arc<Mutex<..>>` и атомики.

**Key Characteristics:**
- Протокол: **JSON-строки** — один запрос `\n` → один ответ `\n` по Unix-сокету (`/run/user/$UID/dicta.sock`, права 0600). Команды: `toggle/start/stop/cancel/status/inject/panel/settings/set` (`crates/dictad/src/main.rs:1-10`).
- Состояние записи — `AtomicUsize` (`State::Idle/Recording/Transcribing`, `crates/dictad/src/main.rs:84-113`); мьютекс демона держится только для быстрых действий, inference — в отдельном воркере с собственным `Arc<Mutex<Engine>>`.
- Все настройки валидируются на стороне демона (`parse_change`, `crates/dictad/src/main.rs:497`); клиенты только отображают и отправляют — «панель нельзя привести в состояние, которого демон не придерживается».
- Композитор-независимость: uinput-клавиатура и wl-copy работают на любом Wayland-композиторе; знание фокусного окна — опциональная подсказка (адаптер/кэш), не запрос к композитору.
- Feature gates: `vulkan` (по умолчанию, GPU whisper) и `atspi` (эксперимент, D-Bus-вставка) — `crates/dicta-core/Cargo.toml`, `crates/dictad/Cargo.toml`.

## Layers

**Клиенты (CLI/GUI):**
- Purpose: UX-обёртки над сокетом; никакой бизнес-логики настроек
- Location: `crates/dictactl/src/main.rs`, `crates/dicta-ui/src/main.rs`, `crates/dicta-settings/src/main.rs`
- Contains: разбор аргументов, рисование, отправка команд; `dictactl` дополнительно умеет работать автономно (infer/record/dictate напрямую через dicta-core)
- Depends on: `dicta-core` (только `Config` для пути сокета у CLI/UI; settings-окно ещё `catalog`/`hf`/`net`/`state`)
- Used by: пользователь, адаптеры DE (`dictactl toggle --app konsole`), systemd-юниты

**Демон:**
- Purpose: единственный владелец состояния, модели и вставки
- Location: `crates/dictad/src/main.rs` (один файл, ~1335 строк)
- Contains: `serve_conn` ( роутер команд), `run_start`/`run_stop` (pipeline записи), `set_setting`/`schedule_engine_reload`, watchdog-поток, hotkey-поток, `spawn_ui`
- Depends on: `dicta-core`
- Used by: все клиенты через сокет

**Ядро (dicta-core):**
- Purpose: весь переиспользуемый код pipeline'а
- Location: `crates/dicta-core/src/` — плоские модули, реэкспорт главных типов в `lib.rs:11-15`
- Contains: чистые функции + обёртки внешнего мира (процессы, ioctl, D-Bus)
- Depends on: `pw-record`, `wl-copy`, `/dev/uinput`, `/dev/input/event*`, whisper-rs; опционально atspi/zbus
- Used by: dictad, dictactl, dicta-settings (только Config), dicta-ui (нет — только свой код + сокет)

## Data Flow

### Primary Request Path (диктовка)

1. Триггер: встроенный evdev-хоткей (поток в `crates/dictad/src/main.rs:294-323`, слушает `dicta_core::hotkey::listen`) **или** клавиша DE → `dictactl toggle --app konsole` → сокет (`crates/dictactl/src/main.rs:129`) **или** systemd-шаблон `dicta-toggle@konsole.service`.
2. `serve_conn` разбирает строку как `Req`, роутит по `req.cmd` (`crates/dictad/src/main.rs:810-1012`).
3. `run_start` (`crates/dictad/src/main.rs:1014`): `Recorder::start` запускает subprocess `pw-record --rate 16000 --channels 1 --format s16` в `$TMPDIR/dicta-<pid>-<ms>.wav`; состояние → `Recording`.
4. Во время записи watchdog-поток тикает каждые 80 мс (`crates/dictad/src/main.rs:216-272`): берёт `handler.try_lock()`, читает уровень из хвоста недописанного WAV (`Recorder::level`, `crates/dicta-core/src/capture.rs:79-93`), пишет в `AtomicU32`, решает авто-остановку (`max_record_ms` — всегда; тишина — если `auto_stop`).
5. `run_stop` (`crates/dictad/src/main.rs:1030-1194`): под **коротким** мьютексом забирает recorder, шлёт SIGINT (pw-record дописывает WAV-заголовок), читает сэмплы `load_wav` + `resample_to_16k`; спавнит воркер-поток и возвращает `mpsc::Receiver<()>` соединению "stop".
6. Воркер (пошагово в `run_stop`):
   - гейт тишины: `speech_level(&samples) < cfg.min_level` → пропуск распознавания (whisper на шуме сочиняет фразы);
   - `Engine` за собственным `Arc<Mutex<..>>` — `eng.transcribe(&samples)` (`crates/dicta-core/src/stt.rs:67-109`), beam/greedy по `beam_size`, `filter::clean` от галлюцинаций;
   - фокус: `dicta_core::focus::active_app(hint)` → `cfg.paste_keys_for(app)` → `Combo::parse` (`crates/dicta-core/src/config.rs:235-243`, `focus.rs:63-71`);
   - `inject::insert_text` по `inject_mode` (`crates/dicta-core/src/inject.rs:415-433`): `paste` = сохранить буфер → `wl-copy` → пауза `paste_delay_ms` → клавиши через uinput → фон-восстановление буфера через `clipboard_restore_ms`; `clipboard`/`stdout`/`atspi` — альтернативы;
   - `finish()`: `compare_exchange(Transcribing → Idle)` — новая запись, начатая во время inference, не затрагивается (`crates/dictad/src/main.rs:1196-1206`);
   - `tx.send(())` — соединение "stop" отвечает клиенту после клавиш, не дожидаясь восстановления буфера.
7. Ответ: `serde_json::to_string(&Reply{base: Res, panel, settings?})` + `\n` (`crates/dictad/src/main.rs:995-1002`). `Res` = `{state, level, seconds, text, error}`.

### Settings Flow (живое изменение)

1. Панель оверлея / окно настроек / `dictactl settings <key> <value>` → `{"cmd":"set","key":"min_level","value":"0.4"}`.
2. `parse_change` (`crates/dictad/src/main.rs:497-610`): ключ обязан быть в `dicta_core::config::EDITABLE_KEYS` (`crates/dicta-core/src/config.rs:142-160`), значение проверяется по диапазону/вариантам; для `model` принимается и путь, и имя из папки моделей.
3. `Change::apply` правит живой `Config` в `Handler`; если значение не изменилось — файл не пишется, модель не перезагружается (`current_value`, `crates/dictad/src/main.rs:615-637`).
4. `dicta_core::config::set_key` переписывает одну строку в config.toml по месту, сохраняя комментарии, атомарно (tmp + rename) (`crates/dicta-core/src/config.rs:185-231`).
5. Если ключ в `ENGINE_KEYS` (`crates/dicta-core/src/config.rs:163-170`) → `schedule_engine_reload` в отдельном потоке; отказ, если идёт запись/распознавание или уже грузится (`reloading` AtomicBool) (`crates/dictad/src/main.rs:744-783`).

### UI Flow (оверлей)

1. dictad c `--ui` (или DICTA_UI=1) держит dicta-ui живым: `spawn_ui` ищет бинарник рядом с собой, рестартует каждые 2 с (`crates/dictad/src/main.rs:1228-1251`).
2. dicta-ui опрашивает `{"cmd":"status"}` раз в `POLL_MS = 100` мс (`crates/dicta-ui/src/main.rs:59`, поток ~`:466-509`), рисует через wl_shm софт-рендер (`render.rs`), слой — `zwlr_layer_shell_v1` (`main.rs` imports `smithay_client_toolkit::shell::wlr_layer`).
3. Клик по пилюле = `stop`; drag к другому краю = переезд оверлея (`drag.rs` — конечный автомат, `edge.rs` — геометрия/Placement, положение в `ui.toml` рядом с конфигом).
4. Панель настроек в оверлее: рисует снимок `{"cmd":"settings"}` (`panel.rs`), по клику шлёт `{"cmd":"set",...}`; видимость панели — `AtomicBool panel` **в демоне**, поэтому её открывают и хоткей, и `dictactl panel`, и сама панель.

### Focus Resolution (какое окно фокусное)

Композиторы не дают единого запроса «фокусное окно» (у KWin `queryWindowInfo` интерактивен). Порядок (`crates/dicta-core/src/focus.rs:63-71`):

1. подсказка адаптера в команде (`--app konsole`, KWin-скрипт передаёт `workspace.activeWindow`);
2. `$DICTA_APP`;
3. кэш-файл `~/.cache/dicta/focus` (его пишет `dictactl focus-report <app>` или любой скрипт слежения);
4. ничего не известно → общий `paste_keys`.

Имя нормализуется `focus::normalize` (`.desktop` срезается, берётся хвост после точки, lower-case, без `-`/`_`) и ищется в `paste_keys_by_app`.

**State Management:**
- `Handler` (Config + Engine + Recorder + last_text/last_error/last_app/paste_combo) за `Arc<Mutex<Handler>>` (`crates/dictad/src/main.rs:115-129`).
- Межпоточные флаги: `state: AtomicUsize`, `level: AtomicU32` (f32 через `to_bits`/`from_bits`), `started_at: AtomicU64`, `panel: AtomicBool`, `reloading: AtomicBool` — все `Ordering::Relaxed`.
- Сигнал завершения вставки — `std::sync::mpsc::channel::<()>`.
- dicta-ui: `Arc<Mutex<Shared>>` + опрос сокета; dicta-settings: `net::Client` — одно постоянное соединение (`crates/dicta-settings/src/net.rs:14-19`).

## Key Abstractions

**`Engine` (STT-движок):**
- Purpose: один экземпляр whisper-модели на процесс; грузится секунды, живёт в VRAM/памяти до конца демона
- Examples: `crates/dicta-core/src/stt.rs:37-115`; хранится как `Option<Arc<Mutex<Engine>>>` в `Handler`, прогрев фоновым потоком при старте (`crates/dictad/src/main.rs:182-201`)
- Pattern: lazy singleton + отдельный мьютекс, чтобы inference не блокировал команды

**`Recorder` (capture):**
- Purpose: запись как subprocess `pw-record` в WAV-файл; RAII — `Drop` убивает процесс и удаляет файл (`crates/dicta-core/src/capture.rs:100-106`)
- Examples: `crates/dicta-core/src/capture.rs:12-98`
- Pattern: обёртка внешнего процесса; уровень читается из хвоста ещё пишущегося файла

**`Combo` (комбинация клавиш):**
- Purpose: парсер `"ctrl+shift+v"` → флаги + linux keycode; копируемый value-тип
- Examples: `crates/dicta-core/src/inject.rs:69-80`
- Pattern: `Combo::parse` + default-комбо из конфига как fallback

**`Change` (типизированное изменение настройки):**
- Purpose: enum Level/Flag/Count/Index/Text с `&'static str` ключами; знает, как примениться к Config, как выглядеть в TOML и нужна ли перезагрузка модели
- Examples: `crates/dictad/src/main.rs:410-484`
- Pattern: whitelist-ключи (`EDITABLE_KEYS`) вместо String-ключей

**`Settings`-снимок (3 копии):**
- Purpose: сериализуемый снимок настроек для клиентов
- Examples: `crates/dictad/src/main.rs:58-82` (сервер), `crates/dicta-ui/src/panel.rs:16-60` и `crates/dicta-settings/src/state.rs:10-` (клиенты, оба `#[serde(default)]` — старый демон без `settings` их не ломает)
- Pattern: дублирование схемы вместо общего типа — сознательная развязка версий клиента и демона

## Entry Points

**dictad (демон):**
- Location: `crates/dictad/src/main.rs:131`
- Triggers: `systemd/dictad.service` (`ExecStart=%h/.local/bin/dictad --ui`), `run.sh`, автозапуск `~/.config/autostart/dicta.desktop` (`--install-autostart`), вручную
- Responsibilities: сокет, прогрев модели, watchdog, hotkey, удержание dicta-ui

**dictactl (CLI):**
- Location: `crates/dictactl/src/main.rs` (clap `Command` enum: Infer/Record/Dictate/Start/Stop/Toggle/Cancel/Status/TestKeys/PasteKeys/AtspiDump/AtspiInsert/Inject/Calibrate/FocusReport/Panel/Settings)
- Triggers: пользователь; адаптеры DE; systemd-шаблоны `dicta-toggle@.service`, `dicta-cancel@.service`, `dicta-probe@.service`
- Responsibilities: диагностика цепочки без демона + управление демоном

**dicta-ui (оверлей):**
- Location: `crates/dicta-ui/src/main.rs`; флаги `--edge`, `--preview out/` (кадры BMP без композитора), env `DICTA_DEBUG`, `DICTA_ANIM`
- Triggers: `spawn_ui` демона, `dicta-panel.service`, `run.sh`

**dicta-settings (окно настроек):**
- Location: `crates/dicta-settings/src/main.rs`
- Triggers: `dictactl settings` (без key/value — запускает окно, `crates/dictactl/src/main.rs:172-181`), пользователь

**Установщики:**
- `Makefile` (build/test/install/fetch-model), `systemd/install.sh` (юниты), `adapters/kwin/install.sh` (KWin-скрипт), `tools/fetch-model.sh` (ggml-модель), `tools/insert-test.sh` (E2E-проверка вставки)

## Architectural Constraints

- **Threading:** только OS-потоки; ни одного async-рантайма. Правило мьютексов: `handler.lock()` — только быстрые действия; inference держит только `engine.lock()`; watchdog — `try_lock`; загрузка модели — вне мьютексов вообще.
- **Порядок памяти:** все атомики `Ordering::Relaxed` — состояние читается только для отображения и сравнения, CAS в `finish()` тоже Relaxed (допустимо: гонка даёт лишний/недостающий тик, не порчу).
- **Внешние процессы — часть контракта:** `pw-record` (PipeWire), `wl-copy` (wl-clipboard). Их отсутствие — ошибка запуска записи, не паника.
- **Права:** сокет 0600; uinput требует `/dev/uinput`; evdev-хоткей — группы `input` или udev-ACL (`crates/dictad/src/main.rs:313-319` подсказывает); без них — hotkey = "off" + клавиша DE.
- **Feature-матрица:** `vulkan` (default) — нужен glslangValidator при сборке; `--no-default-features` = чистый CPU; `atspi` — экспериментальная вставка через D-Bus, собирается отдельно у dictad и dictactl.
- **Версионная совместимость клиентов:** клиенты диктуют `#[serde(default)]` на снимках настроек — демон без команды `settings` не ломает UI (`crates/dicta-ui/src/panel.rs:14-15`).
- **Global state:** `QUIET_LOG_INSTALLED: Once` — глушение логов whisper.cpp (`crates/dicta-core/src/stt.rs:15-30`); синглтон-модели нет — она переезжает в `Handler` под мьютексом.
- **Circular imports:** нет; зависимость строго однонаправленная: клиенты → dictad → dicta-core (dicta-ui слкшишь сокету напрямую, dicta-core не импортирует).
- **Язык:** комментарии и строки для пользователя — русские; это конвенция проекта, не обходить.

## Anti-Patterns

### Схема настроек в трёх местах

**What happens:** `Settings` определён независимо в `crates/dictad/src/main.rs:58-82`, `crates/dicta-ui/src/panel.rs` и `crates/dicta-settings/src/state.rs` — сервер и два клиента.
**Why it's wrong:** добавление поля в конфиг требует правки трёх структур; рассинхрон незаметен до рантайма (клиент просто покажет дефолт из `#[serde(default)]`).
**Do this instead:** это осознанный выбор (развязка версий), но при добавлении настройки обновляй **все три** плюс `Change`-руки в dictad (см. STRUCTURE.md «Новая настройка»). Не выносить общий тип, пока совместимость старого демона не потеряет важность.

### JSON руками в dictactl

**What happens:** `crates/dictactl/src/main.rs:138-144` собирает payload строкой `format!("{{\"cmd\":\"{cmd}\"...}}")`.
**Why it's wrong:** `app` экранируется вручную (`replace(['"', '\\'], "")`) — забываемый шаг; демону нужен валидный JSON.
**Do this instead:** для новых команд используй `serde_json` (как это делает dictad для ответа), либо держи существующий `json_quote()` (`crates/dictactl/src/main.rs:154`).

### Долгие операции под мьютексом handler

**What happens / Why it's wrong:** загрузка модели (секунды) или inference под `handler.lock()` заморозили бы start/toggle/status на всё время.
**Do this instead:** образец уже в коде — `run_stop` забирает данные под коротким локом и уходит в поток (`crates/dictad/src/main.rs:1042-1068`), `schedule_engine_reload` грузит модель вне мьютексов (`:744`). Новые тяжёлые пути делать так же.

## Error Handling

**Strategy:** `anyhow::Result` сквозь весь dicta-core и демона; паник нет — ошибки доходят до пользователя.

**Patterns:**
- Ошибки демона не убивают его: пишутся в `h.last_error` и возвращаются клиенту полем `"error"` ответа JSON (например `crates/dictad/src/main.rs:649-655, 924-932`); в stderr — `eprintln!("dictad: ...")` для journalctl.
- Валидация настроек **до** применения: `parse_change` возвращает внятную русскую ошибку, битый config.toml невозможен (`crates/dictad/src/main.rs:486-610`).
- Fallback-цепочки вместо отказа: `Combo::parse(keys).unwrap_or(paste_combo)` (`crates/dictad/src/main.rs:1160`), неизвестная команда → `"error"` в ответе, соединение не рвётся.
- Вставка деградирует по режимам: `paste` не удался → ошибка в поле, но демон жив; таймаут pw-record 2 c → kill + лучший esfuerzo (`crates/dicta-core/src/capture.rs:55-66`).

## Cross-Cutting Concerns

**Logging:** `eprintln!` в stderr (русский текст, префикс `dictad:` / `dicta/whisper:`), смотрится через `journalctl --user -u dictad`. Логи whisper.cpp глушатся до WARN (`crates/dicta-core/src/stt.rs:15-30`). Никакого log-фреймворка.
**Validation:** значения настроек — демон (`parse_change`); JSON — serde; WAV-заголовок — `hound`; команды — match по `req.cmd`, неизвестная → ошибка в ответе.
**Authentication:** нет; сокет 0600 в `$XDG_RUNTIME_DIR` — защита файловыми правами пользовательской сессии. Секретов в коде нет (вставка — буфер обмена, не клавиши шифрования).
**Persistence:** `~/.config/dicta/config.toml` (атомарная перезапись одного ключа `set_key`), `~/.config/dicta/ui.toml` (Edge-положение оверлея, `crates/dicta-ui/src/edge.rs`), `~/.cache/dicta/focus` (подсказка фокуса), `$TMPDIR/dicta-*.wav` (удаляется в `Drop`).

---

*Architecture analysis: 2026-11-22*

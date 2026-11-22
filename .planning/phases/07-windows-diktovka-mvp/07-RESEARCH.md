# Phase 7: Windows диктовка MVP — Research

**Researched:** 2026-11-22
**Domain:** порт dictad/dicta-core на Windows (захват WASAPI, вставка SendInput + delayed render, RegisterHotKey, named pipe interprocess + DACL, CI)
**Confidence:** HIGH по живому коду и API-верификациям; MEDIUM по поведенческим деталям Windows-вставки (закрываются спиком, см. Facts §5)

## Summary

Главный вывод по аудио: **бэкенд сегодня — НЕ cpal**. Захват живёт через subprocess `pw-record` (PipeWire CLI), пишущий 16 кГц mono s16 WAV во temp-файл, с остановкой SIGINT и чтением «недописанного» WAV-хвоста для индикатора уровня [VERIFIED: crates/dicta-core/src/capture.rs:28-53,79-93]. Это значит, что Win-порт захвата — **новый модуль целиком** (cpal/WASAPI), а не «переход cpal→cpal». Плюсы: интерфейс `Recorder { start, stop, elapsed, level }` чисто платформенный (callers в dictad не меняются), а линейный ресэмпл в 16 кГц **уже существует** [VERIFIED: crates/dicta-core/src/audio.rs:58-74 `resample_to_16k`] и вызывается на выходе записи [VERIFIED: capture.rs:71] — децимация 48кГц→16кГц на cpal-пути НЕ требует нового кода. Конверсия сэмплов для whisper уже есть (`to_f32`, stt.rs:93), whisper принимает i16 16кГц — конверсии частоты в stt нет и не нужна, всё делает capture-путь.

Второй главный вывод: **в репо сегодня ноль cfg(windows)/cfg(unix)-развилок** (grep по crates/ пуст). Практически весь workspace НЕ собирается на Windows: unix-only — capture.rs (libc::kill/SIGINT), hotkey.rs (std::os::unix::io::RawFd, libc::poll), inject.rs (libc::open/ioctl /dev/uinput), atspi.rs (фича, Linux), dictad/main.rs (UnixListener, PermissionsExt), dictactl (UnixStream), dicta-ui (wayland), dicta-settings (UDS net.rs). Порт = cfg-гейтинг этих модулей + новые Windows-модули. Переносимые как есть: audio.rs, filter.rs, config.rs, focus.rs, stt.rs (точечные правки путей — см. Facts §8).

Третий вывод: **whisper Vulkan на Windows сломан upstream** — Vulkan-бэкенд ggml молча не регистрируется в Windows MSVC static-сборках (наши точные версии: whisper-rs 0.16.0 / whisper-rs-sys 0.15.0) [CITED: github.com/ggml-org/whisper.cpp/issues/3750], плюс сборка Vulkan на windows-раннере требует Vulkan SDK (отсутствует) и спотыкается о vulkan-shaders-gen [CITED: whisper.cpp#3973]. Рекомендация: **Windows-MVP = CPU-only** (`--no-default-features`), Vulkan на Windows отложить.

Спик delayed render (locked 5): CI-раннер headless — GUI-эксперимент (вставка в Chromium/elevated/зажатый модификатор) на нём невозможен. Решение: **документальная база уже собрана** (MS Learn по delayed rendering/WM_RENDERFORMAT/message-only окнам; PITFALLS: settle ~400 мс, Handy #502 класс), а экспериментальная часть — маленький `spike/`-bin, который пользователь запускает на своей Windows-машине по протоколу замеров из Facts §5, ДО написания production-кода вставки. Результат фиксируется в 07-SPIKE-RESULT.md.

**Primary recommendation:** cfg-гейтинг unix-модулей + новые `#[cfg(windows)]`-модули (capture via cpal native-format + существующий resample; inject via enigo SendInput + windows-rs delayed render; hotkey via RegisterHotKey thread-queue без окна; transport via interprocess named pipe + SDDL DACL), Windows-CI Rust-джоба CPU-only, спик-протокол как отдельный deliverable до production-вставки.

<user_constraints>
## User Constraints (from CONTEXT.md, verbatim)

### Locked (решения, не подлежащие пересмотру планёром)

1. **Ветка**: вся работа фазы 7 — на ветке `windows` (база 23049d6); кодовые коммиты → windows, .planning-доки → main. Merge в main НЕ делаем.
2. **Протокол заморожен**: тот же JSON-протокол демона поверх named pipe `\\.\pipe\dicta` (interprocess 2.4.8); команды status/settings/set/start/stop/toggle/cancel/inject/panel без изменений; Flutter-сторона (WindowsPipeEngine фазы 6) НЕ переписывается, только подключается к живому серверу.
3. **Linux-путь нетронут**: `cfg(windows)`/`cfg(unix)` развилки в Rust; поведение Linux байт-в-байт сохраняется (гейты сравнения с main).
4. **Стек (из .planning/research/STACK.md, подтверждено research-флагом ROADMAP)**: cpal 0.18.2 (WASAPI), enigo 0.6.1 (SendInput KEYEVENTF_UNICODE), interprocess 2.4.8 (пайп-сервер + DACL), whisper — существующий движок dicta-core (та же модель/конфиг; whisper.cpp уже собирается на Windows CI). Новые зависимости Rust — только эти три + их замыкание; UI-зависимости Flutter НЕ растут.
5. **Спик ОБЯЗАТЕЛЕН до фиксации реализации вставки** (research-флаг ROADMAP): delayed render `SetClipboardData(NULL)` + `WM_RENDERFORMAT` — экспериментальная проверка settle-таймингов и форматов ДО написания production-кода вставки (PITFALLS.md: settle ~400 мс, Handy #502 класс).
6. **DACL-контракт из фазы 6** (README-якорь «named pipe DACL»): security descriptor пайпа — только текущий пользователь (owner SID), SYSTEM; закрывает residual «preempt-спуф пайпа» из 06-SECURITY.
7. **Пути**: конфиг `%APPDATA%\dicta\config.toml` (паритет resolveConfigPath фазы 5), модели `%LOCALAPPDATA%` (обязательство data_local_dir из README фазы 6); env-шов `DICTA_PIPE` (паритет DICTA_SOCK, kDictaPipe дефолт).
8. **Хоткей в ДЕМОНЕ** (паритет Linux-решения): RegisterHotKey + скрытое message-only окно («без открытия окна» = без видимого UI); приложение НЕ регистрирует свои глобальные хоткеи.
9. **Кириллица**: Flutter-код — только через .arb (гейт cyrillic_gate как раньше); Rust-код — ASCII-идентификаторы, русские строки допустимы как в существующих crates (конвенция dictad).
10. **UI-hint**: новых Flutter-виджетов почти нет (подключение к живому серверу); ui-plan-gate → block:false ожидаем; ui-аудит фазы 7 — решает оркестратор после исполнения (фаза 6 SKIPPED с обоснованием).

### Must-NOT (запреты)

1. НЕ трогать Linux-пути без cfg-развилки; НЕ менять протокол; НЕ менять Flutter-транспорт (windows_engine.dart фазы 6).
2. НЕ регистрировать глобальные хоткеи в Flutter-приложении.
3. НЕ портировать dicta-ui (layer-shell overlay) — Windows-оверлей вне вехи; вставка без видимого окна.
4. НЕ вводить новые Flutter-зависимости; Rust — только cpal/enigo/interprocess (+замыкание, все уже в STACK-исследовании).
5. НЕ хранить аудио/тексты диктовок на диске; payload не логируется (паритет T-контрактов).
6. НЕ делать merge windows→main; НЕ пушить без CI-проверки (push ок — CI зелёный).
7. НЕ нарушать DACL-контракт: пайп доступен только текущему пользователю.
8. НЕ пропускать спик: реализация вставки пишется ТОЛЬКО после спик-результата.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| WIN-03 | Захват микрофона (cpal/WASAPI, 16 кГц mono) → whisper.cpp → та же модель/конфиг, запись по RegisterHotKey без открытия окна | Facts §1 (текущий pw-record-бэкенд + переносимый Recorder-интерфейс), §2 (cpal WASAPI: native format + существующий resample_to_16k), §4 (RegisterHotKey: hWnd=NULL → очередь потока, окно вообще не нужно), §7 (whisper CPU-only на Windows) |
| WIN-04 | Вставка SendInput (KEYEVENTF_UNICODE) с буфером через delayed render (SetClipboardData(NULL) + WM_RENDERFORMAT), оптаут-форматы Win+V/облака, восстановление буфера; кейсы: обычное/elevated/UIPI, зажатый модификатор, Chromium | Facts §3 (enigo 0.6.1 API: key(Control/Unicode), text() = батч SendInput KEYEVENTF_UNICODE), §5 (спик-протокол + документальная база delayed render), §6 (UIPI/модификаторы/Chromium — PITFALLS P3, UAT-кейсы) |
</phase_requirements>

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Захват микрофона 16кГц/уровень | Rust-демон (cpal WASAPI, новый cfg(windows)-модуль capture) | — | Аудио всегда было в ядре; уровень течёт в status-ответе по IPC; Dart-аудио запрещено (Must-NOT 4 + STACK What-NOT-to-Use) |
| STT (whisper) | Rust-демон (существующий stt.rs) | — | Та же модель/конфиг (locked 4); движок платформенно-независим |
| Глобальный хоткей | Rust-демон (RegisterHotKey в отдельном потоке) | — | Паритет Linux-решения (locked 8); приложение хоткеи НЕ регистрирует (Must-NOT 2) |
| Вставка текста | Rust-демон (enigo SendInput + windows-rs delayed render) | — | inject.rs — cfg(unix) целиком; Windows-ветка новая (verified fact) |
| IPC-транспорт | Rust-демон — interprocess named pipe сервер с DACL | Flutter-клиент — готовый WindowsPipeEngine фазы 6 (не переписывается) | Протокол заморожен (locked 2); сервер создаёт пайп и несёт DACL-контракт (locked 6) |
| Пути конфига/моделей | Rust (dirs: config_dir → %APPDATA%, data_local_dir → %LOCALAPPDATA%) | — | locked 7; требует cfg-развилки в default_model_path (Facts §8) |
| CI Windows | GitHub Actions windows-latest (Rust-джоба CPU-only + существующая Flutter-джоба) | — | Кросс-компиляция MSVC с Linux невозможна (Facts §9) |

## Standard Stack

### Core (Rust, target.'cfg(windows)'.dependencies)

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `cpal` | 0.18.2 (текущая; crates.io 2026-08-16) | Захват микрофона WASAPI | Единственная зрелая Rust audio-библиотека; WASAPI-бэкенд Win10+; легитимность OK (seam: 896k загрузок/нед, RustAudio/cpal, с 2014) |
| `enigo` | 0.6.1 (текущая; crates.io 2025-08-28) | SendInput: клавиши вставки + Unicode-текст | Windows-бэкенд = statically-linked `windows` crate; `text()` на Windows батчит SendInput KEYEVENTF_UNICODE (Context7, VERIFIED); легитимность OK (139k/нед, с 2017) |
| `interprocess` | 2.4.8 (в STACK) / **2.4.9 вышла 2026-11-18** (crates.io VERIFIED) | Named pipe сервер `\\.\pipe\dicta` + DACL | Sync-API без tokio; `ListenerOptionsExt::security_descriptor(SecurityDescriptor::deserialize(SDDL))` — ровно DACL-контракт (Context7 VERIFIED); легитимность OK (400k/нед, с 2020) |
| `windows` (transitive от enigo; добавляется явно только для delayed render/hotkey/SID) | версия из замыкания enigo 0.6.1 | Win32: clipboard delayed render, RegisterHotKey, GetTokenInformation | Уже в дереве через enigo — «три новых + замыкание» (locked 4) не нарушено |

**Пин-решение (планёру):** interprocess 2.4.8 (как в locked 4) или 2.4.9 — обе валидны, API security_descriptor идентичен; рекомендую 2.4.9 (последний stable), но если планёр держит locked 4 буквально — 2.4.8 не хуже. Пометка drift: STACK/CONTEXT писали 2.4.8 до выхода 2.4.9 (2026-11-18).

**MSRV:** локальный rustc 1.99.0 (2026-09-28) ≥ 1.85 (MSRV cpal) [VERIFIED: `rustc --version` этой сессии]; cpal/enigo/interprocess MSRV 1.85/1.85/1.75 — конфликтов нет.

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| interprocess named pipe | сырой CreateNamedPipeW через windows-rs | ~300 строк руками + DACL руками; interprocess уже проверяет это в API — не надо |
| enigo | прямые SendInput через windows-rs | enigo под капотом тот же код; для MVP быстрее; при желании вырезать потом тривиально (STACK:89) |
| cpal native format + resample | cpal 16кГц запрос + AUTOCONVERTPCM | AUTOCONVERTPCM даёт тишину на Win11 24H2 communications-эндпоинтах (cpal#1200, PITFALLS:217) — native format безопаснее, децимация у нас уже есть |
| windows-rs для спика | winapi / clipboard-ext | winapi старее; clipboard-ext — непроверенная зависимость, против locked 4 |

### Installation (ветка windows)

```toml
# crates/dicta-core/Cargo.toml
[target.'cfg(windows)'.dependencies]
cpal = "0.18"
enigo = "0.6"
windows = { version = "<из замыкания enigo>", features = [
  "Win32_Foundation", "Win32_System_DataExchange", "Win32_System_LibraryLoader",
  "Win32_UI_WindowsAndMessaging", "Win32_Security", "Win32_System_Threading" ] }

# crates/dictad/Cargo.toml
[target.'cfg(windows)'.dependencies]
interprocess = "2.4"
```

## Package Legitimacy Audit

| Package | Registry | Age | Downloads | Source Repo | Verdict | Disposition |
|---------|----------|-----|-----------|-------------|---------|-------------|
| cpal | crates.io | с 2014-12 | ~896k/нед | github.com/RustAudio/cpal | OK | Approved |
| enigo | crates.io | с 2017-01 | ~139k/нед | github.com/enigo-rs/enigo | OK | Approved |
| interprocess | crates.io | с 2020-08 | ~400k/нед | github.com/kotauskas/interprocess | OK | Approved |

Проверено: `gsd-tools query package-legitimacy check --ecosystem crates` → все `OK` (seam этой сессии) + сверка версий через crates.io API (`cpal 0.18.2 2026-08-16`, `enigo 0.6.1 2025-08-28`, `interprocess 2.4.9 2026-11-18`) [VERIFIED: crates.io API + seam]. Пакетов с verdict SLOP/SUS нет. Все три имени взяты из STACK-исследования и подтверждены официальными доками (docs.rs/Context7) — право на [VERIFIED] получено.

## Facts by topic

Каждый факт: `[VERIFIED: file:line]` (живое чтение этой сессии), `[VERIFIED: tool]` (проба), `[CITED: url]` (официальные доки), `[ASSUMED]` (тренировочное знание — в Assumptions Log).

### §1. Аудио-бэкенд сегодня: pw-record, НЕ cpal — объём порта

- Захват — subprocess: `Command::new("pw-record")` с `["--rate","16000","--channels","1","--format","s16"]` в temp-WAV [VERIFIED: crates/dicta-core/src/capture.rs:28-40, дословно `cmd.args(["--rate", "16000", "--channels", "1", "--format", "s16"]);`].
- Остановка — `libc::kill(pid, libc::SIGINT)` («pw-record успевает дописать заголовок WAV») + поллинг `try_wait` 2 с [VERIFIED: capture.rs:51-66].
- Уровень — чтение **недописанного** WAV-файла с диска: последние 1280 байт (≈40 мс s16) → `normalized_rms` [VERIFIED: capture.rs:79-93].
- Ресэмпл уже есть: `resample_to_16k(samples, rate)` — линейная интерполяция, вызывается в `Recorder::stop` [VERIFIED: capture.rs:71 `Ok(crate::audio::resample_to_16k(samples, rate))`; реализация audio.rs:57-74].
- Whisper-конверсия: `to_f32` (i16 → f32 /32768) в stt [VERIFIED: stt.rs:93 + audio.rs:77-79]; конверсии частоты в stt-пути нет и не нужно.
- Интерфейс `Recorder` (start/stop/elapsed/level) — единственная точка, которую знает dictad [VERIFIED: dictad/src/main.rs:959 `run_start(&mut h,...)` → `Recorder::start(&h.cfg)` main.rs:1020; уровень — `rec.level()` main.rs:233]. Callers НЕ меняются: Windows-модуль даёт тот же интерфейс.

**Следствие для порта:** Win-захват = новый `capture_windows` (cpal-стрим в памяти, level по хвосту кольцевого буфера, без temp-файлов — заодно закрывает Must-NOT 5 «не хранить аудио на диске»: pw-record-путь на Linux пишет temp-WAV, Windows-путь может вообще не трогать диск). Объём ≈ 150-200 строк + callback-модель вместо subprocess.

### §2. cpal 0.18.2 на WASAPI: форматы и 16 кГц (RQ2)

- cpal WASAPI-бэкенд: `supported_input_configs` пробует стандартные частоты против `IsFormatSupported` нативно; «только дефолтное число каналов поддерживается в shared-mode» (комментарий самого cpal: "When calling IsFormatSupported with the shared-mode audio engine, only the default number of channels seems to be supported") [CITED: Context7 /RustAudio/cpal, src/host/wasapi/device.rs].
- Канонический пример захвата — `default_input_config()` + `build_input_stream` [CITED: Context7, examples/record_wav.rs].
- **Рекомендация:** НЕ запрашивать 16 кГц mono принудительно. Дефолтный input-конфиг (обычно 44.1/48 кГц, 1-2 канала) → в callback: deinterleave/первый канал → накопление → на stop `resample_to_16k` (уже есть, §1). Это обходит AUTOCONVERTPCM-тишину на Win11 24H2 [CITED: cpal#1200 через PITFALLS.md:217]. Если у устройства `supported_input_configs` содержит нативные 16 кГц — можно взять их, но НЕ полагаться.
- mono: если у девайса 2 канала — берём первый (паритет `deinterleave` audio.rs:50-55, который уже берёт `c[0]`).
- Дefолтное устройство: `host.default_input_device()`; `cfg.mic` (имя PipeWire-источника) на Windows неприменимо — вариант: матчинг по имени устройства cpal, MVP: дефолтный микрофон + честная ошибка если не найден [ASSUMED — решение планёра, MVP-область].

### §3. enigo 0.6.1: API для вставки (WIN-04)

- Создание: `Enigo::new(&Settings::default())`; трейт `Keyboard` [VERIFIED: Context7 /enigo-rs/enigo].
- Клавиши: `enigo.key(Key::Control, Press)` → `key(Key::Unicode('v'), Click)` → `key(Key::Control, Release)` — Windows-путь конвертирует в VIRTUAL_KEY + scan code [VERIFIED: Context7 keyboard-trait].
- Unicode-текст: `enigo.text(&str)` — на Windows «builds one batched SendInput array of KEYEVENTF_UNICODE press/release pairs; `\n` becomes `Key::Return`, `\t` becomes `Key::Tab`» [VERIFIED: Context7, src/win/win_impl.rs:323]. Это ровно механизм WIN-04 fallback-вставки (посимвольный режим ≠ основной paste-режим; PITFALLS:227 — чанки, не 200+ отдельных кейстроков: батч уже решает это одним SendInput-массивом).
- Ошибки: `InputError::InvalidInput/Mapping/Simulate` — честные Result, не тихие [VERIFIED: Context7].
- enigo НЕ делает clipboard — delayed render наш код через windows-rs (замыкание, не новая зависимость).
- Раскладки: `Key::Unicode` не зависит от раскладки (на Windows идёт через VK/VkKeyScanExW для букв, KEYEVENTF_UNICODE для text) [VERIFIED: Context7].

### §4. RegisterHotKey: без окна вообще (RQ3, locked 8)

- Официальная семантика [CITED: learn.microsoft.com/windows/win32/api/winuser/nf-winuser-registerhotkey]: «If this parameter is **NULL**, WM_HOTKEY messages are posted to the message queue of the calling thread and must be processed in the message loop» — **окно для хоткея не нужно**, достаточно выделенного потока с `GetMessageW`-циклом. «Без открытия окна» из locked 8 выполняется буквально.
- Модификаторы: MOD_ALT 0x1 / MOD_CONTROL 0x2 / MOD_SHIFT 0x4 / MOD_WIN 0x8 / MOD_NOREPEAT 0x4000 [CITED: там же]. MOD_WIN «зарезервированы для ОС» — документированное ограничение.
- Занятость: «fails if the keystrokes ... have already been registered for another hot key» → GetLastError `ERROR_HOTKEY_ALREADY_REGISTERED` (1409) — честная ошибка, как требует RQ3 [CITED: там же; код 1409 — ASSUMED числом, поведение — CITED].
- F12 зарезервирован для отладчика [CITED: там же].
- Повторная регистрация того же (hWnd,id): старый хоткей сохраняется, надо явно `UnregisterHotKey` [CITED: там же].
- Парсинг спеки: демон на Linux читает `DICTA_HOTKEY` env или `cfg.hotkey` [VERIFIED: dictad/src/main.rs:293 `let hotkey_spec = std::env::var("DICTA_HOTKEY").unwrap_or_else(|_| cfg.hotkey.clone());`]; дефолт `"off"` [VERIFIED: config.rs:90 `hotkey: "off".into()`]. Windows-ветка парсит ту же спеку ("ctrl", "alt", "shift", "meta"/"super"/"win", буквы/цифры/F1-F12/space/enter — таблица keycode в hotkey.rs:15-66) в MOD_*+VK. Паритет env `DICTA_HOTKEY` сохраняем.
- Архитектурное место: отдельный поток, зеркально evdev-циклу [VERIFIED: dictad main.rs:298-323 — thread::spawn + listen + backoff]; Windows: thread::spawn → RegisterHotKey(NULL, id, mods|MOD_NOREPEAT, vk) → GetMessageW-цикл → WM_HOTKEY → `do_toggle` (та же функция main.rs:1208-1222). Конфликта с poll-циклом нет: message-queue живёт в своём потоке.
- Отказ (занято) — честный eprintln + НЕ ретраить бесконечно (в отличие от evdev-backoff): занятость не проходит сама; повторная попытка только после явного set-ключа. [Рекомендация, не факт]

### §5. Спик delayed render: судьба и протокол (RQ1, locked 5)

**Почему CI невозможен:** windows-latest раннер — сервисная headless-сессия без пользовательского десктопа; нет Chromium/elevated-целей, нет реального фокуса окон, clipboard-семантика в неинтерактивной сессии отличается. GUI-эксперимент вставки на CI невоспроизводим [ASSUMED по устройству GitHub-раннеров — устойчиво, но не прогонялось; решение консервативное].

**Документальная база (уже собрана, HIGH):**
- Delayed rendering: «If a window delays rendering a clipboard format, it must be prepared to render ... The system sends the clipboard owner a WM_RENDERFORMAT message when a request is received» [CITED: learn.microsoft.com/windows/win32/dataxchg/clipboard-operations].
- Message-only окно: «enables you to send and receive messages. It is not visible, has no z-order, cannot be enumerated, and **does not receive broadcast messages**» [CITED: learn.microsoft.com/windows/win32/winmsg/window-features]. WM_RENDERFORMAT шлётся владельцу буфера напрямую (не broadcast) — доставка в message-only окно ожидаема, но прямо в доках не зафиксирована → **пункт замера №7** (message-only vs скрытое WS_OVERLAPPED окно).
- Timing-феномены (settle ~400 мс, Chromium probe-then-read, менеджеры буфера съедают promise, SetClipboardData-promise возвращает NULL **на успехе** с протухшим last-error) — PITFALLS.md P2 с 4+ независимыми источниками (Handy #502, OpenWhispr #829, win-text-inject) [CITED: .planning/research/PITFALLS.md:35-55 + указанные issue-трекеры].

**РЕШЕНИЕ: спик = код-эксперимент `spike/delayed_render` (маленький bin, только windows-rs) + протокол замеров на Windows-машине пользователя (UAT-стиль), ДО production-кода вставки.** Результат — таблица в `07-SPIKE-RESULT.md` (файл фазы). Production-ветка вставки пишется только после него (Must-NOT 8).

**Протокол замеров (что пишет spike-бин в stdout для каждого кейса):**
1. Baseline-буфер: GetClipboardData(CF_UNICODETEXT) сохранить (что восстанавливать).
2. `EmptyClipboard` + `SetClipboardData(CF_UNICODETEXT, NULL)` (promise) + promise-оптауты: `RegisterClipboardFormatW` для «ExcludeClipboardContentFromMonitorProcessing», «CanIncludeInClipboardHistory», «CanUploadToCloudClipboard», «Clipboard Viewer Ignore» — тоже `SetClipboardData(fmt, NULL)`.
3. Зафиксировать: return value SetClipboardData (ожидание: NULL = успех! [CITED: PITFALLS:46]), GetLastError (ожидание: протухший), `IsClipboardFormatAvailable(CF_UNICODETEXT)` (ожидание: true), `GetClipboardOwner()` (должен быть hwnd спика).
4. Пользователь жмёт Ctrl+V в целевом окне (или спик шлёт SendInput). Замерить t1 − t0 (момент WM_RENDERFORMAT относительно SetClipboardData), t2 (SetClipboardData с реальным хендлом внутри обработчика), повторные WM_RENDERFORMAT (Chromium probe-then-read → ожидается ≥2 рендера).
5. Тишина: лог всех сообщений 5 с после вставки — определить фактический settle для конфига `clipboard_restore_ms` (дефолт 2500, config.rs:93).
6. Кейсы-цели: Notepad (обычное), Chromium-браузер, Windows Terminal (Ctrl+Shift+V), elevated-Notepad (UIPI: ожидание — вставка Ctrl+V СКВОЗЬ elevated запрещена, спик фиксирует факт тихого отказа), зажатый модификатор.
7. Message-only окно (HWND_MESSAGE) vs скрытое WS_OVERLAPPED окно: если WM_RENDERFORMAT не приходит в message-only в течение 10 с после paste — пересоздать как скрытое окно, зафиксировать какой вариант работает.
8. `WM_RENDERALLFORMATS` перед выходом: не отрендерить = потеря данных [CITED: clipboard-operations, Vanara-доки] — спик обязан обработать и залогировать.
9. Win+V после прогона: транскрипт в истории? (ручная проверка оптаутов). PowerShell `Get-Clipboard` — ещё один способ дернуть рендер (замер отдельным пунктом).

**Какой crate для спика:** `windows` (transitive от enigo; фичи `Win32_System_DataExchange`, `Win32_UI_WindowsAndMessaging`, `Win32_Foundation`). НЕ winapi (устаревает), НЕ clipboard-ext (не проверен, лишняя зависимость против locked 4).

### §6. UIPI, зажатые модификаторы, Chromium — вставка (WIN-04 кейсы)

Из PITFALLS P3 (все кейсы — обязательные UAT-пункты фазы, автоматизировать на CI нельзя):
- **UIPI:** SendInput в elevated-окно тихо игнорируется. До записи в буфер: сравнить integrity level цели (RID mandatory label) со своим → elevated = честный результат «ClipboardOnly/отказ», не мнимый успех [CITED: PITFALLS.md:63-73].
- **Зажатые модификаторы:** перед синтезом Ctrl+V отпустить физически зажатые (GetAsyncKeyState-чек), НЕ нажимая обратно после [CITED: PITFALLS.md:62 + FluidVoice #940].
- **Гонка фокуса:** GetForegroundWindow в момент хоткея; перед вставкой проверить, что окно всё ещё foreground, иначе честная ошибка [CITED: PITFALLS.md:69].
- **Chromium:** KEYEVENTF_UNICODE ок, но paste-эвент требует settle (probe-then-read) — закрывается спиком §5 [CITED: PITFALLS.md:44].
- dwExtraInfo-тег на синтезированные события — чтобы собственный хоткей/хуки не реагировали на свои вставки [CITED: PITFALLS.md:72].
- paste_keys по приложениям: `paste_keys_by_app` (config.rs:107-138) — konsole/alacritty → ctrl+shift+v и т.д. На Windows фокусное окно определяется по-своему (GetForegroundWindow + класс процесса), XDG-кэш фокуса неприменим; MVP: дефолт `ctrl+v` + терминальные приложения Windows (Windows Terminal = ctrl+v по дефолту? — замерить в UAT; известные терминалы Win поддерживают Ctrl+V, но Ctrl+Shift+V тоже) [ASSUMED — UAT-пункт].

### §7. whisper на Windows: CPU-only

- whisper-rs 0.16.0 / whisper-rs-sys 0.15.0 [VERIFIED: Cargo.lock-дерево этой сессии, `cargo tree -p dicta-core`]: whisper.cpp 1.8.3 в static-сборке MSVC с Vulkan — «Vulkan backend silently fails to register in the ggml backend registry» [CITED: github.com/ggml-org/whisper.cpp/issues/3750 — issue прямо про whisper-rs-sys 0.15.0/whisper-rs 0.16.0 Windows MSVC static]. Даже собрав Vulkan, получаем тихий CPU-fallback.
- Сборка Vulkan на windows-latest требует Vulkan SDK (не предустановлен) и ломается на vulkan-shaders-gen ExternalProject [CITED: whisper.cpp#3973].
- **Рекомендация:** Windows-джоба = `cargo build --no-default-features` (CPU-only, фича `vulkan` откл.), пользовательский конфиг `use_gpu` на Windows игнорирует/честное предупреждение. Vulkan-на-Windows — отдельная будущая работа (после апстрим-фикса), в MVP вне scope.
- Риск производительности: та же модель (locked 4: ggml-medium-q5_0, beam 5) на CPU Windows-десктопа — секундамбы на фразу; конфиг меняется пользователем (`n_threads`, `beam_size`, модель) — не блокер, но ожидание в UAT зафиксировать [ASSUMED].

### §8. Пути и конфиг (locked 7, WIN-05 смежный)

- `config_path()`: `dirs::config_dir().join("dicta").join("config.toml")` [VERIFIED: config.rs:173-178]. dirs 6 на Windows: config_dir = RoamingAppData = `%APPDATA%` [CITED: docs.rs/dirs/6.0.0 — Known Folders] → `%APPDATA%\dicta\config.toml` — locked 7 выполняется существующим кодом без правки.
- **Ловушка:** `default_model_path()` использует `dirs::data_dir()` [VERIFIED: config.rs:246-251], а на Windows `data_dir` = `{FOLDERID_RoamingAppData}` — Roaming [CITED: docs.rs/dirs/6.0.0/fn.data_dir: «Windows | {FOLDERID_RoamingAppData} | C:\Users\Alice\AppData\Roaming»]. Обещание README фазы 6 — модели в `%LOCALAPPDATA%` (data_local_dir) → **требуется cfg-развилка**: `#[cfg(windows)] dirs::data_local_dir()`, `#[cfg(not(windows))] dirs::data_dir()`. Без неё модели поедут в Roaming-профиль (нарушение locked 7).
- Флаттеровский `resolveConfigPath` фазы 5/6 уже Windows-осведомлён (config_path.dart фазы 6 PATTERNS:212) — Rust-сторона подтягивается паритетом.
- Сокет: `default_socket_path()` — XDG_RUNTIME_DIR [VERIFIED: config.rs:253-261]; Windows-ветка заменяется на имя пайпа `\\.\pipe\dicta` (env `DICTA_PIPE` приоритет — паритет resolvePipePath в windows_engine.dart:22-28, VERIFIED git show windows:app/lib/core/windows_engine.dart).
- `focus.rs`: компилируется везде (чистый std), но XDG_CACHE_HOME-кэш на Windows бессмыслен — cfg-развилка в `active_app` (hint → DICTA_APP env → Windows: GetForegroundWindow-детект) [VERIFIED: focus.rs:63-71].

### §9. CI и кросс-чек (RQ5, RQ6)

- Существующий windows-build.yml (ветка windows): Flutter-джоба только (subosito/flutter-action@v2, 3.50.1, zip-артефакт) [VERIFIED: git show windows:.github/workflows/windows-build.yml этой сессии]. Rust-джобы нет.
- windows-latest предустановленный Rust: «Using ubuntu-latests, macos-latest, or windows-latest will automatically bring you the most recent stable version of the Rust compiler» [CITED: rust.code-maven.com/github-actions + docs.github.com Rust tutorial]. Но **пин обязателен**: `dtolnay/rust-toolchain@stable` (стабильная практика; STACK:54) — детерминизм MSRV-класса (1.85+).
- Кеш: `Swatinem/rust-cache@v2` по `hashFiles('**/Cargo.lock')` [CITED: PITFALLS:182, STACK:54].
- CMake/MSVC на раннере предустановлены (runner images) — whisper-rs-sys соберётся [ASSUMED по составу runner-image; Whisper.cpp CI сам так делает].
- cargo test на Windows: **сегодня не соберётся НИ ОДИН crate** (см. §10). После cfg-гейтинга: `cargo test --no-default-features -p dicta-core -p dictad` — переносимые тесты (audio/config/focus-нормализация) зелёные, Windows-модули компилируются; Linux-батарея остаётся в ci.yml (ubuntu, untouched).
- RQ6 кросс-чек с Linux: MSVC с Linux невозможен; windows-gnu: cpal/enigo/windows компилируются под gnu, но whisper-rs-sys требует C-кросс-сборку whisper.cpp (cmake toolchain) — не стоит того. **Pragmatic: локальной Windows-проверки нет; feedback-цикл = push в windows-ветку → CI.** Единственная локальная гарантия: Linux-сборка/тесты не деградируют (cfg-гейты не ломают unix-путь — гейт сравнения с main из locked 3).

### §10. Что НЕ собирается на Windows сегодня (точный список по живым исходникам)

Греп `cfg(windows)|cfg(unix)|cfg(target_os` по crates/ — **пусто** [VERIFIED: grep этой сессии]: развилок нет вообще.

| Модуль | Блокер | file:line |
|--------|--------|-----------|
| dicta-core/src/capture.rs | `libc::kill(pid, libc::SIGINT)` — нет на Windows | capture.rs:53 |
| dicta-core/src/hotkey.rs | `use std::os::unix::io::RawFd` + `libc::open/poll/read` по /dev/input | hotkey.rs:11,122-178 |
| dicta-core/src/inject.rs | uinput: `libc::open("/dev/uinput")`, ioctl UI_*; wl-copy/wl-paste subprocess | inject.rs:14-27,168-183,307-326,398 |
| dicta-core/src/atspi.rs | фича `atspi` (не default) — zbus/atspi Linux-only; на Windows не включаем | Cargo.toml features |
| dictad/src/main.rs | `std::os::unix::net::{UnixListener, UnixStream}`, `PermissionsExt`, `UnixStream::connect` | main.rs:13-14,148,159-161 |
| dictactl/src/main.rs | `std::os::unix::net::UnixStream::connect` (два места) | dictactl main.rs:113,131 |
| dicta-ui | wayland/layer-shell целиком | crate целиком |
| dicta-settings | net.rs UDS | net.rs |

Переносимые без правок: audio.rs (hound), filter.rs, stt.rs (whisper-rs), config.rs (кроме default_model_path/default_socket_path §8), focus.rs (кроме active_app-ветки). lib.rs требует cfg-гейтинг pub mod hotkey/atspi/inject.

**Оценка объёма порта:** cfg-гейтинг ~6 файлов + 4 новых Windows-модуля (capture ≈150-200 строк, hotkey ≈80, inject ≈250-300 с delayed render, transport-ветка dictad ≈100 + DACL ≈40) + CI-джоба ≈30 строк YAML + спик ≈200 строк. Не считая спика и UAT — примерно 700-900 строк Rust. Split на 2 плана (07-01 сервер+захват+хоткей; 07-02 вставка+спик) — обоснован и соответствует Autonomy notes.

### §11. Паритет-долги (IN-01/IN-02) — подтверждённые точки

- **settingsSnapshotProvider уже обрабатывает error** [VERIFIED: app/lib/core/providers.dart:49-61 — `if (parsed.error != null) { throw StateError('settings reply error: ${parsed.error}'); }`]. Долг 05-IN-05 в этой точке ЗАКРЫТ.
- **Фактический остаток IN-02** — две точки:
  1. `hotkeyConfigTextProvider` берёт `SettingsReply.fromReply(reply).settings` и **молча отбрасывает `error`** кортежа [VERIFIED: app/lib/features/settings/hotkey_card.dart:24-26 — `final snapshot = SettingsReply.fromReply(reply).settings;`], плюс `catch (_)` глотает всё [hotkey_card.dart:27-30].
  2. Поллинг: `catch (_)` без текста → `disconnected` без причины [VERIFIED: providers.dart:90-92].
- IN-01 (set/fetchSettings до живого пайпа) закрывается автоматически появлением сервера этой фазы [VERIFIED: .planning/phases/06-windows-ci-karkas/06-REVIEW-DISPOSITION.md:12-13].
- **Один маленький Flutter-коммит** фазы 7: продвигать error в hotkeyConfigTextProvider (паритет с providers.dart:57-59) и в poll-ветку. Новых виджетов нет (locked 10 подтверждается).

### §12. DACL interprocess: API и SID (RQ4, locked 6)

- API подтверждён [VERIFIED: Context7 /kotauskas/interprocess — официальные доки/исходники]:
  ```rust
  use interprocess::os::windows::named_pipe::{pipe_mode, PipeListenerOptions};
  use interprocess::os::windows::security_descriptor::SecurityDescriptor;
  use widestring::u16cstr;
  let sd = SecurityDescriptor::deserialize(u16cstr!("D:P(A;;GA;;;WD)"))?; // пример из доков: всем GA
  let listener = PipeListenerOptions::new()
      .path(r"\\.\pipe\Example")      // полный путь, НЕ авто-префикс
      .security_descriptor(sd)
      .create_duplex::<pipe_mode::Bytes>()?;
  ```
- **DACL-контракт (locked 6):** SDDL `D:P(A;;GA;;;SY)(A;;GA;;;<SID_пользователя>)` — SYSTEM + только текущий пользователь; НЕ `WD` (everyone) из примера. SID берётся: `OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY)` → `GetTokenInformation(TokenUser)` → `ConvertSidToStringSidW` → в SDDL-строку. widestring — лёгкая утилитарная зависимость (замыкание разрешено locked 4) [VERIFIED: API-вызовы — стандартная Win32-цепочка, CITED: learn.microsoft.com; точные bindings — через windows crate].
- Имя пайпа: `PipeListenerOptions::path` НЕ добавляет `\\.\pipe\` сам [VERIFIED: Context7 — «Pipe name; \\.\pipe\ is **not** auto-prepended»] → передаём полный `r"\\.\pipe\dicta"` — точное совпадение с kDictaPipe клиента [VERIFIED: windows_engine.dart:22 `const kDictaPipe = '\\\\.\\pipe\\dicta';`].
- Режим `pipe_mode::Bytes` (не Messages) — построчная JSON-семантика поверх байтового потока, паритет UDS-BufReader [VERIFIED: dictad serve_conn main.rs:820-823].
- Семантика «одна команда — одно соединение» сохраняется тривиально: accept → BufReader → read_line → ответ → drop (статус рвёт соединение сразу: main.rs:1007-1009 `if req.cmd == "status" { break; }`).

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Named pipe транспорт | CreateNamedPipeW руками | interprocess 2.4 | СDC/DACL/instance-менеджмент уже в API; синхронная модель вписывается в thread-архитектуру |
| Дефицитная децимация 48k→16k | Новый ресэмплер | Существующий `resample_to_16k` (audio.rs:58) | Уже написан, протестирован, вызывается из stop() |
| SendInput-синтез клавиш/текста | Ручные KEYBDINPUT-структуры | enigo 0.6 (`key`, `text`) | Тот же код под капотом; батч KEYEVENTF_UNICODE одним массивом |
| Message loop / RegisterHotKey | Крейты типа global-hotkey | Тонкий поток GetMessageW через windows-rs (~80 строк) | global-hotkey привязан к event-loop приложения; демон — свой цикл, dependencies минимизируем |
| SDDL/DACL | Ручной SECURITY_ATTRIBUTES | `SecurityDescriptor::deserialize` + windows-rs SID-цепочка | Парсер SDDL уже в interprocess |

**Key insight:** всё рискованное в Windows-вставке — НЕ библиотечное (delayed render, UIPI-чек, release-паттерн модификаторов) — это наши ~300 строк поверх windows-rs; библиотеки закрывают только транспорт/синтез/аудио.

## Common Pitfalls (фаза-специфичные, сверх PITFALLS.md)

1. **Запрос 16кГц mono у WASAPI через AUTOCONVERTPCM** — тишина на Win11 24H2 (cpal#1200). Лечение: native default_input_config + существующий resample (§2).
2. **`dirs::data_dir()` = Roaming на Windows** — модели уедут в %APPDATA%, нарушение locked 7/README-обещания. Лечение: cfg-развилка на `data_local_dir()` (§8).
3. **SetClipboardData(NULL) возвращает NULL на успехе** — проверять `GetClipboardOwner`/`IsClipboardFormatAvailable`, не GetLastError (баг «1 из 4 вставок падает», win-text-inject 0.1.1) [CITED: PITFALLS:46].
4. **Vulkan на Windows CI** — SDK отсутствует + тихий нерегистринг бэкенда (#3750). Лечение: `--no-default-features` в Windows-джобе (§7).
5. **Перенос Linux-паттерна «буфер → Ctrl+V → таймер → восстановить»** — воспроизведённый баг-класс (Handy #502). Лечение: только delayed render + settle из спика, таймер лишь fallback-сядов (§5).
6. **MOD_WIN в RegisterHotKey** — «reserved for use by the operating system»: хоткеи с Win-клавишей могут не регистрироваться/конфликтовать. Дефолт-рекомендация пользователю: Alt+клавиша [CITED: MS RegisterHotKey].
7. **Pipe name mismatch** — если взять local_socket-имя «dicta» (namespaced, авто-префикс) вместо полного пути в named_pipe-модуле — клиент dart_ipc с kDictaPipe не найдёт сервер. Лечение: `PipeListenerOptions::path(r"\\.\pipe\dicta")` без авто-префикса (§12).
8. **interprocess 2.4.9 drift** — STACK зафиксировал 2.4.8, 2026-11-18 вышла 2.4.9; API security_descriptor идентичен — планёр пинует любую из двух явно (§ Standard Stack).

## Code Examples

### cpal WASAPI recorder (скелет, паритет Recorder-интерфейса)

```rust
// cfg(windows)-модуль; НЕ часть production до сбора на Windows-машине,
// но compile-гейтится на CI (cargo check --no-default-features -p dicta-core).
// Источники: Context7 /RustAudio/cpal (examples/record_wav.rs, device.rs).
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

pub struct Recorder {
    stream: cpal::Stream,           // drop = остановка
    samples: Arc<Mutex<Vec<i16>>>,   // mono, native rate
    rate: u32,
    started: std::time::Instant,
    overflow: Arc<AtomicBool>,       // переполнение буфера callback'а
}

impl Recorder {
    pub fn start(_cfg: &crate::config::Config) -> anyhow::Result<Self> {
        let host = cpal::default_host();
        let device = host.default_input_device()
            .ok_or_else(|| anyhow::anyhow!("нет устройства ввода (WASAPI)"))?;
        let config = device.default_input_config()?; // native: обычно 44.1/48k
        let rate = config.sample_rate().0;
        let channels = config.channels() as usize;
        let samples = Arc::new(Mutex::new(Vec::<i16>::with_capacity(16_000 * 30)));
        let err_fn = |err| eprintln!("dictad: cpal: {err}");
        let buf = samples.clone();
        let overflow = Arc::new(AtomicBool::new(false));
        let ovf = overflow.clone();
        let stream = match config.sample_format() {
            cpal::SampleFormat::I16 => device.build_input_stream(
                &config.into(),
                move |data: &[i16], _| {
                    let mut out = buf.lock().unwrap();
                    if out.len() > 16_000 * 120 { ovf.store(true, Ordering::Relaxed); return; }
                    for frame in data.chunks_exact(channels) { out.push(frame[0]); } // первый канал
                },
                err_fn, None)?,
            cpal::SampleFormat::F32 => device.build_input_stream(
                &config.into(),
                move |data: &[f32], _| {
                    let mut out = buf.lock().unwrap();
                    if out.len() > 16_000 * 120 { ovf.store(true, Ordering::Relaxed); return; }
                    for frame in data.chunks_exact(channels) {
                        out.push((frame[0].clamp(-1.0, 1.0) * 32767.0) as i16);
                    }
                },
                err_fn, None)?,
            other => anyhow::bail!("cpal: неподдерживаемый формат {other:?}"),
        };
        stream.play()?;
        Ok(Self { stream, samples, rate, started: std::time::Instant::now(), overflow })
    }

    pub fn stop(mut self) -> anyhow::Result<Vec<i16>> {
        self.stream.pause()?; // остановить поток до take()
        let raw = std::mem::take(&mut *self.samples.lock().unwrap());
        Ok(crate::audio::resample_to_16k(raw, self.rate)) // существующая децимация (audio.rs:58)
    }

    pub fn elapsed(&self) -> std::time::Duration { self.started.elapsed() }

    /// Уровень по хвосту буфера — паритет pw-record-версии (capture.rs:79-93),
    /// но без файла: последние ~40 мс (640 сэмплов на 16k после прикидки по rate).
    pub fn level(&self) -> f32 {
        let guard = self.samples.lock().unwrap();
        let win = ((self.rate as usize / 25).max(64)).min(guard.len());
        crate::audio::normalized_rms(&guard[guard.len() - win..])
    }
}
```

### interprocess listener + DACL (по верифицированному Context7-примеру, адаптация под контракт locked 6)

```rust
// Source: Context7 /kotauskas/interprocess — официальный пример + ListenerOptionsExt API.
use interprocess::os::windows::named_pipe::{pipe_mode, PipeListenerOptions};
use interprocess::os::windows::security_descriptor::SecurityDescriptor;
use widestring::{u16cstr, U16CString};

fn current_user_sddl() -> anyhow::Result<interprocess::os::windows::security_descriptor::SecurityDescriptor> {
    // OpenProcessToken → GetTokenInformation(TokenUser) → ConvertSidToStringSidW
    // (windows crate, Win32_Security + Win32_System_Threading features)
    let sid = crate::win_sid::current_user_sid_string()?;
    // DACL-контракт фазы 6: SYSTEM + только текущий пользователь. НЕ WD/everyone.
    let sddl = format!("D:P(A;;GA;;;SY)(A;;GA;;;{sid})");
    Ok(SecurityDescriptor::deserialize(u16cstr!(... /* U16CString::from_str(&sddl) */))?)
}

pub fn serve(pipe_path: &str) -> anyhow::Result<()> {
    let listener = PipeListenerOptions::new()
        .path(pipe_path)                 // полный r"\\.\pipe\dicta", БЕЗ авто-префикса
        .security_descriptor(current_user_sddl()?)
        .create_duplex::<pipe_mode::Bytes>()?;
    for stream in listener.incoming() { /* serve_conn — та же, что UDS-цикл dictad */ }
    Ok(())
}
```

### RegisterHotKey: поток без окна (MS Learn-паттерн, §4)

```rust
// Source: learn.microsoft.com .../nf-winuser-registerhotkey — пример с NULL hWnd.
// Парсинг спеки "alt+v" → MOD_ALT + VK_V — таблица по мотивам hotkey.rs:15-66.
pub fn listen(spec: &str, mut on_press: impl FnMut() + Send + 'static) -> anyhow::Result<()> {
    let (mods, vk) = parse_windows_combo(spec)?; // MOD_ALT|MOD_NOREPEAT, 'V' → VK
    unsafe {
        if windows::Win32::UI::WindowsAndMessaging::RegisterHotKey(
            None, // hWnd = NULL: WM_HOTKEY приходит в очередь ЭТОГО потока
            1,
            mods | MOD_NOREPEAT,
            vk,
        ).as_bool() == false {
            anyhow::bail!("хоткей '{}' занят или недоступен (GetLastError={})",
                spec, windows::Win32::Foundation::GetLastError().0); // честная ошибка
        }
    }
    let mut msg = MSG::default();
    while unsafe { GetMessageW(&mut msg, None, 0, 0).as_bool() } > 0 {
        if msg.message == WM_HOTKEY { on_press(); }
    }
    Ok(())
}
```

### enigo: paste-комбо + Unicode-fallback (Context7-верифицированный API)

```rust
// Source: Context7 /enigo-rs/enigo — Keyboard::key / Keyboard::text (win_impl.rs:323).
use enigo::{Direction::{Click, Press, Release}, Enigo, Key, Keyboard, Settings};

fn send_paste_combo(mut enigo: &mut Enigo) -> Result<(), enigo::InputError> {
    enigo.key(Key::Control, Press)?;
    enigo.key(Key::Unicode('v'), Click)?; // Windows: VK + scan code
    enigo.key(Key::Control, Release)
}

fn type_fallback(mut enigo: &mut Enigo, text: &str) -> Result<(), enigo::InputError> {
    enigo.text(text) // Windows: ОДИН батч SendInput KEYEVENTF_UNICODE press/release
}
```

## Runtime State Inventory

> Фаза — платформенный порт (greenfield на Windows), не rename/migration: инвентарь ниже подтверждает, что(runtime)-состояния, которое фаза могла бы сломать, нет.

| Category | Items Found | Action Required |
|----------|-------------|------------------|
| Stored data | Windows-стороны нет (порт новый); Linux-сторону фаза не трогает (locked 3) | None — verified: grep cfg-развилок пуст, Windows-кода в репо нет |
| Live service config | Windows-пайп `\\.\pipe\dicta` ещё не существует ни на одной машине (сервер появляется в этой фазе) | None — verified: сервер = новый код фазы 7 |
| OS-registered state | Windows: регистрации нет (dictad.exe не ставился); Linux systemd-юниты не затрагиваются | None |
| Secrets/env vars | `DICTA_PIPE` (новый env-шов, паритет DICTA_SOCK) + `DICTA_HOTKEY` (существует, main.rs:293) | Только документация; секретов нет |
| Build artifacts | Windows-артефакт CI (zip фазы 6) не содержит dictad.exe — Rust-джобы не было | Новый артефакт: dictad.exe в zip — задача фазы |

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework (Rust) | cargo test 1.99 (workspace, существующий) |
| Framework (Dart) | flutter test — батарея 194/194 (32 файла), analyze 0, cyrillic 0 [VERIFIED: CONTEXT.md:26] |
| Config file | crates: нет (стандартные #[cfg(test)] модули в исходниках); app: app/test/ |
| Quick run (Linux) | `cargo test --workspace && (cd app && flutter test)` |
| Full suite | то же + `cargo build --release --locked --workspace` (паритет ci.yml) |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| WIN-03 | Захват cpal: Recorder-интерфейс возвращает 16кГц mono, level>0 | unit (структура) + CI compile + manual-аудио | `cargo test --no-default-features -p dicta-core` (после гейтинга); реальный звук — UAT | ❌ Wave 0: `crates/dicta-core/src/capture_windows.rs` + cfg-тесты |
| WIN-03 | RegisterHotKey: парсинг спеки "alt+v" → (MOD_ALT\|MOD_NOREPEAT, VK_V) | unit (чистая функция парсинга, без Win32) | `cargo test -p dicta-core hotkey_win::tests` | ❌ Wave 0 |
| WIN-03 | whisper CPU-only на Windows собирается | CI compile | `cargo build --no-default-features -p dictad` (windows-latest) | ❌ Wave 0: windows-build.yml Rust-джоба |
| WIN-04 | SDDL-строитель DACL: содержит SY и user-SID, не WD | unit (строка SDDL) | `cargo test -p dictad win_transport::tests` | ❌ Wave 0 |
| WIN-04 | enigo-комбо: порядок Press/Click/Release (мок-трейт) | unit | `cargo test -p dicta-core inject_win::tests` | ❌ Wave 0 |
| WIN-04 | Delayed render: settle/форматы/двойной рендер Chromium | **manual (спик)** — GUI-эксперимент невозможен на CI | протокол §5, `spike/delayed_render`, результат → 07-SPIKE-RESULT.md | ❌ Wave 0: spike-бин |
| WIN-04 | 4 кейса вставки (обычное/elevated/зажатый/Chromium) | **manual-only (UAT)** — физически на Windows-машине | протокол UAT (CONTEXT:63) | n/a |
| протокол | JSON-протокол не изменён | golden/фрейминг-тесты существующие | `flutter test` (windows_engine_framing_test.dart уже есть, фаза 6) | ✅ |
| locked 3 | Linux не деградировал | CI-гейт | ubuntu-джоба ci.yml байт-в-байт + `git diff main -- crates/` на cfg-гейты | ✅ (ci.yml существует) |

### Sampling Rate

- **Per task commit:** `cargo test --workspace` (Linux) + `cargo check` новых cfg-модулей недоступен локально — потому минимум: `cargo build --workspace && cargo test --workspace` на Linux после каждого Rust-таска.
- **Per wave merge:** push в windows-ветку → windows-build.yml (Flutter + НОВАЯ Rust-джоба) зелёный.
- **Phase gate:** полный UAT-прогон на Windows-машине пользователя (спик-протокол + 4 кейса вставки + живая диктовка) — до `/gsd-verify-work`.

### Wave 0 Gaps

- [ ] `crates/dicta-core/src/capture_win.rs` (cpal) + cfg-гейт в lib.rs/capture.rs — покрывает WIN-03
- [ ] `crates/dicta-core/src/hotkey_win.rs` (RegisterHotKey) — WIN-03; парсинг-тесты чистой функцией
- [ ] `crates/dicta-core/src/inject_win.rs` (enigo + delayed render) — WIN-04; ТОЛЬКО после 07-SPIKE-RESULT.md (Must-NOT 8)
- [ ] `crates/dictad/src/transport_win.rs` (interprocess + DACL) + SDDL-тест — WIN-04-смежный/DACL-контракт
- [ ] `.github/workflows/windows-build.yml` + Rust-джоба (`dtolnay/rust-toolchain@stable`, `Swatinem/rust-cache@v2`, `cargo build/test --no-default-features -p dicta-core -p dictad`) — CI-гейт
- [ ] `spike/delayed_render` bin + 07-SPIKE-RESULT.md — предусловие inject_win
- [ ] Flutter-мини-фикс IN-02: hotkeyConfigTextProvider error-ветка + poll-текст (hotkey_card.dart:24-31, providers.dart:90) — `flutter test` остаётся 194+/194+

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| rustc (локальный Linux) | сборка/тесты Linux-гейтов | ✓ | 1.99.0 (≥ MSRV 1.85) | — |
| cpal WASAPI (windows-latest) | захват | ✓ (через CI) | 0.18.2 | — |
| whisper-rs CPU-only (windows-latest) | STT | ✓ предпосылки (CMake/MSVC на раннере) | 0.16/0.15 | Vulkan на Windows — нет (сломан upstream, §7) |
| Vulkan SDK на раннере | GPU на Windows | ✗ | — | CPU-only (`--no-default-features`) — рекомендовано |
| Windows-машина пользователя | спик + UAT (4 кейса + живая диктовка) | ✓ (по CONTEXT: autonomia) | — | Иного нет: GUI-эксперимент только там |
| windows-gnu кросс-чек с Linux | быстрая проверка типов | ✗ (не ставлен; whisper C-кросс делает его непрактичным) | — | CI-push как цикл фидбека |

**Missing dependencies with no fallback:** нет блокирующих (CPU-only покрывает сборку; GUI-проверки по определению UAT).

## Security Domain

### Applicable ASVS Categories (фаза-специфично)

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V4 Access Control | yes | DACL пайпа: SDDL `D:P(A;;GA;;;SY)(A;;GA;;;<owner-SID>)` — только SYSTEM и текущий пользователь (locked 6); preempt-спуф закрыт тем, что клиент коннектится только к существующему серверу с DACL |
| V5 Input Validation | yes | существующий parse_change (dictad main.rs:497-610) не меняется; inject-лимит 5000 симв. (main.rs:886) сохраняется |
| V12 File Ops | partial | пути %APPDATA%/%LOCALAPPDATA% через dirs; аудио в памяти, НЕ на диске (Must-NOT 5: Windows-путь даже чище pw-record-темпов) |

### Known Threat Patterns (перенос из 06-SECURITY + новые)

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Preempt-спуф пайпа (клиент к фальшивому серверу) | Spoofing | DACL-контракт: сервер создаёт пайп с owner-SID descriptor; валидация остаётся клиентской проблемой фазы 6 → закрыт сервером этой фазы |
| Транскрипты в Win+V истории / облаке MS | Information Disclosure | 4 оптаут-формата на каждый транскрипт (§5 п.2) [CITED: PITFALLS:233] |
| Чтение чужих клавиш (evdev-класс) | Information Disclosure | RegisterHotKey НЕ читает поток клавиш вообще [CITED: PITFALLS:236] — Windows-путь безопаснее Linux-evdev |
| Логи payload | Information Disclosure | паритет T-контрактов: eprintln без текста диктовки (существующий паттерн dictad частично нарушим — вставка логирует text в main.rs:913,1179-1184: **на Windows-ветке не воспроизводить текст в логах** — Must-NOT 5 строже существующего поведения; пометка планёру) |
| UIPI как «фича» | Elevation | честный отказ на elevated-цели, не тихий успех (§6) |

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | windows-latest раннер — headless, clipboard/GUI-эксперимент невоспроизводим | §5 | Если воспроизводим — спик можно автоматизировать; план от этого только выигрывает (остаётся валидным) |
| A2 | Код GetLastError при занятом хоткее = 1409 (ERROR_HOTKEY_ALREADY_REGISTERED) | §4 | Число может отличаться; поведение («fails if already registered») — CITED, для кода хватит честного текста ошибки |
| A3 | CMake/MSVC на windows-latest предустановлены (whisper-rs-sys соберётся) | §9 | Если нет — шаг установки в workflow; известно, что runner-image содержит VS+CMake |
| A4 | Windows Terminal принимает Ctrl+V (и Ctrl+Shift+V) | §6 | Влияет на paste_keys_by_app дефолты Windows; замеряется в UAT |
| A5 | CPU-only medium-q5_0 приемлем по латентности на машине пользователя | §7 | Если медленно — пользователь меняет модель/beam через set (не блокер фазы) |
| A6 | interprocess 2.4.9 API-совместим с 2.4.8 (security_descriptor идентичен) | Stack | Если нет — пин 2.4.8 (locked 4 буквально) |
| A7 | `cfg.mic` (PipeWire source) на Windows не применим; MVP = дефолтный микрофон | §2 | Если нужен выбор микрофона — матчинг по имени cpal-устройства (малая доработка) |
| A8 | dart_ipc-клиент совместим с interprocess-сервером в pipe_mode::Bytes (построчная семантика) | §12 | Риск совместимости transport-пар; проверяется первым же UAT-пунктом U-3 из 06-UAT (поглощается фазой 7) |

## Open Questions

1. **WM_RENDERFORMAT в message-only окно** — доки гарантируют доставку direct-сообщений, но не фиксируют прямо случай clipboard-owner. Что делаем: пункт №7 спик-протокола (message-only → fallback скрытое окно с логом, какой сработал). Планировать inject_win с параметризуемым типом окна до результата спика.
2. **Латентность CPU-inference на Windows-машине пользователя** — та же модель (locked 4), GPU нет (Vulkan сломан). Замер в UAT; если неприемлемо — осознанное решение пользователя по конфигу (n_threads/beam/модель), не архитектурная правка.
3. **interprocess 2.4.8 vs 2.4.9** — планёр выбирает пин (locked 4 писал 2.4.8 до релиза 2.4.9; API-различий в security_descriptor не выявлено).
4. **Дефолт hotkey-спеки на Windows** — Linux-дефолт "off" (config.rs:90); пользователь должен задать сам (DICTA_HOTKEY env или конфиг). Рекомендация: оставить "off" + документация, честная ошибка при занятости (§4).
5. **Порядок двух планов** (Autonomy: split допустим) — рекомендация: 07-01 (cfg-гейт + capture + hotkey + transport/DACL + CI Rust-джоба + IN-02 Flutter-фикс) → 07-02 (спик → inject_win → UAT). Depends_on очевиден: вставка последней, спик — её предусловие.

## Sources

### Primary (HIGH confidence)
- Живые исходники этой сессии: `crates/dicta-core/src/{audio,capture,config,filter,focus,hotkey,inject,stt,lib}.rs`, `crates/dictad/src/main.rs`, `crates/dictactl/src/main.rs`, Cargo.toml (workspace + dicta-core + dictad), `git show windows:.github/workflows/windows-build.yml`, `git show windows:app/lib/core/windows_engine.dart`, `app/lib/core/{providers,settings}.dart`, `app/lib/features/settings/hotkey_card.dart`
- Context7 `/kotauskas/interprocess` — PipeListenerOptions/ListenerOptionsExt::security_descriptor, SecurityDescriptor::deserialize, security_descriptor поле, path без авто-префикса [VERIFIED]
- Context7 `/enigo-rs/enigo` — Keyboard::key/raw/text (win_impl.rs:323: батч SendInput KEYEVENTF_UNICODE), Key::Unicode, InputError [VERIFIED]
- Context7 `/RustAudio/cpal` — supported_input_configs (wasapi/device.rs: только дефолтные каналы, IsFormatSupport нативно), record_wav.rs, build_input_stream [VERIFIED]
- crates.io API: cpal 0.18.2 (2026-08-16), enigo 0.6.1 (2025-08-28), interprocess 2.4.9 (2026-11-18) [VERIFIED]
- learn.microsoft.com: RegisterHotKey (hWnd=NULL → очередь потока; MOD_*; F12 reserved), Clipboard Operations (delayed rendering/WM_RENDERFORMAT), Window Features (message-only окна), docs.rs/dirs/6.0.0 (data_dir = RoamingAppData на Windows)
- gsd-tools `package-legitimacy check --ecosystem crates cpal enigo interprocess` → все OK [VERIFIED]

### Secondary (MEDIUM confidence)
- whisper.cpp#3750 (Vulkan silent non-registration в Windows MSVC static, whisper-rs 0.16.0), whisper.cpp#3973 (vulkan-shaders-gen ExternalProject fail) — GitHub issues, точные наши версии в описании
- rust.code-maven.com + docs.github.com — windows-latest предустановленный stable Rust
- cpal#1200 — AUTOCONVERTPCM-тишина на Win11 24H2 (через PITFALLS.md:217)

### Tertiary (конвергентные)
- Handy #502, OpenWhispr #829, FluidVoice #940, win-text-inject — тайминги delayed render, UIPI, зажатые модификаторы (4+ независимых, через PITFALLS.md)

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — версии+API сверены (crates.io, Context7, seam-legitimacy), rustc 1.99 ≥ MSRV
- Architecture: HIGH — живое чтение всех затрагиваемых модулей (file:line); объём порта посчитан по реальным строкам
- Pitfalls: MEDIUM-HIGH — документальная часть CITED (MS Learn); поведенческие тайминги delayed render закрываются обязательным спиком (locked 5)
- Windows-вставка до спика: MEDIUM by design — план обязан идти через 07-SPIKE-RESULT.md

**Research date:** 2026-11-22
**Valid until:** 2027-01-22 (stable; interprocess/cpal точки версий перепроверить при исполнении позже 30 дней)



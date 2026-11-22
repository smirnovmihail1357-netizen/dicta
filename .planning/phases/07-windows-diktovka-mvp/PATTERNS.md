# Phase 7: Windows диктовка MVP — Pattern Map

**Mapped:** 2026-11-22
**Files analyzed:** 8 точек (транспорт, cfg-развилки, хоткей, вставка, аудио, DACL, CI, Flutter-долг)
**Analogs found:** 6 с аналогом / 8 (2 честных No Analog: cfg-развилки в Rust, DACL/SDDL)
**Ветка анализа:** main (crates идентичны windows@23049d6 — `git diff main windows -- crates` пуст); Flutter-файлы читались с ветки windows.

## Живые факты, закрывающие вопросы CONTEXT

- **Аудио-бэкенд СЕЙЧАС — НЕ cpal**: `capture.rs` спавнит сабпроцесс `pw-record` (PipeWire CLI), стопит SIGINT, читает temp-WAV через `hound`. Win-ветка захвата = новый cpal-Recorder с тем же API. `resample_to_16k` уже существует (линейная децимация) — закрывает research-Q2 про 48 кГц устройства.
- **cfg(windows)/cfg(unix)/cfg(target_os) в Rust — НОЛЬ вхождений** (grep по crates). Код сейчас компилируем только на Linux: `libc` в dicta-core (inject/hotkey/capture), `std::os::unix` в dictad, wayland-стек в dicta-ui. Первая Windows-сборка вскроет все места разом.
- **rustc локально 1.99.0** — MSRV 1.85 (cpal/enigo) покрыт с запасом.
- **dicta-ui не собирается на Windows в принципе** (smithay-client-toolkit + wayland-client в Cargo.toml) — CI-джоба НЕ может быть `cargo test --workspace`; `dictactl` тоже юникс-транспорт (UDS) — решать планёру: cfg-гейт или исключить из Win-джоба.
- **IN-02 (error-поле settingsSnapshotProvider) уже закрыт на обеих ветках** (providers.dart:49-61, diff main↔windows отличается только engineProvider). Реальный остаток долга — 06-REVIEW IN-03 (устаревший doc-комментарий) + IN-01 (покрытие set/fetchSettings тестами) — см. точку 8.

## File Classification

| Новое/меняемое | Role | Data Flow | Closest Analog | Match |
|---|---|---|---|---|
| dictad: named pipe listener (interprocess) | transport/server | request-response (JSON-line) | `crates/dictad/src/main.rs:330-346, 810-1012` | exact (роль+поток) |
| cfg(windows)/cfg(unix) развилки | build-config | — | **No Analog в Rust**; дисциплина — Flutter `Platform.isWindows` fork (windows:providers.dart:18-20) | дисциплина |
| dicta-core: hotkey Windows (RegisterHotKey) | hook (блокирующий слушатель) | event-driven | `crates/dicta-core/src/hotkey.rs:131-204` + интеграция `dictad/main.rs:291-323` | role-match |
| dicta-core: inject Windows (enigo + delayed render) | service/adapter | transform + side-effect | `crates/dicta-core/src/inject.rs:77-107, 330-383, 386-442` | role-match |
| dicta-core: capture Windows (cpal/WASAPI) | service/adapter | streaming → batch | `crates/dicta-core/src/capture.rs:12-98` (API-контракт) + `audio.rs:57-74` (ресэмпл) | role-match |
| dictad: DACL/SDDL пайпа | config/security | — | **No Analog** (семантический якорь: `main.rs:161` 0o600 сокета) | none |
| .github/workflows: Rust-джоба на windows | CI | batch | `.github/workflows/ci.yml:1-39` + windows:`windows-build.yml:1-38` | exact |
| app/lib: settingsSnapshotProvider error-долг | provider (Flutter) | request-response | `app/lib/core/providers.dart:49-61`, windows:`windows_engine.dart:32-36` | exact |

---

## Pattern Assignments

### Точка 1 — Windows-транспорт демона (interprocess named pipe listener)

**Analog: `crates/dictad/src/main.rs`** — принять-цикл и serve_conn:

**Accept-цикл** (330-346):
```rust
for stream in listener.incoming() {
    let Ok(stream) = stream else { continue };
    let handler = handler.clone(); /* … Arc-клоны state/level/started_at/panel/reloading … */
    thread::spawn(move || {
        if let Err(err) = serve_conn(stream, handler, state, level, started_at, panel, reloading, socket2) {
            eprintln!("dictad: {err}");
        }
    });
}
```
Копировать по смыслу: одно соединение — один тред, все Arc<Mutex>/Atomic клоныруются в тред, ошибка соединения — eprintln, цикл не умирает.

**serve_conn — JSON-line протокол** (810-1012, ключевые строки):
- 820-823: `BufReader::new(stream.try_clone()?)` + `read_line` — на пайпе то же самое (interprocess даёт Read/Write);
- 823-824: `serde_json::from_str(line.trim())` в `Req` — структура `Req`/`Res`/`Reply` (25-59, 352-359) НЕ меняется (locked 2);
- 827-992: `match req.cmd` — весь блок команд платформо-независим, не трогать;
- 995-1002: `serde_json::to_string(&Reply{base, panel, settings})` → `write_all` + `b"\n"` + `flush()` — байт-в-байт;
- 1004-1009: «одна команда — одно соединение»: `if req.cmd == "status" { break; }` — сохранить семантику (CONTEXT:23 разрешает держать соединения, но проще паритет).

**Подготовка слушателя, семантические аналоги** (140-161): путь из конфига → env-шов. На Windows: `DICTA_PIPE` env → `\\.\pipe\dicta` дефолт (паритет `DICTA_SOCK`, locked 7; имя пайпа уже живёт во Flutter `kDictaPipe`, windows:windows_engine.dart:23). `std::fs::set_permissions(socket, 0o600)` (161) — семантический аналог DACL пайпа (точка 6).

**НЕ копировать из аналога:**
- 143-157: stale-сокет проба (`UnixStream::connect` → `remove_file`) — именованные пайпы не переживают владельца, в Win-ветке не нужна;
- `std::os::unix::net::{UnixListener, UnixStream}` (12-14) — уходят под `#[cfg(unix)]`, Win-ветка — interprocess `ListenerOptions`/`named_pipe` (STACK:44, sync API, без tokio — вписывается в тред-архитектуру).

**Конвенции аналога:** anyhow + `with_context` с русскими строками; eprintln-префикс `dictad: `; прогрев модели в треде (181-201) и watchdog 80 мс (210-273) — платформо-независимы, не трогать.

---

### Точка 2 — cfg(windows)/cfg(unix) развилки

**No Analog в Rust.** Grep `cfg\((windows|unix|target_os)` по crates — 0 совпадений. Всё платформенное сейчас безусловное.

**Аналог по дисциплине — Flutter-фаза 6 (ветка windows):** ровно ДВЕ точки развилки на весь app/lib:
- `app/lib/core/providers.dart:18-20` — `Platform.isWindows ? WindowsPipeEngine() : DaemonEngine(...)`;
- `app/lib/core/config_path.dart:29` — Win-ветка ДО юникс-XDG-резолюции, с комментарием-контрактом для фазы 7.

**Рекомендуемая форма для Rust** (минимальные точки, модульный уровень):
```rust
// dicta-core/src/lib.rs
#[cfg(unix)]    pub mod hotkey;   // evdev-реализация остаётся как есть
#[cfg(windows)]  pub mod hotkey_win;
#[cfg(unix)]    pub mod inject;   // uinput/wl-clipboard
#[cfg(windows)]  pub mod inject_win; // enigo + delayed render
#[cfg(unix)]    mod capture_pw;   #[cfg(windows)] mod capture_cpal;
```
- Общий API сохраняется (напр. `hotkey::listen(spec, on_press)`, `insert_text(...)`, `Recorder::start/stop`), тела за cfg. dictad main.rs: только транспорт-блок (12-14, 140-161, 330-346) и вызов хоткея (298-322) за cfg; Handler/команды/watchdog общие.
- `atspi` в dicta-core уже feature-gated (`atspi = ["dep:atspi"...]`, Cargo.toml) — НЕ включать на Windows.
- **Гейт регресса (locked 3):** Linux-сборка байт-в-байт — `cargo build --release --locked` и тесты на Linux CI обязаны остаться зелёными без diffs в юникс-путях (сравнение поведения с main).

**Запрет по аналогии:** Flutter-фаза 6 не размазала `Platform.isWindows` по features/ — Rust-фаза тоже: развилки только в core-транспорте, не в UI-логике команд.

---

### Точка 3 — Хоткей Windows (RegisterHotKey + message-only окно)

**Analog 1: `crates/dicta-core/src/hotkey.rs`** — контракт слушателя:
- 131-137: `pub fn listen(spec: &str, mut on_press: impl FnMut()) -> std::io::Result<()>` — блокирующий цикл до конца процесса; невалидный spec → честная ошибка `InvalidInput` с русской строкой;
- 73-103: `parse_combo` — парсер spec-строки (`"alt+v"`); на Windows мапить в `MOD_*` + VK-коды (RegisterHotKey принимает модификаторы+VK, отдельные левые/правые не нужны — у Win один флаг на модификатор);
- 139-144: недоступные устройства → `PermissionDenied` с actionable подсказкой — Win-аналог: `RegisterHotKey` вернул false (комбо занято) → честная ошибка «комбинация занята», НЕ молчание (PITFALLS:252).

**Analog 2: интеграция в демона — `crates/dictad/src/main.rs:291-323`:**
```rust
let hotkey_spec = std::env::var("DICTA_HOTKEY").unwrap_or_else(|_| cfg.hotkey.clone());
if hotkey_spec != "off" {
    thread::spawn(move || { /* listen + колбэк do_toggle */ });
}
```
Копировать по смыслу: порядок env → конфиг → `"off"`-гейт; колбэк = `do_toggle(&handler, &state, &started_at)` (1208-1225) — он сам не блокирует (run_stop уводит работу в воркер); ошибки колбэка — eprintln, не паника; backoff-цикл с удвоением до 900 с (299-319) — Win-аналог: не нужен retry-поллинг, но нужен поток с `GetMessageW`-циклом message-only окна (`CreateWindowExW` с `HWND_MESSAGE`), живущий до конца процесса; окно скрытое = «без видимого UI» (locked 8, must-NOT 3).

**НЕ копировать:** evdev-механику — `open_devices` (106-128), `libc::poll` (163), разбор `input_event` (176-201), левые/правые партнёры модификаторов (91-103), подсказки про группу `input`/udev (313-316). Хоткеи в Flutter-приложении НЕ регистрировать (must-NOT 2).

---

### Точка 4 — Вставка Windows (enigo SendInput + clipboard delayed render)

**Analog: `crates/dicta-core/src/inject.rs`** — структурный каркас вставки:

1. **Combo::parse** (77-107): парсер `"ctrl+v"` — переиспользовать как есть (ASCII-ключи ctrl/shift/alt/v/c/insert/enter; русские bail!-строки). Ключи вставки выбирает `cfg.paste_keys_for(app)` (config.rs:235-239) — общий код, не трогать.
2. **wait_clipboard_ready** (330-341): поллинг-подтверждение вместо слепой паузы; `paste_delay_ms` — ПОТОЛОК ожидания, не фиксированная задержка (комментарий 400-403). Это главная конвенция для Win-ветки: после `SetClipboardData(NULL)` (promise) ждать не таймером, а события `WM_RENDERFORMAT` целевого окна; settle ~400 мс после первого рендера — из PITFALLS:44, уточнить спиком (locked 5/8).
3. **schedule_restore** (345-383): одна отложенная работа на процесс (`static Mutex<Option<PendingRestore>>`), новая вставка ПРОДЛЕВАЕТ дедлайн и сохраняет исходный буфер, восстановление в фоновом треде (поллинг 50 мс), демон не ждёт. На Windows delayed render заменяет это целиком (promise-механика решает гонку таймера — PITFALLS:264), но паттерн «одна работа + дедлайн + фоновый тред» годится для settle-таймера после рендера.
4. **paste_text** (386-412): последовательность [сохранить прежний буфер → положить новый → ПОДТВЕРДИТЬ → послать клавиши → восстановить в фоне]. Win-ветка та же драматургия: [SetClipboardData(NULL)+WM_RENDERFORMAT-promise → settle-wait → enigo paste-combo → release-паттерн модификаторов].
5. **insert_text** (414-442): диспетчер режимов `"paste" | "clipboard" | "stdout" | "atspi"` — сохранить; `atspi`-ветку за `#[cfg(unix)]`/feature (она уже вызывает `crate::atspi`, которого на Win нет).
6. **test_keys** (444-457): диагностика из CLI (`dictactl test-keys`) — сохранить форму для UAT-отладки вставки.

**Вставка клавиш:** enigo 0.6.1, `SendInput` + `KEYEVENTF_UNICODE` (STACK:46); зажатые модификаторы пользователя — release-паттерн перед вставкой (PITFALLS:57-63); UIPI (elevated-цель) — SendInput тихо игнорируется → детектировать и отдавать ЧЕСТНУЮ ошибку в `last_error` (главный ответ несёт `"error"` — main.rs:923-931), не «вставка успешна».

**НЕ копировать:** весь uinput-слой — константы 14-66, `Keyboard`/`with_keyboard` (109-305), тайминги 15/300/600 мс (это время регистрации uinput-девайса в KWin, к Win неприменимо); `wl-copy`/`wl-paste` (`clipboard_cmd` 307-326 — Win-ветка delayed render,外部 процессы не нужны); MIME-комментарии про `text/plain` (395-398) — на Win форматы CF_UNICODETEXT + оптаут-форматы (PITFALLS:297).

**Тайминги-константы для Win-ветки:** settle ~400 мс (спик), всё остальное — из конфига (`clipboard_restore_ms`, `paste_delay_ms` — уже EDITABLE_KEYS, config.rs:142-160).

---

### Точка 5 — Аудио Windows (cpal/WASAPI)

**Факт (закрывает CONTEXT:24):** сейчас бэкенд — сабпроцесс `pw-record`, НЕ cpal.

**Analog: `crates/dicta-core/src/capture.rs`** — API-контракт Recorder (12-98):
```rust
pub struct Recorder { /* … */ }
impl Recorder {
    pub fn start(cfg: &Config) -> Result<Self>            // 19-47
    pub fn stop(mut self) -> Result<Vec<i16>>             // 50-72 → resample_to_16k на выходе
    pub fn elapsed(&self) -> Duration                     // 74-76 (для seconds в status)
    pub fn level(&self) -> f32                            // 79-93 (~40 мс хвост → normalized_rms)
}
impl Drop for Recorder { /* чистка */ }                    // 100-106
```
Win-ветка = cpal-реализация с ЭТИМ API; dictad не замечает подмены (run_start/run_stop, watchdog, level-индикатор — всё поверх контракта).

- **stop()**: pw-record стопится SIGINT и дожидается ≤2 с (55-66) — cpal-аналог: остановить стрим, собрать накопленный буфер (deadlines-паттерн 55-66 годится: try_wait→sleep(20ms)→kill).
- **level()**: сейчас читает недописанный WAV-хвост (80-92) — cpal-версия берёт последние ~1280 сэмплов из кольцевого/канального буфера и тот же `audio::normalized_rms`. Без temp-файла (must-NOT 5 — на диске ничего не хранить; WAV-файл — Linux-артефакт pw-record, Win-ветке не нужен).
- **Частота:** запрашивать 16 кГц mono s16 у WASAPI; если устройство отдаёт только 48 кГц — `audio::resample_to_16k` (audio.rs:57-74) УЖЕ есть и вызывается в stop() (71) — research-Q2 закрыт существующим кодом; whisper-путь (`stt.rs:67-109`, `to_f32` audio.rs:77-79) полностью платформо-независим.

**НЕ копировать:** `Command::new("pw-record")` + аргументы `--rate/--channels/--format` (28-33), SIGINT-комментарий, `wav_header` (128-146 — только для debug-`record_to`).

---

### Точка 6 — DACL/SDDL конфигурация пайпа

**No Analog Found.** В репо нет ни одного security-descriptor/SDDL-кода.

| Файл | Role | Причина |
|---|---|---|
| dictad: создание пайпа с DACL | config/security | Первый Win-специфичный security-код в проекте |

**Семантические якоря (не код):**
- `dictad/main.rs:161` — `set_permissions(socket, 0o600)`: Linux-аналог «только владелец»; Win-эквивалент — SDDL в `ListenerOptions::security_descriptor` (interprocess 2.4.8, STACK:44);
- windows:windows_engine.dart:10-13 — уже записанный КОНТРАКТ (DACL phase 7: только текущий пользователь; клиент не создаёт пайп);
- 06-SECURITY residual: preempt-спуф пайпа закрывается именно DACL (locked 6).

**Минимальный подход:** SDDL вида `D:P(A;;GA;;;SY)(A;;GA;;;<user-sid>)` — owner-SID текущего пользователя + SYSTEM, без ALL_APPLICATION_PACKAGES и без общих групп; SID берётся у текущего токена процесса. Никаких новых зависимостей сверх interprocess (SDDL-строка — просто строка). Рекомендация: выделить функцию `pipe_security_descriptor() -> String` в Win-модуле транспорта (точка 1), юнит-тест на стабильность строки + документирование контракта в README (якорь фазы 6).

---

### Точка 7 — CI Rust-джоба на Windows

**Analog 1: `.github/workflows/ci.yml`** (main, Linux): структура джобы — container `rust:1-trixie`, `cargo build --release --locked --workspace`, CPU-only прогон `--no-default-features`, `cargo test --workspace`, отдельный шаг синтакс-проверок.

**Analog 2: ветка windows `.github/workflows/windows-build.yml`** (git show windows:.github/workflows/windows-build.yml): `runs-on: windows-latest`, `timeout-minutes: 30` (RESEARCH A3 + headroom, строка 11), `actions/checkout@v4`, пины версий (flutter 3.50.1), `workflow_dispatch`, artifact upload.

**Копировать по смыслу для новой Rust-джобы:**
- форма джоба из ci.yml: build --locked + тесты + CPU-only прогон;
- форма раннера из windows-build.yml: `windows-latest`, `timeout-minutes: 30+`, checkout@v4;
- **тулчейн пин** (аналог flutter-version: 3.50.1): dtolnay/rust-toolchain с версией ≥1.85 (MSRV cpal/enigo); локально 1.99.0 — пинать CI-версию явно, НЕ плавающий `rust:1`;
- whisper.cpp на windows-раннере: cmake/clang предустановлены на GH-раннерах; Vulkan-фича может не собраться без SDK → сначала прогон `--no-default-features` (CPU), Vulkan-прогон по желанию;
- **SCOPE: НЕ `--workspace`** — `dicta-ui` (wayland-client, smithay) не собирается на Windows в принципе; `dictactl` — юникс-транспорт. Джоба: `cargo build --locked -p dicta-core -p dictad` + `cargo test -p dicta-core -p dictad` (после cfg-гейтов точки 2), либо `-p dictactl` добавить после его гейта;
- шаг Linux-паритета остаётся в ci.yml нетронутым — это и есть гейт «Linux байт-в-байт» (locked 3).

**НЕ копировать:** apt-get/system-deps шаг (7-16, pipewire/wl-clipboard/vulkan-dev — Linux-специфика), `bash -n` шаг, container: rust-image (на windows-latest тулчейн ставится action-ом).

---

### Точка 8 — Flutter-долг: settingsSnapshotProvider error-поле

**Факт:** основная часть IN-02 (фаза 2) УЖЕ закрыта на обеих ветках — `app/lib/core/providers.dart:49-61`: `SettingsReply.fromReply` возвращает `(settings, error)`, провайдер повышает `parsed.error` в `throw StateError` → AsyncError. Diff main↔windows по этому файлу — только engineProvider (18-20).

**Остаток долга фазы 7 (из 06-REVIEW):**

1. **IN-03** — устаревший doc-комментарий `providers.dart:52-56`: «…may propagate a raw SocketException» → на Windows это `_PipeTransportException` (windows:windows_engine.dart:45-49). Фикс одной строкой комментария: «…a raw transport IOException (SocketException on Linux, _PipeTransportException on Windows) or TimeoutException…».
2. **IN-01** — покрытие set/fetchSettings Windows-ветки тестами: фейк-канон — `app/test/windows_engine_test.dart` (roundtrip через dart_ipc UDS-бинд, destroyCount, budget-в сообщении). Три теста по 06-REVIEW: set с валидным reply (note+settings), set к несуществующему пути (error без payload), fetchSettings к молчащему фейку (TimeoutException эскалирует — контракт settingsSnapshotProvider). «Автоматично при живом демоне» (CONTEXT:27) — но тесты пишутся против фейка, как учёл 06-DISPOSITION.
3. (Опционально, из 05-REVIEW IN-05) — `hotkeyConfigTextProvider` (`app/lib/features/settings/hotkey_card.dart:21`) игнорирует error-поле ответа демона; паритет — как в settingsSnapshotProvider:49-61.

**Конвенции аналога (windows_engine.dart):** русские runtime-строки ТОЛЬКО через `\uXXXX`-эскейпы (32-36 — cyrillic-gate считает байты и в комментариях); ошибки без payload (`'${error.runtimeType}'`, 111-113); различение «таймаут» vs «протокольная ошибка» vs «транспорт» (119-125); таймауты 2/30/120 с как `static const Duration`.

---

## No Analog Found

| Что | Причина | Минимальный подход |
|---|---|---|
| cfg(windows)/cfg(unix) развилки в Rust | В crates ноль cfg-вхождений; платформенный код безусловный | Модульные cfg-гейты (точка 2): транспорт/hotkey/inject/capture за cfg, общий API; дисциплина «минимум точек» из Flutter-фазы 6 |
| DACL/SDDL пайпа | Первый security-descriptor код в проекте | SDDL-строка `D:P(A;;GA;;;SY)(A;;GA;;;<user-sid>)` в Win-модуле транспорта + README-якорь «named pipe DACL» (контракт уже записан в windows_engine.dart:10-13) |
| cpal-захват | Текущий бэкенд — pw-record CLI, не cpal | Новый Recorder с тем же API (capture.rs:12-98); resample уже есть; см. точку 5 |
| delayed render (SetClipboardData(NULL)+WM_RENDERFORMAT) | В репо нет Win32-кода; спик обязателен ДО production-кода (locked 5) | Документарный спик + замер протокола вставки (CONTEXT:51); каркас — inject.rs-драматургия (точка 4) |

## Shared Patterns (обязательные конвенции фазы)

1. **anyhow + русский контекст:** `Result`/`Context` повсюду; ошибки — русские строки с actionable-подсказкой (`main.rs:150, 160`; `hotkey.rs:135-143`); лог-префикс `dictad: ` в eprintln.
2. **Честные ошибки вместо молчания:** занятая комбинация RegisterHotKey → внятная ошибка; UIPI-провал вставки → `"error"` в ответе (главный Res.error, main.rs:923-931), не «успех».
3. **Языки:** Rust — ASCII-идентификаторы, русские строки в коде разрешены (конвенция dictad); Flutter — русские ТОЛЬКО через .arb или `\uXXXX`-эскейпы (cyrillic-gate).
4. **Payload-приватность:** во Flutter-ошибках никогда не логировать текст вставки (windows_engine.dart:111-113 — только тип/путь); аудио/тексты на диск не пишутся (must-NOT 5) — cpal-Recorder без temp-WAV.
5. **Таймауты = потолки, не паузы:** wait-подтверждение поллингом с дедлайном (inject.rs:330-341), не слепой sleep; Win-вставка: settle ~400 мс после первого WM_RENDERFORMAT — фиксировать спиком.
6. **Фоновые работы — в тредах, не под мьютексом:** schedule_restore (inject.rs:368-380), schedule_engine_reload (main.rs:763-781), run_stop-воркер (main.rs:1073-1192); мьютекс handler держится только на быстрых действиях.
7. **Блокирующие слушатели — в выделенных тредах с eprintln-диагностикой:** hotkey-тред (main.rs:298-322); Win: message-only окно + GetMessageW в треде, `"off"`-гейт и env-шов `DICTA_HOTKEY` сохранить.
8. **Минимум cfg-точек, Linux байт-в-байт:** юникс-код не редактировать — только оборачивать в `#[cfg(unix)]`; Linux-CI (ci.yml) остаётся зелёным без изменений поведения — это гейт фазы (locked 3).
9. **CI:** пины версий (тулчейн ≥1.85, как flutter 3.50.1), `timeout-minutes`, `--locked`, scope `-p dictad -p dicta-core` (dicta-ui на Windows не собирается).
10. **Конфиг-гейты без изменений:** EDITABLE_KEYS/ENGINE_KEYS (config.rs:142-170), `set_key` атомарной записью (185-231), `%APPDATA%`-паритет resolveConfigPath (windows:config_path.dart:27-33 — контракт уже записан, Rust-сторона обязана совпасть).

## Metadata

**Аналоги проверены как git-tracked:** все Rust-пути — `git ls-files` на main (= windows для crates); `windows-build.yml` — tracked на ветке windows (на main отсутствует); Flutter-аналоги читались с ветки windows (`git show windows:…`) — в PATTERNS помечены `windows:`.
**Scope поиска:** crates/**, app/lib/core, app/test, .github/workflows, .planning/research/{STACK,PITFALLS}.md, .planning/phases/06-*.
**Прочитано живьём:** dictad/main.rs (1335), inject.rs (457), hotkey.rs (204), capture.rs (146), audio.rs (177), stt.rs (115), config.rs (фрагменты 140-239), lib.rs, Cargo.toml ×3, ci.yml, windows-build.yml (windows), providers.dart, windows_engine.dart, config_path.dart (windows).

---
phase: 07-windows-diktovka-mvp
plan: 07-01
reviewed: 2026-11-22T23:55:00Z
depth: deep
branch: windows
range: 23049d6..8fb0d96 (12 коммитов)
files_reviewed: 13
files_reviewed_list:
  - crates/dicta-core/src/combo.rs
  - crates/dicta-core/src/lib.rs
  - crates/dicta-core/src/inject.rs
  - crates/dicta-core/src/inject_win.rs
  - crates/dicta-core/src/config.rs
  - crates/dicta-core/src/capture_win.rs
  - crates/dicta-core/src/hotkey_win.rs
  - crates/dicta-core/Cargo.toml
  - crates/dictad/src/main.rs
  - crates/dictad/src/transport_win.rs
  - crates/dictad/tests/pipe_roundtrip.rs
  - crates/dictad/Cargo.toml
  - app/lib/features/settings/hotkey_card.dart
  - app/lib/core/providers.dart
  - app/test/hotkey_card_error_test.dart
  - .github/workflows/windows-build.yml
  - Cargo.lock
findings:
  critical: 0
  warning: 2
  info: 8
  total: 10
  necessity: 0
status: issues_found
---

# Phase 07-01: Code Review Report (ветка windows, 23049d6..8fb0d96)

**Вердикт-сводка: 0 BLOCKER / 2 WARNING / 8 INFO.**

План в целом исполнен добротно: Linux-путь байт-в-байт (дифф против 23049d6 —
только cfg-развилки, combo-перенос и новые _win-файлы; аудита списка файлов —
чисто), протокол не тронут, payload-дисциплина на Windows-пути выдержана
абсолютно (ни одной добавленной `{text`-строки во всём диффе), DACL построен из
реального SID процесса. Живые прогоны всё зелёные. Оба WARNING — про
правильность Windows-транспорта на граничных сценариях, не про безопасность
межпользовательской границы (она закрыта корректно).

## Сводная таблица находок

| id | severity | file:line | суть | fix-hint |
|----|----------|-----------|------|----------|
| WR-01 | WARNING | crates/dictad/src/transport_win.rs:106-125 | Нет «уже кто-то слушает»-пробы: второй dictad (или same-user имитатор) молча сосуществует с первым (named-pipe multi-instance); клиенты попадают в любой свободный инстанс — гонки состояния/watchdog | Перед созданием listener сделать probe-`PipeStream::connect_by_path`; успех → bail «уже кто-то слушает» (паритет unix main.rs:161-171) |
| WR-02 | WARNING | crates/dictad/src/main.rs:400; crates/dicta-core/src/config.rs:27,265 | `cfg.socket` (сериализуемое поле конфига) на Windows мёртв: слушатель берёт `resolve_pipe_name()` (DICTA_PIPE → дефолт), а стартовый лог main.rs:304 печатает cfg.socket как будто он активен | Либо прокинуть cfg.socket в цепочку resolve (не-дефолт → слушать его), либо под cfg(windows) убрать socket из «сокет {}»-лога — сейчас лог вводит в заблуждение |
| IN-01 | INFO | crates/dicta-core/src/hotkey_win.rs:81-96 | После успеха RegisterHotKey и сбоя GetMessageW(-1) backoff-ретрай зовёт RegisterHotKey в том же потоке с тем же id=1 без UnregisterHotKey → ERROR_HOTKEY_ALREADY_REGISTERED навсегда; UnregisterHotKey не вызывается ни на одном выходе | UnregisterHotKey(None, 1) в error-пути listen перед возвратом; редкий случай (GetMessageW −1 почти невозможен), но авто-выздоровление backoff здесь ложно |
| IN-02 | INFO | crates/dicta-core/src/hotkey_win.rs:18-59 | Паритет-щели парсера против hotkey.rs:15-66: «printscreen/prtsc» и правые модификаторы (rctrl/ralt/rshift/rmeta) валидны на Linux → на Windows честное «не клавиша»; добавлены сверхнаборные алиасы «control», «insert/ins»; bare-спека («v») регистрируется как global hotkey и ПОГЛОЩАЕТ клавишу (evdev не поглощал) — plan-blessed, но инвазивно | Задокументировать; опционально — отказ от bare-спеки без модификаторов как осознанный Windows-запрет |
| IN-03 | INFO | crates/dicta-core/src/hotkey_win.rs:73 | Сообщение «(GetLastError={})» печатает `err.code().0` — это HRESULT-обёртка windows-crate (0x8007xxxx-вида), не сырой GetLastError; диагностика читается неверно | `err.code()` → использовать `HRESULT::from_win32`-раскрытие или Win32Error; или честно переименовать поле в код ошибки |
| IN-04 | INFO | crates/dictad/src/transport_win.rs:44-77 | Хэндл токена не закрывается (CloseHandle нет); TOKEN_USER читается как `&*(buf.as_ptr() as *const TOKEN_USER)` из Vec<u8> — технически unaligned-reference UB (на x64 работает) | Закрыть token; буфер через `#[repr(align(8))]`-обёртку или `read_unaligned` |
| IN-05 | INFO | crates/dictad/src/main.rs:294-296, 343-346 | Общие блоки печатают на Windows вводящий в заблуждение текст: «виртуальная клавиатура (uinput) готова» после no-op inject_win::warm_up_keyboard; hotkey-backoff советует «группу input / udev-ACL на /dev/input/event*» | cfg-гейт этих двух строк (Linux-текст — только unix) |
| IN-06 | INFO | Cargo.lock (ad6a053, 52dd5f2) | cpal 0.18.2 тянет windows 0.62.2 → в lock ДВЕ мажорные версии windows-crate (0.58 пин + 0.62 замыкание), вопреки комментарию dictad/Cargo.toml «не порождаем вторую мажорную»; побочно windows-sys нескольких пакетов даунгрейднулся 0.61.2→0.52.0 | Не блокер (CI зелёный, это цена cpal 0.18); поправить комментарий в Cargo.toml, при случае выровнять пины |
| IN-07 | INFO | app/test/hotkey_card_error_test.dart | Имя тест-файла разошлось с планом (hotkey_card_test.dart); гейт «33 файла» проходит, чисто docs-расхождение | Переименовать или зафиксировать в SUMMARY |
| IN-08 | INFO | crates/dictad/tests/pipe_roundtrip.rs:38-41,69 | stderr(Stdio::piped()) никогда не читается — при росте стартового шума демона warm-up-тред упрётся в буфер пайпа (serve не затронет); temp-конфиг удаляется только на пути успеха, не из guard | Читальщик stderr в фоновом треде или inherit(null); remove_file перенести в Drop-guard. Флакiness-риск низкий: имя пайпа уникально по pid, ретрай ≤2 с, DaemonGuard убивает ребёнка и при panic — cleanup процесса честный |

## Что проверено живыми прогонами (машина = Linux)

- `cargo build --release --locked --workspace` — ok.
- `cargo test --workspace` — ok (все крафты; pipe_roundtrip на Linux = 0 тестов
  ожидаемо, `#![cfg(windows)]`).
- `cargo check --locked --no-default-features -p dicta-core -p dictad` — ok
  (CPU-only feature-граф собирается и на Linux).
- `cd app && flutter test` — 198 passed; `flutter analyze` — 0 issues;
  `bash tool/cyrillic_gate.sh` — 0; `ls app/test/*.dart | wc -l` — 33 (гейт
  Task 5: 33 файла — проходит, IN-07 про имя).
- CI (gh): run 45064417225 на HEAD 8fb0d96 — **success** (джоба rust на
  windows-latest зелёная на финальном коммите; промежуточные красные прогоны
  b161b26/8ee6154/cf062c0 — задокументированный fix-loop Task 2, каждый закрыт
  атомарным fix-коммитом).
- Аудит заморозки: `git diff 23049d6 --name-only` — только якорные файлы
  плана; audio/filter/stt/focus/atspi/ci.yml/release.yml/windows_engine.dart
  (фаза 6) — не тронуты (пустой дифф, живая проверка).
- Linux-only импорты: `libc::` только в capture/hotkey/inject (все
  `#[cfg(unix)]`-гейты в lib.rs); `std::os::unix::*` в dictad main.rs — обе
  use-строки и stale-блок под cfg(unix); dictactl — однострочный `#![cfg(unix)]`.
- Payload-гейт: `git diff 23049d6..windows | grep '^+.*{text'` — **пусто** (0
  добавленных строк с текстом диктовки во всём диапазоне); по-блочно в
  cfg(windows)-блоках main.rs `{text` — пусто; все новые eprintln в
  transport_win/hotkey_win/capture_win/inject_win прочитаны глазами — только
  keys/app/len/SDDL/ошибки cpal, ни одного текста.

## Убеждённость по фокусам

**Payload-дисциплина: высокая.** Три независимые проверки (grep по всему
диффу, поблочный grep cfg(windows)-регионов main.rs, ручное чтение каждой
новой лог-строки transport_win/hotkey_win/capture_win/inject_win) — ни одного
проявления текста диктовки. Windows-логи вставки несут `{keys}`, `{app:?}`,
`text.len()`; run_stop — тайминги/keys/app/len. Единственный перенос текста —
протокольный `text: Some(text)` в Res (замороженный протокол, это не лог).

**DACL: высокая.** SDDL `D:P(A;;GA;;;SY)(A;;GA;;;<SID>)` строится из
РЕАЛЬНОГО SID (OpenProcessToken(GetCurrentProcess) → GetTokenInformation
(TokenUser) → ConvertSidToStringSidW — хардкода SID нет), флаг D:P защищает от
наследования, ни WD/AU/ALL APPLICATION PACKAGES нет (unit-тест
`sddl_is_system_plus_owner_only` + отрицательный гейт плана `!grep 'A;;GA;;;WD'`
зелёный). Пайп создаёт только сервер, полное имя `\\.\pipe\dicta` без
авто-префикса (PITFALL 7 соблюдён, roundtrip на CI доказал same-user доступ).
Калибровочное замечание WR-01: DACL закрывает МЕЖпользовательский preempt;
same-user сосуществование инстансов никто не проверяет (unix-путь проверял).

**capture_win: ок.** Level из живого буфера в памяти (Mutex<Vec<i16>> +
normalized_rms по хвосту rate/25), native default_input_config (никаких
принудительных 16 кГц), первый канал, resample_to_16k на stop на native rate,
overflow-потолок 120 с честной записью в лог, ring/temp-файлов нет, поток
останавливается pause + drop Stream (утечки нет, контракт start/stop/elapsed/
level — паритет capture.rs; `file()` никто на Windows не зовёт). Пауза/
продолжение семантики dictad не менялась.

**pipe_roundtrip: не флаковый.** Уникальное имя `\\.\pipe\dicta-test-<pid>` +
temp-конфиг по pid, ретрай-коннект ≤2 с (демон стартует мгновенно —
прогрев-модель в отдельном треде, пайп-serve его не ждёт), DaemonGuard убивает
ребёнка в Drop (в т.ч. при panic), connect_by_path против полного имени —
тот же шов, что слушает сервер. Остаточные шероховатости — IN-08.

**hotkey_win: ок с оговорками.** hWnd=NULL + GetMessageW-очередь выделенного
треда, окно не создаётся (grep CreateWindowEx — пусто), занятая комбинация →
честная PermissionDenied с кодом, спека парсится чистой функцией с юнит-тестами
(IN-02 — паритет-щели, IN-03 — код ошибки, IN-01 — редкий ретрай-тупик).

**CI: консистентен.** `--release` соответствует артефакту
`target/release/dictad.exe`; `timeout-minutes: 60` есть; rust-cache@v2 (ключ по
умолчанию хеширует Cargo.lock — не течёт); scope `-p dicta-core -p dictad`, не
workspace; flutter-джоба не тронута; run на HEAD — success.

**Flutter IN-02 (8fb0d96): канон соблюдён.** hotkeyConfigTextProvider
продвигает `parsed.error` в `StateError('settings reply error: …')` — строка
дословно паритетна settingsSnapshotProvider (providers.dart:57-59);
catch сужен до `on IOException`/`on TimeoutException` (транспорт → честный
фолбэк, протокол → AsyncError); poll-ветка несёт `e.runtimeType` без payload
(тест «SECRET PAYLOAD MUST NOT LEAK» это асерит); фейк-канон расширен без
новых зависимостей (fetchSettingsError/statusError уже были в
fake_dicta_engine.dart).

## Summary

Кодовых блокеров нет: протокол заморожен, Linux батарея зелёная, CI на HEAD
зелёный, DACL и payload — главные угрозы плана — закрыты корректно и
проверяемо. Два WARNING рекомендую закрыть до 07-02 (probe-коннект перед
созданием пайпа и честность cfg.socket на Windows) — оба дёшевы, оба про
граничные сценарии реального пользователя. INFO — полировка сообщений,
UB-техника в SID-цепочке и lock-гигиена.

---

_Reviewed: 2026-11-22T23:55:00Z_
_Reviewer: gsd-code-reviewer (GLM-5.3)_
_Depth: deep_
_Branch state: возвращена на main после ревью_

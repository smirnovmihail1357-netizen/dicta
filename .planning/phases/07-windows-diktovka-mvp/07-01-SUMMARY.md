---
phase: 07-windows-diktovka-mvp
plan: "07-01"
subsystem: windows-daemon-core
tags: [rust, windows, named-pipe, dacl, wasapi, cpal, registerhotkey, ci, flutter, in-02]
requires: []
provides:
  - "Windows dictad.yml transport: \\\\.\\pipe\\dicta with owner-SID DACL (WIN-04 foundation)"
  - "capture_win cpal/WASAPI Recorder + hotkey_win RegisterHotKey (WIN-03 core)"
  - "CI Rust job with dictad-windows-x64 artifact (precondition for 07-02 UAT)"
  - "IN-02 residual closed: hotkey card error propagation + poll disconnect cause"
affects: [07-02]
tech-stack:
  added: [interprocess 2.4.8, widestring 1.2.1, cpal 0.18.2, windows 0.58.0]
  patterns: ["cfg(unix)/cfg(windows) module gates", "one command = one connection", "errors-as-data provider"]
key-files:
  created:
    - crates/dicta-core/src/combo.rs
    - crates/dicta-core/src/hotkey_win.rs
    - crates/dicta-core/src/capture_win.rs
    - crates/dicta-core/src/inject_win.rs
    - crates/dictad/src/transport_win.rs
    - crates/dictad/tests/pipe_roundtrip.rs
    - app/test/hotkey_card_error_test.dart
  modified:
    - crates/dicta-core/src/lib.rs
    - crates/dicta-core/src/inject.rs
    - crates/dicta-core/src/config.rs
    - crates/dicta-core/Cargo.toml
    - crates/dictad/src/main.rs
    - crates/dictad/Cargo.toml
    - crates/dictactl/src/main.rs
    - Cargo.lock
    - .github/workflows/windows-build.yml
    - app/lib/features/settings/hotkey_card.dart
    - app/lib/core/providers.dart
decisions:
  - "Default pipe name is a shared const in dicta-core (cross-crate calls from config.rs into dictad are impossible); DICTA_PIPE env shov stays in transport_win."
  - "Single windows-crate major (0.58, already in lock) used by both crates; required sub-features added explicitly."
  - "Production injection deliberately NOT implemented — inject_win returns an honest error until the 07-02 spike."
metrics:
  duration: "66m (2026-11-22T19:04:12Z → 2026-11-22T20:10:04Z)"
  completed: 2026-11-22
  status: complete
actuals:
  tokens: 12419
  tasks: 5
  commits: 12
  plan_head_before: 23049d692e2a2c8ccd526f0deba6b4b21a74db9a
  plan_head_after: 8fb0d963c98df6d7e09cb71bd51d07ad2e7d7eb5
branch: windows
---

# Phase 07 Plan 01: Windows диктовка MVP Summary

Windows-ядро демона: именованный канал `\\.\pipe\dicta` с DACL только для SYSTEM и владельца, WASAPI-захват и RegisterHotKey без окна — доказаны живым CI-прогоном на windows-latest; Linux-путь и протокол остались байт-в-байт нетронутыми.

## Что построено

1. **cfg-развилки (unix-логика не изменена).** `hotkey`/`inject`/`capture` гейтированы `#[cfg(unix)]`, добавлены `*_win` аналоги; `Recorder` и `Combo` реэкспортируются за cfg. `Combo::parse` перенесён дословно в платформо-независимый `combo.rs`. `dictactl` gated to unix. `dictad/main.rs`: только cfg-точки (импорты, stale-сокет, transport, hotkey, `--ui`, два payload-лога).
2. **transport_win.rs (production).** `resolve_pipe_name()` (DICTA_PIPE → дефолт), `pipe_security_descriptor(sid)` = `D:P(A;;GA;;;SY)(A;;GA;;;<owner-SID>)`, SID через OpenProcessToken → GetTokenInformation(TokenUser) → ConvertSidToStringSidW, слушатель `PipeListenerOptions::path(полное имя).security_descriptor(...).create_duplex::<pipe_mode::Bytes>()`, accept-цикл с `serve_conn(impl BufRead, impl Write, …)`. Unit-тест SDDL (SY + owner-SID, без WD/everyone).
3. **Интеграционный tracer.** `crates/dictad/tests/pipe_roundtrip.rs` поднимает dictad.exe с DICTA_PIPE + temp DICTA_CONFIG, шлёт `{"cmd":"status"}`, получает JSON `state=idle, error=null, level=0`; демон гарантированно убивается guard-ом. Прошёл **на живом windows-раннере**.
4. **CI Rust-джоба.** `.github/workflows/windows-build.yml`: новая джоба `rust` (windows-latest, dtolnay/rust-toolchain@stable, Swatinem/rust-cache@v2), `cargo build --release --locked --no-default-features -p dicta-core -p dictad` + такой же `cargo test`, артефакт `dictad-windows-x64` (`target/release/dictad.exe`). Flutter-джоба не тронута.
5. **capture_win.** cpal 0.18.2/WASAPI: native `default_input_config` (без принудительных 16 кГц), первый канал, I16/F32-ветки, overflow ~120 с, `resample_to_16k`, `level()` по хвосту из памяти; **без temp-файлов**. 2 unit-теста без микрофона.
6. **hotkey_win.** `parse_windows_combo` (ctrl/shift/alt/win + буквы/цифры/F1-F12/space/enter) — чистая функция с 4 тестами; `RegisterHotKey(None, …)` → WM_HOTKEY в очередь потока, **ни одного CreateWindowEx**; занятая комбинация/невалидная спека → честные ошибки.
7. **IN-02-остаток.** `hotkeyConfigTextProvider` продвигает `reply.error` в StateError→AsyncError, сужает catch до `IOException`/`TimeoutException` (файл-фолбэк только для транспорта); poll-ветка кладёт `e.runtimeType` в `DictaStatus.error` без payload. 4 новых теста против фейка, 198 тестов / 33 файла.
8. **Payload-приватность.** Windows-ветки логирования в `dictad/main.rs` пишут только keys/app/len — `{text}` в cfg(windows)-блоках отсутствует (отрицательный гейт = 0).

## Как проверить

Локально (Linux):
```bash
cd /home/wardreg/projects/dicta
cargo build --release --locked --workspace
cargo test --workspace
cargo check --locked --no-default-features -p dicta-core -p dictad
(cd app && flutter test && flutter analyze && bash tool/cyrillic_gate.sh)
# аудит заморозки:
git diff 23049d6 --name-only -- crates/          # только файлы из списка Task 1-4
git diff 23049d6 --quiet -- crates/dicta-core/src/{audio,filter,stt,focus,atspi}.rs  # пусто
sed -n '/cfg(windows)/,/^    }/p' crates/dictad/src/main.rs | grep -c '{text'  # 0
```
Живые гейты (Windows): `gh run view <run-id> --repo smirnovmihail1357-netizen/dicta`, артефакт `dictad-windows-x64` в `gh api repos/.../actions/runs/<run-id>/artifacts`.

**Результаты (живые, 2026-11-22):** cargo build --release OK; cargo test --workspace — 14+25+42+3 зелёных; flutter test — **198 passed / 33 файла**; flutter analyze — **No issues found**; cyrillic gate — **0**; DACL/SDDL, hotkey-парсинг, capture-тесты и pipe_roundtrip — **ok на windows-latest**.

## Живые CI run-id

| Ключ | Run ID | Conclusion | Примечание |
|------|--------|------------|------------|
| 07-01-first | 45060102916 | success | первая Windows-компиляция прошла сразу (фикс-луп не потребовался) |
| 07-01-capture | 45060755185 | failure | cpal: нет HostTrait/StreamTrait |
| 07-01-capture | 45061299949 | failure | cpal: нет DeviceTrait |
| 07-01-capture | 45061863952 | failure | cpal 0.18: description().name(), sample_rate()=u32, config by value |
| 07-01-capture | **45062488468** | **success** | capture + pipe_roundtrip зелёные |
| 07-01-hotkey | 45063134323 | failure | GetLastError вне unsafe |
| 07-01-hotkey | **45063612416** | **success** | 4 парсинг-теста + roundtrip зелёные |
| 07-01-final | **45064417225** | **success** | финальный прогон на 8fb0d96, оба джоба зелёные |

Run-ids также зафиксированы в `/tmp/opencode/rust-run-ids.txt` (ключи first/capture/hotkey/final — по одному успешному на ключ).

## Коммиты (ветка windows, 12)

| Хэш | Сообщение |
|-----|-----------|
| 3055c38 | refactor(07-01): extract combo.rs |
| dc425dc | feat(07-01): cfg gates + win stubs |
| ad6a053 | feat(07-01): transport_win named pipe + DACL |
| 3ff1e63 | test(07-01): pipe roundtrip integration |
| 0d75211 | ci(07-01): rust job — dicta-core+dictad CPU-only on windows-latest |
| 52dd5f2 | feat(07-01): capture_win — cpal/WASAPI recorder (WIN-03) |
| b161b26 | fix(07-01): windows compile — cpal trait imports + cfg'd socket path |
| 8ee6154 | fix(07-01): windows compile — import cpal DeviceTrait |
| 1bf2151 | fix(07-01): windows compile — cpal 0.18 API shapes |
| cf062c0 | feat(07-01): hotkey_win — RegisterHotKey thread queue (WIN-03) |
| ef8e512 | fix(07-01): windows compile — unsafe GetLastError + dep hygiene |
| 8fb0d96 | fix(07-01): hotkey card surfaces daemon error (IN-02) |

Все коммиты на `windows`; MERGE windows→main НЕ выполнялся; `.planning` на windows не коммитился. Финальный HEAD windows = 8fb0d96.

## Deviations from Plan

1. **`pub mod combo` вместо `mod combo`.** План требует и `mod combo`, и импорт `dicta_core::combo::Combo` в main.rs — приватный модуль не дал бы второй. Сделан публичным; `pub use combo::Combo` сохранён.
2. **Один `pub use crate::combo::Combo;` в inject.rs** вместо предложенных `use` + `pub use` — два импорта одного имени дают E0252.
3. **windows = "0.58".** План ожидал версию из замыкания enigo 0.6.1, но enigo в дереве отсутствует; взята уже залоченная мажорная 0.58 (исключает вторую мажорную версию в lock).
4. **Дополнительные feature windows-crate:** `Win32_Security_Authorization` в dictad (ConvertSidToStringSidW) и `Win32_UI_Input_KeyboardAndMouse` в dicta-core (RegisterHotKey/MOD_*/VK_* живут там, а не в WindowsAndMessaging). Без них API не существует.
5. **`default_socket_path()` на Windows возвращает `PathBuf::from(DEFAULT_PIPE_NAME)`** — общий const в dicta-core. План предлагал звать `resolve_pipe_name()` из dictad, что невозможно кросс-крейтом; env-шов DICTA_PIPE остался в transport_win.
6. **`transport_win::serve` вызывается на месте accept-цикла**, после создания Arc-ов, а не в блоке bind — иначе Arcs ещё не exist; семантика «один вызов, цикл внутри» сохранена.
7. **interprocess залочен ровно 2.4.8** через `cargo update --precise` (Cargo.toml строка "2.4.8" допускала 2.4.9).
8. **`libc` перенесён в `[target.'cfg(unix)'.dependencies]`** — все его пользователи cfg-gated; убрано предупреждение `unused dependency` на Windows.
9. **Имя теста `hotkey_card_error_test.dart`, а не `hotkey_card_test.dart`.** Последний уже существует на windows (HK-01 coverage); новый файл дал требуемые 33 файла без перезаписи чужого теста.
10. **Инжектированный путь фолбэка** — добавлен провайдер `hotkeyConfigPathProvider` (seam в духе `engineProvider`), чтобы тест транспортной ошибки мог проверить содержимое temp-конфига, не мутируя `Platform.environment`.
11. **Причина отключения передается через существующее поле `DictaStatus.error`** (`DictaStatus(connected: false, state: idle, error: e.runtimeType.toString())`) вместо добавления параметра в `disconnected()` — `engine.dart` не тронут, поведение аддитивно.

## Known Stubs (intentional, for 07-02)

| Файл | Что | Почему |
|------|-----|--------|
| crates/dicta-core/src/inject_win.rs | `insert_text` → честная ошибка «вставка появится после спика (07-02, 07-SPIKE-RESULT.md)»; `warm_up_keyboard` → Ok | Must-NOT 8: production-вставка запрещена до спика. |

Протокол демона, Linux-пути, `app/lib/core/windows_engine.dart`, `ci.yml`/`release.yml` и README-якоря DACL не изменялись.

## Что осталось на 07-02

- **Спик 07-SPIKE-RESULT.md** (обязателен до production-вставки), затем заменить тело `inject_win::insert_text` (delayed render / UIPI / clipboard).
- **Реальная вставка** и 4 кейса вставки (UAT-протокол 07-VALIDATION); `combo.rs` уже готов к переиспользованию.
- UAT backstop-истины (живой звук WASAPI, физическое нажатие хоткея) — помечаются `human_needed`, авто-pass запрещён; артефакт `dictad-windows-x64` из run 45064417225 готов для UAT.

## Self-Check: PASSED

- Все 6 новых Rust-файлов + новый Dart-тест созданы и закоммичены.
- 12 коммитов присутствуют в `git log 23049d6..HEAD`, HEAD windows = 8fb0d96.
- Все `<verify>`-гейты Task 1-5 выполнены вживую; финальный CI 45064417225 = success; Linux-батарея и Flutter-батарея зелёные; аудит диффов и payload-гейт чистые.

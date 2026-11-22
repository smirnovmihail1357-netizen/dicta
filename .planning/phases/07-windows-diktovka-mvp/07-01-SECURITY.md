---
phase: 7
plan: 07-01
audit_date: 2026-11-22
auditor: gsd-security-auditor
verdict: SECURED
threats_total: 5
threats_open: 0
asvs_level: 1
block_on: high
scope: 23049d6..6313014 (ветка windows), 18 файлов, +1299/−102
---

# Security Audit — Plan 07-01 (Windows-ядро демона)

## Вердикт: SECURED — 5/5 угроз закрыто, threats_open: 0

## Threat Verification

| ID | Категория | Severity | Disposition | Статус | Evidence (живой код) |
|----|-----------|----------|-------------|--------|----------------------|
| T-07-01 | Spoofing (preempt-спуф пайпа) | high | mitigate | CLOSED | SDDL `D:P(A;;GA;;;SY)(A;;GA;;;<owner-SID>)` — transport_win.rs:50-52; SID реальный, из токена процесса: OpenProcessToken → GetTokenInformation(TokenUser) → ConvertSidToStringSidW — transport_win.rs:56-93 (не хардкод); probe занятости ДО bind — transport_win.rs:42-47 (`pipe_already_listening`, connect_by_path) и :116-118 (bail «уже кто-то слушает», паритет unix main.rs), WR-01 закрыт; дескриптор навешан на слушателя — :119-123 (полное имя `\\.\pipe\...`, без авто-префикса); unit-тест отрицательного гейта — :165-175: D:P-префикс, SY, owner-SID, **НЕТ WD, НЕТ AU, НЕТ S-1-15-2** (ALL APPLICATION PACKAGES); тест пробы — :178-190. Гейт `! grep 'A;;GA;;;WD'` чист. 06-SECURITY residual #1 (preempt-спуф) закрыт: сервер стартует только если пайп свободен, чужой пользователь не может ни создать инстанс (DACL GA только SY+owner), ни подсоединиться |
| T-07-02 | Information Disclosure (payload в логах) | high | mitigate | CLOSED | Негативный grep `{text` по ВСЕМУ диффу 23049d6..windows → ровно 2 совпадения, оба внутри `#[cfg(unix)]`: main.rs:985 (`inject … {text:?}`, unix-only) и main.rs:1257 (unix-only). Windows-ветки логируют только keys/app/len/тайминги: main.rs:987-988 (`len={} (текст не логируется)`) и :1261-1265 (`распознано за {inferred:?}, вставка {keys} в {app:?}, len={}`); стартовый лог пайпа — только имя/модель/язык (main.rs cfg(windows)-блок). Все 10 добавленных eprintln-строк диффа просмотрены глазами: ошибки cpal (capture_win.rs:67), переполнение буфера (:126), транспортные `dictad: {err}` — ни одна не интерполирует текст/сэмплы. capture_win уровень из памяти (:136-140), без payload. Flutter: hotkey_card_error_test.dart — ассерт `error` isNot(contains('SECRET PAYLOAD')); провайдер передаёт `e.runtimeType.toString()` (не сообщение исключения). Linux-логи не изменены (байт-в-байт) |
| T-07-03 | Tampering (env-шов DICTA_PIPE) | low | accept | CLOSED (accepted) | Принятый риск: план `<threat_model>` (env-шов своего сеанса, test-only, паритет resolvePipePath фазы 6) + 06-SECURITY.md Residuals #2; комментарий кода transport_win.rs:28-31 («env-шов теста», пустое значение игнорируется :35). → accepted risks log |
| T-07-04 | Information Disclosure (аудио-путь) | medium | mitigate | CLOSED | Сэмплы ТОЛЬКО в памяти: `Arc<Mutex<Vec<i16>>>` — capture_win.rs:64, забор `std::mem::take` — :124, децимация на месте `audio::resample_to_16k` — :128; grep pw-record/temp_dir/.wav по capture_win = 0; единственный std::fs в новом коде — temp-конфиг теста, удаляется в Drop (pipe_roundtrip.rs:27); stt.rs/audio.rs/filter.rs/focus.rs/atspi.rs — git diff 23049d6 **пуст** (заморозка не нарушена) |
| T-07-SC | Tampering (supply chain) | high | mitigate | CLOSED | Cargo.lock (дифф 337 строк): ТОЛЬКО добавления, **ноль** `-version`-строк (нет дрейфа существующих пакетов), ноль удалённых пакетов; каждый новый пакет трассируется в заявленное замыкание: cpal 0.18.2 (alsa/alsa-sys/dasp_sample/coreaudio/objc2/block2/mach2/ndk-цепочка — бэкенды cpal), interprocess 2.4.8 ровно (`--precise`; recvmsg — его dep), widestring 1.2.1, windows 0.62.2 + windows-*-семейство — транзитивное замыкание cpal (задокументировано: dictad/Cargo.toml комментарий IN-06, deviation #3; прямые deps — windows 0.58 == залоченная 0.58.0, второй мажорной версии для прямых deps нет). Пины: cpal "0.18.2" (dicta-core/Cargo.toml), interprocess "2.4.8", widestring "1"; lock закоммичен + `--locked` в обоих шагах CI. CI (windows-build.yml rust:40-63): нет env:, нет secrets, нет `$env:`/printenv, нет pull_request_target; scope `-p`, не workspace; артефакт dictad.exe только upload (не исполняется). ci.yml/release.yml — дифф пуст (заморожены) |

## Доп-проверки

- **Протокол заморожен:** 0 изменений парсинга команд/Res/JSON-строки в диффе main.rs; сигнатура serve_conn обобщена `impl BufRead`/`impl Write` (транспорт-агностик, main.rs:882), тело не тронуто; pipe_roundtrip.rs:73-76 ассертит замороженную форму status-ответа (state=idle, error=null, level=0). windows_engine.dart не тронут.
- **Linux-изоляция:** cfg-гейты — lib.rs (8 атрибутов), main.rs (25), dictactl `#![cfg(unix)]` (dictactl/src/main.rs:1-2, windows-бина пустая); uinput/udev-сообщения cfg(unix) (IN-05), cfg.socket unix-only (WR-02); combo.rs — дословный перенос из inject.rs (сверен построчно); `cargo check --locked --no-default-features -p dictad` зелёный локально; батарея `--workspace` зелёная на HEAD 6313014.
- **IN-04 (перепроверка):** transport_win.rs:79 `CloseHandle(token)` после копии SID, :83 `read_unaligned` для TOKEN_USER, :90 `LocalFree` SID-строки — закрыт.
- **IN-08:** pipe_roundtrip.rs:44 stderr→`Stdio::null()` (нет зависания на буфере пайпа), DaemonGuard :22-29 — kill дочернего + удаление temp-конфига в Drop.
- **DICTA_CONFIG на Windows:** шов существовал до фазы (config.rs:174-177 в 23049d6), на Windows стал достижим впервые. Оценка: env-инъекция = контроль окружения того же пользовательского сеанса (тот же trust-домен, что DICTA_PIPE/DICTA_SOCK); может перенаправить демон на произвольный конфиг (путь модели, hotkey), эскалации за пределы сеанса нет; DACL пайпа от этого не зависит. Информационный residual 3.

## Unregistered Flags

Незарегистрированных флагов нет: все 11 deviations из SUMMARY сверены (#3 windows-crate → T-07-SC; #10-11 швы провайдеров/error-runtimeType → T-07-02); новых поверхностей вне реестра нет.

## Residuals (informational)

1. Same-user preempt/instance-спуф остаётся возможным (DACL owner-SID = тот же trust-домен): процесс того же пользователя может создать инстанс пайпа раньше демона — но честный dictad откажется стартовать (probe :116), молчаливой подмены нет; заложенная граница модели.
2. Утечка токен-хэндла на error-путях `current_user_sid_string` (needed==0 :65-67, Err второго GetTokenInformation :70-77 — ранний return без CloseHandle): процесс сразу завершается (стартовый путь), ОС реклеймит хэндлы — negligible.
3. DICTA_CONFIG env-шов достижим на Windows (pre-existing, впервые живой на платформе): тот же сеанс/пользователь, паритет с принятым T-07-03 → accepted risks log.
4. Actions пинуются тегом, не SHA (dtolnay/rust-toolchain@stable, Swatinem/rust-cache@v2): конвенция PATTERNS.md, единая для фаз 1-6; гигиеническая заметка.
5. DICTA_PIPE не валидирует префикс `\\.\pipe\` (test-only шов :32-37): значение = произвольный путь object namespace; тот же trust-домен, задокументирован как шов теста (transport_win.rs:28-31).

**threats_open: 0**

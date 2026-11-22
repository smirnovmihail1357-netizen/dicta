---
phase: 07-windows-diktovka-mvp
plan: "07-01"
verified: 2026-11-22T20:37:52Z
status: human_needed
score: 9/9 must-haves verified
covered_files:
  - .github/workflows/windows-build.yml
  - .planning/phases/07-windows-diktovka-mvp/07-01-PLAN.md
  - .planning/phases/07-windows-diktovka-mvp/07-01-SUMMARY.md
  - .planning/phases/07-windows-diktovka-mvp/07-02-PLAN.md
  - Cargo.lock
  - app/lib/core/providers.dart
  - app/lib/features/settings/hotkey_card.dart
  - app/test/hotkey_card_error_test.dart
  - crates/dicta-core/Cargo.toml
  - crates/dicta-core/src/capture_win.rs
  - crates/dicta-core/src/combo.rs
  - crates/dicta-core/src/config.rs
  - crates/dicta-core/src/hotkey_win.rs
  - crates/dicta-core/src/inject.rs
  - crates/dicta-core/src/inject_win.rs
  - crates/dicta-core/src/lib.rs
  - crates/dictactl/src/main.rs
  - crates/dictad/Cargo.toml
  - crates/dictad/src/main.rs
  - crates/dictad/src/transport_win.rs
  - crates/dictad/tests/pipe_roundtrip.rs
covered_digest: "v4:sha256:b761defdb68ac7b0443ba4df53bbe010c0cf43aa2dc97295b5180ff9d99c1519"
covered_base: "windows HEAD 63130140897eb8214560ee1df825a21ef28517ee (freeze base 23049d6)"
behavior_unverified: 3
behavior_unverified_items:
  - truth: "Живой звук WASAPI: cpal-Recorder на реальной Windows-машине захватывает микрофон и whisper.cpp распознаёт диктовку (roadmap SC п.1, WIN-03)"
    test: "На живой Windows-машине: dictad.exe (артефакт dictad-windows-x64 из CI run 45066485292) + конфиг + модель в %LOCALAPPDATA%\\whisper-cpp; нажать хоткей, проговорить фразу, остановить"
    expected: "Уровень сигнала живой (не тишина — Pitfall 1 закрыт native-конфигом), whisper возвращает текст той же моделью/конфигом; аудио нигде не пишется на диск"
    why_human: "CI-раннер без аудио-устройств; unit-тесты capture_win покрывают чистые функции (кламп/overflow), но не живой поток сэмплов"
  - truth: "Физическое нажатие хоткея: RegisterHotKey(hWnd=NULL) срабатывает в живой сессии пользователя (WIN-03)"
    test: "На живой Windows-машине: DICTA_HOTKEY=<спека> (например alt+v), физически нажать комбинацию при запущенном dictad.exe"
    expected: "Диктовка стартует/останавливается без какого-либо окна; занятая комбинация → честная русская ошибка с win32-кодом, backoff-ретрай при освобождении"
    why_human: "WM_HOTKEY приходит в очередь потока живой интерактивной сессии; CI доказал парсинг (4 unit-теста) и компиляцию, но не доставку сообщения"
  - truth: 'Живое подключение Flutter-приложения к пайп-серверу \\.\pipe\dicta на Windows-машине (06-UAT U-3, поглощение фазой 7)'
    test: "На живой Windows-машине: запустить dictad.exe, затем dicta_app.exe (артефакт фазы 6)"
    expected: "Оболочка переключается с баннера «демон недоступен» на connected-статус через \\\\.\\pipe\\dicta; status/start/stop по JSON-line протоколу без изменений; kill демона → баннер с причиной (IN-02 poll-ветка)"
    why_human: "Межпроцессное взаимодействие двух живых процессов на Windows-десктопе; CI pipe_roundtrip доказал демон-сторону, но не Flutter-клиента"
human_verification:
  - test: "UAT-7-1 (07-UAT.md): живая диктовка на Windows — хоткей без окна → фраза → распознанный текст той же моделью/конфигом"
    expected: "WIN-03 backstop подписан пользователем; артефакт dictad-windows-x64 из run 45066485292"
    why_human: "Живой звук + физическое нажатие хоткея непроверяемы из CI/Linux-окружения"
  - test: "UAT-7-2 (07-UAT.md): подключение Flutter-приложения к живому Windows-демону (поглощает 06-UAT U-3)"
    expected: "Connected-статус, roundtrip команд, баннер с причиной при kill демона"
    why_human: "Живое межпроцессное подключение на Windows-десктопе"
  - test: "UAT-7-3 (07-UAT.md, помечен «(07-02)»): спик delayed render + 4 кейса вставки + оптаут-форматы/восстановление буфера"
    expected: "07-SPIKE-RESULT.md с 9 живыми замерами; вставка SendInput после спика — зона плана 07-02"
    why_human: "Физически manual-кейсы (elevated-окно/UIPI, зажатый модификатор, Chromium) — вне 07-01 по Must-NOT 8"
deferred:
  - truth: "Roadmap SC п.2: вставка SendInput с delayed render проходит 4 обязательных кейса"
    addressed_in: "Plan 07-02 (фаза 7, тот же milestone)"
    evidence: "07-02-PLAN.md requirements [WIN-04]: spike/delayed_render бин + 9 замеров ДО production-вставки, затем inject_win::insert_text"
  - truth: "Roadmap SC п.3: оптаут-форматы Win+V/облака, восстановление исходного буфера"
    addressed_in: "Plan 07-02 (фаза 7)"
    evidence: "07-02-PLAN.md must_haves: «Зажатые модификаторы…» + протокол RESEARCH §5; спик-артефакты — Task 1/2 плана 07-02"
  - truth: "Roadmap SC п.4: спик delayed-render/WM_RENDERFORMAT выполнен экспериментально до фиксации реализации"
    addressed_in: "Plan 07-02 (фаза 7)"
    evidence: "07-02-PLAN.md: Task 1 = spike-бин delayed_render + 07-SPIKE-RESULT.md (Must-NOT 8: спик обязателен до production)"
---

# Phase 07 Plan 01: Windows диктовка MVP — Verification Report

**Phase Goal (ROADMAP §7):** Пользователь на Windows проходит полный цикл: хоткей без окна → диктовка → текст в активном окне — с вставкой, не повторяющей чужие баги
**Plan:** 07-01 — Windows-ядро демона (named pipe + DACL, WASAPI, RegisterHotKey, CI, IN-02-остаток)
**Verified:** 2026-11-22T20:37:52Z
**Status:** human_needed
**Code base:** ветка `windows`, HEAD 63130140897eb8214560ee1df825a21ef28517ee; база заморозки 23049d6; 20 коммитов (12 плана + 8 review-фиксов)
**Re-verification:** No — initial verification (предыдущего VERIFICATION.md не было)

## Goal Achievement

Все 9 истин плана подтверждены ЖИВЫМИ прогонами (локальная Linux-батарея на windows-HEAD-дереве + живые CI-прогоны на windows-latest с чтением логов). Промахов нет. Три backstop-истины (живой звук, физический хоткей, живое подключение приложения) честно вынесены в human_needed → поглощены 07-UAT.md — авто-pass запрещён планом (verification §7) и 07-VALIDATION (Nyquist).

### Observable Truths

| # | Truth | Status | Evidence (живой прогон) |
|---|-------|--------|------------------------|
| 1 | CI Rust-джоба зелёная на ветке windows, scoped CPU-only, артефакт dictad.exe, живой run-id | ✓ VERIFIED (live) | `gh run view` ×5: 45060102916/45062488468/45063612416/45064417225/**45066485292 = success** (последний на HEAD 6313014 — проверено headSha); артефакт `dictad-windows-x64` в run 45066485292 не истёк (gh api artifacts); джоба: `cargo build --release --locked --no-default-features -p dicta-core -p dictad` + тот же test, БЕЗ `--workspace`, dtolnay/rust-toolchain ×1, Swatinem/rust-cache ×1 |
| 2 | pipe_roundtrip на windows-раннере: dictad.exe + DICTA_PIPE/DICTA_CONFIG отвечает JSON status (state=idle, error=null) | ✓ VERIFIED (live) | Лог run 45066485292: `test status_roundtrip_over_named_pipe ... ok` — живой end-to-end «клиент → пайп → serve_conn → JSON»; тест спавнит CARGO_BIN_EXE_dictad с уникальным `\\.\pipe\dicta-test-<pid>`, temp-конфиг `hotkey="off"`, ретрай ≤2 c, assert state=="idle" && error==null && level==0; teardown: DaemonGuard kill + удаление конфига (IN-08) |
| 3 | DACL-контракт: SDDL D:P(A;;GA;;;SY)(A;;GA;;;<owner-SID>), только SYSTEM+владелец, полный путь без авто-префикса | ✓ VERIFIED (live) | transport_win.rs:51 `format!("D:P(A;;GA;;;SY)(A;;GA;;;{sid})")`; unit-тест `sddl_is_system_plus_owner_only` (фейковый SID, отсутствие WD/S-1-15-2/AU) зелёный в CI (dictad: 5 passed); `A;;GA;;;WD` в файле отсутствует (grep); DEFAULT_PIPE_NAME = r"\\.\pipe\dicta" == kDictaPipe windows_engine.dart:20 (полное имя, PITFALL 7); + review WR-01: pipe_already_listening-проба до bind с unit-тестом |
| 4 | Linux батарея зелёная; git diff 23049d6 — только cfg-развилки + combo-перенос + новые _win-файлы | ✓ VERIFIED (live) | Живой прогон 2026-11-22: `cargo build --release --locked --workspace` OK → `cargo test --workspace` OK (14+25+42+3 passed, 0 failed) → `cargo check --locked --no-default-features -p dicta-core -p dictad` OK; `git diff 23049d6 --name-only` = РОВНО 18 якорных файлов (см. Freeze Audit), audio/filter/stt/focus/atspi/ci.yml/release.yml/windows_engine.dart — пусто |
| 5 | hotkey_win: чистый парсер + unit-тесты, RegisterHotKey hWnd=NULL без окна, занятость → честная русская ошибка | ✓ VERIFIED (live) | CI-лог 45066485292: 4 парсинг-теста ok (`parses_alt_v_into_mod_and_vk` — mods==MOD_ALT\|MOD_NOREPEAT, vk==VK_V; multiple modifiers+F2; win+bare; rejects_unknown_spec); `RegisterHotKey` с hWnd=NULL в выделенном потоке, `CreateWindowEx` count=0; занятость → PermissionDenied с win32-кодом (review IN-03: HRESULT & 0xFFFF); IN-01: UnregisterHotKey на выходе/ошибке |
| 6 | capture_win: native default_input_config (НЕ 16 кГц), первый канал, resample_to_16k, level из памяти, аудио без диска | ✓ VERIFIED (live) | cpal = "0.18.2" в [target.'cfg(windows)'.dependencies]; `default_input_config`/`resample_to_16k`/`normalized_rms` присутствуют; pw-record/.wav/temp count=0; CI-лог: `f32_samples_are_clamped_to_i16_range ... ok`, `overflow_triggers_only_above_ceiling ... ok` |
| 7 | Пути: %APPDATA% конфиг, %LOCALAPPDATA% модели (data_local_dir), env-шов DICTA_PIPE | ✓ VERIFIED (live) | config.rs: `data_local_dir` count=1 (cfg-развилка, комментарий про Pitfall 2); config_path() не тронут; DICTA_PIPE ×3 в transport_win.rs (resolve_pipe_name, паритет resolvePipePath фазы 6) |
| 8 | Payload-приватность на Windows-пути: eprintln вставки НЕ содержит текст диктовки | ✓ VERIFIED (live) | `sed -n '/cfg(windows)/,/^    }/p' crates/dictad/src/main.rs \| grep -c '{text'` = **0** (живой прогон); Linux-логи не изменены (diff показывает cfg-пары, юникс-строки как есть) |
| 9 | Flutter IN-02-остаток: error-продвижение + poll-причина; батарея 33 файла, analyze 0, cyrillic 0 | ✓ VERIFIED (live) | Живой прогон: `flutter test` — **+198 All tests passed** / 33 файла; `flutter analyze` — No issues found; `bash tool/cyrillic_gate.sh` — exit 0; `catch (_)` = 0 в hotkey_card.dart и providers.dart; hotkey_card.dart: parsed.error → StateError → AsyncError, catch сужен до IOException/TimeoutException; providers.dart: disconnected несёт e.runtimeType (без payload); новый hotkey_card_error_test.dart (4 кейса против фейка, DICTA_CONFIG/temp без живого пайпа) |

**Score:** 9/9 truths verified-live (0 gaps). Backstop/human: 3 (см. behavior_unverified_items) — поглощены 07-UAT.md (UAT-7-1/7-2/7-3).

### Backstop-истины (human_needed — НЕ passed, авто-pass запрещён)

| # | Истина | Куда поглощена |
|---|--------|----------------|
| B1 | Живой звук WASAPI → whisper.cpp, та же модель/конфиг (roadmap SC п.1, WIN-03 backstop) | 07-UAT.md UAT-7-1 |
| B2 | Физическое нажатие хоткея в живой сессии (RegisterHotKey delivery) | 07-UAT.md UAT-7-1 |
| B3 | Живое подключение Flutter-приложения к пайп-серверу на Windows (06-UAT U-3, RESEARCH A8) | 07-UAT.md UAT-7-2 |

### Prohibitions (8/8 respected)

| Must-NOT | Проверка | Статус |
|----------|----------|--------|
| 1. Linux-пути/протокол/Flutter-транспорт не тронуты | windows_engine.dart/engine.dart в diff ОТСУТСТВУЮТ; framing/golden фаз 1-6 зелёные в живой батарее | ✓ VERIFIED |
| 2. НЕ регистрировать хоткеи в Flutter | app/lib дифф = только hotkey_card/providers + тест; RegisterHotKey только в hotkey_win.rs (Rust-демон) | ✓ VERIFIED |
| 3. НЕ портировать dicta-ui; spawn_ui на Windows → честная ошибка | dicta-ui в diff отсутствует; main.rs:375 `eprintln!("dictad: оверлей доступен только на Linux")` под cfg(windows), БЕЗ спавна | ✓ VERIFIED |
| 4. НЕ вводить новых Flutter-зависимостей; Rust — только cpal/interprocess(+замыкание) | app/pubspec.lock в diff отсутствует; новые Rust-deps: interprocess 2.4.8 (пин), widestring, cpal 0.18.2, windows 0.58 (+Win32_Security_Authorization / Win32_UI_Input_KeyboardAndMouse — объяснимые замыкания, DISPOSITION п.4) | ✓ VERIFIED |
| 5. НЕ хранить аудио/тексты на диске; payload не логируется | capture_win без temp-WAV (grep=0); `{text` в cfg(windows)-блоках = 0 | ✓ VERIFIED |
| 6. НЕ merge windows→main; push только после CI-проверки | ветка windows живая (20 коммитов), main не содержит кода; каждый push-волновой run success | ✓ VERIFIED |
| 7. DACL без everyone/ALL APPLICATION PACKAGES | SDDL-тест + grep WD = 0; S-1-15-2/AU отсутствуют | ✓ VERIFIED |
| 8. inject_win ТОЛЬКО честная ошибка до спика | `anyhow::bail!("вставка на Windows появится после спика (07-02, 07-SPIKE-RESULT.md)")`; warm_up_keyboard → Ok | ✓ VERIFIED |

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|--------|
| crates/dicta-core/src/combo.rs | NEW, дословный перенос Combo::parse | ✓ VERIFIED | md5 блока `pub struct Combo…` == исходному inject.rs@23049d6 (4a41e476cfa2739678a26525e28b8c30) — байт-в-байт |
| crates/dicta-core/src/hotkey_win.rs | NEW, RegisterHotKey без окна | ✓ VERIFIED | 4 unit-теста зелёные в CI; CreateWindowEx = 0 |
| crates/dicta-core/src/capture_win.rs | NEW, cpal/WASAPI Recorder | ✓ VERIFIED | контракт capture.rs сохранён; 2 unit-теста зелёные в CI |
| crates/dicta-core/src/inject_win.rs | NEW, честная ошибка 07-02 | ✓ VERIFIED | bail-строка с ссылкой на спик |
| crates/dictad/src/transport_win.rs | NEW, named pipe + DACL + DICTA_PIPE | ✓ VERIFIED | SDDL-тест + probe-тест зелёные в CI; serve → serve_conn (main.rs:415) |
| crates/dictad/tests/pipe_roundtrip.rs | NEW, интеграционный status-roundtrip | ✓ VERIFIED | живой прогон ok на windows-раннере (CI-лог) |
| app/test/hotkey_card_error_test.dart | NEW, error-ветка против фейка | ✓ VERIFIED | входит в живые 198/33 |
| .github/workflows/windows-build.yml | rust-джоба + артефакт | ✓ VERIFIED | 5 живых success-прогонов; артефакт dictad-windows-x64 доступен |

### Key Link Verification

| From | To | Via | Status |
|------|----|----|--------|
| transport_win.rs | dictad/src/main.rs serve_conn | transport_win::serve → accept-цикл → crate::serve_conn (transport_win.rs:141) | ✓ WIRED |
| windows_engine.dart (фаза 6) | transport_win.rs | kDictaPipe '\\.\pipe\dicta' == DEFAULT_PIPE_NAME, полный путь, pipe_mode::Bytes | ✓ WIRED (файл фазы 6 не тронут) |
| windows-build.yml | GitHub Actions | scoped build/test + upload-artifact, live success ×5 | ✓ WIRED |
| dicta-core/src/lib.rs | Linux-сборка | module-level cfg(unix)/cfg(windows) + Recorder-реэкспорт за cfg | ✓ WIRED (Linux-батарея зелёная) |
| capture_win.rs | dictad main.rs (run_start/run_stop) | тот же API Recorder { start, stop, elapsed, level } (main.rs:29-31 cfg-пара импортов) | ✓ WIRED |
| hotkey_win.rs | dictad main.rs (блок DICTA_HOTKEY) | cfg-пара hotkey::listen / hotkey_win::listen (main.rs:343-345), 'off'-гейт и env-шов общие | ✓ WIRED |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
|----------|--------------|--------|-------------------|--------|
| transport_win.rs | pipe name | DICTA_PIPE env → DEFAULT_PIPE_NAME | реальный слушатель \\.\pipe\dicta (CI-прогон живой) | ✓ FLOWING |
| pipe_roundtrip.rs | status reply | живой dictad.exe через named pipe | JSON state=idle/error=null/level=0 | ✓ FLOWING |
| capture_win.rs | samples | cpal default_input_device (native config) | живой поток — на Windows-машине (backstop B1); в CI — чистые функции | ✓ FLOWING (код) / backstop (живой звук) |
| hotkey_card.dart | config text | engine.fetchSettings() → reply → snapshot.configPath → File | фейк-тесты; живой клиент — backstop B3 | ✓ FLOWING (код) |

### Behavioral Spot-Checks (живые)

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| Rust-батарея | cargo build --release --locked --workspace && cargo test --workspace && cargo check --locked --no-default-features -p dicta-core -p dictad | BUILD-OK / 14+25+42+3 passed / CHECK-OK (exit 0, 23:34–23:35) | ✓ PASS |
| Flutter-батарея | flutter test && flutter analyze && bash tool/cyrillic_gate.sh | +198 All tests passed / No issues found / exit 0 | ✓ PASS |
| pipe_roundtrip на windows | CI run 45066485292 (headSha 6313014) | `status_roundtrip_over_named_pipe ... ok` в живом логе | ✓ PASS |
| hotkey/capture unit-тесты на windows | тот же run | 4 hotkey + 2 capture теста ok; dicta-core 20 passed, dictad 5 passed | ✓ PASS |
| CI-прогоны ×5 | gh run view … --json conclusion | все success; final 45066485292 на HEAD 6313014 | ✓ PASS |

### Probe Execution

План probe-скриптов не декларирует (scripts/*/tests/probe-*.sh отсутствуют); живые гейты плана — батарея + CI, прогнаны выше. N/A.

### Requirements Coverage

| Req | Source Plan | Description | Status | Evidence |
|-----|------------|-------------|--------|----------|
| WIN-03 | 07-01 | Захват cpal/WASAPI + RegisterHotKey без окна | ✓ SATISFIED (код — live CI) / backstop живая диктовка → UAT-7-1 | capture_win + hotkey_win + 6 unit-тестов на windows-раннере |
| WIN-04 | 07-01 (фундамент) + 07-02 | Пайп-сервер + DACL (07-01); вставка SendInput (07-02) | ✓ SATISFIED для фундамента (transport_win + pipe_roundtrip live); вставка — deferred 07-02 (Must-NOT 8) | roadmap SC п.2/п.3/п.4 → 07-02-PLAN.md |

Orphaned requirements: нет (фаза 7 = WIN-03/WIN-04, оба покрыты планами).

### Test Quality Audit

| Test File | Linked Req | Active | Skipped | Circular | Assertion Level | Verdict |
|-----------|-----------|--------|---------|----------|-----------------|---------|
| pipe_roundtrip.rs | WIN-04 | 1 | 0 | нет | Value (state=="idle", error null, level 0) | OK — живой end-to-end |
| hotkey_win tests | WIN-03 | 4 | 0 | нет | Value (mods==MOD_ALT\|MOD_NOREPEAT, vk==VK_V) | OK |
| capture_win tests | WIN-03 | 2 | 0 | нет | Value (кламп диапазона, порог overflow) | OK |
| transport_win tests | WIN-04 | 2 | 0 | нет | Value (SDDL-строка, probe до/после listener) | OK |
| hotkey_card_error_test.dart | IN-02 | 4 | 0 | нет | Behavioral (AsyncError vs фолбэк vs snapshot) | OK |

Disabled tests: 0. Circular: 0 (expected values — константы/VK-коды, не генерированы системой). Insufficient assertions: 0.

### Anti-Patterns Found

Ни одного: TBD/FIXME/XXX/TODO/HACK/PLACEHOLDER — 0 по всем 13 изменённым исходникам; `catch (_)` — 0 (удалён, IN-02); пустых реализаций нет; единственная «заглушка» inject_win — задекларированная честная ошибка (Known Stub 07-02, Must-NOT 8) с bail-строкой.

### Freeze Audit (git diff 23049d6..windows)

18 файлов, все якорные: lib.rs, combo.rs, inject.rs, config.rs, hotkey_win.rs, capture_win.rs, inject_win.rs, dicta-core/Cargo.toml, dictad main.rs, transport_win.rs, dictad Cargo.toml, pipe_roundtrip.rs, dictactl main.rs, Cargo.lock, windows-build.yml, hotkey_card.dart, providers.dart, hotkey_card_error_test.dart. **Посторонних файлов нет**: audio/filter/stt/focus/atspi — пусто; ci.yml/release.yml — пусто; dicta-ui/dicta-settings — не тронуты; windows_engine.dart/engine.dart/protocol — не тронуты; app/pubspec.lock — не тронут.

### Commits Audit (23049d6..windows = 20)

- 12 план-коммитов атомарны per-task: 3055c38 (combo), dc425dc (cfg+stubs), ad6a053 (transport+DACL+lock), 3ff1e63 (roundtrip), 0d75211 (CI), 52dd5f2 (capture), cf062c0 (hotkey; touch capture_win = one-line unused_mut fix, задокументирован в теле), 8fb0d96 (IN-02 Flutter), + 3 fix-loop (b161b26/8ee6154/1bf2151 — cpal 0.18 API, объяснимы: «первая Windows-компиляция вскрывает пропуски», каждая — один файл/одна причина) + ef8e512 (unsafe GetLastError).
- 8 review-коммитов (cad935a, b90623b, 753fe3d, 7984e0a, ab82cb1, fbaa7a9, 6d8810d, 6313014) — по одному на каждый пункт 07-01-REVIEW.md Fixed (WR-01/WR-02/IN-01/IN-03/IN-04/IN-05/IN-06/IN-08), соответствие подтверждено 07-01-REVIEW-DISPOSITION.md; CI после фиксов = success (45066485292 на 6313014).
- Все 20 — предки HEAD 6313014. MERGE windows→main НЕ выполнялся.

### Decision Coverage

Skipped — no trackable decisions (`check.decision-coverage-verify`: `{skipped: true, total: 0}`, CONTEXT.md без `<decisions>`-блока).

### Human Verification Required

Все три пункта поглощены **07-UAT.md** (создан этой верификацией):

### 1. UAT-7-1 — Живая диктовка на Windows (WIN-03 backstop: живой звук + физический хоткей)

**Test:** Скачать dictad-windows-x64 из CI run 45066485292; на живой Windows-машине с конфигом и моделью в %LOCALAPPDATA%\whisper-cpp задать DICTA_HOTKEY, физически нажать комбинацию, проговорить фразу, остановить.
**Expected:** Запись стартует без окна; уровень живой (не тишина — native-config); whisper распознаёт текст той же моделью/конфигом; занятая комбинация → честная ошибка.
**Why human:** CI-раннер без аудио-устройств и без живой очереди сообщений WM_HOTKEY.

### 2. UAT-7-2 — Подключение Flutter-приложения к живому Windows-демону (поглощает 06-UAT U-3)

**Test:** Запустить dictad.exe, затем dicta_app.exe (артефакт фазы 6).
**Expected:** Connected-статус через \\.\pipe\dicta; status/start/stop по замороженному JSON-протоколу; kill демона → баннер с причиной (IN-02 poll-ветка, e.runtimeType).
**Why human:** Живое межпроцессное подключение двух процессов на Windows-десктопе.

### 3. UAT-7-3 — Спик + 4 кейса вставки (07-02)

**Test:** Спик-протокол §5 (9 замеров delayed render) → 07-SPIKE-RESULT.md; затем 4 кейса вставки (обычное/elevated-UIPI/зажатый модификатор/Chromium) + оптаут-форматы/восстановление буфера.
**Expected:** Выполняется планом 07-02 (Must-NOT 8: спик до production-вставки); в 07-01 insert_text честно bail'ит.
**Why human:** Физически manual-кейсы на живой машине.

### Gaps Summary

Промахов нет: 9/9 истин verified-live, 8/8 prohibitions, 6/6 key links, аудит заморозки чист, батарея зелёная живым прогоном, CI ×5 success (финальный на HEAD 6313014). Статус **human_needed** — ровно три backstop-истины плана (verification §7: авто-pass запрещён), поглощённые 07-UAT.md (UAT-7-1/7-2/7-3). Roadmap SC п.2/п.3/п.4 отнесены к 07-02 (deferred).

---

_Verified: 2026-11-22T20:37:52Z_
_Verifier: the agent (gsd-verifier), живые прогоны на windows HEAD 6313014_

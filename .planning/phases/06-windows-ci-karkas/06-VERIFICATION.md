---
phase: 06-windows-ci-karkas
verified: 2026-11-22T18:14:00Z
status: human_needed
score: 12/13 must-haves verified
covered_files:
  - ".github/workflows/windows-build.yml"
  - "README.md"
  - "README.ru.md"
  - "app/lib/core/config_path.dart"
  - "app/lib/core/hf/hf_catalog.dart"
  - "app/lib/core/providers.dart"
  - "app/lib/core/windows_engine.dart"
  - "app/pubspec.lock"
  - "app/pubspec.yaml"
  - "app/test/windows_engine_test.dart"
  - "app/test/windows_paths_test.dart"
covered_digest: "v4:sha256:00ab01bcb78de8bf713eec8d657e318e3224002c9443a0151802bd0333558bb3"
behavior_unverified: 1
overrides_applied: 0
behavior_unverified_items:
  - truth: "ROADMAP SC3 (residual): живая Windows-сборка показывает ту же оболочку и баннер «демон недоступен» при отсутствии пайп-сервера, подключается через named pipe к живому серверу"
    test: "Запустить dicta_app.exe из zip-артефакта на живой Windows-машине (без пайп-сервера фазы 7)"
    expected: "Оболочка с теми же вкладками/темами, баннер «демон недоступен», нет крашей; с сервером фазы 7 — подключение через \\\\.\\pipe\\dicta"
    why_human: "Требует живую Windows-машину и пайп-сервер фазы 7; CI доказывает сборку и артефакт, но не runtime-поведение GUI. Задекларировано backstop'ом в 06-01-SUMMARY.md (D5) и 06-REVIEW-DISPOSITION.md (IN-04 ACCEPTED)"
human_verification:
  - test: "Скачать артефакт dicta-app-windows-x64 (run 45053175664 или новее), распаковать весь zip, запустить dicta_app.exe на живой Windows-машине БЕЗ пайп-сервера"
    expected: "Оболочка стартует из трея, тот же сайдбар (Дашборд/История/Модели/Настройки), тёмная/светлая тема Material You переключается, баннер «демон недоступен» (канон фазы 1), нет крашей"
    why_human: "Визуальный паритет и runtime-поведение GUI на Windows не автоматизируемы из этого окружения (нет Windows-машины); CI доказывает только сборку"
  - test: "На той же машине с Process Monitor (ProcMon) или eyesight проверить пути живого процесса при работе оболочки"
    expected: "Процесс читает %APPDATA%\\dicta\\config.toml и модель из %LOCALAPPDATA%\\whisper-cpp; никаких обращений к /run/user, ~/.config, XDG_*"
    why_human: "Пути кода покрыты injectable-фейк-тестами (windows_paths_test.dart, 10 тестов), но фактическое окружение живого процесса видно только на живой машине"
  - test: "После появления пайп-сервера фазы 7: запустить сервер, затем dicta_app.exe — проверить подключение и полный цикл статуса"
    expected: "Оболочка переключается на connected-статус через \\\\.\\pipe\\dicta, статус/старт/стоп/инъекция работают по тому же JSON-протоколу"
    why_human: "В фазе 6 сервера не существует (locked decision 2) — подключение к живому пайпу не может быть проверено ранее фазы 7"
---

# Phase 6: Windows CI + каркас — Verification Report

**Phase Goal:** Windows-ветка существует и не гниёт: CI зелёный на пустом скелете ДО кода фич, мост Flutter↔движок выбран, пути и безопасность трубы правильные
**Verified:** 2026-11-22T18:14:00Z
**Status:** human_needed
**Re-verification:** No — initial verification

**Ветка-состояние на момент верификации:** код — ветка `windows` (origin/windows = 872fbbf, head после 4 review-фиксов); план/SUMMARY — main (e1c21ab). Fingerprint снят с чекаута ветки `windows` — implementation-файлы существуют только там.

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
| --- | --- | --- | --- |
| 1 | Ветка `windows` создана от d07763a, workflow-коммит ПЕРВЫЙ в истории | ✓ VERIFIED | `git merge-base d07763a windows` = d07763a; `git log --oneline d07763a..windows \| tail -1` = `e217f4a ci(06-01): windows-build.yml` (живой прогон на чекауте windows); origin/windows = 872fbbf |
| 2 | CI зелёный ДВАЖДЫ (скелет + фичи), живые run-id в SUMMARY | ✓ VERIFIED | `gh run list`: 45050118081 (e217f4a, пустой скелет) **success**, 45051000549 (930fc66, фичи) **success**; SUMMARY на main (e1c21ab) содержит оба id дословно; **+ post-review прогон 45053175664 (872fbbf) watched live → conclusion success** (финальное состояние ветки) |
| 3 | Артефакт zip: dicta_app.exe + flutter_windows.dll + data/ | ✓ VERIFIED | `gh run download 45051000549`; `unzip -l`: dicta_app.exe (90,624 B), flutter_windows.dll (21,332,480 B), cnativeapi.dll, dynamic_color_plugin.dll, data/app.so + icudtl.dat + flutter_assets (трей-иконки, шейдеры, шрифты) — 31,194,105 B / 19 файлов, весь Release-каталог |
| 4 | dart_ipc 2.0.2 — единственная новая зависимость, lock-замыкание чистое | ✓ VERIFIED | pubspec.yaml-diff: `+ dart_ipc: 2.0.2` (точный пин, без caret); pubspec.lock-diff: только заголовки dart_ipc (direct) / win32 6.4.0 / ffi_leak_tracker 0.1.2 (transitive), +24 строки, 0 удалений, никаких других пакетов |
| 5 | WindowsPipeEngine implements DictaEngine (lib==2), пайп-литерал только в kDictaPipe, on IOException + .timeout | ✓ VERIFIED | `implements DictaEngine` в lib = 2 (daemon_engine.dart:39, windows_engine.dart:58), test = 1; литерал `\\.\pipe\dicta` ровно 1 (windows_engine.dart:20 kDictaPipe); on IOException ×4, `.timeout()` (connect 2 s / status 2 s / set 30 s / inject 120 s), on SocketException = 0 |
| 6 | Platform.is ровно 2 точки в lib, 0 в features/ | ✓ VERIFIED | providers.dart:18 (engineProvider) + config_path.dart:29 (resolveConfigPath); `grep Platform.is app/lib/features` = 0 |
| 7 | Windows-пути %APPDATA%/%LOCALAPPDATA% через injectable env, без юникс-хардкодов | ✓ VERIFIED | quoted `'APPDATA'`=1 (config_path.dart:48), `'LOCALAPPDATA'`=1 (hf_catalog.dart:65), оба через инжектируемые параметры; grep `^\+.*(/run/user\|~/\.config\|XDG_)` по диффу app/lib = 0; windows_paths_test.dart — 10 контракт-тестов, все зелёные |
| 8 | DACL-контракт записан в README ×2 + обязательства фазы 7 | ✓ VERIFIED | README.md якорь "named pipe DACL" = 1; README.ru.md якорь «DACL именованного канала» = 1; оба содержат: security descriptor только текущий user SID, crate `interprocess`, `data_dir()`→`data_local_dir()` |
| 9 | Протокол/Rust НЕ тронуты | ✓ VERIFIED | `git diff d07763a windows -- crates/` пуст; daemon_engine.dart / ci.yml / release.yml — диффы 0 строк (байт-в-байт) |
| 10 | Linux-UDS путь daemon_engine.dart НЕ изменён; развилка только в провайдере | ✓ VERIFIED | дифф daemon_engine.dart пуст; единственная точка выбора транспорта — engineProvider (providers.dart:18-20) |
| 11 | Linux-батарея зелёная (32 файла: 30 существующих + 2 новых) | ✓ VERIFIED | Живой прогон на чекауте windows: `flutter test` → **194/194 All tests passed** (32 файла); `flutter analyze` → No issues found!; `bash app/tool/cyrillic_gate.sh` → exit 0 |
| 12 | Ни один Linux-сьют не инстанцирует WindowsPipeEngine с реальным пайпом | ✓ VERIFIED | grep kDictaPipe / пайп-литерала в app/test = 0; windows_engine_test.dart использует `dart_ipc.bind` на инжектированном temp-UDS + settable platform interface |
| 13 | ROADMAP SC3 (остаток): живая сборка показывает ту же оболочку, баннер, подключается к живому пайп-серверу | ⚠️ PRESENT_BEHAVIOR_UNVERIFIED | Код и артефакт на месте (CI zip собран; транспортный шов покрыт unit-тестами через dart_ipc UDS), но runtime-поведение на живой Windows-машине требует пайп-сервера фазы 7 — backstop по 06-01-SUMMARY.md D5, 06-VALIDATION.md:24, 06-REVIEW-DISPOSITION.md IN-04 → 06-UAT.md |

**Score:** 12/13 truths verified (1 present, behavior-unverified → 06-UAT.md)

SC1→truth 2 (CI первым на пустом скелете), SC2→truth 1 (ветка от запушенного main = d07763a, Linux-workstream не блокируется: main ушёл вперёд docs-коммитами e1c21ab/f0fb34f), SC3→truths 5/6 + остаток truth 13, SC4→truths 7/8. Дедупликация: план не вычитает roadmap-SC.

### Required Artifacts

| Artifact | Expected | Status | Details |
| --- | --- | --- | --- |
| `.github/workflows/windows-build.yml` | Windows CI, единственный новый workflow | ✓ VERIFIED | 38 строк substantive: runs-on windows-latest, flutter 3.50.1, build --release, Compress-Archive, upload-artifact@v4, timeout-minutes: 30 (WR-04); wired — 3 живых зелёных прогона |
| `app/lib/core/windows_engine.dart` | Windows named pipe транспорт за швом DictaEngine | ✓ VERIFIED | 223 строки substantive; implements DictaEngine через переиспользование build*Line/decodeReplyMap (не дублирует); wired через engineProvider |
| `app/test/windows_engine_test.dart` | Linux-фейк через dart_ipc.bind UDS | ✓ VERIFIED | 6 активных тестов (roundtrip, transport-error snapshot/reply, timeout hint, connect-budget, destroyCount==1); 0 skip-паттернов |
| `app/test/windows_paths_test.dart` | env-фейк контракт-тесты путей | ✓ VERIFIED | 10 активных тестов (APPDATA/LOCALAPPDATA/XDG-ветки/override/регрессия Linux HOME); 0 skip-паттернов |
| README.md / README.ru.md DACL-контракт | Якоря + обязательства фазы 7 | ✓ VERIFIED | Якоря 1/1, user-SID/interprocess/data_local_dir в обоих |
| Ветка `windows` на origin | Живёт после фазы для фазы 7 | ✓ VERIFIED | origin/windows = 872fbbf; merge в main НЕ выполнялся |

### Key Link Verification

| From | To | Via | Status | Details |
| --- | --- | --- | --- | --- |
| providers.dart | windows_engine.dart | `Platform.isWindows ? WindowsPipeEngine() : DaemonEngine(resolveSocketPath())` | ✓ WIRED | providers.dart:18-20, живой код |
| windows_engine.dart | dart_ipc 2.0.2 | `connect(pipePath).timeout(_connectTimeout)` | ✓ WIRED | windows_engine.dart:71-79; dart_ipc не имеет параметра timeout — обёртка с реапом позднего коннекта (WR-02) |
| windows-build.yml | GitHub Actions (ветка windows) | push branches [windows] + workflow_dispatch → build → zip → artifact | ✓ WIRED | 3 прогона success; последний 45053175664 на 872fbbf |
| config_path.dart | Platform.environment['APPDATA'] | `windowsConfigPath({String? appData})` инжектируемый | ✓ WIRED | config_path.dart:29→48; ветка ДО Unix-XDG резолва |
| hf_catalog.dart | Platform.environment['LOCALAPPDATA'] | `destDirFrom(model, xdg, home, localAppData)` | ✓ WIRED | destDir(model) сигнатура не сломана — downloads.dart:92 и models_screen.dart:117 вызывают без изменений; явный parent-путь модели всё ещё выигрывает (WR-03 backslash) |
| README.md / README.ru.md | Фаза 7 (ТЗ) | DACL-контракт: только текущий user SID; interprocess; data_local_dir() | ✓ WIRED | Оба якоря + 3 обязательства (a/b/c) |

Примечание: `query verify.key-links` на main-чекате даёт "source not found" для 4 ссылок — это артефакт хореографии веток (implementation-файлы существуют ТОЛЬКО на `windows`), не разрыв связи. Все 6 линков проверены вручную на чекауте `windows` (файл:строка выше); на чекауте windows они существуют и связаны.

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
| --- | --- | --- | --- | --- |
| windows_engine.dart | pipePath | kDictaPipe / DICTA_PIPE override | ✓ (константа by design — транспорт не рендерит данные) | ✓ FLOWING |
| engineProvider | DictaEngine instance | Platform.isWindows fork | ✓ | ✓ FLOWING |
| config_path.dart windows-ветка | roaming | `Platform.environment['APPDATA']` (инжектируемый) | ✓ | ✓ FLOWING |
| hf_catalog.dart windows-ветка | localAppData | `Platform.environment['LOCALAPPDATA']` (инжектируемый) | ✓ | ✓ FLOWING |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
| --- | --- | --- | --- |
| Linux-батарея на ветке windows | `flutter test` (app/) | 194/194 All tests passed, 32 файла | ✓ PASS |
| Статический анализ | `flutter analyze` | No issues found! | ✓ PASS |
| Кириллица-гейт | `bash app/tool/cyrillic_gate.sh` | exit 0 | ✓ PASS |
| CI скелет (первый deliverable) | `gh run list` → 45050118081 | conclusion=success | ✓ PASS |
| CI после фич-кода | `gh run list` → 45051000549 | conclusion=success | ✓ PASS |
| CI после review-фиксов (финальное состояние) | `gh run watch 45053175664 --exit-status` (headSha 872fbbf) | exit 0, conclusion=success, job 3m28s | ✓ PASS |
| Артефакт содержит exe+dll+data | `gh run download` + `unzip -l` | 19 файлов, 31,194,105 B | ✓ PASS |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
| --- | --- | --- | --- | --- |
| WIN-01 | 06-01-PLAN.md | workflow зелёный zip на windows-раннере, ДО кода фич | ✓ SATISFIED | adjacency e217f4a первым + run 45050118081 success (скелет) + artifact verified |
| WIN-02 | 06-01-PLAN.md | та же оболочка, named pipe `\\.\pipe\dicta`, тот же JSON-протокол | ✓ SATISFIED (код) / ⚠ UAT (живой runtime) | WindowsPipeEngine за швом DictaEngine, unit-тесты через dart_ipc UDS, протокол переиспользован; живая оболочка → 06-UAT.md |
| WIN-05 | 06-01-PLAN.md | %APPDATA%/%LOCALAPPDATA% без юникс-хардкодов; DACL записан | ✓ SATISFIED | truth 7 + truth 8; реализация DACL — фаза 7 (locked decision 3, контракт записан) |

Orphaned requirements: нет — REQUIREMENTS.md маппит WIN-01/02/05 на Phase 6, все покрыты планом 06-01.

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
| --- | --- | --- | --- | --- |
| — | — | TBD/FIXME/XXX/TODO/HACK/PLACEHOLDER/skip в диффе фазы (app/lib, app/test, workflows, README) | 0 находок | — |
| — | — | Прочие | 0 находок | — |

Debt-marker gate: дифф `d07763a..windows` по всем кодовым путям фазы не содержит ни одного маркера долга.

### Test Quality Audit

| Test File | Linked Req | Active | Skipped | Circular | Assertion Level | Verdict |
| --- | --- | --- | --- | --- | --- | --- |
| windows_engine_test.dart | WIN-02 | 6 | 0 | 0 | Behavioral (roundtrip snapshot, destroyCount==1, budget в сообщении) | OK |
| windows_paths_test.dart | WIN-05 | 10 | 0 | 0 | Value (точные пути, precedence, регрессия Linux) | OK |

Disabled tests: 0. Circular: 0 (фейки — собственный dart_ipc platform interface + UDS, не система-под-тестом). Существующие сьюты не модифицированы (дифф app/test называет только 2 новых файла).

### Human Verification Required

Три backstop-пункта (живая Windows-машина) — вынесены в **06-UAT.md**:

1. **Оболочка/баннер на живой Windows** — запустить dicta_app.exe без пайп-сервера: та же оболочка, баннер «демон недоступен», нет крашей.
2. **Пути живого процесса** — ProcMon: %APPDATA%\dicta\config.toml, %LOCALAPPDATA%\whisper-cpp; никаких XDG_//run/user/~/.config.
3. **Подключение к живому пайп-серверу** — после фазы 7: connected-статус через `\\.\pipe\dicta`, полный цикл команд по тому же JSON-протоколу.

### Gaps Summary

Провалов нет. Все 12 истин плана подтверждены живыми прогонами (CI ×3 success, unzip артефакта, git-диффы заморозок, grep-гейты, 194/194 тестов, analyze 0, cyrillic 0). Единственный незакрытый пункт — остаток SC3 (runtime-поведение на живой Windows-машине), который по locked decision 2 физически не может быть проверен в фазе 6 (пайп-сервера ещё не существует) и задекларирован backstop'ом → 06-UAT.md, отсюда `status: human_needed`.

Замечание по хореографии веток: локальная ветка `windows` опережает origin на 1 docs-коммит (23049d6 review disposition) — не влияет на CI и не является гэпом верификации.

### Decision Coverage

CONTEXT.md (main) использует свободный формат `## Locked Decisions` (8 решений), не `<decisions>`-блок — формальный гейт пропускается по формату. Ручная сверка: все 8 locked decisions отражены в артефактах фазы — 1 (named pipe мост) → truth 5; 2 (демон не портируется, баннер) → UAT-пункт 1/3; 3 (DACL контракт) → truth 8; 4 (CI первым на пустом скелете) → truth 1/2; 5 (отдельная ветка, .planning на main) → truth 1 + SUMMARY e1c21ab на main; 6 (единственная зависимость dart_ipc) → truth 4; 7 (протокол/Rust/UDS не тронуты) → truth 9/10; 8 (кириллица/батарея) → truth 11.

---

_Verified: 2026-11-22T18:14:00Z_
_Verifier: the agent (gsd-verifier)_

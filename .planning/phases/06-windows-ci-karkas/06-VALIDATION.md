---
phase: 6
phase_name: Windows CI + каркас
requirements: [WIN-01, WIN-02, WIN-05]
plan: 06-01-PLAN.md
status: draft
---

# Phase 6 Validation Strategy

## Nyquist Edge Coverage (edge-probe резолюции)

| REQ | Грани | Категория | Резолюция | Верификация |
|---|---|---|---|---|
| WIN-01 | adjacency (workflow ДО фич-кода в истории ветки); idempotency (повторный прогон зелёный); empty-input (zip непустой, содержит exe); concurrency (workflow не зависит от main-Rust-CI) | adjacency + idempotency | explicit | git-гейт порядка коммитов на ветке windows (workflow-коммит первый, до dart_ipc); живой прогон `gh run list --workflow windows-build.yml` conclusion=success ×2 (первый + после фич-кода); artifact zip содержит dicta_app.exe |
| WIN-02 | паритет оболочки; транспорт изолирован швом; штатный баннер при недоступном пайпе | mixed | explicit + backstop | explicit: WindowsPipeEngine implements DictaEngine (grep==1 lib), имя пайпа в единственной константе, `on IOException` + `.timeout()` ошибки (research: Win32NamedPipeException implements IOException, не SocketException), Linux-фейк-тесты всех команд; backstop: та же оболочка глазами на Windows-сборке — UAT-пункт |
| WIN-05 | отсутствие юниксовых хардкодов в Windows-ветках кода; DACL-контракт записан | mixed | explicit + backstop | explicit: grep `/run/user\|~/.config\|XDG_` в windows-ветках кода ==0 (достижимость только через изолированные env-инжектируемые функции по паттерну config_path.dart:14-35); %APPDATA%/%LOCALAPPDATA% через Platform.environment с injectable-параметрами; DACL-контракт — README/decision якорь [HEAD=0 EXPECT==1] + запись обязательства фазы 7 (data_local_dir, PITFALLS:234); backstop: живая Windows-машина — UAT |

## Sampling / Verification Modes

- **Linux-локально (каждая задача)**: flutter test (все существующие сьюты не ломаются), flutter analyze, bash tool/cyrillic_gate.sh, grep-гейты структуры (implements==1 lib/test, Platform.is* в lib — только точка развилки в providers).
- **Linux-фейки для Windows-кода**: dart_ipc компилируется на Linux (research: живая проба UDS roundtrip), WindowsPipeEngine тестируется через FakeTransport-инжекцию; исключения обёртки `.timeout()` проверяемы на Linux.
- **Живой CI-прогон**: gh CLI 2.104.0 authenticated (workflow scope); верификация `gh run list/watch --workflow windows-build.yml` на ветке windows; фиксировать conclusion в SUMMARY/VERIFICATION (не блокировать задачи ожиданием — финальный статус в гейт-задаче).
- **UAT (backstop)**: та же оболочка на Windows-машине, баннер при отсутствии демона, визуальный паритет вкладок — пункты 06-UAT.md.

## Per-Task Verification Map

Task IDs: TBD — привязывается к 06-01-PLAN.md сразу после планирования (до чекера).

| Проверка | Task ID | Режим |
|---|---|---|
| workflow-файл + ветка windows + триггеры | TBD | Linux grep + git |
| живой CI прогон (пустой скелет) conclusion=success | TBD | gh CLI |
| dart_ipc пин + WindowsPipeEngine шов | TBD | Linux grep + тесты |
| команды/таймауты/ошибки пайп-движка | TBD | Linux фейк-тесты |
| платформенная развилка в providers | TBD | Linux grep + тесты |
| %APPDATA%/%LOCALAPPDATA% пути | TBD | Linux env-фейк тесты |
| DACL-контракт + README якорь | TBD | grep [HEAD=0 EXPECT==1] |
| повторный CI прогон после фич-кода | TBD | gh CLI |
| финальная батарея + контракты заморозки | TBD | Linux полный прогон |

## Mandatory Test Commands (базовые)

```
cd app && flutter test && flutter analyze && bash tool/cyrillic_gate.sh
cd app && grep -rn "implements DictaEngine" lib/ test/ | wc -l   # lib==1, test==1
git log --oneline windows -- .github/workflows/windows-build.yml | head -1   # первый коммит ветки
gh run list --workflow windows-build.yml --branch windows --limit 2 --json conclusion
```

# Phase 6 Context — Windows CI + каркас

**Phase**: 6 — Windows CI + каркас
**Requirements**: WIN-01, WIN-02, WIN-05
**Status**: Locked for planning (директива автономии: оркестратор принял решения, обсуждение с пользователем пропущено)

## Locked Decisions (НЕ обсуждать, НЕ пересматривать в плане)

1. **Мост Flutter ↔ движок: named pipe IPC — РЕШЕНО.**
   `\\.\pipe\dicta` + тот же JSON-line протокол; Dart-сторона — `dart_ipc` 2.0.2
   (socket-совместимый API, чистый Dart + win32 FFI). FRB-embed ОТКЛОНЁН:
   нарушает ограничение PROJECT.md «приложение = клиент демона, протокол не
   трогаем» (конвергенция STACK+PITFALLS 2/3, Pitfall 5). Это подтверждение
   research-флага, которое ROADMAP требовал «в plan-phase» — здесь оно.
2. **Демон на Windows в этой фазе НЕ портируется.** Фаза 6 = CI + каркас
   приложения. Windows-клиент подключается к несуществующему серверу → штатный
   баннер «демон недоступен» (поведение фазы 1 на Linux без демона). Пайп-сервер
   (`interprocess`, WASAPI, SendInput) — фаза 7.
3. **DACL пайпа — контракт фазы 7, зафиксировать документально** в решении/README
   фазы 6: сервер создаёт пайп с security descriptor «только текущий user SID»
   (PITFALLS.md:234). В фазе 6 верифицируется ЗАПИСЬ контракта, не реализация.
4. **CI — первый deliverable, на ПУСТОМ скелете**: `windows-build.yml` коммитится
   и зеленеет ДО любого Windows-фич-кода (dart_ipc, платформенные ветки).
   Текущее приложение (оболочка фаз 1-5) обязано собираться на windows-раннере
   как есть.
5. **Отдельная ветка `windows`** от запушенного main (origin/main = d07763a,
   push выполнен 2026-11-22). Linux-workstream (main) не блокируется.
   `.planning/`-артефакты фазы коммитятся на main (текущая рабочая ветка),
   кодовые коммиты фазы — на ветке `windows`.
6. **Единственная новая зависимость этой фазы: `dart_ipc` 2.0.2** (пин точно).
   `interprocess` — фаза 7, НЕ добавлять.
7. Протокол/крейты/Rust не трогаем; Linux UDS-путь (`daemon_engine.dart`) не
   трогаем — Windows-транспорт = НОВАЯ реализация рядом, выбор по
   `Platform.isWindows` в провайдере.
8. Кириллица — только через .arb; локальные проверки на Linux: flutter analyze
   + flutter test + cyrillic gate обязаны остаться зелёными.

## Проверенные факты (живой код на 2026-11-22)

- `app/windows/` — полный скелет от `flutter create` (runner/, CMakeLists.txt,
  flutter/); НИ ОДНОЙ Windows-сборки ещё не делалось.
- `.github/workflows/ci.yml` (Rust, ubuntu, container rust:1-trixie) и
  `release.yml` (tag-релизы) — аналоги синтаксиса; windows-build.yml — ОТДЕЛЬНЫЙ
  файл, существующие не трогать.
- В `app/lib/` НЕТ ни одного `Platform.isWindows/isLinux` — платформенный
  выбор ещё не существует; `daemon_engine.dart:19` читает XDG_RUNTIME_DIR.
- `app/lib/core/config_path.dart:24-25` — XDG_CONFIG_HOME-логика (Linux);
  `pubspec.yaml` БЕЗ `flutter:`-пина (flutter-version-file потребует добавить).
- Локальный SDK 3.50.1; CI-рецепт из STACK.md:54: `runs-on: windows-latest`,
  `subosito/flutter-action@v2`, `flutter-version: 3.50.0` ИЛИ
  `flutter-version-file: pubspec.yaml` (требует точное `flutter: 3.50.0` в pubspec).
- `dart_ipc` 2.0.2 (2026-09): bind/connect/listen, named pipe Win / UDS Unix,
  150 pub points (STACK.md:36).
- Артефакт сборки: `build/windows/x64/runner/Release/*` (exe+dll+data) → zip.

## Цели (из ROADMAP SC 1-4)

1. WIN-01: `windows-build.yml` зелёный zip-артефакт на пустом скелете, ПЕРВЫМ.
2. WIN-02: Windows-сборка показывает ту же оболочку, подключение через named
   pipe `\\.\pipe\dicta` (развилка подтверждена — locked decision 1).
3. WIN-05: пути %APPDATA%/%LOCALAPPDATA% без хардкода юниксовых; DACL-контракт
   записан (locked decision 3).

## Ограничения на план

- Задачи CI — первыми волнами; фич-код — после зелёного CI.
- Верификация «CI зелёный» — живой прогон на GitHub Actions после push ветки
  (gh CLI доступен; пайплайн ~10-20 мин; финальный статус фиксировать в
  SUMMARY/VERIFICATION, не дожидаться внутри задачи).
- Проверки на Linux (flutter test/analyze/cyrillic) не ломать: весь новый
  код платформенно-изолирован, тесты Windows-веток — на фейках.
- 17-ключевой set/формы/браузер не трогать (фазы 4 артефакты).

## Валидационные关注 (edge-probe материал)

- WIN-01: adjacency (workflow ДО dart_ipc в истории коммитов ветки),
  idempotency (повторный прогон workflow зелёный), concurrency (push-force не
  ломает), empty-input (zip непустой, содержит exe).
- WIN-02: паритет оболочки (те же вкладки), транспорт изолирован швом
  DictaEngine, fallback-баннер при недоступном пайпе.
- WIN-05: отсутствие `\\.\pipe\dicta`-хардкода вне константы транспорта;
  отсутствие `/run/user`, `~/.config` в Windows-ветках кода.

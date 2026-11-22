# Phase 6: Windows CI + каркас - Pattern Map

**Mapped:** 2026-11-22
**Files analyzed:** 8 (2 новых файла кода + workflow + тесты; 4 модифицируемых)
**Analogs found:** 5 / 8 (3 позиции — No Analog: платформенный выбор, CI-контент workflow, DACL-контракт; у всех трёх есть поведенческий источник в RESEARCH)

Все названные аналоги — git-tracked (сверено `git ls-files` 8/8: оба workflow,
весь `app/lib/core`, `app/test`, `app/pubspec.yaml`; зеркал нет). Гейты живые:
`implements DictaEngine` в `app/lib` == 1 (`daemon_engine.dart:39`), в
`app/test` == 1 (`fake_dicta_engine.dart:21` — единственная фабрика фейков);
`Platform.isWindows|isLinux` в `app/lib` — НИ ОДНОГО (CONTEXT:45, платформенный
выбор ещё не существует). Кириллица в `app/lib/**/*.dart` вне
`lib/l10n/generated/` — гейт `app/tool/cyrillic_gate.sh`; русские
строковые константы — Unicode-escape (канон `daemon_engine.dart:25-37`).

**Вход исследователя:** локального RESEARCH.md в каталоге фазы НЕТ; развилки
уже закрыты locked decisions CONTEXT.md:9-37, поведенческие источники —
глобальные `.planning/research/STACK.md` (:17-20, :36, :54, :107-117, :130) и
`PITFALLS.md` (:182, :212, :234). Открытых researcher-вопросов для планёра нет.

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|---|---|---|---|---|
| `.github/workflows/windows-build.yml` (new) | config (CI workflow) | batch (push → build → zip-артефакт) | `ci.yml:1-15` (name/on/jobs/checkout-структура) + `release.yml:42-45` (upload-artifact@v4) | role-match (синтаксис exact; Flutter/Windows-шагов в проекте нет — см. No Analog) |
| `app/lib/core/windows_engine.dart` (new; имя за планёром: pipe/named-pipe engine) | service (transport impl за шовом DictaEngine) | request-response (JSON-line, connection-per-command) | `app/lib/core/daemon_engine.dart:39-190` (весь файл) | exact (роль+поток; транспорт другой) |
| `app/lib/core/providers.dart` (modify: выбор реализации по `Platform.isWindows`) | store | — | сам файл :16-17 (engineProvider); сама платформенная развилка — No Analog | role-match |
| `app/lib/core/config_path.dart` (modify: Windows-ветка %APPDATA%) | utility | transform (env → путь) | сам файл :14-35 (resolveConfigPath) + :12-13 (инжектируемые параметры) | exact |
| `app/lib/core/hf/hf_catalog.dart` (modify: Windows-ветка %LOCALAPPDATA% в destDir) | utility | transform (env → путь) | сам файл :31-51 (destDirFrom чистая + destDir production-wrapper) | exact |
| `app/pubspec.yaml` (modify: `dart_ipc: 2.0.2`; опц. flutter-пин) | config | — | сам файл :33-36 (точечные версии) | exact |
| `app/test/windows_*_test.dart` (new: path-резолв + pipe framing юниты; имя за планёром) | test | transform / request-response (без живого транспорта) | `config_path_test.dart:1-40` (verbatim+injectable) + `daemon_engine_framing_test.dart:1-9` («no live socket») | exact |
| DACL-контракт пайпа (док-запись в новом engine + заметка фазы) | doc | — | No Analog (ср. канон док-комментариев `daemon_engine.dart:1-3,12-15`) | none (источник PITFALLS.md:234) |

## Pattern Assignments

### `.github/workflows/windows-build.yml` (CI workflow, batch)

**Analog 1 (структура workflow):** `.github/workflows/ci.yml:1-15`

```yaml
name: CI

on:
  push:
    branches: [main]
  pull_request:

jobs:
  build:
    name: build + tests (Vulkan и CPU-only)
    runs-on: ubuntu-latest
    ...
    steps:
      - uses: actions/checkout@v4
```

Копировать СКЕЛЕТ: `name:` → «Windows build», `on: push: branches: […]` →
ветка `windows` (locked decision 5; ci.yml:4-5 — точная форма записи branches),
`jobs: build:` c честным `name:` шага, `actions/checkout@v4` — та же мажорная
версия, что в обоих существующих workflow.

**Analog 2 (артефакт):** `.github/workflows/release.yml:42-45`

```yaml
      - uses: actions/upload-artifact@v4
        with:
          name: dicta-${{ matrix.name }}
          path: dist/
```

Копировать `actions/upload-artifact@v4` с `name`/`path`; путь — содержимое
`app/build/windows/x64/runner/Release/` (exe+dll+data, CONTEXT:54), упакованное
в zip (Compress-Archive на windows-раннере — PowerShell-канон; STACK.md:54
«+ рядом dictad.exe» — НЕ фаза 6, демона на Windows ещё нет).

**Контент шагов (No Analog — источник STACK.md:54, PITFALLS.md:182):**

```yaml
    runs-on: windows-latest
    defaults:
      run:
        working-directory: app
    steps:
      - uses: actions/checkout@v4
      - uses: subosito/flutter-action@v2
        with:
          flutter-version: "3.50.0"   # ИЛИ flutter-version-file: pubspec.yaml
      - run: flutter pub get
      - run: flutter config --enable-windows-desktop
      - run: flutter build windows --release
```

Вилка пина (CONTEXT:49-51): `flutter-version: "3.50.0"` прямо в workflow —
БЕЗ правки pubspec; ИЛИ `flutter-version-file: pubspec.yaml` — тогда в
`pubspec.yaml` environment обязан появиться точный `flutter: "3.50.0"`
(STACK.md:130: диапазоны не резолвятся action'ом; сейчас pubspec БЕЗ
flutter-пина, pubspec.yaml:21-22). Решение за планёром; оба варианта
равноценны, зафиксировать выбранный.

**Отличия от аналогов:** НЕ container-rust (ci.yml:12-13 — не копировать);
НЕ tag-триггер (release.yml:3-5); НЕ `permissions: contents: write`
(release.yml:7-8 — артефакт загружается без прав на релизы); отдельный
файл, ci.yml/release.yml не трогать (CONTEXT:42-44). `working-directory:
app` — новый элемент (существующие workflow живут в корне репо, ci.yml
оперирует crates/ с корня); без него `flutter pub get` упадёт.

---

### `app/lib/core/windows_engine.dart` (service, request-response)

**Analog (полный структурный клон):** `app/lib/core/daemon_engine.dart`

Копировать СТРУКТУРУ файла, менять только транспорт:

1. **Library-шапка** (:1-3) — «The only file that knows the Linux
   transport…» → «The only file that knows the Windows named pipe transport.
   `\\.\pipe\dicta`, one fresh connection per command, one JSON line out and
   one JSON line back.» Транспорт живёт ТОЛЬКО в core/ (канон фазы 5,
   hf_client.dart:1-3).
2. **Резолв пути** (:16-23, resolveSocketPath) → `resolvePipePath()`:
   `DICTA_PIPE` (test override) → константа `\\.\pipe\dicta`. Env-override
   первым — точный канон :17-18. Хардкод пайпа — ТОЛЬКО в этой константе
   (edge-probe WIN-05: отсутствие `\\.\pipe\dicta` вне константы транспорта).
3. **Русские сообщения** (:25-37) — Unicode-escape-константы; hint для
   Windows ДРУГОЙ: не «systemctl --user start dictad» (:30), а
   «демон недоступен»-баннер-семантика фазы 1 (пайп-сервера нет до фазы 7 —
   CONTEXT:15-18). `_badReply`/`_timeoutHint(budget)`-структуру (:31-37,
   отчёт секунд таймаута) — копировать как есть.
4. **Конструктор с инжектируемым путём** (:40-41,
   `DaemonEngine([String? socketPath])`) → `WindowsPipeEngine([String?
   pipePath])` — канон тестируемости (config_path.dart:12-13 ссылается именно
   на него).
5. **Таймаут-бюджеты** (:45-48): `_connectTimeout 2s`, `_statusTimeout 2s`,
   `_settingsTimeout 30s`, `_injectTimeout 120s` — те же ЧИСЛА, те же имена.
   Бюджет 120 s — контракт протокола, не транспорта.
6. **`_call`** (:50-76): connect → `write(line)` → `flush()` → `.first`
   JSON-line → `destroy()` в `finally` (:73-75). Транспорт — socket-
   совместимый API `dart_ipc` 2.0.2 (bind/connect/listen, STACK.md:36);
   ЛОГИКА чтения первой строки НЕ меняется. Обёртка connect-ошибки (:58-63):
   тип + путь пайпа + hint + `error.runtimeType` — payload privacy
   (комментарий :59 «Error type and socket path only»).
7. **`_commandLine`/`_snapshotCommand`/`set`** (:78-127, :163-189) —
   копировать byte-в-байт форму трёх веток: transport-error →
   `DictaReply(ok:false, error: msg)` / `DictaStatus(connected:false, …)`;
   `TimeoutException` → `_timeoutHint`; `catch (_)` → `_badReply`
   (комментарии :91, :184 «never mask a SocketException again» — сохранить
   смысл: транспортную ошибку НЕ маскировать форматной веткой).
8. **Все `@override`** (:129-160): `status/start/stop/inject/toggle/cancel/
   fetchSettings/set` — тот же интерфейс `DictaEngine` (engine.dart:71-91);
   wire-строки `buildStatusLine/buildSetLine/buildInjectLine` и
   `decodeReplyMap` (engine.dart:93-105) — ПЕРЕИСПОЛЬЗОВАТЬ, не дублировать
   (протокол заморожен, locked 7).

**DACL-контракт (locked 3):** док-комментарием в шапке нового engine (стиль
:12-15 — контекстный комментарий над функцией):

```dart
// DACL contract (phase 7, PITFALLS.md:234): the pipe SERVER creates
// \\.\pipe\dicta with a security descriptor granting access ONLY to the
// current user SID. Phase 6 verifies this CONTRACT is recorded, not
// implemented — the client connects, it does not create the pipe.
```

**Отличия от аналога:** импорт `dart_ipc` вместо чистого `dart:io`-сокета;
исключения dart_ipc — НЕ dart:io SocketException → ветка transport-error
ловит фактический тип пакета, но文案-канон (:58-63) сохранён; hint-строка без
systemctl; `InternetAddress(…, type: unix)` (:53-55) → dart_ipc connect по
имени пайпа.

**Безопасность на Linux-тестах:** dart_ipc 2.0.2 — чистый Dart, win32 FFI
срабатывает только при connect (STACK.md:36) → импорт файла в
providers.dart НЕ валит Linux-тесты, пока connect не вызывается вне
`Platform.isWindows`-ветки. Планёр зафиксировать: ни один Linux-сьют не
инстанцирует `WindowsPipeEngine` с реальным connect.

---

### `app/lib/core/providers.dart` (modify: платформенный выбор)

**Analog (точка выбора):** сам файл :16-17

```dart
final engineProvider = Provider<DictaEngine>((ref) =>
    DaemonEngine(resolveSocketPath()));
```

Копировать ФОРМУ: тот же `Provider<DictaEngine>`, doc-комментарий перед ним
(:13-15 «DaemonEngine holds no disposable resource…» — для Windows-ветки тот
же аргумент connection-per-command, `ref.onDispose` НЕ нужен; если dart_ipc
потребует close — гарантировать в `_call`-finally, не в провайдере).

Новая строка (контент — No Analog, источник STACK.md:20 «платформа меняет
только транспорт»):

```dart
final engineProvider = Provider<DictaEngine>((ref) =>
    Platform.isWindows ? WindowsPipeEngine() : DaemonEngine(resolveSocketPath()));
```

(импорты `dart:io` Platform + windows_engine.dart добавляются к :1-11;
форматирование — `dart format` решит.)

**Запреты:** провайдер ОСТАЁТСЯ одним швом — тесты подменяют его целиком
(`engineProvider.overrideWithValue(fake)`, fake_dicta_engine.dart:201);
выбор не выносится ни в features/, ни в main.dart; второй полл-цикл/второй
провайдер статуса не заводится (`dictaStatusProvider` :19-20 и
`DictaStatusNotifier` :65-102 не меняются — пайп и UDS дают одинаковые
 DictaStatus-снимки, errors-as-data уже внутри движков).

---

### `app/lib/core/config_path.dart` (modify: Windows %APPDATA%)

**Analog:** сам файл :14-35 (`resolveConfigPath`) + канон инжектируемых
параметров :12-13 («Parameters are injectable because Platform.environment
is immutable in widget tests»).

Копировать ПАТТЕРН разрешения: override первым на ОБЕИХ платформах
(`DICTA_CONFIG` :19-22 — не трогать), затем платформенная развилка.
Windows-ветка (по канону fallback-цепочки :24-34):

```dart
  // Windows: %APPDATA% (roaming) — the daemon's Windows contract (phase 7).
  // No Rust parity reference exists yet: this records the CONTRACT, not a
  // mirror of config.rs (contrast with the Linux branch below).
  final appData = appDataOverride ?? Platform.environment['APPDATA'];
  if (appData != null && appData.isNotEmpty) {
    return '$appData\\dicta\\config.toml';  // или Platform.pathSeparator
  }
```

Порядок ветвей — `Platform.isWindows` ДО юниксового XDG-разбора (иначе на
Windows XDG-переменные отсутствуют и сработает last-resort :33). Расширить
инжектируемые параметры (`{String? appData, …}`) — тот же паттерн, что
`:14-18`; НИКАКИХ `/run/user`, `~/.config` в Windows-ветке (edge-probe
WIN-05). Паритет-комментарии стиля :8-10/:25-27 копировать по форме, но
честно: эталона в Rust нет — пишем «contract», не «mirrors config.rs».

---

### `app/lib/core/hf/hf_catalog.dart` (modify: Windows %LOCALAPPDATA% для dest_dir)

**Analog:** сам файл :31-51.

```dart
/// Pure version of catalog.rs:70-88 for tests: no process environment reads.
String destDirFrom(String model, String? xdgDataHome, String? home) { … }

/// Production wrapper: XDG_DATA_HOME -> HOME/.local/share -> whisper-cpp.
String destDir(String model) => destDirFrom(…, Platform.environment[…], …);
```

Копировать ДВУХСЛОЙНУЮ форму: чистая функция с инжектируемыми env-параметрами
(тесты без мутации Platform.environment) + тонкий production-wrapper, читающий
`Platform.environment`. Windows-ветка: `%LOCALAPPDATA%` (НЕ %APPDATA% —
модели это данные, не roaming-настройки) → `…\whisper-cpp`. Расширение —
параметром (`String? localAppData`) в `destDirFrom` с развилкой до XDG-ветки,
production-wrapper :47-51 передаёт `Platform.environment['LOCALAPPDATA']`.
Ни одного хардкода юниксового пути в Windows-ветке; DownloadEngine
(downloads.dart:92-93) потребляет `destDir(model)` — сигнатуру НЕ ломать.

---

### `app/pubspec.yaml` (modify)

**Analog:** сам файл :33-36 — канон точечных версий:

```yaml
  flutter_riverpod: 3.5.0
  dynamic_color: 2.1.0
  window_manager: 0.7.0
  tray_manager: 0.9.0
```

Новая строка в тот же блок: `dart_ipc: 2.0.2` — точечно, БЕЗ caret
(locked 6: единственная новая зависимость фазы; `interprocess` НЕ добавлять —
фаза 7). Если планёр выберет `flutter-version-file` в workflow — добавить
`flutter: "3.50.0"` в `environment:` (:21-22) точной строкой (STACK.md:130).
После правки — `flutter pub get` локально, pubspec.lock коммитится (канон
репо).

---

### Тесты: `app/test/windows_*_test.dart` (new)

**Analog 1 (стиль verbatim/паритет):** `config_path_test.dart:1-9` — шапка
«Parity tests… copied verbatim from the frozen Rust/TOML reference»,
fixtures через `File('test/fixtures/…').readAsStringSync()`. Для
Windows-веток honest-шапка: «contract tests — no Rust reference exists yet».

**Analog 2 (юнит без живого транспорта):** `daemon_engine_framing_test.dart:1`
— «pure unit tests, no live socket». Тесты Windows-веток:
- `resolvePipePath`/`resolveConfigPath`/`destDirFrom` — чистые функции с
  инжектируемыми параметрами (env в тесте не мутабелен, канон
  config_path.dart:12-13): override-set/unset/empty, APPDATA/LOCALAPPDATA
  присутствует/отсутствует, last-resort;
- ошибочные ветки WindowsPipeEngine — БЕЗ реального connect: только
  обёртки/hint-константы (если `_call` не вынесен — тестировать формы
  сообщений на чистых данных, не пайп).

**Запрет:** НИ ОДИН Linux-сьют не инстанцирует `WindowsPipeEngine` с живым
connect (dart_ipc win32 FFI на Linux несовместим); реальный `DaemonEngine`
в тестах тоже никогда не создавался — канон «No dart:io transport is
reachable through this class» (fake_dicta_engine.dart:4). Widget-сьюты
продолжают идти через `buildFakeApp` + `FakeDictaEngine`
(fake_dicta_engine.dart:181-208, override-канон «null = не подменять»
:186-191) — новых параметров фейкам не нужно: движок подменяется ЦЕЛИКОМ,
платформенная развилка в провайдере на тесты не влияет.

**Гейт:** `implements DictaEngine` в `app/test` остаётся == 1
(fake_dicta_engine.dart:21); второй фейк НЕ заводить. В `app/lib` после фазы
станет 2 (DaemonEngine :39 + WindowsPipeEngine) — гейт был про фабрику
фейков, lib-реализации не ограничены.

---

## Shared Patterns

### Один шов — один провайдер
**Source:** `providers.dart:16-33` (engineProvider/hfClientProvider/
downloadEngineProvider/trayServiceProvider — все однострочные с
doc-комментом «overridden by Fake… in tests»).
**Apply to:** engineProvider остаётся ЕДИНСТВЕННОЙ точкой выбора
UDS/pipe; никакого второго статус-провайдера.

### Errors-as-data, три ветки
**Source:** `daemon_engine.dart:84-93` (transport/timeout/format —
`catch (_)` только для JSON-ветки, комментарий :91 «never mask a
SocketException again»).
**Apply to:** windows_engine.dart — byte-в-байт та же форма; UI никогда не
видит исключений из транспорта.

### Payload privacy
**Source:** `daemon_engine.dart:59, :172` («Error type and socket path only —
never log reply payloads»; «never log the set payload»).
**Apply to:** windows-обёртки ошибок — тип + `\\.\pipe\dicta` + runtimeType,
не тексты команд.

### Unicode-escape для русских констант
**Source:** `daemon_engine.dart:25-37` (кириллица-гейт считает байты и в
комментариях).
**Apply to:** все русские строки windows_engine.dart («демон недоступен»,
hint без systemctl).

### Инжектируемые env-параметры
**Source:** `config_path.dart:12-18`, `hf_catalog.dart:31-32`,
`daemon_engine.dart:40-41` (DaemonEngine([socketPath])).
**Apply to:** resolvePipePath, Windows-ветки config_path/hf_catalog —
Platform.environment immutable в тестах.

### CI-канон
**Source:** `ci.yml:15` + `release.yml:24,42-45` (checkout@v4, upload-
artifact@v4, честные step-имена).
**Apply to:** windows-build.yml; существующие workflow не трогать.

### Локальные проверки Linux
**Source:** CONTEXT:36-37 — flutter analyze + flutter test + cyrillic gate.
**Apply to:** гейт каждой задачи фазы: весь новый код платформенно-изолирован,
Linux-сьюты зелёные без Windows-раннера.

## Запреты-по-аналогии

1. **Не трогать Linux UDS-путь** — `daemon_engine.dart` и
   `resolveSocketPath` (:16-23) байт-в-байт; Windows-транспорт = НОВЫЙ файл
   рядом, выбор только в провайдере (locked 7; CONTEXT:32-34).
2. **Не хардкодить пути** — никаких `/run/user`, `~/.config` в
   Windows-ветках (edge-probe WIN-05); `\\.\pipe\dicta` — только константа
   транспорта; %APPDATA%/%LOCALAPPDATA% — через `Platform.environment` с
   инжектируемыми параметрами.
3. **Не добавлять транспорт в lib/features** — транспорт живёт ТОЛЬКО в
   `core/` (канон daemon_engine.dart:1-3, hf_client.dart:1-3); features/
   знают только `DictaEngine`.
4. **Не слать настоящие команды в тестах** — «pure unit tests, no live
   socket» (daemon_engine_framing_test.dart:1); ни один Linux-сьют не
   инстанцирует WindowsPipeEngine с connect; `implements DictaEngine` в
   test == 1 (второго фейка НЕ заводить).
5. **Не трогать ci.yml / release.yml** — windows-build.yml отдельный файл
   (CONTEXT:42-44); их триггеры/контейнеры не копировать в новый workflow.
6. **Новые пакеты** — ТОЛЬКО `dart_ipc: 2.0.2`, точечно (locked 6);
   `interprocess` — фаза 7, НЕ добавлять; никаких TOML/FRB-пакетов.
7. **Протокол/Rust не трогать** — `build*Line`/`decodeReplyMap`
   (engine.dart:93-105) переиспользуются, НЕ дублируются в windows-файле;
   никакой `subscribe`-команды (заморозка; ARCHITECTURE-идея отклонена).
8. **FRB-embed отклонён** (locked 1) — никакого in-process Rust-моста в
   этой фазе и далее без нового решения.
9. **Кириллица в lib/ вне generated** — только .arb; escape-канон
   daemon_engine.dart:25-37 для НЕ-user-visible констант.
10. **CI раньше фич-кода** (locked 4) — windows-build.yml коммитится и
    зеленеет на ПУСТОМ скелете ДО dart_ipc/платформенных веток; порядок
    задач в плане обязан это отражать (edge-probe adjacency).

## No Analog Found

| Артефакт | Почему нет аналога | Поведенческий источник |
|---|---|---|
| Платформенный выбор `Platform.isWindows` в провайдере | В `app/lib` НИ ОДНОГО `Platform.is*` (grep пуст; CONTEXT:45-46) | RESEARCH STACK.md:20 «платформа меняет только транспорт»; locked 7 |
| Контент Windows-workflow (subosito/flutter-action@v2, flutter build windows, zip) | Оба существующих workflow — Rust/ubuntu/container; ни одного Flutter/Windows шага в репо | STACK.md:54, :130; PITFALLS.md:182 (recipe, включая `flutter config --enable-windows-desktop`) |
| DACL security descriptor контракта пайпа | Ни одного security-контракта в кодовой базе; реализация — фаза 7 | PITFALLS.md:234 («доступ только текущему user SID»); фиксация док-комментарием + строкой SUMMARY (locked 3) |
| Windows-резолв путей %APPDATA%/%LOCALAPPDATA% | Паритет-эталона в Rust НЕТ (демон на Windows не портирован, фаза 7) | КОНТРАКТ фазы 7 в стиле паритет-комментариев config_path.dart:8-10; модели — %LOCALAPPDATA% (не roaming), конфиг — %APPDATA% |

Родственные, но НЕ идентичные: `resolveSocketPath` (daemon_engine.dart:16-23)
— структурный аналог для `resolvePipePath`, но env-override-ключ ДРУГОЙ
(`DICTA_PIPE`, не `DICTA_SOCK` — тесты UDS- и pipe-резолвов не должны
пересекаться); `destDirFrom` (hf_catalog.dart:32-44) — аналог формы, но
Windows-ветка НЕ зеркалирует catalog.rs (эталона нет).

## Предупреждения о расхождениях (для планёра)

1. **Ветка и триггер:** workflow обязан жить на ветке `windows`
   (locked 5) — `on: push: branches: [windows]`; `.planning/`-артефакты
   фазы коммитятся на main, код — на windows. Планёру зафиксировать
   checkout/push-шаги задач.
2. **Вилка пина Flutter SDK:** `flutter-version: "3.50.0"` в workflow (0
   правок pubspec) vs `flutter-version-file: pubspec.yaml` (+ точный
   `flutter: "3.50.0"` в environment, STACK.md:130). Локальный SDK 3.50.1
   (CONTEXT:49) — CI-пин 3.50.0/локально 3.50.1 допустимы, но планёр
   выбирает ОДИН вариант и фиксирует.
3. **dart_ipc на Linux-раннере:** пакет импортируется в providers.dart
   через windows_engine.dart — Linux-тесты идут мимо FFI, пока connect
   только в `Platform.isWindows`-ветке. Планёр обязан добавить к задаче
   верификацию: `flutter test` на Linux зелёный С ИМПОРТОМ dart_ipc
   (analyze + pub get уже ловят большинство проблем).
4. **Timeout-исключения dart_ipc:** точный тип исключений транспорта
   (аналог SocketException в ветках :84/:108/:171) — сверить с API
   dart_ipc 2.0.2 при реализации; форма трёх веток НЕ меняется, меняется
   только ловимый тип.
5. **Существующие сьюты не должны сломаться:** framing-тесты
   (daemon_engine_framing_test.dart) и config_path_test.dart не меняются —
   Windows-ветки ДОБАВЛЯЮТСЯ отдельными group/файлами; grep
   `resolveSocketPath`/`destDir(` перед правками (call-sites:
   providers.dart:17, downloads.dart:92-93, models_screen.dart:117).

## Metadata

**Analog search scope:** `.github/workflows/*`, `app/lib/core/**`
(engine/daemon_engine/providers/config_path/hf), `app/test/**`,
`app/pubspec.yaml`, `app/windows/` (tracked-скелет: CMakeLists.txt, runner/,
flutter/), `.planning/research/{STACK,PITFALLS}.md` (READ-ONLY рецепты),
`.planning/phases/05-*/PATTERNS.md` (формат).
**Grep-проверки:** `implements DictaEngine` (lib==1, test==1),
`Platform.isWindows|isLinux` по `app/lib` (пусто), `APPDATA|LOCALAPPDATA`
по `app/` (пусто), `dart_ipc` по pubspec (пусто — зависимости ещё нет);
`git ls-files` 8/8 tracked, зеркал нет.
**Pattern extraction date:** 2026-11-22.

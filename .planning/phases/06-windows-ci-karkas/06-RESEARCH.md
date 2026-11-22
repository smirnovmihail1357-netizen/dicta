# Phase 6 Research

**Phase:** 6 — Windows CI + каркас
**Requirements:** WIN-01, WIN-02, WIN-05
**Researched:** 2026-11-22
**Domain:** GitHub Actions Windows-раннер + named pipe IPC-каркас (dart_ipc) + Windows-пути
**Confidence:** HIGH (ядро — dart_ipc и pubspec-пин — верифицировано ЖИВЫМИ пробами этой сессии на Linux/Dart 3.14.0: pub get + компиляция + runtime-roundtrip + фальсификация пина; CI-факты сверены с actions/runner-images и subosito/flutter-action README 2026-11-22; gh CLI проверен живым вызовом)

<user_constraints>

## User Constraints (from CONTEXT.md)

### Locked decisions (НЕ обсуждать, НЕ пересматривать)

1. **Мост Flutter ↔ движок: named pipe IPC — РЕШЕНО.** `\\.\pipe\dicta` + тот же JSON-line протокол; Dart-сторона — `dart_ipc` 2.0.2 (socket-совместимый API, чистый Dart + win32 FFI). FRB-embed ОТКЛОНЁН.
2. **Демон на Windows в этой фазе НЕ портируется.** Windows-клиент подключается к несуществующему серверу → штатный баннер «демон недоступен». Пайп-сервер (`interprocess`, WASAPI, SendInput) — фаза 7.
3. **DACL пайпа — контракт фазы 7, зафиксировать документально** в решении/README фазы 6: сервер создаёт пайп с security descriptor «только текущий user SID». В фазе 6 верифицируется ЗАПИСЬ контракта, не реализация.
4. **CI — первый deliverable, на ПУСТОМ скелете:** `windows-build.yml` коммитится и зеленеет ДО любого Windows-фич-кода.
5. **Отдельная ветка `windows`** от запушенного main (origin/main = d07763a). `.planning/`-артефакты — на main, кодовые коммиты фазы — на ветке `windows`.
6. **Единственная новая зависимость: `dart_ipc` 2.0.2** (пин точно). `interprocess` — фаза 7, НЕ добавлять.
7. Протокол/крейты/Rust не трогаем; Linux UDS-путь (`daemon_engine.dart`) не трогаем — Windows-транспорт = НОВАЯ реализация рядом, выбор по `Platform.isWindows` в провайдере.
8. Кириллица — только через .arb; локальные проверки на Linux (flutter analyze + flutter test + cyrillic gate) обязаны остаться зелёными.

### Must-NOT (из CONTEXT/ограничений)

- НЕ трогать `.github/workflows/ci.yml` и `release.yml` — windows-build.yml отдельный файл.
- НЕ добавлять пакетов, кроме dart_ipc 2.0.2.
- НЕ хардкодить юниксовые пути в Windows-ветках кода (`/run/user`, `~/.config`).
- НЕ ждать живой CI-прогон внутри задачи — статус фиксировать в SUMMARY/VERIFICATION через gh после.
- 17-ключевой set/формы/браузер — не трогать (фаза 4).

</user_constraints>

<phase_requirements>

## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| WIN-01 | `windows-build.yml` собирает zip-артефакт на windows-раннере и зелёный на ПУСТОМ скелете — первый deliverable | Готовый YAML-рецепт ниже (Architecture Patterns → Pattern 1): windows-latest = Server 2025 + VS 2026 Enterprise с полным C++-ворклоадом [VERIFIED: actions/runner-images]; flutter-action@v2 синтаксис сверен с README [VERIFIED]; артефакт `build/windows/x64/runner/Release/*` [VERIFIED: docs.flutter.dev/release/breaking-changes/windows-build-architecture]. Пустой скелет собирается без изменений кода — трассировка рантайма в Pattern 3 |
| WIN-02 | Windows-сборка показывает ту же оболочку, транспорт — named pipe `\\.\pipe\dicta` (развилка locked) | dart_ipc 2.0.2 верифицирован живой пробой на Linux: `connect()` возвращает `Socket`, паттерн `_call` из daemon_engine.dart работает без изменений (проба: roundtrip 0.4 мс/вызов); Windows-движок = НОВАЯ реализация через dart_ipc за швом DictaEngine, выбор в engineProvider по `Platform.isWindows` |
| WIN-05 | Пути %APPDATA%/%LOCALAPPDATA% без юниксовых хардкодов; DACL-контракт записан | Таблица dirs-соответствий [VERIFIED: dirs-rs README]: config_dir→Roaming, data_local_dir→LocalAppData, **data_dir→тоже Roaming** (критическая нюанс-находка: текущий dicta-core default_model_path через `dirs::data_dir()` на Windows попал бы в Roaming — см. Pitfall 6); контракт DACL = документ, верификация ревью-гейтом |

</phase_requirements>

## Summary

Главный вопрос фазы — «не сгниёт ли Windows-ветка» (Pitfall 8 из PITFALLS.md) — закрывается CI-first, и исследование подтвердило, что рецепт из STACK.md:54 собирается почти дословно, с двумя уточнениями, найденными фальсификацией. Первое: `dart_ipc` 2.0.2 **компилируется и работает на Linux живьём** (probe этой сессии: pub get резолвит win32 6.4.0, полный daemon_engine-паттерн write→flush→`cast<List<int>>()`→LineSplitter→first→destroy прошёл UDS-roundtrip за 0.4 мс/вызов; connect к несуществующему пути бросает `SocketException` за 1 мс) — значит Windows-транспорт пишется и **тестируется на Linux-фейках без единой Windows-машины**, что и требует CONTEXT. Публичный API пакета — ровно `bind(path)`/`connect(path)`, оба возвращают `dart:io` `ServerSocket`/`Socket`; Windows-реализация `Win32NamedPipeSocket implements Socket` — весь стрим/sink-контракт совпадает с тем, что уже пишет `daemon_engine.dart:50-76`.

Второе уточнение — критическое для локальной разработки: опция `flutter-version-file: pubspec.yaml` из STACK.md:54 **отвергнута живой фальсификацией**. flutter-action требует ТОЧНЫЙ пин (`flutter: 3.50.0`), а такой пин ломает локальный `flutter pub get` на машине с Flutter 3.50.1 (воспроизведено: «Try using the Flutter SDK version: 3.50.0», pub get Failed). Рекомендация: в workflow явный `flutter-version: 3.50.1` (паритет с локальным SDK), в pubspec — либо ничего (как сейчас), либо диапазон `flutter: ">=3.50.0 <3.51.0"` (проверено: локальный pub get на 3.50.1 проходит). CI-среда: windows-latest сегодня = Windows Server 2025 + Visual Studio Enterprise 2026 18.12 с полным C++-ворклоадом (VC.Tools.x86.x64, CMake 4.5, Ninja, Win10 SDK) — ничего доустанавливать не нужно; поддержка VS 2026 во flutter_tools влита 2025-11-01 (PR #177458, issue #176399 closed as fixed), т.е. задолго до 3.50.

Рантайм пустого скелета на Windows прослежен по коду: `resolveSocketPath()` (daemon_engine.dart:16-23) на раннере не находит `DICTA_SOCK`/`XDG_RUNTIME_DIR` и падает на `Process.runSync('id', ['-u'])` (у `id` нет exe на Windows → ProcessException), исключение глотается `catch (_)` поллера (providers.dart:90) → `DictaStatus.disconnected` → штатный баннер «демон недоступен» — ровно поведение фазы 1 на Linux без демона, как требует locked decision 2. Компиляция скелета кросс-платформенна: `InternetAddressType.unix` объявлен безусловно в общем dart:io (локальный SDK-сурс: `static const InternetAddressType unix = InternetAddressType._(2);`).

**Primary recommendation:** ветка `windows` от d07763a → первым коммитом `windows-build.yml` (flutter-action@v2, `flutter-version: 3.50.1`, cache:true, build → Compress-Archive → upload-artifact@v4) → зелёный прогон на пустом скелете → только потом pubspec+dart_ipc 2.0.2 и WindowsPipeEngine; пути — injectable-функции с env-параметрами по прецеденту config_path.dart; ошибки транспорта ловить `on IOException` (не `on SocketException` — на Windows это Win32NamedPipeException), connect оборачивать `.timeout()`.

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Windows CI (сборка, zip, артефакт) | .github/workflows/windows-build.yml (единственный файл) | — | CI = первый deliverable; ci.yml/release.yml не трогаем (CONTEXT:42-44) |
| Windows-транспорт IPC | app/lib/core/windows_engine.dart (НОВАЯ реализация DictaEngine) | engineProvider (providers.dart:16-17) — fork по `Platform.isWindows` | locked decision 7: Linux UDS-путь не трогаем, новая реализация рядом |
| JSON-line протокол | переиспользуется как есть (engine.dart:99-105 buildStatusLine и др.) | — | протокол не меняется во всей вехе |
| Пути конфига/моделей Windows | injectable-функции (env-параметры) рядом с config_path.dart/hf_catalog.dart | — | прецедент injectable-параметров из-за immutable Platform.environment в тестах (config_path.dart:12-14) |
| DACL-контракт | документ (README фазы / windows-engine docstring) | ревью-гейт фазы 7 | locked decision 3: верифицируется ЗАПИСЬ |
| Ветка/история | git branch windows от origin/main d07763a | .planning/ — на main | locked decision 5 |
| Верификация CI из CLI | gh run list/watch (gh 2.104.0 authenticated, живой вызов этой сессией) | SUMMARY/VERIFICATION фиксация | CONTEXT:67-69 |

## Standard Stack

### Core

| Технология | Версия | Назначение | Почему |
|-----------|--------|-----------|--------|
| `subosito/flutter-action@v2` | @v2 (README main, чтение 2026-11-22) | Flutter SDK на windows-раннере | Официально рекомендуемый путь; `flutter-version: 3.50.1` ИЛИ `flutter-version-file` (требует точный пин — см. Pitfall 1); `cache: true` + `pub-cache: true`, cache-key `flutter-:os:-:channel:-:version:-:arch:-:hash:` [VERIFIED: github.com/subosito/flutter-action README] |
| `runs-on: windows-latest` | = Windows Server 2025, image 20261116.315.1 | Сборка | VS 2026 Enterprise 18.12 + полный C++-ворклоад (VC.Tools.x86.x64, CMake 4.5.0, Ninja 1.13.2, Windows 10 SDK), Rust 1.100, gh CLI 2.104 предустановлены [VERIFIED: actions/runner-images README] |
| `dart_ipc` | **2.0.2** (пин точно) | Windows-транспорт `\\.\pipe\dicta` | pub.dev verified publisher monkeywie.cn, MIT, 17.4k загрузок/нед, 150 points; `connect()` → `Socket`, Windows = Win32NamedPipeSocket implements Socket; Linux-компиляция и работа верифицированы живой пробой этой сессии |
| `actions/upload-artifact@v4` | v4 | zip-артефакт | v3 отключён GitHub-ом с 2025-01; v4-синтаксис `with: name/path/retention-days` [CITED: github.com/actions/upload-artifact] |
| `actions/checkout@v4` | v4 | чекаут | как в существующем ci.yml:15 (единый стиль репо; в README flutter-action примерах v6 — не критично) |
| PowerShell `Compress-Archive` | встроен в windows-runner (pwsh 7.6.10 на образе) | zip каталога Release | нулевая зависимость; `shell: pwsh` — дефолт на windows-раннере [CITED: learn.microsoft.com/powershell] |

### Supporting

| Технология | Версия | Назначение | Когда |
|-----------|--------|-----------|-------|
| `flutter: ">=3.50.0 <3.51.0"` в pubspec environment | диапазон | защита от случайного SDK-дрейфа | опционально; локальный pub get на 3.50.1 проверен-зелёный. НЕ точный пин (Pitfall 1) |
| dart_ipc bind() в тестах | 2.0.2 | фейк-сервер UDS на Linux для юнит-тестов WindowsPipeEngine | проба доказала: bind/connect идентичны на обеих платформах, тесты транспорта гоняются на Linux |

### Alternatives Considered

| Вместо | Можно было | Отвергнуто, потому что |
|--------|-----------|------------------------|
| dart_ipc (named pipe) | AF_UNIX на Windows (Dart ≥3.11) | драйвер afunix есть не на всех Win10, не поддерживает datagrams/abstract — STACK.md:84; named pipe = FlClash-проверенный путь (locked decision 1) |
| dart_ipc (named pipe) | прямой win32 FFI (CreateFile/ReadFile/WriteFile) | dart_ipc 2.0.2 УЖЕ это делает (overlapped I/O через `package:win32`, без C++-плагин-слоя); самописный FFI = сотни строк кода синхронизации OVERLAPPED/ивентов, которые dart_ipc отладил в 2.0.x (CHANGELOG 2.0.0-2.0.2) |
| flutter_rust_bridge embed | — | отклонён вехой (locked decision 1; PITFALLS P5) |
| flutter-version-file | явный `flutter-version:` | фальсифицировано живьём: точный пин ломает локальный pub get на 3.50.1 |
| MSIX (package msix) | zip | STACK.md:55: для каркаса избыточен |

## Package Legitimacy Audit

> Seam `package-legitimacy check` поддерживает только npm/pypi/crates — pub вне seam. Выполнена ручная верификация по всем сигналам seam, включая ЖИВОЙ pub get + компиляцию + runtime-пробу.

| Package | Registry | Возраст | Загрузки | Source Repo | Verdict | Disposition |
|---------|----------|---------|----------|-------------|---------|-------------|
| dart_ipc 2.0.2 | pub.dev | первый релиз 2025, 2.0.2 от 2026-09-06 | 17.4k/нед | github.com/monkeyWie/dart_ipc (MIT) | OK (ручной аудит: verified publisher `monkeywie.cn`, 150 pub points, 10 likes, живой pub get + живая runtime-проба) | Approved |

**Packages removed due to SLOP:** none.
**Packages flagged as suspicious [SUS]:** none — dart_ipc проверен глубже seam-аудита (не только реестр, но и исполнение).

## Architecture Patterns

### System Architecture Diagram (фаза 6)

```
[git] main (d07763a, Linux-workstream, CI ci.yml — НЕ трогаем)
        │ branch 'windows'
        ▼
[commit 1] .github/workflows/windows-build.yml ──► GitHub Actions windows-latest
        │                                            (Server 2025 + VS2026, flutter 3.50.1)
        │ push ветки → on: push [windows] + workflow_dispatch
        ▼                                            │
 ЗЕЛЁНЫЙ ПРОГОН НА ПУСТОМ СКЕЛЕТЕ                    ▼
 (zip: dicta_app.exe + dll + data) ───────────► upload-artifact@v4 (WIN-01 ✓)
        │
        ▼ (только после зелёного)
[фич-код] pubspec: dart_ipc 2.0.2 → WindowsPipeEngine (lib/core/)
        │                                            │
        │  engineProvider (providers.dart:16) ── Platform.isWindows fork
        │        ├─ Linux  → DaemonEngine (UDS, НЕ тронут)
        │        └─ Win   → WindowsPipeEngine → dart_ipc.connect('\\\\.\\pipe\\dicta')
        │                          │ нет сервера (фаза 7) → Win32NamedPipeException
        │                          ▼ (через ~5 с ретраев, см. Pitfall 2 → .timeout)
        │                   catch on IOException → DictaStatus.disconnected → баннер (WIN-02 ✓)
        ▼
[пути] resolveConfigPath/destDir Windows-ветки: %APPDATA% / %LOCALAPPDATA%
        + README-контракт DACL «только текущий user SID» (WIN-05 ✓, запись = ревью-гейт)
```

### Pattern 1: windows-build.yml (полный рецепт, WIN-01)

```yaml
# .github/workflows/windows-build.yml — ОТДЕЛЬНЫЙ файл, ci.yml не трогать
name: Windows build

on:
  push:
    branches: [windows]
  workflow_dispatch:

jobs:
  build:
    name: flutter build windows (release zip)
    runs-on: windows-latest
    defaults:
      run:
        working-directory: app
    steps:
      - uses: actions/checkout@v4

      - uses: subosito/flutter-action@v2
        with:
          channel: stable
          flutter-version: 3.50.1   # НЕ flutter-version-file: точный пин в pubspec
          cache: true               # кеш SDK + pub-cache по :os:-:channel:-:version:-:hash:
          pub-cache: true

      - run: flutter pub get
      - run: flutter build windows --release

      - name: Package release zip
        shell: pwsh    # дефолт на windows-раннере; Compress-Archive встроен
        run: Compress-Archive -Path build/windows/x64/runner/Release/* `
          -DestinationPath dicta-app-windows-x64.zip

      - uses: actions/upload-artifact@v4
        with:
          name: dicta-app-windows-x64
          path: app/dicta-app-windows-x64.zip
          retention-days: 14
```

[VERIFIED по элементам: триггеры/labels — actions/runner-images README; flutter-action@v2 инпуты и cache-key — flutter-action README (fetched 2026-11-22); путь артефакта `build/windows/x64/runner/Release` — docs.flutter.dev/release/breaking-changes/windows-build-architecture «Example build path after the migration if targeting x64: build\windows\x64\runner\Release\hello_world.exe»; рабочий каталог — pubspec живёт в app/]

Триггеры не конфликтуют с main-workflow: ci.yml срабатывает на `push: branches: [main]` + `pull_request` (ci.yml:3-6); push в `windows` его не будит; PR windows→main разбудит Linux-CI — это желаемое поведение к merge-времени. `workflow_dispatch` даёт idempotency-проверку (повторный прогон без push — валидационный кейс WIN-01).

### Pattern 2: WindowsPipeEngine — шов и фейк-стратегия

**What:** третья реализация `DictaEngine` (после DaemonEngine и FakeDictaEngine) поверх dart_ipc; выбор — в провайдере:

```dart
// providers.dart (fork по платформе — locked decision 7)
final engineProvider = Provider<DictaEngine>((ref) => Platform.isWindows
    ? WindowsPipeEngine()          // dart_ipc.connect('\\\\.\\pipe\\dicta')
    : DaemonEngine(resolveSocketPath()));
```

```dart
// lib/core/windows_engine.dart — транспортный скелет (фаза 6)
import 'dart:async';
import 'dart:convert';
import 'dart:io';
import 'package:dart_ipc/dart_ipc.dart';
import 'engine.dart';

const kDictaPipe = '\\\\.\\pipe\\dicta'; // единственное место с хардкодом трубы (валидационный кейс WIN-05)

class WindowsPipeEngine implements DictaEngine {
  // dart_ipc.connect не имеет параметра timeout (публичный API — только path):
  // на Windows коннект к отсутствующей трубе ретраится ~5 c (ERROR_FILE_NOT_FOUND),
  // поэтому таймаут — обёрткой.
  Future<Socket> _connect() => connect(kDictaPipe)
      .timeout(const Duration(seconds: 2));

  Future<String> _call(String line, {required Duration timeout}) async {
    final socket = await _connect(); // может бросить Win32NamedPipeException
    try {
      socket.write(line);
      await socket.flush();
      return await socket
          .cast<List<int>>()
          .transform(utf8.decoder)
          .transform(const LineSplitter())
          .first
          .timeout(timeout);
    } finally {
      socket.destroy();
    }
  }
  // ... команды status/start/stop/... — переиспользуют buildStatusLine и др.
  // Ошибки ловить on IOException (охватывает и SocketException на Unix,
  // и Win32NamedPipeException на Windows) + on TimeoutException. НЕ on SocketException —
  // на Windows он не поймает НИЧЕГО (см. Pitfall 3).
}
```

**Почему паттерн верен (всё — по чтению живого кода dart_ipc 2.0.2 из pub-cache):**
- Публичный API пакета — ровно две функции: `Future<ServerSocket> bind(String path)` и `Future<Socket> connect(String path)` (lib/src/dart_ipc.dart); path на Windows — формат `\\.\pipe\dart_ipc`, на Unix — путь файла (docstring той же функции).
- Windows-сокет — `class Win32NamedPipeSocket implements Socket` (lib/src/named_pipe_socket.dart): write/flush/destroy/cast/transform — весь контракт dart:io Socket, паттерн `_call` из daemon_engine.dart:50-76 идентичен.
- Коннект-ретраи: `const _pipeTimeoutMs = 5000;` + retryable `ERROR_PIPE_BUSY`/`ERROR_FILE_NOT_FOUND` (lib/src/dart_ipc_windows.dart) — до 5 с на отсутствующий сервер.
- Ошибка Windows — `class Win32NamedPipeException implements IOException` (не SocketException!) и в 2.0.2 НЕ экспортируется из пакета (dart_ipc_io.dart экспортирует только dart_ipc.dart + dart_ipc_unix.dart) — ловить можно только по супертипу IOException.
- Живая проба на Linux (Dart 3.14.0, эта сессия): `bind`→`_ServerSocket`, `connect`→`_Socket`, roundtrip `{"cmd":"status"}` → 0.4 мс/вызов на свежем соединении; connect к несуществующему пути → `SocketException` за 1 мс (`is IOException: true`).

**Фейк-стратегия тестов (без единой Windows-машины):** dart_ipc-транспорт платформенно-агностичен — юнит-тесты WindowsPipeEngine на Linux поднимают фейк-сервер через `bind('${Directory.systemTemp.path}/dicta_test_$pid.sock')` и инжектят путь в конструктор (прецедент `DaemonEngine([String? socketPath])`, daemon_engine.dart:40-41). Существующие 29 тест-файлов не биндят реальных сокетов (grep этой сессии: ServerSocket.bind/Socket.connect в app/test/ отсутствуют) — Flutter-сьют остаётся зелёным.

### Pattern 3: пустой скелет на Windows-раннере — трассировка рантайма (locked decision 2)

Текущее приложение (фазы 1-5) собирается на windows-раннере **без изменений**:
- **Компиляция:** `InternetAddressType.unix` объявлен безусловно в общем dart:io — локальный SDK-сурс `~/development/flutter/bin/cache/dart-sdk/lib/io/socket.dart:15`: `static const InternetAddressType unix = InternetAddressType._(2);` [VERIFIED: чтение исходника SDK этой сессией]. `Socket.connect(..., timeout:)` и `Process.runSync` — кросс-платформенные API dart:io.
- **Рантайм:** `resolveSocketPath()` (daemon_engine.dart:16-23): на раннере `DICTA_SOCK` и `XDG_RUNTIME_DIR` не заданы → `Process.runSync('id', ['-u'])` → `id.exe` на Windows не существует → ProcessException (документированное поведение dart:io: программа не найдена → исключение) [CITED: api.dart.dev dart:io Process]. Исключение бросается в теле `engineProvider` при первом `ref.read` → ловится `catch (_)` в `DictaStatusNotifier.poll()` (providers.dart:82-101) → `state = DictaStatus.disconnected()` → баннер «демон недоступен». Это и есть «штатный баннер» до подключения dart_ipc — идентично поведению фазы 1 на Linux с убитым демоном.
- **Побочный чтение конфига:** HK-карточка через `resolveConfigPath()` (config_path.dart:14-35) на Windows получит фолбэк `'dicta-config.toml'` (относительный путь, HOME нет) → файл не найден → карточка честно показывает дефолт «off». Не фатально, только чтение.
- ЕДИНСТВЕННОЕ, что требует живого прогона — сама сборка (MSVC/CMake); рантайм-баннер верифицируется пользователем на своей Windows-машине в UAT фазы (PITFALLS:250 чек-лист «артефакт ни разу не запускали»).

### Pattern 4: Windows-пути (WIN-05) — injectable-функции

Прецедент — config_path.dart:12-18 («Parameters are injectable because Platform.environment is immutable in widget tests»). Windows-ветки:

```dart
// рядом с config_path.dart (НЕ внутри него — Linux-логику не трогаем)
/// Windows: %APPDATA%\dicta\config.toml — паритет dirs::config_dir()
/// (RoamingAppData, dirs-rs README table). Linux-фолбэк не используется.
String resolveConfigPathWindows({String? appData}) {
  final roaming = appData ?? Platform.environment['APPDATA'];
  if (roaming == null || roaming.isEmpty) return r'dicta\config.toml'; // честный фолбэк
  return '$roaming\\dicta\\config.toml';
}

/// Модели: %LOCALAPPDATA%\whisper-cpp — контракт WIN-05 (см. Pitfall 6 про data_dir).
String modelsDirWindows({String? localAppData}) {
  final local = localAppData ?? Platform.environment['LOCALAPPDATA'];
  return local == null || local.isEmpty
      ? 'whisper-cpp'
      : '$local\\whisper-cpp';
}
```

Соответствие dirs crate [VERIFIED: dirs-rs README platform table]: `config_dir` Win = `{FOLDERID_RoamingAppData}`; `data_local_dir` Win = `{FOLDERID_LocalAppData}`; `runtime_dir` Win = `None` (UDS-пути на Windows не резолвятся вовсе — и не нужны, транспорт = труба).

### Pattern 5: DACL-контракт (locked decision 3) — документ, не код

В README фазы/windows-модуле фиксируется (без реализации):
- сервер фазы 7 (`interprocess`/named pipe) создаёт трубу с security descriptor «доступ только текущему user SID» (PITFALLS.md:234);
- dart_ipc 2.0.2-КЛИЕНТ к DACL не имеет отношения (CreateFile-коннект проходит по умолчанию);
- НЮАНС для фазы 7, найденный в сурсе dart_ipc: его bind() на Windows передаёт `null` security attributes в CreateNamedPipe (т.е. сервер dart_ipc = default DACL) — но пайп-сервер у dicta будет в Rust (interprocess), так что контракт применим именно к Rust-серверу;
- верификация фазы 6 = наличие записи (ревью-гейт), реализация+тест — фаза 7.

### Anti-Patterns to Avoid

- **«Быстро добавим dart_ipc в первый же коммит с workflow»:** нарушение locked decision 4 — CI обязан позеленеть на ПУСТОМ скелете; adjacency-валидация WIN-01 проверяет порядок коммитов (workflow ДО dart_ipc в истории ветки).
- **Ловить `on SocketException` в WindowsPipeEngine:** на Windows это не ловится вообще (Win32NamedPipeException — IOException, не SocketException); ловить `on IOException` + `on TimeoutException`.
- **Хардкод `C:\Users\...` или `/run/user` в Windows-ветках:** валидационный кейс WIN-05 (grep по `\\.\pipe\dicta` вне константы + отсутствие юниксовых путей в Windows-ветках).
- **zip только .exe:** релиз-каталог содержит dll + data/ — без них запуск падает; zip весь `Release/*` [VERIFIED: docs.flutter.dev «the executable is generated in the build\windows\runner\Release directory» + breaking-change про arch-сегмент].

## Don't Hand-Roll

| Проблема | Не писать руками | Использовать | Почему |
|----------|------------------|--------------|--------|
| Named pipe FFI (CreateFile/ReadFile/WriteFile + OVERLAPPED + события + ретраи) | свой win32-FFI-слой | `dart_ipc` 2.0.2 | сотни строк кода синхронизации overlapped-I/O; баги этого слоя чинились релизами 2.0.0→2.0.2 (handle ownership, idempotent shutdown — CHANGELOG) |
| Транспортный протокол | новый wire-format | существующий JSON-line (engine.dart) | протокол менять запрещено вехой |
| Установка VS/тулчейна на раннере | шаги choco/cmake | ничего — VS 2026 уже на образе | [VERIFIED: runner-images] |
| Zip | кастомный PowerShell-скрипт по файлам | `Compress-Archive -Path Release/*` | каталог самодостаточен |
| Кеш Flutter SDK | actions/cache вручную | `cache: true` в flutter-action | ключи `:os:-:channel:-:version:-:arch:-:hash:` из коробки |
| Фейк-демон для тестов Windows-транспорта | мок Socket вручную | `dart_ipc.bind()` UDS-сервер на Linux | probe: идентичный API на обеих платформах |

**Key insight:** фаза 6 — почти ноль новой прикладной логики; её ядро — правильные обёртки (workflow, engine-развилка, пути) над уже проверенными кусками (протокол, шов DictaEngine, dart_ipc).

## Runtime State Inventory

> Фаза создаёт ветку и CI-регистрацию — не рефакторинг существующих данных, но инвентаризация обязательных «живых» состояний:

| Category | Items Found | Action Required |
|----------|-------------|------------------|
| Stored data | None — фаза не мигрирует данные; config.toml/модели читаются, не пишутся (app — тонкий клиент) | нет |
| Live service config | GitHub Actions: workflow `windows-build.yml` регистрируется на push ветки (server-side GitHub); существующие ci.yml/release.yml НЕ трогаются | код-коммит ветки `windows` + push |
| OS-registered state | git-ветка `windows` (локально + origin) — ЕЩЁ НЕ СУЩЕСТВУЕТ (git branch -a этой сессии: только main, container) | `git branch windows d07763a && git push -u origin windows` |
| Secrets/env vars | gh CLI авторизован (scopes: repo, workflow — push workflow-файла разрешён; проверено живым `gh auth status` этой сессией) | нет |
| Build artifacts | `app/build/` локально — gitignored; артефакт CI — на GitHub Actions (retention-days) | нет в git |

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| gh CLI | верификация CI из CLI | ✓ (живой вызов) | 2.104.0, authenticated (repo+workflow scopes) | — |
| Flutter SDK (локальный) | локальные проверки | ✓ | 3.50.1 stable (живой `flutter --version`) | — |
| Dart SDK (локальный) | фейк-тесты dart_ipc | ✓ | 3.14.0 standalone (живой `dart --version`) | flutter-bundled |
| Windows-раннер | сборка артефакта | ✓ (GitHub-hosted, windows-latest = Server 2025 + VS 2026, image 20261116.315.1) | — | pin `windows-2022` при сбоях тулчейна (Pitfall 4) |
| Локальная Windows-машина | UAT баннера/оболочки | ✗ (в этой среде нет; машина пользователя) | — | рантайм-трассировка Pattern 3 + UAT на машине пользователя |
| dart_ipc pub-cache | тесты | ✓ | 2.0.2 загружен живым pub get | — |

**Missing dependencies with no fallback:** локальная Windows-сборка — отсутствует by design (CI-only, Pitfall 8); рантайм-верификация оболочки на Windows откладывается в UAT пользователя, что уже отражено в валидационных кейсах CONTEXT:67-69 и PITFALLS:250.

## Common Pitfalls

### Pitfall 1: flutter-version-file / точный пин flutter в pubspec ломает локальную разработку
**What goes wrong:** `flutter-version-file: pubspec.yaml` требует ТОЧНОГО `flutter: 3.50.0` (README: диапазоны — «Bad»); такой пин в pubspec делает локальный `flutter pub get` на Flutter 3.50.1 невыполнимым.
**Фальсификация (живая, эта сессия):** pubspec с `flutter: 3.50.0` + локальный 3.50.1 → «Failed to update packages… Try using the Flutter SDK version: 3.50.0». Диапазон `flutter: ">=3.50.0 <3.51.0"` → pub get проходит.
**How to avoid:** workflow: явный `flutter-version: 3.50.1`; pubspec: либо не трогать (текущее состояние), либо диапазон.
**Warning signs:** «flutter pub get» падает у разработчика сразу после CI-коммита.

### Pitfall 2: 5-секундный коннект-ретрай на отсутствующую трубу
**What goes wrong:** dart_ipc Windows `connect()` ретраит `ERROR_FILE_NOT_FOUND` до `_pipeTimeoutMs = 5000` (исходник 2.0.2) — каждый полл статуса против несуществующего сервера фазы 7 висит ~5 с; поллер с 100 мс-тик + inFlight-гейт превращается в 5-секундные такты с бэкоффом.
**How to avoid:** `.timeout(2 s)`-обёртка вокруг connect (Pattern 2) → быстрый TimeoutException → баннер как на Linux.
**Warning signs:** в Windows-сборке баннер «недоступен» появляется с 5-секундными паузами между опросами.

### Pitfall 3: on SocketException не ловит ошибки Windows-транспорта
**What goes wrong:** `Win32NamedPipeException implements IOException`, НЕ SocketException (исходник), и в 2.0.2 даже не экспортируется из пакета — скопированный из daemon_engine.dart:58 `on SocketException catch` в Windows-коде молча пропустит все ошибки трубы вверх.
**How to avoid:** `on IOException` (покрывает SocketException на Unix и Win32NamedPipeException на Windows) + отдельный `on TimeoutException`; сообщение — по образцу daemon_engine (тип+путь, без payload).
**Warning signs:** красный стек Win32NamedPipeException в консоли Windows-сборки.

### Pitfall 4: миграция label windows-latest и углы VS2026
**What goes wrong:** `windows-latest` уже = Server 2025/VS 2026 (label мигрирует silently); VS 2026 поддержан flutter_tools с PR #177458 (merged 2025-11-01) — но угол flutter/flutter#180481 (выбор тулчейна при наличии нескольких VS) открыт.
**How to avoid:** строить на windows-latest как в рецепте (flutter-action README сам использует windows-latest); при необъяснимом «Unable to find suitable Visual Studio toolchain» — пин `windows-2022` (Server 2022 + VS 2022) одной строкой.
**Warning signs:** сборка падает на CMake/«could not find any instance of Visual Studio».

### Pitfall 5: недо-зип артефакта
**What goes wrong:** zip только dicta_app.exe без flutter_windows.dll и data/ → артефакт незапускаем.
**How to avoid:** `Compress-Archive -Path build/windows/x64/runner/Release/*` (весь каталог); валидационный кейс WIN-01 «zip непустой, содержит exe» — расширить до «exe+dll+data».
**Warning signs:** артефакт 1-2 МБ вместо ~30-80 МБ.

### Pitfall 6: dirs::data_dir() на Windows = Roaming, а не %LOCALAPPDATA%
**What goes wrong:** наивное «модели кладём как dirs::data_dir()» кладёт их в Roaming-профиль; dicta-core живой код `default_model_path()` использует именно `dirs::data_dir()` (config.rs:246-251), который по таблице dirs-rs на Windows = `{FOLDERID_RoamingAppData}`.
**How to avoid:** контракт WIN-05 (= формулировка требования: «%APPDATA%/%LOCALAPPDATA%»): конфиг → %APPDATA%\dicta (config_dir, паритет), модели → %LOCALAPPDATA%\whisper-cpp (data_local_dir). ЗАПИСАТЬ в контракте фазы 6, что фаза 7 при cfg(windows)-порте обязана переключить модельный путь с `data_dir()` на `data_local_dir()` — это осознанное отклонение от Linux-паритета (большие бинарные данные не в роуминге, PITFALLS:218), а не случайность.
**Warning signs:** Windows-порт фазы 7 «работает», но модели живут в Roaming.

### Pitfall 7: «улучшить» resolveSocketPath под Windows
**What goes wrong:** соблазн сделать resolveSocketPath кросс-платформенным в фазе 6 — а это трогает Linux UDS-путь (запрещено locked decision 7) и маскирует штатный баннер-путь пустого скелета.
**How to avoid:** resolveSocketPath не трогать ВООБЩЕ; Windows-развилка только в провайдере (новая реализация рядом).
**Warning signs:** диффы по daemon_engine.dart в ветке windows.

### Pitfall 8: кеш-пессимизм и «поставим всё каждый раз»
**What goes wrong:** 20-минутный холодный билд без кеша (PITFALLS:216); либо самодельный actions/cache по pubspec.lock.
**How to avoid:** `cache: true` + `pub-cache: true` во flutter-action (ключи включают :version: и :hash: — минорные версии не смешиваются).
**Warning signs:** каждый прогон windows-build.yml > 25 мин.

## Code Examples

### Windows-фейк для юнит-теста WindowsPipeEngine (Linux, гоняется живьём)

```dart
// test/windows_engine_test.dart — фейк-сервер через dart_ipc на UDS (Linux CI)
// Проба этой сессии доказала: API идентичен, ~0.4 мс на fresh-connection roundtrip.
final server = await bind('${Directory.systemTemp.path}/dicta_test.sock');
final sub = server.listen((client) {
  client.cast<List<int>>().transform(utf8.decoder)
      .transform(const LineSplitter()).listen((line) {
    final req = jsonDecode(line) as Map<String, dynamic>;
    if (req['cmd'] == 'status') {
      client.write('${jsonEncode({
        'state': 'idle', 'level': 0.0, 'seconds': 0.0})}\n');
    }
  });
});
// ... WindowsPipeEngine(pipePathOverride: тот же путь) → expect status idle
await sub.cancel();
await server.close();
```

[VERIFIED: этот паттерн исполнен живьём этой сессией (probe roundtrip + 20 последовательных fresh-connection вызовов: 8 мс суммарно)]

### Верификация зелёного CI из CLI (доступно в этой среде)

```bash
gh run list --repo smirnovmihail1357-netizen/dicta --workflow windows-build.yml --branch windows --limit 5
gh run watch <run-id> --repo smirnovmihail1357-netizen/dicta --exit-status   # exit 0 = зелёный
gh run download <run-id> --repo smirnovmihail1357-netizen/dicta -n dicta-app-windows-x64 -D /tmp/opencode/win-zip
unzip -l /tmp/opencode/win-zip/*.zip   # exe + dll + data (Pitfall 5)
```

[VERIFIED: gh 2.104.0 установлен, аутентифицирован (repo+workflow scopes), `gh run list` живой вызов вернул реальные прогоны CI этой сессией; watch/download — CLI-контракт gh 2.104]

## State of the Art

| Старый подход | Текущий | Когда изменилось | Влияние |
|---------------|---------|------------------|---------|
| windows-latest = Server 2022 + VS 2022 | Server 2025 + VS 2026 18.12 | label-миграция 2026 | flutter 3.50 собирает против VS 2026 (PR #177458 влита 2025-11); фолбэк — pin windows-2022 |
| upload-artifact v3 | v4 | v3 отключён 2025-01 | только `with: name/path` v4-синтаксис |
| Путь сборки `build/windows/runner/Release` | `build/windows/**x64**/runner/Release` | Flutter 3.13 (breaking change arch-сегмент) | zip-шаг в workflow обязан включать x64 |
| dart_ipc = Flutter-метод-канал-плагин | чистый Dart + win32 FFI overlapped I/O | 2.0.0 (2026) | нет C++-плагин-слоя, компилируется на Linux (проверено) |
| `flutter-version-file` с диапазоном | только точный пин | README текущий | отвергнуто для dicta: ломает локальный 3.50.1 |

**Deprecated/outdated:** flutter-version-file для нашего кейса (см. Pitfall 1); AF_UNIX-on-Windows как транспорт (driver-зависимость); system_tray и прочее — вне фазы.

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | Учётка windows-раннера — runneradmin, `APPDATA`=`C:\Users\runneradmin\AppData\Roaming`, `LOCALAPPDATA`=`...\Local` | Pattern 4, Q4 | низкий: значения стандартных профильных env-переменных; но сама фаза не зависит от конкретного имени — код читает только переменные |
| A2 | `Process.runSync('id', ['-u'])` на Windows бросает ProcessException (exe не найден) — документированное поведение dart:io, воспроизведено аналитически, НЕ исполнено на Windows | Pattern 3 | средний: если бы не бросал, resolveSocketPath вернул бы мусорный путь → SocketException на connect → баннер тем же путём (через catch(_)); итоговый UX идентичен |
| A3 | Холодный Flutter-Windows билд на windows-latest ~15-25 мин, с cache ~10-15 | Environment | низкий: влияет только на ожидание прогона, не на результат |
| A4 | Flutter 3.50.1 + VS 2026 (image 20261116) собираются без ручных шагов — поддержка влита 2025-11-01 (#177458), но угол #180481 открыт | Standard Stack | средний: первый прогон может упасть на тулчейне → готовый фолбэк pin `windows-2022` (Pitfall 4) |
| A5 | zip-артефакт 30-80 МБ (типовой размер Flutter-Windows release-каталога) | Pitfall 5 | низкий: только порог валидации «zip непустой» |

Все прочие факты — VERIFIED (живые пробы этой сессии: dart_ipc, pubspec-пин, gh) или CITED (официальные README/docs: flutter-action, runner-images, dirs-rs, dart:io, docs.flutter.dev).

## Open Questions

Все вопросы ТЗ на фазу закрыты этой сессией:

1. **dart_ipc на Linux — РЕШЕНО (VERIFIED живой пробой):** компилируется и работает (pub get → compile → UDS roundtrip 0.4 мс/вызов; connect к несуществующему пути → SocketException за 1 мс). КРИТИЧЕСКОГО отсутствия Linux-поддержки НЕТ — фейк-стратегия тестов реализуема на Linux целиком. Найденные API-нюансы: нет timeout-параметра у connect (→ обёртка), 5-с Windows-ретрай (Pitfall 2), ошибки Windows = IOException-совместимые, но не SocketException и не экспортируемые (Pitfall 3).
2. **flutter-action синтаксис — РЕШЕНО (VERIFIED README):** `flutter-version: 3.50.1`; `flutter-version-file` отвергнут фальсификацией (Pitfall 1); VS-шаг не нужен (образ содержит VS 2026 + C++); артефакт `build/windows/x64/runner/Release/*`; zip Compress-Archive; upload-artifact@v4.
3. **Триггеры/ветка — РЕШЕНО:** `push: branches: [windows]` + `workflow_dispatch` в отдельном windows-build.yml; с ci.yml (`push:[main]` + `pull_request`, ci.yml:3-6) конфликтов нет.
4. **Пути на CI-раннере — РЕШЕНО:** APPDATA/LOCALAPPDATA — стандартные профильные переменные (A1); паритет dirs: config=Roving (config_dir), модели по WIN-05 = LOCALAPPDATA (data_local_dir) с ЗАПИСАННЫМ обязательством фазы 7 переключить `data_dir()`→`data_local_dir()` (Pitfall 6).
5. **Пустой скелет на Windows — РЕШЕНО (компиляция VERIFIED по SDK-сурсу, рантайм трассирован, A2):** собирается без изменений; рантайм = штатный баннер.
6. **Верификация CI из CLI — РЕШЕНО (VERIFIED живым gh):** gh 2.104.0 authenticated со scope workflow; время ~15-25 мин (A3).
7. **Пин flutter — РЕШЕНО фальсификацией (Pitfall 1):** диапазон в pubspec (или ничего) + явный flutter-version в workflow; локальная разработка на 3.50.1 не ломается (живой pub get прошёл).

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | flutter_test (SDK), сьют app/test/ — 29 файлов, все зелёные (фазы 1-5) |
| Config file | app/analysis_options.yaml (flutter_lints); отдельного CI-конфига нет |
| Quick run command | `cd app && flutter test` (~30-60 c) + `flutter analyze` |
| Full suite command | `cd app && flutter test && flutter analyze` + cyrillic gate + `cargo test --workspace` (Linux, не трогается) |

### Phase Requirements → Test Map

| Req ID | Behavior | Тип проверки | Где проверяется | File Exists? |
|--------|----------|--------------|-----------------|---------------|
| WIN-01 | Workflow-файл существует, синтаксис валиден | статика | `gh workflow view windows-build.yml` / actionlint-глазом | ❌ Wave 1 |
| WIN-01 | Zip-артефакт собирается, зелёный прогон на пустом скелете | живой CI-прогон | `gh run list --workflow windows-build.yml --branch windows` + `gh run watch --exit-status` | ❌ Wave 1 (первый deliverable) |
| WIN-01 | Zip содержит exe+dll+data, непустой | CI-проверка в workflow (шаг с `Test-Path`/список) + локально `unzip -l` скачанного | CLI после прогона | ❌ Wave 1 |
| WIN-01 | Idempotency (повторный прогон зелёный) | живой CI (workflow_dispatch без push) | CLI после прогона | ❌ Wave 2 |
| WIN-02 | Оболочка = та же (паритет вкладок) | ревью + рантайм-трассировка (Pattern 3) + UAT на машине пользователя | код-гейт + UAT | — |
| WIN-02 | Windows-транспорт за швом, баннер при недоступной трубе | юнит-тест на Linux-фейке (dart_ipc bind UDS) | `test/windows_engine_test.dart` | ❌ Wave 2 |
| WIN-02 | Выбор реализации по Platform.isWindows в провайдере; UI не знает о транспорте | юнит (фейки) + grep-гейт «нет plugin/transport-вызовов из виджетов» | провайдер-тест | частично (shell_test.dart фазы 1-5) |
| WIN-05 | Конфиг-путь Windows = %APPDATA%\dicta\config.toml | юнит (injectable env) | `test/config_path_windows_test.dart` | ❌ Wave 3 |
| WIN-05 | Модели Windows = %LOCALAPPDATA%\whisper-cpp | юнит (injectable env) | рядом с hf-тестами | ❌ Wave 3 |
| WIN-05 | Нет юниксовых хардкодов в Windows-ветках; `\\.\pipe\dicta` только в константе | grep-гейт (ревью) | верификация фазы | — |
| WIN-05 | DACL-контракт записан | ревью-гейт: наличие записи в README/решении фазы | верификация фазы | ❌ Wave 3 |
| (гейт) | Linux-проверки не сломаны | flutter test + analyze + cyrillic gate на Linux | каждая задача фазы | ✅ (29 файлов) |
| (гейт) | Протокол/демон/Rust не тронуты | diff-гейт: crates/, adapters/, systemd/, daemon_engine.dart, ci.yml, release.yml — нулевые диффы в ветке windows | ревью фазы | — |

### Sampling Rate

- **Per task commit:** `cd app && flutter test && flutter analyze` (Linux, ~1 мин).
- **Per wave merge:** полный Linux-сьют + push ветки windows → живой CI-прогон (`gh run watch --exit-status`), статус фиксируется в SUMMARY (не блокируя задачи — CONTEXT:67-69).
- **Phase gate:** зелёный windows-build.yml на HEAD ветки + Linux-сьют зелёный + DACL-контракт в доках.

### Wave 0 Gaps

- [ ] `.github/workflows/windows-build.yml` — WIN-01 (первый deliverable, до dart_ipc).
- [ ] git-ветка `windows` от `d07763a` + push.
- [ ] `app/test/windows_engine_test.dart` — WIN-02 (фейк через dart_ipc.bind на UDS).
- [ ] `app/test/config_path_windows_test.dart` — WIN-05.
- [ ] DACL-контракт README-запись — WIN-05 (фаза 7 executor читает её как ТЗ).

**Рекомендация по волнам:** Wave 1 = ветка + workflow + первый зелёный прогон на ПУСТОМ скелете (только WIN-01; adjacency: workflow-коммит ПЕРВЫМ в истории ветки). Wave 2 = pubspec dart_ipc 2.0.2 + WindowsPipeEngine + провайдер-fork + windows_engine_test (WIN-02; dart_ipc после зелёного CI — locked decision 4). Wave 3 = Windows-ветки путей (config/models) + DACL-контракт-запись + повторный idempotency-прогон (WIN-05). Финальный гейт: полный Linux-сьют + живой CI на HEAD.

## Security Domain

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V5 Input Validation | да (JSON-line от транспорта) | decodeReplyMap (engine.dart:93-97) — переиспользуется без изменений |
| V6 Cryptography | нет | — |
| Transport pipe access | да (фаза 7 реализует) | DACL-контракт «только текущий user SID» (PITFALLS.md:234) — ЗАПИСЬ в фазе 6, реализация в фазе 7; dart_ipc-клиент к DACL-серверу подключается штатно (CreateFile-коннект) |
| Secrets | нет | в ветке нет секретов; payload-приватность сохраняется (сообщения об ошибках без текста — паттерн daemon_engine.dart:58-62) |

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Локальный процесс подключается к трубе и шлёт команды | Elevation/Spoofing | DACL фазы 7 (контракт записан в фазе 6 — Pattern 5) |
| Транскрипты/payload в логах | Information Disclosure | сохранённый паттерн «тип+путь, без payload» в новой WindowsPipeEngine |

## Sources

### Primary (HIGH — живые пробы и чтение исходников этой сессии)
- dart_ipc 2.0.2: pub.dev (verified publisher monkeywie.cn, MIT, 17.4k/нед, 150 points) + pub-cache `~/.pub-cache/hosted/pub.dev/dart_ipc-2.0.2` (lib/src/dart_ipc.dart — публичный API bind/connect; named_pipe_socket.dart — `Win32NamedPipeSocket implements Socket`; dart_ipc_windows.dart — 5-с ретраи, Win32NamedPipeException, bind с null security attributes; CHANGELOG 2.0.0-2.0.2)
- Живая Linux-проба (эта сессия, Dart 3.14.0): pub get + compile + UDS roundtrip 0.4 мс/вызов + SocketException за 1 мс на отсутствующий путь
- Живая фальсификация pubspec-пина (эта сессия): точный `flutter: 3.50.0` → локальный pub get Failed на 3.50.1; диапазон `>=3.50.0 <3.51.0` → зелёный
- dart:io SDK-сурс: `~/development/flutter/bin/cache/dart-sdk/lib/io/socket.dart:12-15` — InternetAddressType.unix безусловен
- Живой код app/: pubspec.yaml (sdk ^3.14.0, БЕЗ flutter-пина), daemon_engine.dart:16-76, engine.dart:71-105, providers.dart:16-17/65-103, config_path.dart:14-35, hf_catalog.dart:32-51, main.dart, .github/workflows/ci.yml:3-6; crates/dicta-core/src/config.rs:246-261 (data_dir/config_dir/runtime_dir)
- gh CLI: живые `gh auth status` (repo+workflow scopes), `gh run list` против smirnovmihail1357-netizen/dicta (реальные прогоны)

### Secondary (MEDIUM-HIGH — официальные доки, fetched 2026-11-22)
- github.com/subosito/flutter-action README (main): flutter-version/flutter-version-file/cache/pub-cache, windows-latest сниппет, yq auto-install
- github.com/actions/runner-images README + Windows2025-VS2026-Readme (image 20261116.315.1: VS 2026 Enterprise 18.12, VC.Tools.x86.x64, CMake 4.5.0, Ninja, pwsh 7.6.10)
- docs.flutter.dev/release/breaking-changes/windows-build-architecture (build\windows\x64\runner\Release), docs.flutter.dev/platform-integration/windows/building
- github.com/flutter/flutter: issue #176399 + PR #177458 «Add support for Visual Studio 2026» (merged 2025-11-01, closed as fixed)
- github.com/dirs-dev/dirs-rs README: platform table (config_dir/data_dir/data_local_dir/runtime_dir на Windows)
- flutter/flutter#180481 (открытый угол выбора VS-тулчейна — монитор-пункт, фолбэк windows-2022)

### Tertiary
- [CITED] api.dart.dev dart:io Process (ProcessException при ненаходимой программе — основа A2); actions/upload-artifact v4; learn.microsoft.com Compress-Archive; GitHub docs триггеров (on.push.branches/workflow_dispatch)

## Metadata

**Confidence breakdown:**
- Транспорт (dart_ipc): HIGH — исходники прочитаны, API исполнено живьём на Linux
- CI-рецепт: HIGH — синтаксис сверен с README образов/action-ов; фолбэк для единственного живого риска (VS-тулчейн) предопределён
- Windows-пути: HIGH — таблица dirs + живой Rust-код; runtime-баннер: MEDIUM (A2 — трассировка, не прогон)
- Время билда: LOW-MEDIUM (A3 — оценка)

**Research date:** 2026-11-22
**Valid until:** ~2026-12-22 (стабильно; label windows-latest может мигрировать — Pitfall 4)

---
*Phase 6 research for: dicta v1.1 «Dicta App» — Windows CI + каркас*
*Дайджесты 5 ключевых вопросов закэшированы в research-store (ключи research-plan, тир HIGH/MEDIUM)*

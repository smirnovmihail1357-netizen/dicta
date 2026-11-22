---
phase: 06-windows-ci-karkas
reviewed: 2026-11-22T19:40:12Z
depth: deep
files_reviewed: 11
files_reviewed_list:
  - .github/workflows/windows-build.yml
  - README.md
  - README.ru.md
  - app/lib/core/config_path.dart
  - app/lib/core/hf/hf_catalog.dart
  - app/lib/core/providers.dart
  - app/lib/core/windows_engine.dart
  - app/pubspec.lock
  - app/pubspec.yaml
  - app/test/windows_engine_test.dart
  - app/test/windows_paths_test.dart
findings:
  critical: 0
  warning: 4
  info: 4
  necessity: 0
  total: 8
status: issues_found
---

# Phase 6: Code Review Report

**Reviewed:** 2026-11-22T19:40:12Z
**Depth:** deep (живые пробы + cross-file трейсинг, включая исходник dart_ipc 2.0.2 из pub-cache)
**Files Reviewed:** 11 (дифф `d07763a..windows`, 5 коммитов e217f4a…930fc66; ревью выполнялось на ветке `windows`, main возвращён)
**Status:** issues_found

## Summary

Ревью фазы 6 «Windows CI + каркас» каноном фаз 2-5: не вера в SUMMARY, а живые
пробы на ветке `windows`. Каркас честный: оба CI-прогона перепроверены живым
`gh run view` (skeleton:45050118081 на head e217f4a, features:45051000549 на
head 930fc66 — оба `success`), артефакт скачан этим ревью и распакован
(dicta_app.exe + flutter_windows.dll + 15 записей `data/` — не недо-зип),
контракты заморозки байт-в-байт (crates/, daemon_engine.dart, ci.yml,
release.yml — нулевой дифф), Linux-батарея 191/191 зелёная одним прогоном,
analyze чистый, cyrillic-гейт 0. Замыкание pubspec.lock ровно
dart_ipc/win32/ffi_leak_tracker (ffi и meta уже были в lock, версии не
сдвинуты). Wire-формат байт-в-байт с daemon_engine (buildStatusLine/
buildSetLine/buildInjectLine/decodeReplyMap переиспользованы, дублирования
нет), `on IOException` вместо запрещённого `on SocketException` (гейт 0),
пайп-литерал только в kDictaPipe, Platform.is ровно 2 точки, юниксовых
хардкодов в Windows-ветках нет. README DACL-контракт технически верен —
включая нетривиальное утверждение про `bind()` dart_ipc: проверено по исходнику
пакета, `CreateNamedPipe(..., null)` действительно передаёт default DACL.

Найдено 0 BLOCKER, 4 WARNING, 4 INFO, necessity-нарушений 0. Ключевые: (1)
connect-таймаут (2 с) отчётливо лжёт в подсказке — start/stop/inject сообщают
«таймаут 120 с», set — «таймаут 30 с», хотя реальное ожидание было 2 с (в
каноне `Socket.connect(timeout:)` кидает SocketException с отменой задачи и
честной формой без бюджета); (2) `.timeout()`-обёртка не отменяет внутренний
5-с ретрай-цикл dart_ipc — поздний успех оставляет никогда не
destroy()-енный Win32-хендл (фантомное соединение для сервера фазы 7);
(3) parent-проверка кастомного пути модели слепа к `\` — на Windows
«explicit custom-model parent wins» не выполняется, закачка/удаление уйдут в
%LOCALAPPDATA%\whisper-cpp; (4) job без `timeout-minutes` (default 6 ч на
застрявшем раннере).

## Live gates (замерены этим ревью, не скопированы из SUMMARY)

| Gate | Result |
|---|---|
| `cd app && flutter test` (ветка windows) | 191/191 passed (32 файла) |
| `flutter analyze` | No issues found! |
| `bash tool/cyrillic_gate.sh` | 0 |
| `gh run view 45050118081 / 45051000549 --json conclusion` | success / success (head e217f4a / 930fc66) |
| `gh run download 45051000549` + `unzip -l` | exe + flutter_windows.dll + data/ (15 записей), скачано этим ревью |
| `git diff --stat d07763a windows -- crates/` | пусто (RUST-FROZEN) |
| `git diff d07763a windows -- daemon_engine.dart ci.yml release.yml` | пусто (байт-в-байт) |
| lock-diff: добавленные заголовки | только dart_ipc, ffi_leak_tracker, win32; ffi/meta уже в lock, sha256 не тронуты |
| adjacency `git log d07763a..windows \| tail -1` | e217f4a ci(06-01) workflow-коммит первый |
| `implements DictaEngine` app/lib / app/test | 2 / 1 |
| `Platform.is` в app/lib | 2 (providers.dart:18, config_path.dart:29), features/ — 0 |
| пайп-литерал `\\.\pipe\dicta` в app/lib | 1 (kDictaPipe, windows_engine.dart:20) |
| `SocketException` в windows_engine.dart | 0 (гейт Pitfall 3) |
| `'APPDATA'` quoted / `LOCALAPPDATA` в lib | 1 / 1 |
| `/run/user\|XDG_\|~/.config` в windows_engine + добавленных строках config/hf | 0 |
| `flutter-version-file` в yml (исключённый литерал) | 0 |
| README-якоря: `named pipe DACL` EN / `DACL именованного канала` RU; `data_local_dir` ×2 | 1 / 1; есть |
| dart_ipc 2.0.2 из исходника (pub-cache): `Win32NamedPipeException implements IOException`; `connect()` ретраит ERROR_FILE_NOT_FOUND/ERROR_PIPE_BUSY 5 с; public API только `bind`/`connect` | подтверждено |

## Narrative Findings (AI reviewer)

### WR-01: Connect-таймаут (2 с) мисрепортит чужой бюджет — «таймаут 120 с» после двух секунд ожидания

**Severity:** WARNING
**File:** `app/lib/core/windows_engine.dart:71` (обёртка), catch-точки `:107-108` (_commandLine), `:133-137` (_snapshotCommand), `:196-199` (set)
**Issue:** `_connect()` = `connect(pipePath).timeout(_connectTimeout)` кидает
`TimeoutException`. В `_commandLine`/`_snapshotCommand`/`set` этот исключение
ловится общей веткой `on TimeoutException` → `_timeoutHint(timeout)`, где
`timeout` — бюджет **команды** (120 с для start/stop/inject/toggle/cancel,
30 с для set). Итог: при отсутствующем пайпе (штатнейший кейс до фазы 7 —
dart_ipc ретраит ERROR_FILE_NOT_FOUND все свои 5 с, движок срезает на 2 с)
`start()` через 2 секунды возвращает пользователю «демон не отвечает (таймаут
120 с)». Это нарушает канон честных ошибок («budget is reported so a wedged
paste is distinguishable», комментарий `:40-42` сам обещает различимость) и
расходится с каноном: dart:io `Socket.connect(timeout:)` на таймауте кидает
**SocketException** с отменой задачи (проверено по SDK,
socket_patch.dart:1178-1189 → `task.cancel()` + OSError «Connection timed
out»), т.е. DaemonEngine попадает в `on SocketException`-обёртку с честным
сообщением без бюджета. Только status() избегает лжи случайно (у него оба
бюджета по 2 с). Тесты этого не ловят: timeout-тест гоняет только status().
**Fix:** В `_call` выделить connect-таймаут в транспортную обёртку до общей
IOException-ветки:

```dart
try {
  socket = await _connect();
} on TimeoutException {
  // Connect has its own 2 s budget - report THAT, not the command budget.
  throw _PipeTransportException(
    '$_daemonNotResponding $pipePath: ${_timeoutHint(_connectTimeout)}',
  );
} on IOException catch (error) {
  throw _PipeTransportException(
    '$_daemonNotResponding $pipePath: ${error.runtimeType}',
  );
}
```

(_PipeTransportException — implements IOException, так что существующие
catch-точки подхватят сообщение как `_transportMessage`). + тест: silent fake
+ `start()` → `reply.error` содержит «таймаут 2», не «120».

### WR-02: `.timeout()` не отменяет 5-с ретрай-цикл dart_ipc — поздний успех оставляет осиротевший Win32-хендл

**Severity:** WARNING
**File:** `app/lib/core/windows_engine.dart:71`; свидетельство — dart_ipc 2.0.2 `lib/src/dart_ipc_windows.dart` (`connect()`: while(true) ретраи `ERROR_PIPE_BUSY`/`ERROR_FILE_NOT_FOUND`, deadline 5000 мс, задержка 20 мс) + `named_pipe_socket.dart` (фоновый read-цикл, `_activeOperationCount > 0` → `keepIsolateAlive = true`)
**Issue:** `Future.timeout()` не отменяет нижележащий future. После того как
движок получил TimeoutException на 2-й секунде, ретрай-цикл dart_ipc живёт
ещё ~3 с; если пайп появляется (или освобождается после ERROR_PIPE_BUSY) в
этом окне, `CreateFile` успешен, конструируется `Win32NamedPipeSocket` — и
его **никто никогда не destroy()-ит**: вызывающая сторона уже ушла по
TimeoutException. Утечка: открытый нативный pipe-хендл + фантомное
подключение, которое сервер фазы 7 увидит как клиента, который ничего не
шлёт и не отключается. При поллинге status каждые ~2 с окно гонки почти
всегда открыто. Канон свободен от этого: `Socket.connect(timeout:)`
вызывает `task.cancel()` (SDK, там же).
**Fix:** Срезать поздний успех в самой обёртке:

```dart
Future<Socket> _connect() {
  final connectFuture = connect(pipePath);
  return connectFuture.timeout(_connectTimeout, onTimeout: () {
    // dart_ipc's retry loop cannot be cancelled: reap a late success so
    // an orphaned Win32 handle is not left open.
    connectFuture.then((socket) => socket.destroy()).ignore();
    throw TimeoutException('connect $pipePath', _connectTimeout);
  });
}
```

### WR-03: destDirFrom слеп к `\` — Windows-путь кастомной модели обходит ветку «explicit parent wins», которой обещает комментарий

**Severity:** WARNING
**File:** `app/lib/core/hf/hf_catalog.dart:38-48` (parent-проверка `:38` против комментария `:43-45`); call-sites: `app/lib/core/downloads.dart:92`, `app/lib/features/models/models_screen.dart:117`
**Issue:** Фаза фиксирует Windows-контракт destDirFrom, и её собственный
комментарий гарантирует: «an explicit custom-model parent above still wins».
Но parent-проверка — `model.lastIndexOf('/')`. На Windows (платформа, для
которой пишется контракт) кастомный путь будет с обратными слешами
(`C:\models\ggml.bin`) → `/` не найден → вернётся
`%LOCALAPPDATA%\whisper-cpp`. Rust-эталон (catalog.rs:70-88,
`Path::new(model).parent()`) на Windows оба разделителя понимает — т.е.
клиент разъедется с демоном: закачка при активной кастомной модели уйдёт не
в её каталог, а `canDelete` пометит честно скачанные модели как
`outsideDestDir`. Materialизуется в фазе 7, но фаза 6 поставляет именно
контракт — и поставленный контракт ошибочен для Windows-разделителя, что
противоречит и комментариям, и тесту «explicit model parent path wins»
(который проверяет только Unix-форму `/opt/models/...`).
**Fix:** Учитывать оба разделителя в чистой функции (Platform.is здесь не
нужен — параметр-инжекция сохраняется):

```dart
final slash = model.lastIndexOf('/');
final backslash = model.lastIndexOf('\\');
final index = slash >= 0 && backslash > slash ? backslash : slash;
if (index >= 0) { ... } // index == 0 → корень текущего разделителя
```

+ тест-кейс `destDirFrom(r'C:\models\ggml.bin', null, '/home/u',
r'C:\Users\u\AppData\Local') == r'C:\models'`.

### WR-04: Job без timeout-minutes — застрявшая Windows-сборка держит раннер до 6-часового дефолта

**Severity:** WARNING
**File:** `.github/workflows/windows-build.yml:7-11` (job `build:` c `runs-on: windows-latest`, без `timeout-minutes`)
**Issue:** Точка отсечения из cicd-review-patterns: job с `runs-on:` и без
job-level `timeout-minutes` — застрявший flutter-build (известная категория
сбоёв на self-hosted/flaky раннерах) держит раннер до платформенного лимита
(360 минут). Оценка самой фазы (RESEARCH A3) — 15-25 мин; 6 часов штрафа на
каждый hang несоразмерны. Замечание честности: ci.yml/release.yml тоже без
timeout-minutes (канон репо), но новый workflow — то место, где фикс
бесспорен и ни с чем не конфликтует.
**Fix:** Одной строкой под `runs-on:`:

```yaml
    runs-on: windows-latest
    timeout-minutes: 30   # RESEARCH A3: 15-25 min + headroom
```

## Info

### IN-01: set()/fetchSettings() Windows-ветки не покрыты тестами — 30-с бюджет и SetReply-ошибки не проверены

**Severity:** INFO
**File:** `app/test/windows_engine_test.dart` (файл целиком) против `app/lib/core/windows_engine.dart:174-208`
**Issue:** Фейк-сьют гоняет status/stop/start (и timeout через status).
Крупнейшие непокрытые ветки нового кода: `set()` — все три error-ветки
(транспорт/таймаут-30с/формат) и SetReply-форма; `fetchSettings()` —
сырая эскалация ошибок (паритет с каноном, но на Windows это
_PipeTransportException, который никто не asserts). План тестов
requirements не нарушал (его список веток покрыт), но фаза заявила
«транспортный шов end-to-end» — 5 из 8 публичных команд движка не имеют ни
одного Windows-пути в тестах.
**Fix:** Против существующего фейка — 3 теста: set с валидным JSON-reply
(SetReply.note/settings), set к несуществующему пути (error без payload),
fetchSettings к молчащему фейку (TimeoutException эскалирует — контракт
settingsSnapshotProvider).

### IN-02: Error-строки status-поллинга нигде не отображаются — типизированный снимок девиации это display-dead данные

**Severity:** INFO
**File:** `app/lib/features/shell/shell.dart:46-58` (баннер читает только `connected`); `app/lib/core/windows_engine.dart:123-146`; мотивация — 06-01-SUMMARY.md «Deviations» п.1
**Issue:** Девиация «status() через _snapshotCommand» обоснована как «the
same banner path the poll notifier expects». Живой трейсинг: баннер
(shell.dart:46-58) ключуется на `!status.connected` и выводит статичный
`l10n.daemonUnavailable` («демон недоступен») — `DictaStatus.error`,
который честно строит _snapshotCommand, UI никогда не читает (grep
`status.error` в features/ пуст). На Linux поллинг тоже отбрасывает ошибку
(catch → disconnected) — UX-паритет сохранён, девиация безвредна, но её
заявленное обоснование переобещает: различие «таймаут 2 с» vs «демон
недоступен» пользователь не увидит. Ошибки stop/inject доходят до UI честно
(dicta_control lastError) — там канон работает.
**Fix:** Ничего не чинить в фазе 6. Либо в UAT-пункте 2 заменить обещание
«banner shows … the hint» на фактическое «generic daemon-unavailable
banner within ~2 s», либо (фаза 7) отдать error-поле в баннер.

### IN-03: Doc-комментарий settingsSnapshotProvider устарел для Windows-транспорта

**Severity:** INFO
**File:** `app/lib/core/providers.dart:58-59`
**Issue:** «Note the throwing contract: fetchSettings() itself may propagate
a raw SocketException/TimeoutException on a transport/timeout failure» — на
Windows сырой ошибкой будет приватный `_PipeTransportException` (toString =
«PipeTransportException: демон недоступен …»), не SocketException.
Поведение корректно (FutureProvider ловит всё), неточен только тип в
документации разделяемого провайдера.
**Fix:** «…a raw transport IOException (SocketException on Linux,
_PipeTransportException on Windows) or TimeoutException…».

### IN-04: resolveConfigPath-ветка Platform.isWindows недостижима на Linux CI — precedence-тест доказывает меньше, чем обещает его имя

**Severity:** INFO
**File:** `app/lib/core/windows_paths_test.dart:27-40` против `app/lib/core/config_path.dart:29`
**Issue:** Тест называется «DICTA_CONFIG override wins over every platform
branch», но на Linux CI выполняется только Unix-ветка: сама строка 29
(`if (Platform.isWindows) return windowsConfigPath(...)`) не исполняется ни
одним автоматическим тестом (Platform.isWindows не фейкается без
dart:io-override). Честность соблюдена (шапка файла прямо говорит «no Rust
Windows reference exists yet»), но порядок «override до Windows-ветки»
доказан только статически чтением кода; runtime-проверка осталась UAT-пункту
на Windows-машине — этого в SUMMARY не проговорено.
**Fix:** Одной строкой в SUMMARY/UAT: «resolveConfigPath Windows-routing
(строка 29) проверяется только живой Windows-машиной (UAT п.3); CI
проверяет windowsConfigPath инжектируемо».

## Structural Findings (fallow)

Structural-блок в приглашении ревью не передавался — секция опущена.

## Незакрытые вопросы фокуса ревью (сверено, дефектом не признано)

- **Wire-формат байт-в-байт:** все 8 команд против daemon_engine.dart
  сверены построчно — buildStatusLine/buildSetLine/buildInjectLine импортом,
  inline-строки start/stop/toggle/cancel/settings идентичны, `'app': ?app`
  форма та же; дублирования протокольных функций нет (PATTERNS запрет 7).
- **Таймаут-бюджеты:** имена и числа совпадают с каноном (:64-67 против
  daemon_engine.dart:45-48) — 2/2/30/120.
- **Payload privacy:** _PipeTransportException несёт только
  daemonNotResponding + pipePath + runtimeType; сырые read-path IOException
  (Win32NamedPipeException.toString = «<operation> failed with Win32 error
  <code>») payload не содержат; ни одна строка движка не интерполирует
  dictation-текст в ошибки. Гейт чист.
- **dispose/StreamController:** движок stateless (connection-per-command),
  StreamController в app-коде нет; в dart_ipc `destroy()` закрывает хендл и
  гасит read-цикл (named_pipe_socket.dart finally) — finally-destroy в
  `_call` достаточен.
- **Провайдер-fork:** единая точка Platform.isWindows; `resolveSocketPath()`
  на Windows не вызывается (short-circuit до Process.runSync('id') — на
  Windows его бы не было); onDispose не нужен — оба движка без ресурсов;
  переподключений нет, утечек нет.
- **Env-инжекция путей:** windowsConfigPath/destDirFrom — чистые функции с
  параметрами, Platform.environment читается только в production-обёртках;
  пустые значения покрыты тестами; Linux-диффы поведения нулевые
  (regression-группа зелёная, существующие сьюты не тронуты — diff-stat
  подтверждает только 2 новых тест-файла).
- **Workflow:** actions запинены тегами (v4/v2 — pins по канону репо и
  cicd-reference «version tag is a pin»); секретов нет; `defaults.run.working-directory:
  app` корректен для run-шагов, upload-artifact путь `app/…` корректен для
  uses-шага (живая загрузка артефакта это подтвердила); Compress-Archive
  `Release/*` рекурсивно включил data/ (проверено unzip -l);
  concurrency-группа не нужна (CI-only workflow, negative-space правила);
  Node20-annotation на @v4 — принятая, незаблокированная деградация, оба
  прогона зелёные.
- **README DACL:** все три технических утверждения сверены — CreateNamedPipe
  dart_ipc действительно передаёт null security attributes (исходник
  пакета), data_dir→Roaming/data_local_dir→Local (dirs-rs), interprocess —
  штатный Rust-путь; якоря EN/RU greppable, data_local_dir в обоих.
- **Замыкание lock:** только dart_ipc/win32/ffi_leak_tracker; ffi/meta уже
  были в lock и не сдвинуты (sha256 сверены d07763a vs windows).
- **Мёртвых тестов нет:** оба новых файла исполняются (13 тестов в счётчике
  191), ни один сьют не пропущен/не помечен skip.

---

_Reviewed: 2026-11-22T19:40:12Z_
_Reviewer: the agent (gsd-code-reviewer)_
_Depth: deep_

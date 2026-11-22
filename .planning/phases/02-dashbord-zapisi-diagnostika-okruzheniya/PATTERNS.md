# Phase 2: Дашборд записи + диагностика окружения - Pattern Map

**Mapped:** 2026-11-22
**Files analyzed:** 7 (5 new, 2 modified)
**Analogs found:** 7 / 7

Все аналоги — git-tracked (сверено `git ls-files`). Язык комментариев/doc — русский
(CONVENTIONS.md), идентификаторы английские; кириллица в `app/lib/**/*.dart` вне
`lib/l10n/generated/` запрещена гейтом `app/tool/cyrillic_gate.sh` — весь русский UI
только через `.arb`, русский runtime-текст в core/ — Unicode-escape (паттерн
`daemon_engine.dart:26-31`).

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|---|---|---|---|---|
| `app/lib/features/dashboard/dashboard_screen.dart` (new) | component | event-driven (poll-driven) | `app/lib/features/shell/shell.dart` | exact |
| `app/lib/features/dashboard/level_history.dart` (new: notifier + CustomPainter) | store + component | streaming (100 мс сэмплы) | `app/lib/core/providers.dart` + `crates/dicta-ui/src/render.rs:705-729` | role-match |
| `app/lib/features/dashboard/diagnostics_screen.dart` (new: чеклист DIAG-01) | component | request-response | `app/lib/features/shell/shell.dart` (banner) + `app/lib/core/settings.dart` (разбор ответа) | role-match |
| `app/lib/core/engine.dart` (modify: +`inject(text)`) | service interface | request-response | сам файл + `daemon_engine.dart:99-127` | exact |
| `app/lib/core/providers.dart` (modify: +histtory/settings provider) | store | streaming | сам файл (`DictaStatusNotifier`) | exact |
| `app/lib/features/shell/shell.dart` (modify: стыковка экрана) | component | — | сам файл | exact |
| `app/test/dashboard_test.dart` (new) | test | request-response | `app/test/shell_test.dart` + `app/test/settings_model_test.dart` | exact |

## Pattern Assignments

### `app/lib/features/dashboard/dashboard_screen.dart` (component, poll-driven)

**Analog:** `app/lib/features/shell/shell.dart`

**Структура и l10n** (shell.dart:23-26, 119-123): `ConsumerWidget`/`ConsumerStatefulWidget`,
`final l10n = AppLocalizations.of(context)!;`, `final status = ref.watch(dictaStatusProvider);`,
цвета — только `Theme.of(context).colorScheme` (Material You, как banner: `colors.errorContainer`
shell.dart:32-41). Состояние — `switch (status.state)` по трём реальным значениям
`DaemonState` (shell.dart:119-123) — четвёртого состояния «пауза» не изобретать
(CONTEXT: демон имеет ровно idle/recording/transcribing).

**Правило APP-03 (заголовок shell_test.dart:1-2):** под `features/` нет transport-кода и
нет `dart:io` — экран зовёт только методы `ref.read(engineProvider)` (start/stop/cancel/set/inject)
и читает `ref.watch(dictaStatusProvider)`.

**Кнопки (DASH-02):** честность по `status.state` — во время `transcribing` кнопка
cancel `onPressed: null`; после stop НЕ делать свой UX-таймаут: команда stop уже
ушла с `_injectTimeout` 120 c, спиннер «распознаю…» держится до следующего status-тика
(контекст §7).

**Copy/inject (DASH-03):** «скопировать» — `Clipboard.setData` (локально, без демона);
«вставить заново» — `ref.read(engineProvider).inject(status.text!)`. inject требует
непустой текст ≤5000 символов (`dictad main.rs:881-887`) — кнопка disabled при
`status.text == null || status.text.isEmpty`.

**Слайдер min_level (DASH-04):** паттерн «одна команда на жест» из CONVENTIONS.md:151-177
(egui `drag_stopped`): локальная `pending`-копия значения, демону — только по
`onChanged`-завершению (Flutter: `Slider.onChangeEnd`), формат отправки —
`set('min_level', pending.toStringAsFixed(2))` — тот же формат, что шлёт панель
(`panel.rs:291,296`).

### `app/lib/features/dashboard/level_history.dart` (store + painter, streaming)

**Analog 1 (notifier):** `app/lib/core/providers.dart:19-57` — `class X extends Notifier<T>`
с `build()`, `ref.onDispose`, guards `_inFlight`/`_disposed` (Riverpod 3 не имеет legacy
mounted-API — комментарий providers.dart:17-18). Ring-буфер сэмплов `base.level` копится
ЗДЕСЬ (не в DictaStatus): подписка на `ref.listen(dictaStatusProvider, ...)` в `build()`,
окно последних N сэмплов в памяти, не persist (контекст §открытые вопросы п.3).
Сэмпл берём в каждом тике поллинга 100 мс — уже ~10 Гц.

**Analog 2 (семантика уровня):** `crates/dicta-ui/src/render.rs:705-729` — `level` это float
0..1; новейший сэмпл справа; при короткой истории колонка пустая (0.0); высота
`max(2, level * H)`; пороги оттенка `>0.6` / `>0.3` / остальное. Для гистограммы
калибровки (DASH-04) рисуем стандартными виджетами: `CustomPainter` + `LayoutBuilder`,
никаких chart-пакетов (контекст scope fence). Отметка min_level — вертикальная
линия поверх, значение для отметки — из settings-ответа (`SettingsReply.minLevel`,
settings.dart:93), не из локальной догадки.

**Anti-pattern (CONCERNS.md «panel_rest_sig»):** в dicta-ui рендер кэшируется по строковой
подписи кадра и «зависает», если новое динамическое поле забыли внести в sig. В Flutter
этот класс проблемы не воспроизводится (Ref.watch перерисовывает сам) — НЕ вводить
никакого ручного кэша кадров.

### `app/lib/features/dashboard/diagnostics_screen.dart` (component, request-response)

**Analog (banner):** `app/lib/features/shell/shell.dart:31-42` — контейнер на всю ширину
с `colors.errorContainer/onErrorContainer`; для чеклиста — строки-пункты с иконкой
✓/✗ (`Icons.check_circle` / `Icons.error`, тон через `colorScheme`) и подписью-подсказкой.

**Данные (DIAG-01):** п.1–2 из `ref.watch(dictaStatusProvider)` (`connected`, живой state);
п.3 «хоткей-путь» — БЕЗ дёргания хоткея: помечать как «проверьте руками dictactl toggle»
(контекст §3, §открытые вопросы п.2); п.4 «модель найдена/загружена» — из одного
`fetchSettings()` через `SettingsReply.fromReply` (`settings.dart:115-132`):
`models` непуст, `model` непуст, `reloading == false` — reloading на свежем демоне
нормален, не ошибка (doc settings.dart:6-7). Один fetch на открытие экрана, не поллинг.

**Anti-pattern (CONCERNS «3 копии схемы»):** НЕ заводить пятую копию разбора
settings-ответа — только существующий `SettingsReply.fromReply`. Новых команд демону нет.

### `app/lib/core/engine.dart` + `daemon_engine.dart` (modify: +inject)

**Analog:** соседние команды в `daemon_engine.dart:99-127` — `_command(cmd, app)` c
`_injectTimeout`. Новый метод интерфейса `Future<DictaReply> inject(String text)`:
payload строго через `jsonEncode` (`{'cmd': 'inject', 'text': text}`) — как все строки
в engine.dart:88-91. **Anti-pattern (CONCERNS «4 способа собрать JSON»):** никакой
ручной конкатенации/вырезания кавычек (баг dictactl: `dictactl inject 'он сказал "да"'`
теряет кавычки) — Dart-слой уже делает правильно через `jsonEncode`, не копировать
сломанный rust-паттерн. Ошибку демон вернёт русским текстом в `error` — наружу через
существующий `DictaReply(ok:, error:)` (engine.dart:62-67), без исключений.

Заодно обновить `FakeDictaEngine` в `shell_test.dart:11-40` и `_FakeEngine` в
`widget_test.dart:10-36` (interface расширился — оба фейка обязаны реализовать `inject`).

### `app/lib/core/providers.dart` (modify: +settings/history провайдеры)

**Analog:** `DictaStatusNotifier` (providers.dart:19-57): `build()` + `ref.onDispose`,
guard-флаги вместо mounted-API, backoff 200 мс→2 с не нужен для разовых fetch — только
для поллинга. Settings-провайдер — FutureProvider/Notifier поверх
`ref.read(engineProvider).fetchSettings()`, кэширует `SettingsReply` для отметки
min_level и диагностики; сбрасывать не по таймеру.

### `app/lib/features/shell/shell.dart` (modify: стыковка)

Заменить `_DashboardPlaceholder` (shell.dart:112-146) в `IndexedStack` (shell.dart:76-84)
на `const DashboardScreen()`; placeholder-класс удалить. Диагностика — либо отдельный
destination-слот, либо секция внутри дашборда (решение за планом); навигация не меняется.

### `app/test/dashboard_test.dart` (test)

**Analog:** `app/test/shell_test.dart` — точь-в-точь каркас:
- `FakeDictaEngine implements DictaEngine` (shell_test.dart:11-40) — расширить
  сценарностью: `status()` выдаёт программируемую последовательность снапшотов
  (level меняется между тиками), `set`/`inject` записывают вызовы в список для
  assert «одна команда на жест».
- `buildTestApp(fake)` (shell_test.dart:42-52): `ProviderScope(overrides:
  [engineProvider.overrideWithValue(fake)])` + `MaterialApp` с
  `AppLocalizations.localizationsDelegates`, `locale: Locale('ru')`.
- Паттерн ожидания поллинга: `await tester.pump(const Duration(milliseconds: 300))`
  (shell_test.dart:60); финальный `await tester.pumpWidget(const SizedBox.shrink())`
  дляdispose scope (shell_test.dart:68,80) — обязателен, иначе dangling-timer leak.
- Дополнительно (аналог `settings_model_test.dart:56-67`): отравленные/битые ответы
  не роняют UI — fake возвращает мусорный map, expect: экран рисует disconnected-ветку.

## Shared Patterns

### Русский UI / l10n
**Source:** `app/lib/l10n/app_ru.arb`
**Apply to:** все новые строки экранов. Формат записи: `"key": "русский текст"` +
`"@key": {"description": ..., "placeholders": {...}}` (пример с placeholders —
`stateLabel`/`levelSeconds`, arb:44-58). Новые ключи для дашборда: кнопки
(запуск/стоп/отмена/распознаю…), подписи гистограммы, пункты чеклиста. После правки —
`flutter gen-l10n` (committed generated output, l10n.yaml). В widget-тестах текст
проверяется русскими литералами (`find.text('Дашборд')`, shell_test.dart:62) — в test/
гейт не действует.

### Ошибки без исключений
**Source:** `app/lib/core/engine.dart:62-67` + `daemon_engine.dart:83-97`
**Apply to:** все новые пути команд: transport-ошибка → `DictaReply(ok: false,
error: actionableHint)`; UI показывает error, никогда не crash. Никаких своих
try/catch с rethrow в features/.

### Anti-patterns (из CONCERNS.md — не тащить в Dart)

1. **Не спамить `set` на каждый кадр слайдера** — только по отпусканию
   (CONVENTIONS.md:151-177; CONTEXT §8).
2. **Не собирать JSON руками** — только `jsonEncode` (баг потери кавычек в dictactl).
3. **Не выдумывать состояния/команды демона** — три state, команды из
   status/start/stop/cancel/inject/set/settings (CONTEXT «Проверенные факты»).
4. **Не читать config.toml демона приложением** — всё через сокет-протокол
   (CONTEXT §3; «Ручной разбор config.toml» как анти-пример).
5. **Не таймаутить stop на стороне UX** — ждать status-тик (CONTEXT §7).

## No Analog Found

Пусто — для каждого файла найден tracked-аналог в `app/lib` / `app/test` /
`crates/dicta-ui`.

## Metadata

**Analog search scope:** `app/lib/**`, `app/test/**`, `crates/dicta-ui/src/{panel,render}.rs`
**Files scanned:** 10
**Pattern extraction date:** 2026-11-22

# Phase 3: История диктовок - Pattern Map

**Mapped:** 2026-11-22
**Files analyzed:** 9 (4 new, 5 modified)
**Analogs found:** 9 / 9

Все аналоги — git-tracked (сверено `git ls-files`, 12/12). Кириллица в
`app/lib/**/*.dart` вне `lib/l10n/generated/` запрещена гейтом
`app/tool/cyrillic_gate.sh` — весь русский UI только через `.arb`; русский
текст в core/features при необходимости — Unicode-escape (паттерн
`dicta_control.dart:10-15`). Гейт единственной фабрики фейков живой:
`grep -c "implements DictaEngine" app/test/*.dart` → только
`fake_dicta_engine.dart:1` (== 1, см. 02-SECURITY T-2-SC).

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|---|---|---|---|---|
| `app/lib/features/history/history.dart` (new: HistoryEntry + HistoryNotifier) | store (model + riverpod) | event-driven (append из stop-ответа) | `app/lib/features/dashboard/level_history.dart:7-24` + `dicta_control.dart:21-60` | exact |
| `app/lib/features/dashboard/dicta_control.dart` (modify: врезка в `stop()`) | store (существующий) | request-response | сам файл, `stop()` :87-106 | exact |
| `app/lib/features/history/history_screen.dart` (new) | component | event-driven + локальный CRUD (filter/remove) | `app/lib/features/dashboard/dashboard_screen.dart` + `last_text_card.dart` | exact |
| `app/lib/core/theme_ext.dart` (new: ThemeExtension `muted(context)`) | config (тема) | — | процесс-аналог: коммит `3fcd6ac` (UI-долг фазы 2 одним свипом) | role-match |
| `app/lib/main.dart` (modify: регистрация extension в ThemeData) | config | — | сам файл :44-46 | exact |
| `app/lib/features/shell/shell.dart` (modify: слот «История») | component | — | сам файл :79-87 | exact |
| разморозка ×5 копий muted-стиля (modify: 5 файлов features/) | refactor | — | `02-UI-REVIEW.md:75` (реестр копий) | exact |
| `app/test/history_test.dart` (new) + сценарии в существующих сьютах | test | request-response + state | `app/test/last_text_test.dart` + `level_history_test.dart` | exact |
| `app/lib/l10n/app_ru.arb` + generated (modify) | config (l10n) | — | сам файл (canon имён ключей) | exact |
| `app/tool/diag_smoke.dart` (modify: `error.runtimeType`) | tool | request-response | `02-SECURITY.md:30-36` (residual) | exact |

## Pattern Assignments

### `app/lib/features/history/history.dart` (new: HistoryEntry + HistoryNotifier)

**Analog 1 (notifier-каркас):** `app/lib/features/dashboard/level_history.dart:7-24`

```dart
class LevelHistoryNotifier extends Notifier<List<double>> {
  static const capacity = 200;

  @override
  List<double> build() => const <double>[];

  void push(double level) {
    final next = <double>[...state, level];
    state = next.length > capacity
        ? next.sublist(next.length - capacity)
        : next;
  }

  void clear() => state = const <double>[];
}

final levelHistoryProvider =
    NotifierProvider<LevelHistoryNotifier, List<double>>(LevelHistoryNotifier.new);
```

Копировать: `class X extends Notifier<T>` + `build()` + публичные мутаторы
(`add`/`remove`) + `NotifierProvider` в том же файле. Doc-коммент «in-memory,
deliberately not persisted» (`level_history.dart:1-2`) — переформулировать для
истории (HIST-04: память = MVP, персистентность = FUT-03).

**Analog 2 (иммутабельная модель-значение):** `dicta_control.dart:21-60`
(`DictaControlState`: const-конструктор, final-поля, `copyWith` с
clear-флагами). `HistoryEntry` — та же дисциплина: const-конструктор
`{required int id, required String text, required DateTime at}`, final-поля,
без copyWith (запись неизменяема). Но НЕ та же локация: HistoryEntry —
UI-домен, класть рядом с нотифаером в features/, не в `core/engine.dart`
(протокол заморожен, `engine.dart:69-86` не трогаем).

**Analog 3 (приватный счётчик-поле):** `providers.dart:40` — `_backoff`
как образец приватного мутабельного поля нотифаера. Для стабильного id —
`int _nextId = 0;` инкремент внутри `add` (A1: int-счётчик, новые-сверху:
`state = [entry, ...state]`).

**Что НЕ копировать:** `providers.dart:36-51` (таймер, backoff,
`ref.onDispose`) — у истории нет поллинга и ресурсов; НЕ подписываться на
`dictaStatusProvider` (status никогда не несёт текст, A4-поправка) — запись
добавляется ТОЛЬКО явным вызовом из `DictaControlNotifier.stop()`.
Емкость-обрезку из `level_history.dart:15-17` НЕ вводить без указания планёра
(MVP — сотни записей, A1; overflow-политика — на усмотрение оркестратора).

### `app/lib/features/dashboard/dicta_control.dart` (modify: врезка захвата)

**Analog:** сам файл, `stop()` :87-106 — единственная точка врезки.

```dart
final snapshot = await ref.read(engineProvider).stop();
state = state.copyWith(
  commandInFlight: false,
  clearLastText: snapshot.text == null,
  lastText: snapshot.text,
  ...
);
```

Копировать существующую структуру `stop()` без изменений (busy-флаг :88,
try/catch :89/:99) и дописать ОДИН захват после успешного ответа:
`snapshot.text != null && snapshot.error == null` (guard A3 —
`snapshot.text!.trim().isNotEmpty`) → `ref.read(historyProvider.notifier)
.add(...)`. Внутри Notifier `ref` уже доступен — паттерн чтения соседнего
провайдера как в `dashboard_screen.dart:41` (`ref.read(...notifier)`).
Комментарий-замок в шапке файла (`dicta_control.dart:1-3`: «Recognized text
is captured only from our own stop/inject replies - status never carries
text») — дополнить строкой про историю.

**Что НЕ копировать:** `inject()` :129-148 НЕ трогать — inject-эхо не
дописывается (CONTEXT locked №2: иначе дубликаты при HIST-03 re-insert).
`cancel()`/`start()` :66-85 не трогать. НЕ «достаивать» захват в
`DictaStatusNotifier.poll()` (`providers.dart:53-73`) — status.text всегда
null (`crates/dictad/src/main.rs:828-833`), это грабли A4.

### `app/lib/features/history/history_screen.dart` (new)

**Analog 1 (скелет экрана):** `dashboard_screen.dart:17-52` —
`ConsumerStatefulWidget`, `final l10n = AppLocalizations.of(context)!;`,
`ref.watch(...)` в build, цвета только `Theme.of(context).colorScheme`.
Отличие по A1: вместо `SingleChildScrollView`+Column —
`ListView.builder` (сотни записей), search-поле через стандартный
`TextField` c локальным `setState(_query)` — паттерн локального editable
состояния как `_marker` в `calibration_card.dart:61-67`.

**Analog 2 (карточка-строка + кнопки + snackbar):** `last_text_card.dart`

- Карточный скелет `Card > Padding(16) > Column` c заголовком `titleMedium`
  (`last_text_card.dart:24-33`) — но заголовок-существительное по
  UI-наследию: заголовок `titleSmall/bodyMedium`, значение `titleLarge` с
  цветом по тону (02-UI-REVIEW.md:41, 97).
- Копирование: `ref.read(clipboardWriterProvider)(text)` +
  `ScaffoldMessenger...SnackBar(l10n.dashCopied)`
  (`last_text_card.dart:55-58`) — тот же шов, второй call-site разрешён
  (шов для того и создан; call-site аудит в 02-SECURITY T-2-02 считал
  production-вызовы — обновить нечем, паттерн тот же).
- Re-insert: ТОЛЬКО `ref.read(dictaControlProvider.notifier).inject(text)`
  (`last_text_card.dart:70-72`), гард `!busy` (`:52,68` —
  `commandInFlight` из CONTEXT №9), после `await` — `if (!context.mounted)
  return;` (`:74`) и чтение результата для snackbar (:75-81).
- Delete: локальный `historyProvider.notifier.remove(id)` + snackbar
  «удалено» по канону `dashCopied`/`dashInjected` (feedback-словарь
  CONTEXT №7).
- Честный empty-state с подсказкой (`last_text_card.dart:38-45` +
  `dashNoTextHint` :90-93): пустая история обязана честно сказать, что
  диктовки глобальным хоткеем демона невидимы (A4-ограничение, HIST-04).

**Analog 3 (таймстампы):** `dashboard_screen.dart:80-84` —
`fontFeatures: const [FontFeature.tabularFigures()]` для цифр времени
(UI-наследие №7). Форматирование даты/времени — локальный helper по образцу
`_formatSeconds` (`dashboard_screen.dart:188-194`: чистая функция без
DateTime-библиотек свыше core).

**Что НЕ копировать:** НЕ `GestureDetector`+`CustomPaint` для интерактива
(анти-паттерн BLOCKER из 02-UI-REVIEW.md:82 — калибровку чинили реальным
`Slider` в `3fcd6ac`; список/поиск — только стандартные виджеты
`ListView`/`TextField`). НЕ icon-only кнопки (02-UI-REVIEW.md:80 — все
кнопки текстовые; если IconButton — обязателен tooltip/Semantics).
НЕ выдумывать команду демона «удалить» — удаление чисто локальное, протокол
заморожен.

### `app/lib/core/theme_ext.dart` (new: ThemeExtension `muted(context)`)

**Analog (процесс, не код):** коммит `3fcd6ac` — так фаза 2 вводила
сквозное UI-изменение одним свипом: правки всех consumers +
`app_ru.arb` + regen + тесты в одном коммите (см. `git show 3fcd6ac
--stat`: 6 lib-файлов, .arb, generated, 3 теста). Здесь так же: новый файл
+ регистрация в main.dart + замена всех копий + `flutter analyze` 0.

**Копии для разморозки (grep по состоянию на HEAD):**

| Файл:строка | Копия |
|---|---|
| `app/lib/features/dashboard/last_text_card.dart:22` | `final weak = TextStyle(color: colors.onSurfaceVariant);` |
| `app/lib/features/dashboard/diagnostics_card.dart:18` | то же |
| `app/lib/features/dashboard/calibration_card.dart:167,173,180` | inline в style |
| `app/lib/features/dashboard/dashboard_screen.dart:81,95` | inline |
| `app/lib/features/shell/shell.dart:108` | `Theme.of(context).colorScheme.onSurfaceVariant` |

(Реестр из 02-UI-REVIEW.md:75 указывал :140,146,151/:79,87 — после `3fcd6ac`
строки сместились; ориентироваться на grep по
`TextStyle(color: colors.onSurfaceVariant)`.)

**Регистрация:** `main.dart:44-46` — в оба `ThemeData(colorScheme: ...)`
добавить `extensions: [MutedExt.light/dark]`; структура `ColorScheme.fromSeed`
(:37-43) не меняется. Новый экран истории использует extension с первого
дня — НЕ добавлять шестую копию.

**Что НЕ копировать:** НЕ тащить `ThemeExtension` в features/ — он
кросс-фичерный, место в `core/` (как `settings.dart`); НЕ заводить
extension для всего подряд — только muted-стиль, остальное на токенах M3
(02-UI-REVIEW.md:47: «образцовое использование токенов» — не сломать).

### `app/lib/features/shell/shell.dart` (modify: слот «История»)

**Analog:** сам файл :79-87 — `IndexedStack` со слотами; заменить ВТОРОЙ
`_SectionPlaceholder()` (:83, слот history) на `const HistoryScreen()`;
импортировать по образцу :7. NavigationRail (:49-76) и баннер (:31-45) не
трогать; `_SectionPlaceholder` остаётся для слотов «Модели»/«Настройки»
(:84-85) — класс НЕ удалять. `shell.dart:108` — заодно разморозить копию
muted (theme_ext).

### Тесты: `app/test/fake_dicta_engine.dart` (modify) + `history_test.dart` (new)

**Analog 1 (фабрика — ТОЛЬКО расширять):** `fake_dicta_engine.dart:15-124`.
Для сценариев истории СУЩЕСТВУЮЩИХ крюков достаточно: `stopReply`
(:43-44, выдача :85-90) программирует stop-ответ с текстом — это и есть
сценарий захвата. Новые крюки добавлять ТОЛЬКО если план докажет
необходимость — паттерн добавления: поле + использование в методе
(:53-59 `statusError`). Категорически НЕ клонировать класс: `implements
DictaEngine` остаётся ровно 1 (`fake_dicta_engine.dart:15`; гейт из
02-SECURITY T-2-SC: `grep "implements DictaEngine" app/test/*.dart | wc -l
== 1`).

**Analog 2 (виджет-сценарии):** `last_text_test.dart` — точь-в-точь каркас
для истории:
- helper `tick()` (:11-14), helper `enabled(tester, key)` (:16-17);
- захват текста через реальный stop: `captureTextViaStop` (:19-23) —
  tap `dashStopButton`, pump ×2 — история должна получить запись ЭТИМ же
  путём;
- сценарий: `fake.stopReply` с `text: 'привет'` (:47-51) → после stop в
  истории появилась запись; copy-кнопка истории → `onCopy`-shriver
  (`buildFakeApp(fake, onCopy: ...)` :74) + `expect(find.text('скопировано'))
  `(:84);
- инъекция из истории: `fake.injectTexts` (:112) — точный текст, гард
  busy при `injectGate` (:99,113);
- финальный `await tester.pumpWidget(const SizedBox.shrink())` (:61 и все
  сьюты) — dispose scope, без него dangling-timer leak;
- русские литералы в expect разрешены — гейт на test/ не действует
  (last_text_test.dart:34-39).

**Analog 3 (unit-тесты нотифаера):** `level_history_test.dart:6-33` —
`ProviderContainer()` + `addTearDown(container.dispose)` + прямые вызовы
`add`/`remove` + expect порядка (новые-сверху) и стабильных id.

**Analog 4 (assert состояния через контейнер):** `dashboard_state_test.dart:56-61`
— `captureContainer` в `buildFakeApp` + `container.read(historyProvider)`
для проверки списка записей.

**Существующие сьюты:** `shell_test.dart:33-36` (find.text('История'))
остаётся зелёным — rail не меняется; `placeholderScreen`-текст в слоте
истории исчезает — проверить, не asserts ли его кто-то (grep
`placeholderScreen` по test/ перед правкой).

### `app/lib/l10n/app_ru.arb` + generated (modify)

**Analog:** сам файл — канон структуры: `"key": "текст"` +
`"@key": {"description": ...}` (пример с placeholders — `stateLabel`
:44-58). Именование по образцу словаря обратной связи:
`dashCopied` (:98-101), `dashInjected` (:122-125) → новые
`histCopied`/`histDeleted`/`histReinjected`/`histEmptyHint`/`histSearchHint`
/… (~15-25 ключей, CONTEXT №5). После правки — `flutter gen-l10n`;
generated (`app_localizations*.dart`) коммитится, но руками НЕ правится
(в `3fcd6ac` generated обновлён в том же коммите :24-29 stat).

### `app/tool/diag_smoke.dart` (modify: погашение residual 02-SECURITY)

**Analog:** `02-SECURITY.md:30-36` (сама находка) + дисциплина фиксированных
строк `daemon_engine.dart` (error-пути несут тип ошибки и путь сокета, не
payload). Фикс: `diag_smoke.dart:28`
`stderr.writeln('DIAG SMOKE FAIL $error')` →
`stderr.writeln('DIAG SMOKE FAIL ${error.runtimeType}')` — одна строка,
больше в файле не менять (:26 фиксированная строка уже образцовая).

## Shared Patterns

### Русский UI / l10n / кириллический гейт
**Source:** `app_ru.arb` (canon) + `dicta_control.dart:10-15` (escape-паттерн).
**Apply to:** все новые строки экрана истории. Русский текст — только .arb;
комментарии с кириллицей — \uXXXX (гейт считает и комментарии).

### Ошибки без исключений
**Source:** `dicta_control.dart:66-106` — try/catch → `genericFailure: true`,
демонская ошибка — данные (`last_text_card.dart:91-110`: verbatim показ,
никаких выдуманных текстов).
**Apply to:** врезка захвата в stop(): catch-ветка уже существует — в неё
НЕ добавлять запись истории (захвата нет — нет записи).

### busy-gating `commandInFlight`
**Source:** `dicta_control.dart:23,30` + `last_text_card.dart:52,68`.
**Apply to:** кнопки re-insert из истории (единственный путь вставки —
`dictaControlProvider.inject()`, CONTEXT №9).

### Feedback-словарь (snackbar)
**Source:** `last_text_card.dart:56-58,78-81` + `dashCopied`/`dashInjected`.
**Apply to:** «скопировано»/«удалено»/«вставлено заново» — transient
SnackBar только по факту успеха, паттерн `if (!context.mounted) return;`
после await.

### A11y-минимум фазы 2 (не откатывать)
**Source:** `dashboard_screen.dart:63-69,76-99` (Semantics liveRegion /
excludeSemantics), `calibration_card.dart:156-164` (настоящий Slider).
**Apply to:** экран истории: liveRegion на transient-изменениях не нужен
(список сам читается), но semantic-подписи и текстовые кнопки обязательны.

## Запреты-по-аналогии

1. **Пятый парсер Settings** — не заводить: единственный парсер
   `SettingsReply.fromReply` (`providers.dart:20-32`, lock-комментарий
   :18-19). История не парсит ответы демона вовсе — ей хватает
   `DictaStatus.text`.
2. **Второй `implements DictaEngine`** — гейт ==1
   (`fake_dicta_engine.dart:15`); клоны фейков из фазы 1 уже слиты,
   обратно не разводить.
3. **Кириллица в lib/ вне generated** — только .arb; обход гейта
   \uXXXX-эскейпами для USER-VISIBLE строк запрещён (урок
   02-UI-REVIEW.md:74: `dashCommandFailedBody` вынесли в .arb в `3fcd6ac`).
4. **Drag-жесты вместо стандартных контролов** — BLOCKER-урок
   калибровки (02-UI-REVIEW.md:82, чинился `3fcd6ac`): поиск — `TextField`,
   список — `ListView.builder`, никаких CustomPaint-интерактивов.
5. **Свой путь вставки** — только `dictaControlProvider.inject()`
   (CONTEXT №9); параллельный `engine.inject` из экрана истории мимо
   commandInFlight запрещён.
6. **JSON руками** — только `jsonEncode` (`engine.dart:94-100`); истории
   протокол не нужен вовсе — новых строк в провод НОЛЬ.
7. **Персистентность** — никаких файлов/JSONL/SharedPreferences/sqflite
   (CONTEXT №3); паттерн doc-замка `level_history.dart:1-2`.
8. **Inject-эхо в историю** — не писать (CONTEXT №2): re-insert известного
   текста не создаёт диктовку, иначе дубликаты при HIST-03.
9. **Захват мимо stop()-шва** — не подписываться на status-поток
   (`providers.dart:53-73` не трогать): status.text всегда null, «достать
   текст постфактум» нельзя (`dictad main.rs:828-833,1030-1036`, A4).
10. **UX-таймауты / выдуманные состояния демона** — три DaemonState, чужих
    состояностей не изобретать (`dashboard_screen.dart:46-50` switch).

## No Analog Found

| Артефакт | Reason |
|---|---|
| Поиск `TextField` по списку | Прямого аналога в lib/ нет (единственный editable-state — `_marker` `calibration_card.dart:61-67`, частичный); стандартный виджет Flutter, planёр берёт M3-дефолты |
| Формат таймстампа записи | Аналога нет (`_formatSeconds` `dashboard_screen.dart:188-194` — только mm:ss без дат); планёр выбирает формат сам, tabular figures обязателен |

## Metadata

**Analog search scope:** `app/lib/**`, `app/test/**`, `app/tool/`,
`.planning/phases/02-*/02-{UI-REVIEW,SECURITY}.md`, git-история (`3fcd6ac`)
**Files scanned:** 16
**Tracked-source gate:** все названные аналоги сверены `git ls-files` (12/12 tracked)
**Pattern extraction date:** 2026-11-22

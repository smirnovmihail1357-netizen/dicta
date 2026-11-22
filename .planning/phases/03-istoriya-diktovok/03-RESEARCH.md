# Phase 3: История диктовок - Research

**Researched:** 2026-11-22
**Domain:** in-memory журнал диктовок в Flutter-приложении поверх замороженного протокола dictad (захват в точке шва `DictaControlNotifier.stop()`, экран списка + поиск + действия)
**Confidence:** HIGH — все факты прочитаны из живого кода приложения, SDK и демона в этой сессии; новых пакетов ноль, весь стек уже стоит; A4-поправка подтверждена повторным чтением main.rs.

<user_constraints>

## User Constraints (from CONTEXT.md)

### Locked Decisions (CONTEXT §«Захваченные решения», verbatim по смыслу, номера как в CONTEXT)

1. **Протокол/демон заморожены**: никакие правки crates/, сокета, формата ответов.
2. **Механизм захвата**: только шов `DictaControlNotifier.stop()` (см. A4 ниже). `inject()`-эхо НЕ дописывается (re-insert уже известного текста не создаёт новую диктовку — иначе дубликаты при HIST-03).
3. **Хранилище MVP — в памяти** (HIST-04): история живёт, пока приложение открыто/в трее. Персистентность (JSONL/файлы) — вне вехи (FUT-03). Никаких записей на диск.
4. **Никаких глобальных хоткеев приложения** на Linux (фаза 5, гонки двойного stop).
5. **Стиль кода** (фазы 1-2): l10n через .arb (ключи camelCase, ~15-25 новых), русские строки только в .arb; в lib/ не-комментная кириллица запрещена (гейт `app/tool/cyrillic_gate.sh` → 0); в комментариях/строках кода при необходимости — \uXXXX-эскейпы. Riverpod Notifier-паттерны как в существующем коде.
6. **Фабрика фейков**: `app/test/fake_dicta_engine.dart` РАСШИРЯЕТСЯ (не клонируется); `implements DictaEngine` в тестах остаётся ровно 1 (гейт `[HEAD=2 EXPECT==1]` — см. фазу 2).
7. **UI-наследие фазы 2** (из 02-UI-REVIEW.md), ввести в этой фазе: ThemeExtension `muted(context)` вместо ×5 копий muted-стиля; заголовочный паттерн карточек (существительное-заголовок, значение titleLarge с цветом по тону); tabular figures для таймстампов; feedback-словарь: snackbar-паттерн фазы 2 (dashInjected) как канон для новых действий (скопировано/удалено/вставлено заново).
8. **Security-residual фазы 2** (02-SECURITY.md, informational): `app/tool/diag_smoke.dart:28` печатает объект ошибки целиком — в этой фазе поправить на `error.runtimeType`.
9. **In-flight команды**: существующий `commandInFlight` замораживает кнопки — re-insert из истории обязан идти через существующий `DictaControlNotifier.inject()`, никаких параллельных путей вставки.

### A4-поправка (locked, из 02-RESEARCH.md — перепроверена в этой сессии)

`status` НИКОГДА не несёт текст [VERIFIED: crates/dictad/src/main.rs:828-833, ветка `"status"` жёстко возвращает `text: None, error: None`]. `run_stop` при `State != Recording` возвращает пусто сразу [VERIFIED: main.rs:1030-1038, `if State::from_index(...) != State::Recording { return Ok(None); }`]. Текст появляется ТОЛЬКО в ответах `stop`/`inject`, и приложение уже ловит обе точки: `stop()` dicta_control.dart:87-106 (`lastText: snapshot.text`, :94), `inject()` :129-148 (:136). Единственный честный механизм захвата — запись в историю в точке шва `stop()` при `snapshot.text != null && snapshot.error == null`. Хоткейные диктовки демона приложению невидимы — документируемое ограничение (вместе с HIST-04), без протокольных расширений.

### the agent's Discretion (открытые дополнения, разрешены оркестратору; не пересуживаются)

- **A1**: запись = текст + timestamp (DateTime) + стабильный int-id; новые-сверху; ListView.builder без пагинации.
- **A2**: поиск — case-insensitive substring; пустой запрос = все записи.
- **A3**: пустые/пробельные тексты в историю не пишутся.
- **A4**: перенесённая поправка фазы 2 — см. выше.

### Запреты (OUT OF SCOPE, попадают в план как prohibitions)

НЕ трогать crates/, pubspec.yaml deps, systemd, adapters; НЕ менять REQUIREMENTS/ROADMAP; НЕ вводить персистентность на диск (SharedPreferences, sqflite, JSONL); НЕ регистрировать глобальные хоткеи; НЕ дублировать `implements DictaEngine` (гейт ==1); НЕ читать исходники FlClash (GPL); НЕ добавлять не-комментную кириллицу в lib/ вне generated.

</user_constraints>

<phase_requirements>

## Phase Requirements

| ID | Description (REQUIREMENTS.md) | Research Support |
|----|-------------|------------------|
| HIST-01 | Пассивное накопление диктовок в локальный store; протокол не меняется | A4-механизм: врезка в `DictaControlNotifier.stop()` (§R1); store = Riverpod Notifier in-memory (§R2). Формулировка REQUIREMENTS «из status-потока» неверна — закрытие через CONTEXT A4 |
| HIST-02 | Листание журнала + поиск по тексту | `HistoryScreen` на IndexedStack-слоте 1 shell.dart:84 (§R3); ListView.builder + TextField-поиск (первый TextField в lib/, §R3) |
| HIST-03 | Копия записи / re-insert (`inject`) / удаление | Существующие швы: `clipboardWriterProvider` dicta_control.dart:155-157, `DictaControlNotifier.inject()` :129-148, удаление = локальный `remove(id)` (§R2); single-путь inject через `commandInFlight` (§R3) |
| HIST-04 | Ограничение MVP зафиксировано в docs | In-memory store, переживает только жизнь приложения; хоткейные диктовки невидимы (A4) — обе оговорки в docs фазы (SUMMARY) |

</phase_requirements>

## Summary

Фаза 3 — почти чистое добавление app-side: ни одного нового пакета, ни одной правки демона, ни одного расширения протокола. Вся работа — (а) in-memory `HistoryNotifier` по образцу `LevelHistoryNotifier`, (б) 3-5 строк врезки в существующий `DictaControlNotifier.stop()`, (в) экран `features/history/history_screen.dart` на слоте 1 IndexedStack вместо заглушки, (г) ThemeExtension muted + разморозка пяти копий, (д) мелкий долг diag_smoke.dart:28.

**Главное подтверждение (A4):** захват только в `stop()` — обе посылки заново прочитаны в живом коде этой сессии: status-ветка демона жёстко возвращает `text: None, error: None` [VERIFIED: main.rs:828-833], а `run_stop` вне Recording возвращает `Ok(None)` без текста [VERIFIED: main.rs:1030-1038]. `DictaEngine.stop()` уже возвращает `DictaStatus` с текстом [VERIFIED: app/lib/core/engine.dart:75], так что **сейм движка менять не нужно вообще** — ни новых сигнатур, ни новых крюков фейка (`stopReply` уже достаточен, fake_dicta_engine.dart:43,85-90).

**Ключевое открытие для безопасности батареи:** `IndexedStack` в Flutter 3.50.1 прячет невыбранные вкладки от тест-файндеров на уровне element-обхода: `_IndexedStackElement.debugVisitOnstageChildren` посещает ТОЛЬКО выбранный ребёнка [VERIFIED: flutter SDK packages/flutter/lib/src/widgets/indexed_stack.dart:196-203], а `find.text`/`find.byKey` по умолчанию ходят именно по onstage-обходу (`skipOffstage: true`, finders.dart:124,1207-1210) [VERIFIED: flutter SDK packages/flutter_test/lib/src/finders.dart]. Значит: после врезки истории запись «привет» будет существовать в дереве одновременно на дашборде и на экране истории, но все 51 существующих тестов остаются на вкладке 0 и не увидят дублей — батарея не ломается без единой правки.

**Primary recommendation:** `HistoryNotifier extends Notifier<List<HistoryEntry>>` в новом `app/lib/features/history/` (модель `{id, text, at}`), врезка `ref.read(historyProvider.notifier).add(...)` внутри try-блока `stop()` после успешного снапшота, экран истории на слоте 1 `shell.dart:84`, ThemeExtension регистрируется в main.dart И в `buildFakeApp` (иначе падают widget-тесты), таймстампы — `DateFormat.Hm('ru')` из уже стоящего intl 0.20.3.

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Захват диктовки (HIST-01) | Flutter notifier (`DictaControlNotifier.stop()`) | — | status текст не несёт (A4); точка шва уже ловит текст ответа |
| Хранение журнала (HIST-01/04) | Flutter Riverpod state (in-memory) | — | протокол/демон заморожены; диск запрещён (FUT-03) |
| Листание/поиск (HIST-02) | Flutter UI (ListView.builder + TextField) | HistoryNotifier (данные) | сотни записей — без пагинации (A1) |
| Копия записи (HIST-03) | Flutter UI → `clipboardWriterProvider` | — | Clipboard SDK, только явный клик (прецедент last_text_card.dart:55) |
| Re-insert (HIST-03) | Flutter UI → существующий `DictaControlNotifier.inject()` | daemon (валидация ≤5000, фокус) | single-путь через commandInFlight (CONTEXT lock 9) |
| Удаление (HIST-03) | Flutter notifier (`remove(id)`) | — | локальная операция, демон не участвует |

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| flutter_riverpod | 3.5.0 (pubspec.yaml:29) | состояние истории/запроса | единственный state-паттерн проекта; StateNotifier-легаси нет [VERIFIED: providers.dart:34-35 комментарий «Riverpod 3 has no legacy notifier/mounted API»] |
| intl | 0.20.3 [VERIFIED: pubspec.yaml:36 `intl: ^0.20.3`; pubspec.lock:163-170 `dependency: "direct main", version: "0.20.3"`] | `DateFormat.Hm` для таймстампов | уже прямая зависимость; generated-l10n уже импортирует intl [VERIFIED: app/lib/l10n/generated/app_localizations_ru.dart:2] |
| flutter_test | SDK (Flutter 3.50.1, проверен `flutter --version` в этой сессии) | виджет-батарея | 51/51 зелёных (фаза 2), пересчитано в этой сессии: `grep -cE "^\s*(testWidgets\|test)\(" test/*.dart` → 51 |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| — | — | новых пакетов ноль | гейт supply-chain: pubspec.yaml не трогаем (байт-в-байт к базе фазы) |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| intl DateFormat | ручной padLeft как `_formatSeconds` (dashboard_screen.dart:189-194) | ручной вариант уже в коде для mm:ss; для HH:mm DateFormat чище и уже зависимость —intl предпочтителен |

**Installation:** ничего. **Package Legitimacy Audit:** фаза не устанавливает внешних пакетов — аудит не требуется; гейт фазы 2 (`git diff --quiet <base> HEAD -- app/pubspec.yaml app/pubspec.lock`) остаётся частью контрактов.

## Architecture Patterns

### System Architecture Diagram (поток диктовки → журнал)

```
[Кнопка Стоп на дашборде]
        │  ref.read(dictaControlProvider.notifier).stop()
        ▼
DictaControlNotifier.stop()  (dicta_control.dart:87-106)
        │  await engine.stop() → DictaStatus snapshot
        │  [guard: text != null && error == null && text.trim().isNotEmpty]
        │  state = state.copyWith(... commandInFlight:false, lastText ...)   ← существующее
        ▼
HistoryNotifier.add(text)  ← ВРЕЗКА (новое; §R1)
        │  entry {id: nextId, text, at: DateTime.now()}
        │  state = [entry, ...state]   (новые сверху)
        ▼
historyProvider (in-memory List<HistoryEntry>) ── HIST-04: переживает только жизнь приложения
        │
        ├─► HistoryScreen (shell.dart слот 1, IndexedStack) ── ListView.builder, новые сверху
        │         ├─ поиск: TextField → HistoryQueryNotifier → фильтр substring (A2)
        │         ├─ «скопировать» → clipboardWriterProvider (явный клик) + snackbar
        │         ├─ «вставить заново» → dictaControlProvider.inject(text) ─ commandInFlight гейт
        │         └─ «удалить» → historyProvider.remove(id) + snackbar
        ▼
[inject-эхо: DictaControlNotifier.inject() :129-148 — запись НЕ создаётся (CONTEXT lock 2)]
```

Внешние зависимости системы: демон dictad (только команды stop/inject уже существующим путём), Clipboard SDK (только явный клик). Ветвление: снапшот с `error != null` или пустым текстом → запись не пишется (A3); хоткейная диктовка демона → в журнал не попадает никогда (A4, документируемое ограничение).

### Recommended Project Structure
```
app/lib/
├── core/
│   └── theme_ext.dart          # НОВОЕ: ThemeExtension muted (регистрация в main.dart + buildFakeApp)
└── features/
    ├── dashboard/dicta_control.dart   # ПРАВКА: 3-5 строк врезки в stop()
    ├── history/                # НОВОЕ:
    │   ├── history_entry.dart  #   модель {id, text, at} (+immutable)
    │   ├── history_store.dart  #   HistoryNotifier + HistoryQueryNotifier + форматтер времени
    │   └── history_screen.dart #   экран: поиск + список + действия
    └── shell/shell.dart        # ПРАВКА: слот 1 → HistoryScreen (shell.dart:84)
app/test/
├── fake_dicta_engine.dart      # БЕЗ ИЗМЕНЕНИЙ движковых крюков (stopReply уже есть); добавить theme-ext в MaterialApp
├── history_store_test.dart     # НОВОЕ: unit (порядок/id/пустышки/удаление/поиск/форматтер)
└── history_screen_test.dart    # НОВОЕ: widget (захват через stop, эхо, поиск, действия)
```

### R1 — механика врезки в `DictaControlNotifier.stop()`

**Где именно:** внутри try-блока `stop()` (dicta_control.dart:87-106), ПОСЛЕ обновления control-состояния (:91-98) — тогда `lastText` и запись истории никогда не разойдутся, и даже теоретический сбой в add не искажает кнопочную семантику. Гард по A3/A4:

```dart
// dicta_control.dart — stop(), после существующего state = state.copyWith(...)
if (snapshot.text != null &&
    snapshot.error == null &&
    snapshot.text!.trim().isNotEmpty) {
  // History is the journal of dictations made THROUGH the app: the stop
  // reply is the only honest capture point (status never carries text).
  ref.read(historyProvider.notifier).add(snapshot.text!);
}
```

`ref.read(...notifier)` — существующий паттерн этого же файла (dicta_control.dart:70,90,111,132). Provider создаётся лениво при первом read: в старых тестах, где история не нужна, создание пустого Notifier'а без таймеров ничего не меняет.

**Почему не ломаются 51 существующих тестов** (главный вопрос R1):
1. Все сценарные батареи сидят на вкладке 0 (`ShellWidget._selected = 0`, shell.dart:20); флайндеры по умолчанию не заходят в невыбранные вкладки IndexedStack [VERIFIED: SDK indexed_stack.dart:196-203 — `debugVisitOnstageChildren` обходит только выбранный элемент; finders.dart:124 — `skipOffstage = true` дефолт, finders.dart:1207-1210 — обход через `debugVisitOnstageChildren`]. Дубль «привет» на экране истории не виден тестам дашборда.
2. Дефолтный `stopReply` фейка — `DictaStatus(connected: true, state: DaemonState.idle)` без текста [VERIFIED: fake_dicta_engine.dart:88-89] → существующие сценарии, не задающие текст, вообще не порождают записей.
3. `HistoryNotifier.build()` — синхронный `const []` без таймеров (по образцу level_history.dart:11): нет новых pump-обязательств.
4. Новый крюк фейка НЕ нужен: захват происходит в control-нотифаере (app-слой), а не в движке; `stopReply`/`injectReply` (fake_dicta_engine.dart:43-44, 85-103) уже полностью скриптуют «stop с текстом» и «inject-эхо». OnSet-паттерн (fake_dicta_engine.dart:59,121) — прецедент добавления крюков, но здесь их ноль: фабрика движком истории не расширяется вовсе.

**Inject-эхо (CONTEXT lock 2):** в `inject()` (:129-148) врезки нет вообще — тест «эхо не пишет запись» фиксирует это как контракт.

### R2 — Riverpod-паттерн списочного состояния

В кодовой базе ровно один паттерн: `Notifier<T>` + иммутабельное состояние копированием. Прецеденты:
- `LevelHistoryNotifier extends Notifier<List<double>>` — список, push через `[...state, level]`, лимит ёмкости, `build() => const <double>[]` [VERIFIED: level_history.dart:7-24].
- `DictaControlState.copyWith` с clear-флагами для nullable-полей [VERIFIED: dicta_control.dart:39-59] — образец, если состоянию понадобятся nullable-поля; для `HistoryEntry` это не нужно.
- Никакого StateNotifier-легаси [VERIFIED: providers.dart:34-35 комментарий; все провайдеры — NotifierProvider/Provider/FutureProvider].

```dart
// history_store.dart (скелет; модель — immutable класс с final id/text/at)
class HistoryEntry {
  const HistoryEntry({required this.id, required this.text, required this.at});
  final int id;
  final String text;
  final DateTime at;
}

class HistoryNotifier extends Notifier<List<HistoryEntry>> {
  @override
  List<HistoryEntry> build() => const <HistoryEntry>[];

  void add(String text) {
    // Id derives from the newest entry: no mutable counter field to reset on
    // rebuild; new entries always sit at the head (A1).
    final id = state.isEmpty ? 1 : state.first.id + 1;
    state = [HistoryEntry(id: id, text: text, at: DateTime.now()), ...state];
  }

  void remove(int id) => state = state.where((e) => e.id != id).toList();
}

final historyProvider =
    NotifierProvider<HistoryNotifier, List<HistoryEntry>>(HistoryNotifier.new);

class HistoryQueryNotifier extends Notifier<String> {
  @override
  String build() => '';
  void set(String value) => state = value;
}
final historyQueryProvider =
    NotifierProvider<HistoryQueryNotifier, String>(HistoryQueryNotifier.new);
```

Рекомендации по R2-вопросам: **id-счётчик** — выводится из головы списка (`state.first.id + 1`), без мутируемого поля: rebuild Notifier'а сбрасывает и список, и счётчик согласованно (in-memory MVP — HIST-04 честно переживает только жизнь провайдера); **reverse-порядок** — новые сверху прямо в состоянии (`[entry, ...state]`), ListView.builder идёт по списку как есть, index 0 = новейшая; **поиск** — фильтрация в билде экрана от `historyQueryProvider` (`e.text.toLowerCase().contains(q.toLowerCase())`), пустой запрос = весь список (A2); Dart `toLowerCase()` Unicode-aware, кириллица приводится корректно. **Форматтер времени** — чистая функция `String formatEntryTime(DateTime at, String localeName)` в history_store.dart: unit-тестируемо без pump, с фиксированным DateTime.

### R3 — UI-аналоги и UI-наследие фазы 2

**Карточный скелет (канон):** `Card > Padding(16) > Column[Text(заголовок-существительное, titleMedium), SizedBox(12), …]` [VERIFIED: last_text_card.dart:24-34; заголовки titleMedium — calibration_card.dart:115, diagnostics_card.dart:74]. Текст записи — `SelectableText` (прецедент last_text_card.dart:36) — часть диктовки можно выделить и скопировать руками.

**Заголовочный паттерн (CONTEXT lock 7):** заголовок-существительное; динамическое значение — крупнее и с цветом по тону. Для истории динамических «значений» нет — паттерн применим к счётчику записей при желании; обязательная часть для этой фазы — noun-title карточек списка.

**Tabular figures (канон фазы 2):** `TextStyle(color: colors.onSurfaceVariant, fontFeatures: const [FontFeature.tabularFigures()])` [VERIFIED: dashboard_screen.dart:80-84 и :94-97 — точный образец для таймстампов записи].

**Поиск (TextField):** в lib/ нет ни одного TextField (grep → 0 хитов) — это первый TextField приложения. Рекомендация: `TextField(key: Key('historySearchField'))` с `onChanged → historyQueryProvider.set(...)`, префикс-иконкой `Icons.search` и hintText из .arb; Label для семантики (a11y-планка фазы 2 — фикс 3fcd6ac уже ввёл Semantics/liveRegion в lib/).

**Snackbar-паттерн (канон dashInjected):** `ScaffoldMessenger.of(context).showSnackBar(SnackBar(content: Text(l10n.…)))` [VERIFIED: last_text_card.dart:56-58 (dashCopied) и :78-81 (dashInjected)]. Async-вариант с mounted-гардом после await inject [VERIFIED: last_text_card.dart:73-81] — точный образец для «вставить заново» из истории: после `await …inject(entry.text)` проверить `context.mounted`, затем по `dictaControlProvider` решить, показывать ли «текст вставлен» (посылка успеха: `lastError == null && !genericFailure`).

**Кнопки:** доступность через те же условия, что в last_text_card: copy всегда доступен по клику; re-insert — `!busy` от `ref.watch(dictaControlProvider).commandInFlight` [VERIFIED: last_text_card.dart:52,68 — паттерн гейта]; delete — локален, без гейта. M3-варианты кнопок — как в фиксе 3fcd6ac (Elevated/FilledButton.tonal/Outlined по смыслу; delete — OutlinedButton с error-foreground по образцу cancel, dashboard_screen.dart:139-152).

**Врезка в shell:** слот 1 IndexedStack [VERIFIED: shell.dart:79-87, второй `_SectionPlaceholder()` на :84] → `const HistoryScreen()`. Заглушка остаётся на слотах 2-3 (models/settings — фазы 4+); ключ `historyTitle` уже существует [VERIFIED: app_ru.arb:12-15 «История»] — новый ключ заголовка не нужен. IndexedStack держит экран истории смонтированным всегда: список живёт и пополняется, даже когда пользователь на дашборде.

**ThemeExtension muted (CONTEXT lock 7):** сегодня muted-стиль `TextStyle(color: colors.onSurfaceVariant)` размножен копиями в 5 файлах [VERIFIED: shell.dart:108; dashboard_screen.dart:81,95; last_text_card.dart:22; diagnostics_card.dart:18; calibration_card.dart:152,167,173,180]. Вводить: `class AppTextStyles extends ThemeExtension<AppTextStyles>` (поле `muted`), регистрация в `ThemeData(…, extensions: [AppTextStyles.from(colors)])` в main.dart — обе темы, light и dark [VERIFIED: main.dart:41-43]. **Критичный подводный камень:** расширение обязано быть зарегистрировано и в `buildFakeApp` (MaterialApp в fake_dicta_engine.dart:145-150), иначе `Theme.of(context).extension<AppTextStyles>()` вернёт null во всех widget-тестах и экран истории упадёт/деградирует. Lookup — null-safe helper `AppTextStyles.mutedOf(context)` c явным fallback-стилем или `!` при гарантии регистрации в обоих местах. Разморозка копий — той же задачей, что ввод расширения (5 файлов выше), чтобы не оставить шестую копию в новом коде.

**L10n-ключи (~8-12 новых; CONTEXT оценил 15-25 — фактическая потребность ниже, т.к. кнопки/снеки переиспользуют dash*-канон):** новые `histEmpty` («журнал пуст»), `histEmptyHint` («записи появятся после диктовок через приложение — хоткей демона приложение не видит»), `histSearchHint` («поиск по тексту»), `histSearchNoMatches` («ничего не найдено»), `histDeleteButton` («удалить»), `histDeleted` («удалено»), `histSearchSemantics` (метка поля). Переиспользовать без дублей: `dashCopyButton`/`dashCopied` («скопировать»/«скопировано»), `dashInjectButton`/`dashInjected` («вставить заново»/«текст вставлен») — идентичные строки и семантика, дублировать .arb-ключи для той же строки хуже переиспользования.

### R4 — тест-паттерны

Каркас сценария (все прецеденты из last_text_test.dart):
- `buildFakeApp(fake, onCopy: …, captureContainer: …)` — единый скаффолд [VERIFIED: fake_dicta_engine.dart:130-154]; `captureContainer` (:132-133,144) даёт прямой доступ к контейнеру для assert'ов состояния `historyProvider`.
- `tick()` — pump 100 мс + кадр [VERIFIED: last_text_test.dart:11-14]; `captureTextViaStop()` — tap dashStopButton + два pump [VERIFIED: last_text_test.dart:19-23]; `enabled(key)` — проверка onPressed != null [VERIFIED: last_text_test.dart:16-17].
- Навигация на историю: `await tester.tap(find.text('История'))` (лейбл NavigationRail, shell.dart:63) + `tick(tester)` — до этого флайндеры вкладку не видят (см. R1).
- Кириллица в `find.text(...)` легальна: гейт сканирует только lib/ (cyrillic_gate.sh), тестовые файлы не затрагиваются [VERIFIED: прецедент last_text_test.dart:34-39].

Сценарии → грани:

| Сценарий | Скриптовка | Грань |
|---|---|---|
| «stop с текстом → запись появилась» | `fake.stopReply = DictaStatus(…, text: 'привет')` (паттерн last_text_test.dart:47-51) → captureTextViaStop → tap «История» → `find.text('привет')` в списке; либо без навигации: captureContainer → `container.read(historyProvider)` length 1, state.first.text == 'привет' | HIST-01 |
| «inject-эхо НЕ пишет запись» | после stop-capture tap dashInjectButton (дефолт injectReply эхом возвращает текст, fake_dicta_engine.dart:97-102) → `container.read(historyProvider).length` остался 1; `fake.injectTexts` — точный текст | HIST-01/HIST-03 |
| «stop с ошибкой/пустышкой — записи нет» | `stopReply` с `error: 'слишком тихо…'` / `text: ''` (паттерн last_text_test.dart:152-158) → length 0 | HIST-01, A3 |
| «порядок новые-сверху, id монотонный» | два captureTextViaStop с разными stopReply → state[0] новее, state[0].id > state[1].id (unit-уровень: без pump) | HIST-01, A1 |
| «поиск фильтрует» | capture → `tester.enterText(find.byKey(Key('historySearchField')), 'мир')` + pump → видна подходящая запись, `histSearchNoMatches` при кривом запросе; пустой запрос = все | HIST-02, A2 |
| «копия по клику» | `onCopy: copied.add` → tap «скопировать» у записи → `copied == [entry.text]`, snackbar «скопировано» | HIST-03 |
| «re-insert идёт существующим путём» | tap «вставить заново» → `fake.injectTexts` == [entry.text]; busy-гейт: кнопка disabled при `commandInFlight` (injectGate-паттерн last_text_test.dart:91-99,115-117) | HIST-03 |
| «удаление» | tap «удалить» (Key('histDeleteButton-N') или по ключу записи) → entry исчезла из state и UI, snackbar «удалено»; поиск+удаление сочетаются | HIST-03 |
| «форматтер времени» | unit: `formatEntryTime(DateTime(2026,1,1,9,5), 'ru') == '09:05'` и т.п. | HIST-02 |

Крюки фабрики, существующие и достаточные [VERIFIED: fake_dicta_engine.dart:16-27]: `stopReply` (:43), `injectReply` (:44), `stopGate`/`injectGate` (:34,37), `injectTexts` (:65), `setCalls`/`onSet` (:59,62), `statusError` (:54), `captureContainer` в buildFakeApp (:133). Для истории новых движковых крюков не требуется; `stopReply` — тот самый крюк («stopReply!» из вопроса R4).

### R5 — долг diag_smoke.dart:28

Подтверждено по строке [VERIFIED: app/tool/diag_smoke.dart:28 — `stderr.writeln('DIAG SMOKE FAIL $error')`, объект ошибки печатается целиком]. 02-SECURITY.md (Residual Risks, informational) рекомендует именно это: печатать `error.runtimeType`/фиксированную строку. Правка одной строки:

```dart
} catch (error) {
  // Type only: never print the error object (02-SECURITY residual).
  stderr.writeln('DIAG SMOKE FAIL ${error.runtimeType}');
  exitCode = 1;
}
```

Поведение smoke не меняется (исключения движка сегодня несут только тип transport-ошибки и путь сокета); закрывает единственный нефиксированный канал вывода. Smoke-прогон остаётся ручным backstop'ом (`dart run tool/diag_smoke.dart`, живой демон).

### R6 — intl/DateFormat для таймстампов

Факты зависимости:
- intl — прямая main-зависимость: pubspec.yaml:36 `intl: ^0.20.3`, lock «direct main» 0.20.3 [VERIFIED: pubspec.lock:163-170]. Generated-l10n уже импортирует intl [VERIFIED: app_localizations_ru.dart:2], т.е. DateFormat доступен без всяких правок манифеста (гейт pubspec не затронут — зависимость уже стоит).
- Локаль-данные дат грузятся автоматически: `AppLocalizations.localizationsDelegates` включает `GlobalMaterialLocalizations.delegate` [VERIFIED: app_localizations.dart:86-91], чей load вызывает `loadDateIntlDataIfNotLoaded()` [VERIFIED: SDK flutter_localizations/src/material_localizations.dart:735], регистрирующий date-символы ВСЕХ локалей, включая ru [VERIFIED: SDK flutter_localizations/src/utils/date_localizations.dart:18-33]. То же касается buildFakeApp — те же delegates (fake_dicta_engine.dart:146-147) → DateFormat работает и в widget-тестах без `initializeDateFormatting` руками.
- Русские форматы: TIMEFORMATS ru = `['HH:mm:ss zzzz', 'HH:mm:ss z', 'HH:mm:ss', 'HH:mm']` [VERIFIED: SDK generated_date_localizations.dart, 'ru' symbols (строка 8518)] → `DateFormat.Hm('ru')` = «09:05», `DateFormat.Hms('ru')` = «09:05:03», 24-часовые с нулями; datePatterns ru: `'Md': 'dd.MM'` и далее [VERIFIED: generated_date_localizations.dart:14392-14395].

Рекомендация: таймстамп записи — `DateFormat.Hm(localeName)` (минутной точности достаточно; журнал in-memory живёт одну сессию, дата внутри одной сессии почти всегда одна и та же — HIST-04). Если захочется дата для переходящих через полночь записей — `DateFormat.Md('ru')` → «22.11» рядом со временем. `localeName` берётся с существующего объекта локализации (`AppLocalizations.of(context)!.localeName` — поле уже канонизировано в generated-коде, app_localizations.dart:65). Форматтер — чистая функция (см. R2) для unit-теста. Tabular figures — на стиле текста, не на DateFormat (R3).

### Anti-Patterns to Avoid
- **Захват из status-потока** — текст туда не приходит [VERIFIED: main.rs:828-833]; любое чтение `status.text` как «последнего текста» — ошибка по построению (гейт код-ревью фазы 2: grep `status.text` по lib/features → 0).
- **Врезка в inject()** — дубликаты при HIST-03 re-insert (CONTEXT lock 2).
- **Забыть ThemeExtension в buildFakeApp** — экран истории упадёт в тестах при null-extension.
- **Мутировать список состояния** — только `[entry, ...state]` / `.where().toList()` (паттерн level_history.dart:14-17).
- **Параллельный путь вставки** — любой inject мимо `DictaControlNotifier.inject()` ломает commandInFlight-инвариант (CONTEXT lock 9).
- **Литеральные русские строки в lib/** — только .arb (гейт cyrillic_gate.sh → 0).

## Don't Hand-Roll

| Проблема | Не строить | Использовать | Почему |
|---------|-------------|-------------|--------|
| Русские строки | литералы/escapes | .arb + gen-l10n | гейт + стиль фаз 1-2 |
| Списочное состояние | свой ChangeNotifier | Riverpod Notifier (R2-скелет) | единый паттерн проекта |
| Формат времени | ручной padLeft для HH:mm | `DateFormat.Hm('ru')` (intl 0.20.3 уже стоит) | локаль-данные уже грузятся delegates'ами |
| Буфер обмена | каналы платформы | `clipboardWriterProvider` (существующий шов) | единственный call-site по клику — прецедент last_text_card.dart:55 |
| Список сотен записей | пагинация/virtualization-пакеты | `ListView.builder` (A1) | MVP-мастаб; без новых пакетов |
| Muted-стиль | шестая копия TextStyle | ThemeExtension + разморозка 5 копий | CONTEXT lock 7 |

## Common Pitfalls

### Pitfall 1: «Дубль текста» в существующих тестах
**Что не так:** после врезки текст записи существует на дашборде и в истории одновременно; `expect(find.text('привет'), findsOneWidget)` (last_text_test.dart:55) мог бы найти два виджета.
**Почему не происходит:** флайндеры дефолтно не заходят в невыбранные вкладки IndexedStack [VERIFIED: SDK indexed_stack.dart:196-203 + finders.dart:124,1207-1210]; батарея сидит на вкладке 0.
**Как сломать всё-таки:** `find.text(…, skipOffstage: false)` или навигация в тестах истории без pump. Предупреждение в план: в history-тестах навигация → pump → assert; в дашборд-тестах ничего не менять.

### Pitfall 2: ThemeExtension зарегистрирован только в main.dart
**Что не так:** `Theme.of(context).extension<AppTextStyles>()` → null во всех widget-тестах (buildFakeApp ставит голый MaterialApp, fake_dicta_engine.dart:145-150).
**Как избежать:** регистрировать расширение в ОБОИХ местах одной задачей; lookup через null-aware helper.

### Pitfall 3: inject-эхо дописано в историю
**Что не так:** re-insert из истории создаёт вторую запись того же текста → журнал заполняется дубликатами от HIST-03.
**Как избежать:** врезка только в stop(); тест «эхо не пишет запись» (R4) фиксирует контракт навсегда.

### Pitfall 4: поиск пересобирает весь список в onChanged посимвольно
**Что не так:** на сотнях записей filter на каждый ввод — дёшево, но onChanged может прилетать чаще кадра; дребезг не страшен, но нечего и усердствовать с debounce.
**Как избежать:** фильтр — чистая функция в build от `historyQueryProvider` (Riverpod сам пересобирает только при изменении стейта); без TextEditingController-двойного стейта.

### Pitfall 5: пустая запись от «тихого» stop
**Что не так:** демон отдаёт `error: "слишком тихо … — распознавание пропущено"` без текста; наивная врезка «на каждый успешный stop» запишет мусор или упадёт на null.
**Как избежать:** гард A3/A4 из R1 (text != null && error == null && trim().isNotEmpty) + тест «stop с ошибкой — записи нет».

### Pitfall 6: забыть, что заглушка placeholderScreen остаётся для моделей/настроек
**Что не так:** «оптимизация» — удалить ключ `placeholderScreen` из .arb сломает экраны фаз 4-5.
**Как избежать:** трогать только слот 1 IndexedStack (shell.dart:84); ключ `placeholderScreen` (app_ru.arb:28-31) остаётся в словаре.

## Code Examples

### Врезка захвата (полный диф-скелет stop())
```dart
// Source: живой код dicta_control.dart:87-106 + врезка фазы 3 (позиция после :98)
Future<void> stop() async {
  state = state.copyWith(commandInFlight: true, clearLastError: true);
  try {
    final snapshot = await ref.read(engineProvider).stop();
    state = state.copyWith(
      commandInFlight: false,
      clearLastText: snapshot.text == null,
      lastText: snapshot.text,
      clearLastError: snapshot.error == null,
      lastError: snapshot.error,
      lastErrorCommand: DictaCommand.stop,
    );
    // Phase 3: journal only dictations made through the app. The stop reply
    // is the sole honest capture point; empty/whitespace text never lands.
    if (snapshot.error == null &&
        snapshot.text != null &&
        snapshot.text!.trim().isNotEmpty) {
      ref.read(historyProvider.notifier).add(snapshot.text!);
    }
  } catch (_) { /* существующий путь genericFailure не меняется */ }
}
```

### Re-insert из истории (mounted-гард по канону last_text_card.dart:66-82)
```dart
onPressed: busy ? null : () async {
  await ref.read(dictaControlProvider.notifier).inject(entry.text);
  if (!context.mounted) return;
  final result = ref.read(dictaControlProvider);
  if (result.lastError == null && !result.genericFailure) {
    ScaffoldMessenger.of(context).showSnackBar(
      SnackBar(content: Text(l10n.dashInjected)),
    );
  }
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| muted-стиль ×5 копиями | ThemeExtension (фаза 3 вводит) | UI-ревью фазы 2 (17/24) | один источник второстепенного стиля |
| заглушка «История» в shell | HistoryScreen на слоте 1 | эта фаза | rail-лейбл и `historyTitle` уже существуют — ничего renaming-ового |

**Deprecated/outdated:** формулировка HIST-01 в REQUIREMENTS.md («из status-потока») — технически неверна, заменяется A4-поправкой CONTEXT (план опирается на CONTEXT, REQUIREMENTS не правим — запрет).

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | Переиспользование dashCopyButton/dashCopied/dashInjectButton/dashInjected на экране истории (вместо hist*-дублей ключей) | R3, l10n | Нет — строки идентичны, семантика та же; планёр может предпочесть hist*-префикс ценой +4 ключей |
| A2 | `DateFormat.Hm` (минуты) достаточно; дата не нужна в MVP-таймстампе | R6 | Нет — при желании рядом добавляется Md('ru')='dd.MM' одной строкой |
| A3 | id, выводимый из `state.first.id + 1`, достаточен (без отдельного счётчика-поля) | R2 | Нет — монотонность в пределах жизни провайдера гарантирована головой списка |
| A4 | Пустой `stopReply`-дефолт фейка не создаёт записей — новые кнопки на дашборде не «засоряют» старые сценарии | R1 | Нет — guard A3 отсекает и пустые дефолты |

A1-A4 из CONTEXT.md не пересуживаются (разрешены оркестратору автономно); таблица выше — только НОВЫЕ дополнения исследования.

## Open Questions

1. **Дебаунс поиска не нужен?** Знаем: фильтр по сотням записей дешёвый (A1). Неясно: эстетика «на каждый keystroke». Рекомендация: без дебаунса (KISS), Riverpod сам сжимает rebuild'ы.
2. **Место ThemeExtension-файла: core/ или features/?** Знаем: стиль跨фичный (5 файлов по всем features + shell). Рекомендация: `app/lib/core/theme_ext.dart` — расширение темы не принадлежит одной фиче.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Flutter SDK | всё | ✓ | 3.50.1 (проверено `flutter --version` в этой сессии) | — |
| flutter_riverpod | состояние | ✓ | 3.5.0 (pubspec.yaml:29) | — |
| intl | таймстампы | ✓ | 0.20.3 (pubspec.lock:163-170) | ручной padLeft (dashboard_screen.dart:189-194) |
| dictad (живой демон) | smoke/UAT | ✓ | сокет `/run/user/1000/dicta.sock` присутствует (проверено в этой сессии) | тесты на фейках |
| Новые пакеты | — | не требуются | — | — |

**Missing dependencies with no fallback:** нет.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | flutter_test (SDK, Flutter 3.50.1 — проверено в этой сессии) |
| Config file | `app/analysis_options.yaml` (flutter_lints 6, с фазы 1) |
| Quick run command | `cd app && flutter test` |
| Full suite command | `cd app && flutter test && flutter analyze && ./tool/cyrillic_gate.sh` |

Батарея сегодня: **51/51** (пересчитано в этой сессии: 8+10+5+4+4+5+5+2+5+2+1 по файлам). Прогноз фазы: +13-16 тестов → ~64-67.

### Phase Requirements → Test Map
| Req ID | Behavior (грань) | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| HIST-01 | stop-ответ с текстом → запись в store (capture через captureContainer) | widget | `flutter test test/history_screen_test.dart -x --plain-name 'stop'` | ❌ Wave 0 |
| HIST-01 | inject-эхо НЕ пишет запись (после re-insert length неизменен) | widget | там же | ❌ Wave 0 |
| HIST-01 | stop с error/пустым текстом → записи нет (A3) | widget | там же | ❌ Wave 0 |
| HIST-01 | порядок новые-сверху, id монотонный, remove по id | unit | `flutter test test/history_store_test.dart -x` | ❌ Wave 0 |
| HIST-02 | экран списка: запись видна на вкладке «История» после навигации | widget | `flutter test test/history_screen_test.dart -x --plain-name 'renders'` | ❌ Wave 0 |
| HIST-02 | поиск фильтрует (enterText), пустой = все, no-matches-строка | widget | там же | ❌ Wave 0 |
| HIST-02 | форматтер времени: `formatEntryTime(fixed DateTime, 'ru')` | unit | `flutter test test/history_store_test.dart -x --plain-name 'format'` | ❌ Wave 0 |
| HIST-03 | копия через onCopy-шов + snackbar | widget | history_screen_test.dart | ❌ Wave 0 |
| HIST-03 | re-insert через `dictaControlProvider.inject` (`fake.injectTexts`), busy-гейт (injectGate) | widget | там же | ❌ Wave 0 |
| HIST-03 | удаление: entry исчезает (state + UI), snackbar «удалено» | widget | там же | ❌ Wave 0 |
| HIST-04 | обратный гейт: ноль persistence-поверхностей (grep SharedPreferences/sqflite/File write → 0 в lib/features/history) | static (гейт-скрипт в задаче docs) | в задаче HIST-04 docs | ❌ (docs-задача) |
| долг-02 | diag_smoke печатает только runtimeType | review (одна строка) + ручной smoke | `dart run tool/diag_smoke.dart` | файл есть, правка ❌ Wave 0 |
| ThemeExt | muted-стиль из extension в app + тестах; 5 копий разморожены | widget + analyze | `flutter analyze` + существующие сьюты остаются зелёными | ❌ Wave 0 |

### Sampling Rate
- **Per task commit:** `cd app && flutter test && ./tool/cyrillic_gate.sh`
- **Per wave merge:** full suite + `flutter analyze`
- **Phase gate:** full suite + контракты (`git diff --quiet <base> HEAD -- crates/ app/pubspec.yaml app/pubspec.lock`, гейт `implements DictaEngine == 1`) + UAT ниже.

### Wave-структура (рекомендация планёру)
- **Wave 0 (фундамент, без UI):** .arb-ключи + `flutter gen-l10n`; `history_entry.dart` + `history_store.dart` + unit-тесты; ThemeExtension + регистрация в main.dart И buildFakeApp + разморозка 5 копий + analyze/батарея зелёные; правка diag_smoke.dart:28.
- **Wave 1 (шов + экран):** врезка в stop() + `history_screen.dart` (поиск/список/действия) + слот 1 shell.dart + все widget-сценарии таблицы выше.
- Зависимости: Wave 1 строго после Wave 0 (ключи/провайдеры/extension); врезка stop() и экран — одна задача или соседние в одной волне (тест «stop → запись появилась» требует обоих).

### Manual UAT (backstop, живьём — глазами)
- Живая диктовка через приложение: Запись → Стоп → вкладка «История»: запись с реальным текстом и корректным временем (HH:mm).
- Хоткейная диктовка демона НЕ появляется в журнале — честное ограничение A4 наблюдаемо.
- Re-insert записи в реальное окно (фокус-семантика dashInjectHint) и копирование в буфер.
- Удаление и поиск на живом скролле; «распознаю…» → запись появляется только по завершении.

### Wave 0 Gaps
- [ ] `app/test/history_store_test.dart` — unit: HIST-01 (порядок/id), HIST-02 (форматтер, фильтр)
- [ ] `app/test/history_screen_test.dart` — widget: захват/эхо/поиск/действия (HIST-01..03)
- [ ] Новые ключи .arb (~8-12) + `flutter gen-l10n` — до первого виджета Wave 1
- [ ] `app/lib/core/theme_ext.dart` + регистрация в main.dart и fake_dicta_engine.dart (buildFakeApp)
- [ ] Правка diag_smoke.dart:28 (долг 02-SECURITY)

## Security Domain

ASVS-контекст фазы: app-only, протокол заморожен, новых зависимостей/файлов/env — ноль.

### Applicable ASVS Categories
| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no | same-user сокет, вне скоупа |
| V3 Session Management | no | — |
| V4 Access Control | no | локальный UI, новых привилегий нет |
| V5 Input Validation | yes | поиск — локальный substring (A2), без exec/инъекций; inject-текст из записи идёт демону, валидация остаётся на демоне (T-2-03 фазы 2: клиент НЕ перевалидирует) |
| V6 Cryptography | no | — |

### Known Threat Patterns (Flutter/dicta-стек, продолжение 02-SECURITY)
| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Утечка распознанного текста через вывод/логи | Info disclosure | новый код истории не печатает payload (grep-дисциплина фазы 2); долг diag_smoke.dart:28 → `error.runtimeType` гасит последний нефиксированный канал — задача этой фазы |
| Тамперинг через re-insert изменённого текста | Tampering | единственный путь — `DictaControlNotifier.inject()` (commandInFlight), демон валидирует ≤5000/пустоту (main.rs:886) |
| Персистентность секретных диктовок на диск | Info disclosure | запрет CONTEXT: in-memory only (HIST-04); гейт-проверка grep File/SharedPreferences/sqflite → 0 в features/history |
| Supply chain | Tampering | pubspec.yaml не трогается (байт-в-байт к базе); новых пакетов ноль |

## Sources

### Primary (HIGH confidence, прочитано в этой сессии)
- `app/lib/features/dashboard/dicta_control.dart` — stop :87-106 (врезка), inject :129-148, copyWith :39-59, clipboardWriterProvider :155-157
- `app/lib/features/shell/shell.dart` — IndexedStack :79-87, слот 1 :84, rail-лейбл «История» :63, muted-копия :108
- `app/lib/core/{providers,engine}.dart`, `app/lib/features/dashboard/{level_history,last_text_card,dashboard_screen,calibration_card,diagnostics_card}.dart`, `app/lib/main.dart` :41-43, `app/lib/l10n/app_ru.arb` (historyTitle :12, placeholderScreen :28)
- `app/test/{fake_dicta_engine,last_text_test,shell_test,fake_dicta_engine_test}.dart` — крюки, скаффолд, паттерны батареи; 51 тест пересчитан
- `crates/dictad/src/main.rs` — status :828-833, run_stop :1030-1038 (A4 перепроверена)
- Flutter SDK 3.50.1 (локально): `indexed_stack.dart:196-203`, `finders.dart:124,1207-1210`, `material_localizations.dart:735`, `utils/date_localizations.dart:18-33`, `generated_date_localizations.dart` (ru TIMEFORMATS / patterns :14392)
- `app/pubspec.yaml:29,36`, `app/pubspec.lock:163-170` (intl 0.20.3 direct)
- `.planning/phases/02-*/02-RESEARCH.md`, `02-SECURITY.md` (residual diag_smoke), `02-UI-REVIEW.md` (muted ×5, header-паттерн)

### Metadata
**Confidence breakdown:** механика захвата/фейков — HIGH (A4 перепроверена чтением main.rs; все швы прочитаны с file:line); безопасность батареи (IndexedStack/offstage) — HIGH (SDK-исходник); intl/локаль — HIGH (SDK generated-данные ru прочитаны); UI-детали экрана — MEDIUM (рекомендации поверх канона; финальная полировка — UAT глазами).
**Research date:** 2026-11-22. **Valid until:** протокол заморожен на всю веху; устаревание — только от чужих правок app/lib, которых фаза сама и делает.

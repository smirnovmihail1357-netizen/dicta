---
phase: 03-istoriya-diktovok
reviewed: 2026-11-22T12:46:29Z
depth: standard
files_reviewed: 21
files_reviewed_list:
  - app/lib/features/history/history.dart
  - app/lib/features/history/history_screen.dart
  - app/lib/core/theme_ext.dart
  - app/lib/features/dashboard/dicta_control.dart
  - app/lib/features/dashboard/last_text_card.dart
  - app/lib/features/dashboard/diagnostics_card.dart
  - app/lib/features/dashboard/calibration_card.dart
  - app/lib/features/dashboard/dashboard_screen.dart
  - app/lib/features/shell/shell.dart
  - app/lib/main.dart
  - app/lib/l10n/app_ru.arb
  - app/lib/l10n/generated/app_localizations.dart
  - app/lib/l10n/generated/app_localizations_ru.dart
  - app/tool/diag_smoke.dart
  - app/test/fake_dicta_engine.dart
  - app/test/history_store_test.dart
  - app/test/history_capture_test.dart
  - app/test/history_screen_test.dart
  - app/test/shell_test.dart
  - README.md
  - README.ru.md
findings:
  critical: 0
  warning: 1
  info: 4
  total: 5
  necessity: 0
status: issues_found
---

# Phase 3: Code Review Report

**Reviewed:** 2026-11-22T12:46:29Z
**Depth:** standard (дифф 305ed18..HEAD: de9e94c, 37c1f3b, 7a4fb90, 9ce9071)
**Files Reviewed:** 21 (все исходники фазы; .planning-артефакты исключены)
**Status:** issues_found — **0 blocker, 1 warning, 4 info**

## Summary

Ревью живого кода диффа фазы 3 (store истории, врезка stop-шва, экран истории,
ThemeExtension, l10n, тесты, README) против locked-контракта CONTEXT.md и
03-01-PLAN.md. Контракт фазы исполнен точно: захват — только в шве
`DictaControlNotifier.stop()` (dicta_control.dart:103-107) с полным гардом
`error == null && text != null && trim().isNotEmpty`; inject-эхо не пишется
(в `inject()` :138-157 нет ни одного обращения к historyProvider — grep даёт
единственную ссылку на :106 внутри `stop()`); catch-путь не журналирует
(исключение = нет ответа = нет записи); порядок новые-сверху и head-derived id
по формуле плана. Riverpod-дисциплина чистая: `ref.read` соседних нотифаеров,
копия состояния вместо мутации (`[entry, ...state]`, `where+toList`),
`if (!context.mounted) return;` после await в inject-обработчике экрана
(history_screen.dart:135). Security-тезисы держатся: payload нигде не
логируется (grep print/debugPrint/log по новой коде — пусто), diag_smoke
печатает только `error.runtimeType` (долг 02-SECURITY погашен), clipboard —
только по клику через `clipboardWriterProvider`, ноль persistence-API,
`status.text` не читается, контракты crates/pubspec байт-в-байт к базе фазы.

Claims 03-SUMMARY.md перепроверены живым прогоном и не завышены: батарея
действительно 65/65, analyze «No issues found!», cyrillic 0, implements==1,
mutedOf-кол-во 12 (9 размороженных + 3 на экране), weak-копий 0, все 4
README-якоря == 1 и дословны.

Нecessity-оценка выполнялась по рубрике necessity-review.md; новых
necessity-находок нет: `catch (_)`-блоки уже диспозицированы в 02-REVIEW
(IN-03), а assert+bang в `mutedOf` — fail-fast на границе регистрации, не
невозможное состояние.

Слабые места — один a11y-дефект обёртки Semantics у поля поиска (WR-01) и
честные дыры покрытия (IN-01..04). Ни один не блокирует shipping фазы.

## Проверенные гейты (живой прогон 2026-11-22)

- `cd app && flutter test` — **65/65 passed** ✓
- `flutter analyze` — **No issues found!** ✓
- `bash tool/cyrillic_gate.sh` — **0** ✓
- `grep -rn "implements DictaEngine" test/*.dart | wc -l` — **1** ✓
- `grep -rn "SharedPreferences|sqflite|writeAsString" lib` — **0** ✓
- `grep -rn "status\.text" lib` — **0** ✓
- `grep -rn "mutedOf(context)" lib | wc -l` — **12**; weak-копий — **0** ✓
- `git diff --quiet <phase-base> HEAD -- crates/ app/pubspec.yaml app/pubspec.lock` — **пусто** ✓
- `dart run tool/diag_smoke.dart` — `state=idle connected=true` + `INJECT-REJECTED-AS-EXPECTED`, **exit 0** (живой демон) ✓
- README-якоря (`cleared on app restart`, `never enter the journal`, `очищаются при перезапуске`, `глобальным хоткеем демона`) — каждый **== 1** ✓
- `grep -rn "historyProvider" lib/features/dashboard/dicta_control.dart` — **одна** ссылка, :106, внутри `stop()` ✓

## Warnings

### WR-01: Semantics-обёртка поля поиска не связывает метку с самим TextField — метка повисает отдельным a11y-узлом

**File:** `app/lib/features/history/history_screen.dart:30-44`
**Issue:** Поле поиска обёрнуто в голый `Semantics(label: l10n.histSearchSemantics,
child: TextField(...))` без `MergeSemantics`/`excludeSemantics`. Виджет
`Semantics` создаёт в semantics-дереве отдельный узел с меткой, а `TextField`
(EditableText) остаётся независимым дочерним узлом со своей семантикой
(editable text + hint «поиск по тексту»). Итог для screen reader: **два фокуса
на одном поле** — сначала узел «поиск по журналу» без роли текстового поля,
затем сам edit box; mandated-метка `histSearchSemantics` никогда не доходит до
семантического узла редактируемого поля. Это дефект accessibility-планки,
которую план обязывал ввести именно для этого поля («семантика поля через
l10n.histSearchSemantics», Task 3 шаг 1). Тест на метку не написан (см. IN-02),
поэтому дефект не пойман батареей.
**Fix:** Сгруппировать метку с полем через merge, чтобы узел был один:

```dart
MergeSemantics(
  child: Semantics(
    label: l10n.histSearchSemantics,
    child: TextField(key: const Key('historySearchField'), ...),
  ),
)
```

— `MergeSemantics` склеивает конфигурации в один узел (метка + editable), один
фокус-стоп. Альтернатива — убрать обёртку и положиться на `InputDecoration`
(labelText/hint уже локализованы), но тогда ключ `histSearchSemantics`
останется мёртвым. Проверить дампом: `tester.binding`/semantics tree или
`SemanticsDebugger`.

## Info

### IN-01: Контракт «inject-эхо не пишет запись» протестирован только для кнопки дашборда — путь re-insert из истории не ассертится

**File:** `app/test/history_capture_test.dart:49-73`, `app/test/history_screen_test.dart:127-156`
**Issue:** CONTEXT №2 объявляет контракт «навсегда»: эхо inject не создаёт
диктовку. Тест покрывает его через `dashInjectButton` (capture-сьют :65), но
сценарий re-insert из истории (`histInjectButton-1`, screen-сьют :145)
проверяет только `injectTexts`, busy-гейт и снек — длину журнала после эха не
ассертит. Структурно путь безопасен (в `inject()` нет `add`, единственная ссылка
на historyProvider — :106 в `stop()`), но контракт «навсегда» держится на grep,
а не на втором тестовом якоре; регрессия в `inject()` сломала бы его молча.
**Fix:** В сценарий re-insert добавить одну строку после `gate.complete()`:
`expect(container.read(historyProvider), hasLength(1));` (потребуется
`captureContainer` в buildFakeApp этого сценария).

### IN-02: Ветка `if (!context.mounted) return;` и семантика поиска не покрыты ни одним тестом

**File:** `app/lib/features/history/history_screen.dart:135`, `app/test/history_screen_test.dart` (весь файл)
**Issue:** Два поведенческих элемента экрана не имеют тестового якоря:
(а) mounted-гард после `await ref.read(...).inject(...)` — размонтирование
экрана во время in-flight inject (сценарий: tap → переключение вкладки до
`gate.complete()`) ни разу не проигран; (б) метка `histSearchSemantics`
(и корректность Semantics-обёртки, см. WR-01) не ассертится. Оба элемента —
именно те места, где регрессия пройдёт батарею незамеченной.
**Fix:** Один сценарий: `injectGate`-Completer → tap по `histInjectButton-1`
→ не дожидаясь завершения, tap `find.text('Дашборд')` → `gate.complete()` →
pump → ожидать отсутствие исключений и отсутствие снека. Для семантики —
`expect(find.bySemanticsLabel('поиск по журналу'), findsOneWidget)` после
исправления WR-01.

### IN-03: Комментарий theme_ext.dart называет две точки регистрации, а их уже три — doc-дрейф после девиации Task 1

**File:** `app/lib/core/theme_ext.dart:3-4`
**Issue:** Комментарий: «must be registered in EVERY ThemeData (main.dart and
the test scaffold in fake_dicta_engine.dart)». Фактически точек три:
main.dart (обе темы), `buildFakeApp` и — добавленная по девиации Rule 1
(03-SUMMARY.md Deviations 1) — третий скаффолд `shell_test.dart:buildTestApp`
(shell_test.dart:15-25). Следующий скаффолд (фаза 4/5 добавит свой) напишут по
комментарию и пропустят регистрацию — узнают об этом только по assert-крашу
`mutedOf` в debug (в release это null-bang краш). Комментарий вводит в ошибку
относительно реального инварианта.
**Fix:** Поправить комментарий на перечисление фактических точек: «main.dart
(both themes), buildFakeApp, and any suite-local scaffold (see
shell_test.dart)» — либо ссылку на grep-гейт: `grep -rn
"AppTextStyles.from" lib test`.

### IN-04: head-derived id переиспользует значения после удалений — свойство не задокументировано и не протестировано

**File:** `app/lib/features/history/history.dart:32`, `app/test/history_store_test.dart:35-59`
**Issue:** Формула `state.first.id + 1` (плюс `isEmpty ? 1`) переиспользует id
после удаления головы: add(1), add(2) → remove(2) → add → снова id 2; полное
очищение → ids снова с 1. Сегодня это безопасно (в живом списке дубликатов нет,
ключи `hist*Button-${id}` уникальны в кадре, A1 требует лишь стабильности
внутри списка), но store-тесты покрывают remove только без последующего add —
свойство «id не монотонен сквозь удаления» нигде не зафиксировано. Если фаза 4+
ключуется на id сквозь очистки (undo, выбор), это выстрелит.
**Fix:** Минимум — тест-документация в history_store_test.dart
(`remove(2); add('x'); expect(entries.first.id, 2)`) и строка в doc-замке
history.dart: «ids are unique within the live list only; they are reused
after removals (A1)». Кода менять не требуется.

---

_Reviewed: 2026-11-22T12:46:29Z_
_Reviewer: the agent (gsd-code-reviewer)_
_Depth: standard_
_Verified live: flutter test 65/65, analyze 0, cyrillic 0, contracts byte-identical, diag_smoke exit 0 на живом демоне._

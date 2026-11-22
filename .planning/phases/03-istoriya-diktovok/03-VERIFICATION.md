---
phase: 03-istoriya-diktovok
verified: 2026-11-22T14:10:00Z
status: human_needed
score: 13/13 must-haves verified
covered_files:
  - .planning/phases/03-istoriya-diktovok/03-01-PLAN.md
  - .planning/phases/03-istoriya-diktovok/03-SUMMARY.md
  - README.md
  - README.ru.md
  - app/lib/core/theme_ext.dart
  - app/lib/features/dashboard/calibration_card.dart
  - app/lib/features/dashboard/dashboard_screen.dart
  - app/lib/features/dashboard/diagnostics_card.dart
  - app/lib/features/dashboard/dicta_control.dart
  - app/lib/features/dashboard/last_text_card.dart
  - app/lib/features/history/history.dart
  - app/lib/features/history/history_screen.dart
  - app/lib/features/shell/shell.dart
  - app/lib/l10n/app_ru.arb
  - app/lib/l10n/generated/app_localizations.dart
  - app/lib/l10n/generated/app_localizations_ru.dart
  - app/lib/main.dart
  - app/test/fake_dicta_engine.dart
  - app/test/history_capture_test.dart
  - app/test/history_screen_test.dart
  - app/test/history_store_test.dart
  - app/test/shell_test.dart
  - app/tool/diag_smoke.dart
covered_digest: "v4:sha256:6985d61929fe17e26cf7f81b44f9bb9ce5d37601a640afdfa1e7f1f6264a7a7b"
behavior_unverified: 0
overrides_applied: 0
re_verification:
  previous_status: none
  previous_score: none
  gaps_closed: []
  gaps_remaining: []
  regressions: []
human_verification:
  - test: "UAT-3-1: живая диктовка через приложение (Запись→Стоп) → запись в «Истории» с HH:mm; перезапуск приложения → история пуста"
    expected: "Продиктованный через кнопки приложения текст появился в журнале; после рестарта приложения журнал пуст (честный MVP)"
    why_human: "Требует живого микрофона, живого демона и человеческих глаз; widget-тесты доказали механику только на фейке"
  - test: "UAT-3-2: диктовка ГЛОБАЛЬНЫМ хоткеем демона при открытом приложении → в журнал НЕ попала"
    expected: "Запись не появилась; подписать принятие задокументированного ограничения A4 (README: never enter the journal)"
    why_human: "Наблюдаемое поведение живого демона + юридически значимое принятие ограничения (не дефект)"
  - test: "UAT-3-3: поиск/копия/re-insert/удаление глазами на живом журнале"
    expected: "Поиск регистронезависим; копия в буфере; вставка заново работает; удаление убирает запись, undo возвращает"
    why_human: "Живое UX-качество на реальных данных глазами пользователя"
---

# Phase 3: История диктовок — Verification Report

**Phase Goal (ROADMAP):** Журнал распознаваний — центральная ценность окна: диктовки не теряются, их можно найти и использовать повторно
**Verified:** 2026-11-22T14:10:00Z
**Status:** human_needed
**Re-verification:** No — initial verification

**Метод:** goal-backward. HIST-01 сверен с A4-поправкой CONTEXT.md
(честный механизм: захват только в шве `DictaControlNotifier.stop()`), а не с буквой
REQUIREMENTS («из status-потока» — формулировка, написанная вопреки протоколу,
A4 это фиксирует). Диктовки глобального хоткея демона невидимы — документированное
ограничение, не дефект.

## Goal Achievement

### Observable Truths

Roadmap Success Criteria + must_haves плана (SC-4 и truth-8 дедуплицированы — оба об README-фиксации ограничений):

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | SC-1: каждая завершённая диктовка через приложение автоматически появляется в журнале; протокол не изменился | ✓ VERIFIED | `flutter test` 71/71: capture-сцена́рии «a stop reply with text adds exactly one journal entry», порядок/id; `dicta_control.dart:103-107` — врезка в `stop()` под гардом; `git diff --quiet 305ed18 HEAD -- crates/ app/pubspec.yaml app/pubspec.lock` → пусто (exit 0) |
| 2 | SC-2: журнал листается и находится поиском по тексту | ✓ VERIFIED | `history_screen.dart:23-25,82-86` (ListView.builder, новые сверху) + тест «search is case-insensitive substring, no debounce, empty = all» зелёный |
| 3 | SC-3: копия / re-insert (`inject`) / удаление | ✓ VERIFIED | `history_screen.dart:110-123` (clipboardWriterProvider, явный клик), `:130-142` (единственный путь `dictaControlProvider.inject()` под `commandInFlight`), `:199-229` (локальный remove + undo); тесты copy/re-insert-gated/delete/undo зелёные |
| 4 | SC-4/truth-8: ОБА ограничения MVP зафиксированы в README.md и README.ru.md (память + невидимость хоткейных диктовок A4) | ✓ VERIFIED | 4 якоря grep == 1: `cleared on app restart`, `never enter the journal`, «очищаются при перезапуске», «глобальным хоткеем демона» |
| 5 | truth-1: захват ТОЛЬКО в шве stop(), гард `error==null && text!=null && trim().isNotEmpty`; status-поток не читается как источник текста (A4) | ✓ VERIFIED | `grep historyProvider dicta_control.dart` → ровно 1 ссылка (`:106`), внутри `stop()`; `grep "status\.text" lib` → 0 |
| 6 | truth-2: inject-эхо НЕ пишет запись — контракт зафиксирован тестом | ✓ VERIFIED | `inject()` (`:138-157`) не содержит history-врезки; тест «inject echo does not create a second entry» зелёный (`fake.injectTexts == ['привет']`, длина остаётся 1) |
| 7 | truth-3: каждая диктовка с HH:mm-таймстампом (tabular figures), новые сверху, пустые не пишутся | ✓ VERIFIED | `formatEntryTime` = `DateFormat.Hm` (history.dart:72-73); `FontFeature.tabularFigures()` в экране (`:96-101`); head-derived id, newest-first (`history.dart:31-40`); тесты форматтера 09:05/23:59 + whitespace-сцена́рий зелёные |
| 8 | truth-4: поиск case-insensitive substring без дебаунса; пустой запрос = все записи | ✓ VERIFIED | `history_screen.dart:23-25` (toLowerCase().contains) + `:45-46` (onChanged → historyQueryProvider, без дебаунса/контроллера); тест зелёный |
| 9 | truth-5: копия — только явный клик через clipboardWriterProvider; re-insert — только inject() под commandInFlight; удаление — локальный remove | ✓ VERIFIED | Код-аудит экрана (см. truth-3): `onPressed` копии всегда активен, вставка гейтится `busy`, удаление «Local removal only: no daemon command»; тесты «re-insert goes through inject and is gated while in flight», «delete removes the entry locally» зелёные |
| 10 | truth-6: ThemeExtension AppTextStyles зарегистрирован в ОБОИХ точках (main.dart обе темы И buildFakeApp); muted-стиль — единственный источник | ✓ VERIFIED | `main.dart:49,54` + `fake_dicta_engine.dart:156` (+ `shell_test.dart:23` — третий scaffold, отклонение №1 из SUMMARY, легитимный Rule-1 фикс); `mutedOf(context)` 12 использований, weak-копий 0 |
| 11 | truth-7: historyTitle не дублируется; кнопки/снеки переиспользуют dash*-канон | ✓ VERIFIED | `grep -c '"historyTitle"' app_ru.arb` == 1; экран использует `dashCopyButton/dashCopied/dashInjectButton/dashInjected/dashInjectFailed`; hist*-ключей 8 (7 плановых + `histUndo` из UI-аудит-фикса eb7fd7b — не дубль dash-строки, у dash нет undo-лейбла) |
| 12 | truth-9: протокол/демон/pubspec/systemd/adapters не изменились — байт-в-байт к базе фазы 305ed18 | ✓ VERIFIED | `git diff --quiet 305ed18 HEAD -- crates/ app/pubspec.yaml app/pubspec.lock` → exit 0, пусто (живой прогон) |

**Score:** 13/13 truths verified (0 present, behavior-unverified)

Все behavior-dependent истины (захват, эхо, порядок, поиск, гейты) подтверждены
**проходящими тестами** (батарея 71/71, прогнана целиком один раз), не только присутствием символов.

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `app/lib/features/history/history.dart` | store + форматтер + провайдеры | ✓ VERIFIED | 73 строк, immutable-модель, head-id, doc-замок «deliberately not persisted» |
| `app/lib/features/history/history_screen.dart` | экран листания/поиска/действий | ✓ VERIFIED | 243 строки, полный экран, все швы |
| `app/lib/core/theme_ext.dart` | ThemeExtension muted | ✓ VERIFIED | зарегистрирован в 3 scaffolds (main ×2 темы, fake, shell_test) |
| `app/test/history_store_test.dart` | unit-батарея store | ✓ VERIFIED | зелёный в батарее |
| `app/test/history_capture_test.dart` | 4 сценария захвата | ✓ VERIFIED | захват/эхо-нет/пустышка-нет/порядок — все зелёные |
| `app/test/history_screen_test.dart` | widget-сценарии HIST-02/03 | ✓ VERIFIED | 17+ сценариев, включая undo и a11y — зелёные |
| `.arb` hist*-ключи + generated | 7 плановых ключей | ✓ VERIFIED | 8 ключей (7 + histUndo из UI-фикса); generated перегенерирован |
| README-секции (en+ru) | оба ограничения MVP | ✓ VERIFIED | 4 якоря == 1 |
| `app/tool/diag_smoke.dart:28` | `error.runtimeType` (долг 02-SECURITY) | ✓ VERIFIED | `:29` — только runtimeType + комментарий-замок |
| `phase-base.txt` | хеш базы фазы | ✓ VERIFIED | `305ed18c...` — совпадает с git |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|----|--------|---------|
| dicta_control.dart | history.dart | `ref.read(historyProvider.notifier).add(...)` внутри stop() | ✓ WIRED | `:106`, единственная ссылка, под гардом |
| shell.dart | history_screen.dart | `const HistoryScreen()` слот 1 IndexedStack | ✓ WIRED | `:85`; слоты 2-3 — `_SectionPlaceholder` на месте (фазы 4-5) |
| history_screen.dart | dicta_control.dart | `inject()` под commandInFlight | ✓ WIRED | `:134-142`, busy-гейт |
| history_screen.dart | dicta_control.dart | `clipboardWriterProvider` | ✓ WIRED | `:113`, явный клик |
| main.dart + fake_dicta_engine.dart | theme_ext.dart | `extensions: [AppTextStyles.from(...)]` | ✓ WIRED | main.dart:49,54; fake:156; shell_test:23 |
| history.dart | intl DateFormat.Hm | `formatEntryTime` | ✓ WIRED | `:72-73`, тесты 09:05/23:59 зелёные |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
|----------|---------------|--------|--------------------|--------|
| history_screen.dart | `entries` | `ref.watch(historyProvider)` ← шов stop() ← реальный ответ демона | Да (реплика stop с text) | ✓ FLOWING |
| history_screen.dart | `query` | `ref.watch(historyQueryProvider)` ← onChanged TextField | Да | ✓ FLOWING |
| history_screen.dart | таймстампы | `formatEntryTime(entry.at)` ← DateTime.now() при add() | Да | ✓ FLOWING |

Статических заглушек нет: пустые состояния (`histEmpty`, `histSearchNoMatches`) — честные empty-states, не фейковые данные.

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| Полная батарея | `cd app && flutter test` | **71/71 passed, All tests passed!** (51 унаследованных + unit store + 4 capture + widget экрана, вкл. undo/a11y из ревью-фиксов) | ✓ PASS |
| Статический анализ | `cd app && flutter analyze` | **No issues found!** | ✓ PASS |
| Кириллица в lib вне generated | `bash tool/cyrillic_gate.sh` | **0** (exit 0) | ✓ PASS |
| Живой диагностический смоук | `cd app && dart run tool/diag_smoke.dart` | `DIAG SMOKE state=idle connected=true` + `INJECT-REJECTED-AS-EXPECTED`, **exit 0** (живой демон, read-only) | ✓ PASS |
| Контракт заморозки | `git diff --quiet 305ed18 HEAD -- crates/ app/pubspec.yaml app/pubspec.lock` | пусто, exit 0 | ✓ PASS |
| Фабрика фейков | `grep -rn "implements DictaEngine" test/*.dart \| wc -l` | **1** | ✓ PASS |
| Анти-персистентность | `grep -rn "SharedPreferences\|sqflite\|writeAsString\|File(" lib` | **0** | ✓ PASS |
| Анти-A4 (status.text) | `grep -rn "status\.text" lib` | **0** | ✓ PASS |
| README-якоря HIST-04/A4 | grep × 4 (en+ru) | **1/1/1/1** | ✓ PASS |

### Probe Execution

Проб-скриптов `probe-*.sh` в фазе не заявлено (PLAN/VALIDATION используют grep-гейты + diag_smoke, прогнанные выше). Не применимо.

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|------------|-------------|--------|----------|
| HIST-01 | 03-01-PLAN | Пассивное накопление диктовок, протокол не меняется (через A4-механизм: шов stop(), НЕ status-поток) | ✓ SATISFIED | truths 1, 5, 6, 7 + контрактный diff пуст |
| HIST-02 | 03-01-PLAN | Листание журнала + поиск по тексту | ✓ SATISFIED | truth 8 + widget-тесты |
| HIST-03 | 03-01-PLAN | Копия / re-insert / удаление | ✓ SATISFIED | truth 9 + widget-тесты (copy/inject-gate/delete/undo) |
| HIST-04 | 03-01-PLAN | Ограничение MVP зафиксировано в docs | ✓ SATISFIED | truth 4: 4 README-якоря == 1; анти-персистентность grep 0 |

Orphaned requirements: нет — REQUIREMENTS отображает на фазу 3 ровно HIST-01..04, план заявляет все четыре. Live-проявление HIST-01 (реальный микрофон + демон) → UAT-3-1/3-2.

### Prohibitions (must-NOT, все с grep/тест-доказательством)

| Prohibition | Status | Evidence |
|-------------|--------|----------|
| НЕ трогать crates/, pubspec.yaml/lock, systemd, adapters | ✓ HELD | git diff байт-в-байт → exit 0 |
| НЕ вводить персистентность (File/SharedPreferences/sqflite/JSONL/диск) | ✓ HELD | grep → 0; doc-замок history.dart:1-5 |
| НЕ регистрировать глобальные хоткеи на Linux | ✓ HELD | grep hotkey в lib → только l10n-строки/комментарии/диагностические подсказки; ни одной регистрации |
| НЕ дублировать implements DictaEngine (гейт ==1) | ✓ HELD | grep → 1 (единственная фабрика fake_dicta_engine.dart) |
| НЕ добавлять не-комментную кириллицу в app/lib вне generated | ✓ HELD | cyrillic_gate → 0 |
| НЕ менять REQUIREMENTS/ROADMAP (A4 живёт в CONTEXT) | ✓ HELD | HIST-01 в REQUIREMENTS неизменен; ROADMAP-diff — только плановые мета-строки (`Plans: TBD→0/1`, таблица статусов), SC не тронуты (см. Anti-Patterns ℹ️) |
| НЕ строить свой путь вставки — только inject() под commandInFlight | ✓ HELD | единственный путь history_screen.dart:134-142; тест gated |
| НЕ врезаться в inject()/start()/cancel(), НЕ подписываться на status-поток | ✓ HELD | historyProvider в dicta_control.dart — 1 ссылка, только в stop() |
| НЕ удалять placeholderScreen / _SectionPlaceholder | ✓ HELD | shell.dart:86-87,100-101; ключ в .arb == 2 (описание+значение), класс на месте |
| НЕ дублировать historyTitle / hist*-дубли dash-канона | ✓ HELD | historyTitle == 1; dash*-ключи переиспользованы, дубликатов нет (histUndo — новый лейбл, не дубль) |
| НЕ читать исходники FlClash (GPL) | ✓ HELD (by judgment) | ни одного следа в коде/коммитах; паттерны соответствуют собственным канонам фаз 1-2 |

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| — | — | — | — | Долг-маркеры TBD/FIXME/XXX/TODO в 13 файлах фазы: **0**. Пустых реализаций, console.log-заглушек, хардкода пустых данных — не найдено |

ℹ️ Наблюдение (не finding): ROADMAP.md в диапазоне фазы изменился на плановые мета-строки (строки `Plans:` и таблица Progress) — это бухгалтерия GSD-тулинга, содержательные Success Criteria и REQUIREMENTS не тронуты; запрет «НЕ менять REQUIREMENTS/ROADMAP» имел в виду A4-контент — он честно остался в CONTEXT.md.

ℹ️ histUndo — 8-й hist*-ключ против 7 плановых: добавлен UI-аудит-фиксом eb7fd7b (delete undo — top-3 приоритет 03-UI-REVIEW), не дублирует dash-канон. Легитимное расширение, отражённое в 03-UI-REVIEW.md.

### Test Quality Audit

| Test File | Linked Req | Active | Skipped | Circular | Assertion Level | Verdict |
|-----------|-----------|--------|---------|----------|-----------------|---------|
| history_store_test.dart | HIST-01/02 | 4 | 0 | 0 | Value (09:05/23:59, порядок, remove) | OK |
| history_capture_test.dart | HIST-01 | 4 | 0 | 0 | Behavioral (захват/эхо/пустышка/порядок через fake stopReply) | OK |
| history_screen_test.dart | HIST-02/03 | 17+ | 0 | 0 | Behavioral (поиск, швы, гейты, undo, a11y) | OK |

Disabled-тестов нет (`it.skip`/`@Skip` → 0). Циркулярных паттернов нет: fake-движок — намеренный тест-двойник шва DictaEngine, ожидаемые значения независимы (жёстко заданные строки «привет»/«первый»/«второй»).

### Decision Coverage

`check.decision-coverage-verify` → skipped («no trackable decisions»: CONTEXT.md использует прозаические списки, не `<decisions>`-блоки). Ручная трассировка 9 locked-решений CONTEXT: все 9 соблюдены в коде (шов stop-only, без персистентности, без хоткеев, фабрика ==1, cyrillic 0, l10n-канон, ThemeExtension обе точки, diag_smoke-долг, inject-единственный путь).

### Human Verification Required

Все 13 истин верифицированы автоматикой; остаются **живые интеграционные проверки** (реальный микрофон + реальный демон + UX глазами) из 03-VALIDATION.md §Backstop / Manual UAT. Вынесены в **03-UAT.md (status: testing)**:

### 1. UAT-3-1: Живая диктовка через приложение → запись в журнале
**Test:** Запусти dicta-app, Запись→диктовка→Стоп, открой «Историю»; затем перезапусти приложение.
**Expected:** Текст в журнале с HH:mm; после рестарта журнал пуст (честный MVP).
**Why human:** Требует живой микрофон/демон; widget-тесты доказали механику только на фейке.

### 2. UAT-3-2: Хоткейная диктовка демона НЕ попадает в журнал — подписать принятие
**Test:** Продиктуй ГЛОБАЛЬНЫМ хоткеем демона при открытом приложении, открой «Историю».
**Expected:** Записи нет — документированное ограничение A4 (README: never enter the journal). Подписать принятие ограничения (не дефект).
**Why human:** Наблюдаемое поведение живого демона + юридически значимое принятие.

### 3. UAT-3-3: Поиск/копия/re-insert/удаление глазами
**Test:** 3+ записи; поиск в другом регистре, кривой запрос, очистка; копия; вставить заново; удалить + undo.
**Expected:** Все действия работают на живом журнале.
**Why human:** Живое UX-качество на реальных данных.

### Gaps Summary

Гэпов, блокирующих цель, **нет**: все 13 must-haves верифицированы живыми прогонами (батарея 71/71, analyze 0, cyrillic 0, контрактный diff пуст, все grep-гейты в EXPECT, шов/эхо/гейты подтверждены кодом и тестами). Статус **human_needed** только из-за трёх живых UAT-пунктов — они уже вынесены в 03-UAT.md и не могут быть закрыты автоматикой (микрофон, живой демон, подписание принятия A4).

Отклонения, зафиксированные SUMMARY, подтверждены как доброкачественные:
- третий scaffold регистрации ThemeExtension (shell_test.dart) — Rule-1 фикс, тест-только;
- `restoreAt`/`histUndo` (undo удаления) — добавление из UI-аудита eb7fd7b, не сужает контракт;
- 03-SUMMARY.md вместо 03-01-SUMMARY.md — по директиве оркестратора.

Backstop-пункты VALIDATION честно не закрыты молча: grep-backstop анти-персистентности прогнан (0), но живой UAT — в 03-UAT.md, статус testing.

---

_Verified: 2026-11-22T14:10:00Z_
_Verifier: the agent (gsd-verifier)_

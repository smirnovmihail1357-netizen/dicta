# VALIDATION — Phase 3: История диктовок

_status: draft_  
_scope: HIST-01, HIST-02, HIST-03, HIST-04_  
_research: 03-RESEARCH.md (a820069, HIGH confidence)_

## Validation Architecture (из 03-RESEARCH.md)

Тест→грань мэппинг (все сценарии на существующих крюках фабрики, без новых крюков):

| Грань | Закрывается | Механизм |
|---|---|---|
| HIST-01 захват | stop-реплика с текстом → ровно одна запись | `fake.stopReply` (fake_dicta_engine.dart:43,85-90) + `captureContainer` |
| HIST-01 анти-дубли | inject-эхо НЕ пишет запись; пустой текст НЕ пишет | `fake.injectTexts`, stopReply с пустым text |
| HIST-01 порядок/id | новые сверху, id монотонный | unit-тест historyProvider (level_history_test.dart:6-33 паттерн) |
| HIST-02 список+поиск | ListView.builder рендерит все; enterText фильтрует substring case-insensitive | widget-сценарий навигацией `find.text('История')` (shell.dart:63) |
| HIST-02 таймстамп | `DateFormat.Hm('ru')` → «HH:mm», tabular figures | форматтер-тест + образец dashboard_screen.dart:80-84 |
| HIST-03 copy | `onCopy` перехват + snackbar histCopied | last_text_card.dart:55-58 паттерн |
| HIST-03 re-insert | `fake.injectTexts` + busy-гард commandInFlight | last_text_card.dart:70-81 паттерн |
| HIST-03 delete | запись исчезает из state и из UI | unit (remove) + widget (tap «удалить») |
| HIST-04 docs | README фиксирует ограничения (память + хоткей) | grep-гейт по README |
| HIST-04 анти-персистентность | нет записи на диск | grep-гейт отсутствия File/SharedPreferences/sqflite в app/lib |

Wave-структура: Wave 0 — store/entry/unit-тесты + .arb + ThemeExtension (ДВЕ точки
регистрации: main.dart:41-43 И buildFakeApp fake_dicta_engine.dart:145-150, иначе
падают тесты) + diag_smoke-долг (:28 → error.runtimeType); Wave 1 — врезка stop() +
экран + слот shell.dart:84 + widget-сценарии; финальный гейт — батарея + analyze +
cyrillic + контракты.

## Edge-Probe резолюции (5 граней)

| Грань | Категория | Резолюция | Verification |
|---|---|---|---|
| HIST-01 | idempotency | одна stop-реплика → ровно одна запись (гард text!=null && error==null && trim().isNotEmpty); одинаковые тексты двух РАЗНЫХ stop'ов — две легитимные записи (разные диктовки); двойной тап заморожен commandInFlight | explicit (тесты: захват/эхо-нет/пустышка-нет) |
| HIST-01 | concurrency | add синхронен по порядку после await; команды сериализованы commandInFlight; убийство приложения = потеря in-memory — принято HIST-04 | explicit (busy-гард тест) |
| HIST-02 | unclassified | ListView.builder без пагинации (A1: сотни записей); поиск case-insensitive substring, пустой запрос = все | explicit (widget: рендер + enterText-фильтр) |
| HIST-03 | unclassified | copy → clipboardWriterProvider (явный клик); re-insert → ТОЛЬКО dictaControlProvider.inject() (кнопка неактивна при busy); delete → локальный remove | explicit (три сценария) |
| HIST-04 | unclassified | docs: README фиксирует «история в памяти, пока приложение открыто» + «хоткейные диктовки демона в журнал не попадают (A4)» | explicit (grep README) + backstop (grep отсутствия persistence-API) |

## Per-Task Verification Map

_Привязка к задачам 03-01-PLAN.md (Task 1..4):_

| Task | Что строит | Тесты/гейты |
|---|---|---|
| 1 | HistoryEntry+HistoryNotifier (unit: порядок/id/remove) + .arb-ключи + ThemeExtension (обе регистрации) + diag_smoke-долг | unit-батарея store; analyze; cyrillic 0 |
| 2 | Врезка stop() (гард-условия) + сценарии захвата (HIST-01) | capture/эхо/пустышка/порядок; analyze |
| 3 | Экран истории + слот shell + поиск + copy/inject/delete (HIST-02/03) | widget-сценарии; snackbar-ключи; tabular figures |
| 4 | Гейт-прогон + README HIST-04 + контракты | полная батарея (~64-67 ожидается); analyze «No issues»; cyrillic 0; pubspec/crates байт-в-байт к базе фазы; implements DictaEngine == 1; grep README-ограничений; grep отсутствия persistence |

## Backstop / Manual UAT (human_needed —/items в UAT-файл)

- **UAT-3-1**: живая диктовка через приложение (Запись→Стоп) → запись появилась в
  «Истории» с временем HH:mm; перезапуск приложения → история пуста (честное MVP).
- **UAT-3-2**: живая диктовка через ГЛОБАЛЬНЫЙ хоткей демона → в журнал НЕ попала
  (наблюдаемое ограничение A4; подписать, что принято).
- **UAT-3-3**: поиск/копия/re-insert/удаление глазами на живом журнале.

## Команды проверки (канон фазы 2)

```
cd app && flutter test                                   # батарея, 0 провалов
cd app && flutter analyze                                # No issues found!
bash app/tool/cyrillic_gate.sh                           # 0
cd app && grep -rn "implements DictaEngine" test/*.dart | wc -l   # ==1 (гейт [HEAD=2 EXPECT==1])
git diff <base> HEAD --stat -- crates/ app/pubspec.yaml  # пусто
cd app && dart run tool/diag_smoke.dart                  # exit 0 (живой демон)
grep -n "пока приложение" ../README.md                   # HIST-04 фиксация
grep -rn "SharedPreferences\|sqflite\|writeAsString" app/lib     # пусто
```

## Assumptions-границы

A1 (масштаб/id/порядок), A2 (substring-поиск без дебаунса), A3 (пустышки не пишем) —
locked в CONTEXT.md; планёр не пересуживает.

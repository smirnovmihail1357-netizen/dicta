# REVIEW DISPOSITION — Phase 3: История диктовок

_Ревью: 03-REVIEW.md (0 blocker / 1 warning / 4 info). Фиксы: commit 3cb280e. Батарея после фиксов: 68/68, analyze 0, cyrillic 0, implements ==1._

## Триаж

| Finding | Severity | Диспозиция |
|---|---|---|
| WR-01 Semantics поиска (два фокуса у скрин-ридера) | warning | **FIXED** — MergeSemantics вокруг пары (history_screen.dart:30-48); тест семантики доказывает один фокус-стоп (RegExp по склеенной метке, isTextField, hasLength(1)). Урок фазы 2 (калибровка) применён: a11y-долг гасится немедленно, не копится. |
| IN-01 inject-эхо не ассертилось из сценария re-insert | info | **FIXED** — expect(historyProvider, hasLength(1)) после эха (history_screen_test.dart:139-161). |
| IN-02 mounted-гард и семантика поиска без тестов | info | **FIXED** — тест семантики + тест гарда через подмену home (fake buildFakeApp получил опциональный home; провайдеры живы, контекст мёртв — гард покрыт без UnmountedRefException). |
| IN-03 комментарий theme_ext называл 2 точки регистрации, их 3 | info | **FIXED** — doc обновлён (main.dart/buildFakeApp/buildTestApp) + grep-подсказка. |
| IN-04 head-derived id после удалений не документирован | info | **FIXED** — doc-комментарий HistoryNotifier (уникальность среди живых записей) + unit-тест remove→add без конфликта. |

## Принятые девиации исполнения (из 03-SUMMARY + фиксов)

1. Третья точка регистрации ThemeExtension (shell_test buildTestApp) — необходимость:
   тестовый скаффолд строит собственный MaterialApp; без регистрации падали 2
   унаследованных теста, что нарушало fails_when плана. Задокументировано (IN-03).
2. Summary-файл 03-SUMMARY.md (не 03-01-SUMMARY.md) — по прямой инструкции
   оркестратора (канон фазы 2).
3. RegExp вместо bySemanticsLabel в тесте семантики — MergeSemantics склеивает label
   и hint; RegExp + ролевая проверка точнее доказывает контракт. Документировано в
   тесте.

## Отложенные хуки (осознанно, как в фазе 2)

- **nyquist validate-phase** → после UAT-раунда (backstop-пункты UAT-3-1..3 живые;
  карта VALIDATION полная, автопокрытие 68/68 подтверждено).
- **ui-review** → запускается как advisory-шаг сейчас (экран истории — центральная
  ценность окна); находки → вход планирования фазы 4.

## Остаток для следующих фаз

- Идём в фазу 4 с чистым a11y-долгом: поиск, список, карточки покрыты Semantics;
  следующий шаг — экраны настроек (CONF/MODEL), там ждут in-flight индикаторы и
  заголовочный паттерн (наследие фазы 2 полностью или частично закрыто здесь:
  muted-тема ✓, tabular figures ✓, feedback-словарь ✓).

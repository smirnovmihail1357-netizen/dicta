# Phase 3 — UI Review (advisory)

**Audited:** 2026-11-22
**Baseline:** abstract 6-pillar standards (UI-SPEC.md не существует) + наследие 02-UI-REVIEW.md / 02-REVIEW-DISPOSITION.md
**Scope:** код фазы 3 — экран истории (`history.dart`, `history_screen.dart`), слот в `shell.dart`, `theme_ext.dart`; сверка с дашбордом фазы 2 (пост-фиксы)
**Screenshots:** not captured — Flutter Linux desktop; кодовый аудит с живыми grep/чтением
**Качество базы:** analyze 0 issues, 68/68 зелёные, кириллический гейт 0

## Pillar Scores

| Pillar | Score | Key Finding |
|--------|-------|-------------|
| 1. Иерархия/пространство | 3/4 | Карточный скелет повторяет дашборд; copy/inject равновесные ElevatedButton, длинные записи не ограничены |
| 2. Типографика | 4/4 | Ноль литеральных размеров; tabular figures + minute-precision — наследие закрыто образцово |
| 3. Цвет/контраст | 3/4 | Роли ColorScheme; delete без error-тона (дрейф от канона cancel) |
| 4. Состояние/feedback | 3/4 | Snackbar-канон закрыт; провал inject на истории — тишина; удаление без confirm/undo |
| 5. Консистентность/паттерны | 3/4 | muted→ThemeExtension закрыт (14 использований, assert-гейт); заголовочный паттерн — OPEN → фаза 4 |
| 6. Доступность | 3/4 | MergeSemantics поиска — образец; ноль liveRegion на экране истории |

**Overall: 19/24** (фаза 2: 17/24).

## Наследие фазы 2 — сверка

| Обещание | Статус | Доказательство |
|---|---|---|
| muted ×5 → theme-extension | CLOSED | theme_ext.dart:11-38; 14 mutedOf; assert-гейт регистрации |
| Snackbar-канон | CLOSED | history_screen.dart:114-119,144-150,168-173 + last_text_card.dart:57-59,79-81 |
| Tabular figures таймстампов | CLOSED | history_screen.dart:95-100 (DateFormat.Hm + FontFeature) |
| Заголовочный паттерн карточек | OPEN → фаза 4 | dashboard_screen.dart:58-70 без изменений |
| «команда не выполнена» вне l10n | CLOSED | dicta_control.dart:100-108 читает dashCommandFailedBody |

## Top 3 Priority Fixes (гашатся немедленно в этой фазе)

1. **Delete без подтверждения и undo** (history_screen.dart:162-176) — SnackBarAction «вернуть»: записать удалённый entry, восстановить по индексу.
2. **Провал inject на истории — тишина** (history_screen.dart:130-152) — error-snackbar (dashInjectFailed + verbatim-тело как last_text_card.dart:98-111).
3. **Деструктивный/приоритетный дрейф кнопок** (history_screen.dart:105-157) — delete → error-foreground (канон cancel, dashboard_screen.dart:137-151); приоритет: inject FilledButton.tonal или copy понизить — один приоритет выбрать.

## Detailed Findings (краткая выжимка; полные — в выводе аудитора)

- WARNING: плоская кнопочная пара copy/inject; записи без maxLines (стена текста).
- MINOR: низ ListView без отступа; два empty-состояния в разной упаковке.
- MINOR: «модель: …» префиксы в .arb (:205,213) — фаза 4.
- MINOR: busy-gating copy дрейфует между экранами (рекомендация: убрать gate на дашборде).
- MINOR: inject in-flight без индикатора (паттерн dashboard_screen.dart:162-166).
- WARNING a11y: ноль liveRegion на экране истории (область результатов); SnackBar может не звучать для AT на desktop — проверить semanticsEnabled-прогоном.
- MINOR a11y: «ничего не найдено» без Semantics-заголовка.

## Вход планирования фазы 4 (7 пунктов)

1. Заголовочный паттерн статус-карточки (последнее наследие фазы 2).
2. Деструктивный канон: error-foreground + undo — до появления сбросов конфигурации.
3. Единый механизм ошибок cross-screen команд (не только карточка дашборда).
4. In-flight индикаторы async-кнопок — загрузка модели первое долгое действие.
5. Свёртка длинных записей (maxLines/превью).
6. liveRegion области результатов + live-проверка SnackBar AT.
7. Мелочи одним коммитом: bottom-padding, busy-gating copy, тон «модель: …», clear-search suffix, empty-упаковка.

## Registry Safety

Неприменимо (Flutter, не shadcn); новых зависимостей нет.

**UI REVIEW COMPLETE — Overall 19/24, рекомендации: 3 priority (гасятся сейчас) + 6 minor → фаза 4.**

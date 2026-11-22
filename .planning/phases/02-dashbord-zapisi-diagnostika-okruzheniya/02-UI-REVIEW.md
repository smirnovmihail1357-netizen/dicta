# Phase 2 — UI Review

**Audited:** 2026-11-22
**Baseline:** abstract 6-pillar standards (UI-SPEC.md не существует)
**Scope:** код фаз 1–2 — Material You shell, дашборд записи, last-text, калибровка, диагностика
**Screenshots:** not captured — Flutter Linux desktop, веб-dev-сервера нет (порт 3000 занят посторонним node-приложением); кодовый аудит
**Interaction captures:** off (workflow.ui_interaction_capture не включён; нет web-URL для драйвера)

---

## Pillar Scores

| Pillar | Score | Key Finding |
|--------|-------|-------------|
| 1. Иерархия/пространство | 3/4 | Чистая 4px-сетка и логичный порядок карточек; но 6/6 кнопок — одинаковые ElevatedButton, и статус-карточка без заголовка ломает паттерн остальных |
| 2. Типографика | 3/4 | 100% на текстовых токенах M3; живое состояние не выделено, 10-Гц числа без tabular figures |
| 3. Цвет/контраст | 3/4 | Токены везде (единственный hex — fallback-семя); маркер порога в colors.error — семантический мисюз |
| 4. Состояние/feedback | 3/4 | Полное покрытие loading/error/empty/disabled; inject без подтверждения успеха, калибровка без числового значения |
| 5. Консистентность/паттерны | 3/4 | Идентичная структура карточек; «слабый» стиль дублируется ×5, одна user-visible строка вне l10n |
| 6. Доступность | 2/4 | BLOCKER: калибровка недоступна клавиатуре/AT; ноль Semantics/liveRegion в lib/ |

**Overall: 17/24**

---

## Top 3 Priority Fixes

1. **Калибровочный surface недоступен клавиатуре и assistive tech** — единственный способ задать min_level — мышиный horizontal drag по CustomPaint без Semantics, Focus и numeric readout (`calibration_card.dart:89–134`); клавиатурный/AT-пользователь полностью отрезан от функции, а зрячий мышиный пользователь калибрует вслепую (нет числа) — обернуть в `Semantics(slider...)` + `Focus`/стрелки + показывать текущее значение; либо заменить жест настоящим `Slider`, оставив гистограмму read-only визуализацией.
2. **Маркер порога нарисован в `colors.error`** (`calibration_card.dart:46–47`) — красная вертикаль во всю высоту гистограммы читается как «сбой», а не как ручка настройки; заменить на `primary`/`secondary`, а `colors.error` оставить только ошибкам.
3. **Нет анонсов смены состояния и кнопочной иерархии** — смена idle→recording→transcribing и появление баннера «демон недоступен» не звучат для screen reader (ноль `Semantics(liveRegion)` в lib/, `shell.dart:31–42`, `dashboard_screen.dart:63–66`); одновременно выровнять кнопки по M3: Start=filled, Stop=filled-tonal, Cancel=outlined/error-foreground, «перепроверить»=TextButton (`dashboard_screen.dart:103–137`, `diagnostics_card.dart:99–107`).

---

## Detailed Findings

### Pillar 1: Иерархия/пространство (3/4)

**Хорошо:** пять карточек в логическом порядке (статус → управление → текст → калибровка → диагностика); единая сетка 16 padding / 12 меж-карточно / 8–4 внутри — греп по lib/ дал только значения {4, 8, 12, 16} (+ высота гистограммы 120, оправдано); `SingleChildScrollView` — нет overflow при min 800×600; `CrossAxisAlignment.stretch` держит кнопки на всю ширину карточки.

- **WARNING — плоская кнопочная матрица.** Все 6 интерактивных кнопок приложения — `ElevatedButton` (греп: `dashboard_screen.dart:103,115,127`, `last_text_card.dart:50,66`, `diagnostics_card.dart:99`). В состоянии recording одновременно активны «остановить» и «отмена» — и обе неотличимы: деструктивное действие не выделено ни тоном, ни вариантом. «Перепроверить» весит как «начать запись». Fix: M3-иерархия — Start `ElevatedButton` (filled), Stop `FilledButton.tonal`, Cancel `OutlinedButton` с error-foreground, «перепроверить» `TextButton`.
- **WARNING — статус-карточка ломает заголовочный паттерн.** У трёх из четырёх карточек заголовок — существительное в `titleMedium` («последний текст», «порог тишины», «диагностика»); статус-карточка вместо заголовка сразу выдаёт `stateLabel` «состояние: ожидание» (`dashboard_screen.dart:63–66`) — на глаз это выглядит как забытый заголовок, и главная динамика экрана весит как статика. Fix: дать карточке заголовок («запись»), состояние поднять в `titleLarge`/`headlineSmall` с цветом по состоянию.
- **WARNING — мёртвая анимация.** `AnimatedContainer(duration: 120ms)` (`dashboard_screen.dart:68–70`) оборачивает `LinearProgressIndicator`, но ни одно свойство контейнера не меняется — анимации нет; сам бар обновляется 10 раз/с ступеньками без сглаживания. Fix: убрать AnimatedContainer; для сглаживания — tween-значение уровня (например, `TweenAnimationRunner`/короткий lerp), либо осознанно оставить ступеньки и удалить вводящий в заблуждение контейнер.
- **MINOR — идентичные строки диагностики.** «демон жив» и «сокет отвечает» — один и тот же тест, одинаковые иконка и цвет (`diagnostics_card.dart:77–78`); честность задокументирована и подкреплена подписью, но визуально это дубль: сгруппировать строки под один заголовок или дать второй производный признак (напр. время последнего ответа).

### Pillar 2: Типографика (3/4)

**Хорошо:** греп по lib/ — ноль `fontSize`/`fontWeight`/`FontFeature`; всё на дефолтной шкале M3 (`titleMedium` для заголовков карточек, `bodyMedium` для контента, `onSurfaceVariant` для второстепенного). Это образцовое использование токенов.

- **WARNING — живое состояние не выделено.** Значение состояния (самая динамичная информация экрана) набрано тем же `titleMedium`, что и статические заголовки карточек (`dashboard_screen.dart:63–66`); значение внутри «состояние: …» не выделено ни весом, ни цветом. Fix: состояние — `titleLarge`+, окрасить по семантике (recording=primary, transcribing=tertiary).
- **WARNING — 10-Гц цифры без tabular figures.** «время 01:05» и «уровень 0.42 · 12.3 с» (`dashboard_screen.dart:77–88`) обновляются 10 раз/с в пропорциональном шрифте — цифры «дышат» по ширине, таймер визуально дёргается. Fix: `TextStyle(fontFeatures: [FontFeature.tabularFigures()])` для обеих строк (или моноширинный readout).
- **MINOR — шум в idle.** В idle отображаются «уровень 0.00 · 0.0 с» — мёртвые нули как постоянный шум (`dashboard_screen.dart:82–88`). Fix: скрывать/приглушать строку вне recording.

### Pillar 3: Цвет/контраст (3/4)

**Хорошо:** единственный hex в lib/ — fallback-семя `0xFF35689F` с комментарием (`main.dart:36`); весь остальной цвет — роли ColorScheme (fromSeed от KDE-акцента через DynamicColorBuilder, `main.dart:30–43`). Баннер недоступности — корректная пара `errorContainer`/`onErrorContainer` (`shell.dart:32–41`). Контраст relies на M3-дефолты — ручных пар «цвет-на-цвете» в коде нет, нарушений контраста не найти.

- **WARNING — семантический мисюз error для маркера порога.** Вертикальная линия порога тишины рисуется `colors.error` (`calibration_card.dart:46–47`). Красный в M3 = ошибка/деструктивность; постоянный красный элемент UI читается как аварийный. Fix: `primary`/`secondary`, возможно утолщение+подпись при drag.
- **WARNING — `outline` как цвет данных.** Уровень ≤0.3 рисуется `colors.outline` (`calibration_card.dart:31–35`) — это роль бордюров/разделителей, не данных; тиры primary(>0.6)/tertiary(>0.3) семантически произвольны. Fix: один цвет данных с прозрачностью по уровню (primary с alpha-градацией) — и читабельнее, и без ложных ролей.
- **MINOR — иконки «успеха» в primary.** Положительные строки диагностики — `check_circle` в `colors.primary` (`diagnostics_card.dart:61–63`). M3-идиоматично (роли success нет), допустимо; при появлении нового акцента от KDE-semов следить, чтобы primary не совпал тоном с error в тёмной теме.

### Pillar 4: Состояние/feedback (3/4)

**Хорошо — сильнейший столп:** честная кнопочная матрица (`canStart/canStop/canCancel`, busy-gating, `dashboard_screen.dart:32–35`); spinner+подпись «распознаю…» только от полленного состояния, а не от Future (`dashboard_screen.dart:141–155`); честные empty-states с подсказками (`last_text_card.dart:35–45`, `calibration_card.dart:142–148`); ошибки демона — verbatim, с различением inject/прочих команд (`last_text_card.dart:80–95`, WR-01); провал fetch настроек ≠ «нет модели» (`diagnostics_card.dart:41–64`, IN-01/IN-02); при ошибке set маркер не откатывается на стейл (`calibration_card.dart:105–110`); снекбар после копирования (`last_text_card.dart:56–58`).

- **WARNING — успешный inject без подтверждения.** После успешной re-inject пользователь не получает никакого отклика (`dicta_control.dart:120–139`, `last_text_card.dart:66–74`) — непонятно, вставилось ли. Fix: transient SnackBar «текст вставлен» по пути успеха.
- **WARNING — калибровка без числового readout.** Во время и после drag значение порога нигде не показано (`calibration_card.dart:89–134`) — единственная обратная связь — положение линии. Fix: подпись значения (напр. «порог 0.35») под гистограммой, обновляется локально из `_marker`.
- **MINOR — «перепроверить» без отклика.** Кнопка дергает poll+invalidate (`diagnostics_card.dart:99–107`), но результата нажатия не видно — ни спиннера, ни снека. Fix: краткий in-flight индикатор на кнопке (async-кнопка).
- **MINOR — команды in-flight гасят всю матрицу без индикации.** `commandInFlight` дизейблит все кнопки разом (`dashboard_screen.dart:32–35`) — на локальном IPC это миллисекунды, но при таймауте сокета пользователь увидит просто «всё серое». Fix: прогресс-индикатор на активной кнопке.

### Pillar 5: Консистентность/паттерны (3/4)

**Хорошо:** все карточки — один и тот же скелет `Card > Padding(16) > Column[titleMedium, 12, …]`; иконки rail — outlined/filled пара по канону M3 (`shell.dart:52–71`); UI-тексты все через .arb (29 ключей, кириллический гейт 0); тестовые Key на всех интерактивах — воспроизводимость в widget-тестах.

- **WARNING — user-visible строка вне l10n.** «команда не выполнена» захардкожена в `dicta_control.dart:13–14` юникод-эскейпами (чтобы обойти кириллический гейт) и показывается пользователю как тело ошибки (`last_text_card.dart:91–94`). Гейт обойдён техникой, а не архитектурой: при добавлении второй локали строка останется русской. Fix: ключ в .arb (`dashCommandFailedBody`), передавать через состояние или читать l10n на месте показа.
- **WARNING — «слабый» стиль дублируется ×5.** `TextStyle(color: colors.onSurfaceVariant)` конструируется заново в `last_text_card.dart:22`, `diagnostics_card.dart:18`, `calibration_card.dart:140,146,151`, `dashboard_screen.dart:79,87` — дрейф неизбежен. Fix: theme-extension (напр. `ThemeExt.muted(context)` или кастомный `textTheme`-оверлей) — один источник второстепенного стиля.
- **MINOR — тон копии.** «модель: данных нет» / «модель: не удалось получить данные» используют префикс с двоеточием, остальные строки — прямые предикаты (`app_ru.arb:158,166`). Fix: выровнять тон («модель не загружена», «данные модели недоступны»).

### Pillar 6: Доступность (2/4)

**Хорошо:** все кнопки — текстовые (ноль icon-only), у rail `labelType.all` (`shell.dart:50`); `SelectableText` для распознанного текста (`last_text_card.dart:36`); min-окно 800×600 исключает тесноту; сигнализация в диагностике не чисто цветовая — форма иконки меняется (check/error/hourglass).

- **BLOCKER — калибровка недоступна клавиатуре и AT.** Греп по lib/: ноль `Semantics`, `Focus`, `KeyboardListener`, `Slider`. Поверхность калибровки — голый `GestureDetector` над `CustomPaint` (`calibration_card.dart:89–134`): нет роли slider, нет значения, нет фокуса, нет стрелок. Клавиатурный/скрин-ридер-пользователь не может выполнить ключевую операцию фазы 2 вообще; гистограмма так же без semantic-описания. Fix (минимально): `Semantics(container, label: 'порог тишины', value: '0.35', increasedValue/decreasedValue, onSetSemanticValue → set)` + `Focus` + обработка стрелок; (надёжно): реальный `Slider(value/onChanged/onChangeEnd)` под гистограммой — жест на гистограмме оставить как шорткат, slider станет доступным путём. Это фикс до какой-либо UI-фазы, т.к. касается уже поставленной функции.
- **WARNING — ноль live regions.** Смена состояния записи (`dashboard_screen.dart:63–66`) и появление баннера «демон недоступен» (`shell.dart:31–42`) не анонсируются скрин-ридеру; при этом 10-Гц readout наоборот был бы спамом, если попадёт в semantics по умолчанию. Fix: `Semantics(liveRegion: true)` на state-лейбле и баннере; `excludeSemantics`/статичный `semanticsLabel` на уровне и таймере.
- **WARNING — spinner без semantics-лейбла.** `CircularProgressIndicator` в «распознаю…» (`dashboard_screen.dart:146–150`) визуальный; спасает соседний читаемый текст — достаточно, но при рефакторинге подпись связать с индикатором (`Semantics(label…)`).

---

## Registry Safety

`components.json` отсутствует (Flutter-проект, не shadcn/React) — registry-аудит пропущен. Сторонних UI-блоков из registries не установлено; зависимости приложения за фазы 1–2 не менялись (`pubspec.yaml`/`pubspec.lock` diff пуст, фаза 2 gate).

---

## Disposition (orchestrator, 2026-11-22)

- **Чинится немедленно (UI-долг фазы 2, фиксер):** BLOCKER a11y-калибровки (Slider + readout), live regions, маркер порога + тиры цвета, кнопочная иерархия M3, tabular figures, «команда не выполнена» → .arb, мёртвый AnimatedContainer, snackbar на успех inject.
- **В план фаз 3–5 (design-system наследие):** theme-extension `muted(context)` (×5 копий), заголовочный паттерн статус-карточки, тон «модель: …» в .arb, дубль-строки диагностики, in-flight индикаторы async-кнопок, idle-нули уровня.

## Files Audited

- app/lib/main.dart (тема, dynamic color, окно)
- app/lib/features/shell/shell.dart (NavigationRail, баннер, плейсхолдеры)
- app/lib/features/dashboard/dashboard_screen.dart (статус-карточка, матрица кнопок, spinner)
- app/lib/features/dashboard/dicta_control.dart (command state, clipboard seam)
- app/lib/features/dashboard/last_text_card.dart (empty state, copy/inject, ошибки)
- app/lib/features/dashboard/calibration_card.dart (CustomPainter, drag-калибровка)
- app/lib/features/dashboard/diagnostics_card.dart (чеклист, recheck)
- app/lib/features/dashboard/level_history.dart (ring buffer)
- app/lib/core/providers.dart, engine.dart (контракт состояния/полла)
- app/lib/l10n/app_ru.arb (29 ключей фазы 2)
- .planning/phases/02-dashbord-zapisi-diagnostika-okruzheniya/02-SUMMARY.md (контракт реализации)

**Аудитор:** gsd-ui-auditor (фон, автономный режим), 2026-11-22.

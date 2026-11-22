# Phase 4 — UI Review (retroactive, 6-pillar)

**Audited:** 2026-11-22
**Baseline:** abstract 6-pillar standards (UI-SPEC.md не существует) + каноны 02/03-UI-REVIEW.md (ThemeExtension muted, snackbar-канон, кнопочный канон, tabular figures, MergeSemantics на полях, liveRegion на статусе, 3+1 точка регистрации)
**Scope:** `settings_screen.dart` (17-ключевая форма, 4 группы, ENGINE-confirm, reload), `models_screen.dart` (список/активная/guarded delete), `hf_browser.dart` (каталог/поиск/дерево/VRAM/ссылка/загрузка), фаза-4 часть `shell.dart` (оба слота), новые ключи `app_ru.arb`; коммиты d646e3a..HEAD
**Screenshots:** not captured — порт 3000 отвечает 200, но это постороннее node-приложение (задокументировано в 02-UI-REVIEW:6); dicta — Flutter Linux desktop без web-таргета; кодовый аудит
**Interaction captures:** off (workflow.ui_interaction_capture не включён; нет web-URL для драйвера)
**Качество базы:** analyze 0, 131/131, кириллический гейт 0 (04-SUMMARY, final gate)

---

## Pillar Scores

| Pillar | Score | Key Finding |
|--------|-------|-------------|
| 1. Иерархия/типографика | 3/4 | Статус-карточка-заголовок (последнее наследие фазы 2) закрыт образцово; но заголовок секции поиска — это переиспользованный hint-ключ |
| 2. Цвет/тон | 3/4 | error повсюду семантичен; confirm-delete без `onError`-пары, VRAM-over в каталоге и дереве — два разных тона |
| 3. Раскладка/отступы | 2/4 | Внутри групп формы нулевой вертикальный ритм (17 контролов); паддинг карточек 12 vs канон 16, межкарточный 4 vs 12 |
| 4. Консистентность/паттерны | 3/4 | Кнопочная матрица по канону (Filled/tonal/Outlined+error/Text); англ. литералы ошибок (`delete failed:`, `HTTP 404`) — класс дефекта, закрытый в фазе 2, воссоздан |
| 5. Доступность | 3/4 | MergeSemantics на ВСЕХ текстовых полях, включая прямую ссылку ✓; но 10 слайдеров + 3 сегмента + VRAM-слайдер анонсируют значение без имени |
| 6. Состояние/отклик | 2/4 | Форма настроек рисует сфабрикованные дефолты при провале fetch; отказ ENGINE при выборе модели — тишина; таймауты поиска не пойманы |

**Overall: 16/24** (фаза 2: 17/24, фаза 3: 19/24).

Траектория 17 → 19 → 16 честная: **всё наследие фаз 2–3 закрыто и проверено** (таблица ниже), regression — целиком в новых поверхностях фазы 4 (длинный хвост формы/браузера): три дефекта отклика + нулевой ритм формы.

## Сверка наследия фаз 2–3

| Обещание | Статус | Доказательство |
|---|---|---|
| №1 Заголовок статус-карточки (последний item фазы 2) | CLOSED | dashboard_screen.dart:64-80 — titleMedium «запись» + titleLarge состояние с цветом (idle=onSurface, recording=primary, transcribing=tertiary) + liveRegion |
| №2 Деструктивный confirm + канон | CLOSED | models_screen.dart:66-84 confirm; :193-199 OutlinedButton+error; history delete undo сохранён |
| №3 Единый механизм ошибок cross-screen | PARTIAL | settings: liveRegion confStatus ✓ (settings_screen.dart:518-527); models select: отказ молчит (models_screen.dart:50) |
| №4 In-flight индикаторы (download/reload) | CLOSED | reload-спиннер settings_screen.dart:419-433 + disabled ENGINE; download-панель hf_browser.dart:439-481; search/retry — НЕТ |
| №5 Свёртка длинных записей | DEFERRED → фаза 5 | по плану (CONTEXT:33), подтверждено |
| №6 liveRegion + SnackBar-AT | CLOSED | liveRegion ×4 (shell:36, dashboard:69, settings:519, hf_browser:441); ensureSemantics-пробы history_screen_test.dart:142,365 |
| №7 Мелочи: bottom-padding/clear-suffix/empty-упаковка/тон «модель: …» | CLOSED | history_screen.dart:98, 43-50, 62-95, .arb:202,210 («данных о модели нет» — префикс убран) |
| muted → ThemeExtension, 3+1 регистраций | CLOSED | theme_ext.dart:31-38 assert-гейт; регистрации: main.dart:49,54 + shell_test.dart:23 + fake_dicta_engine.dart:193 |
| Tabular figures на живых числах | CLOSED | readout'ы settings:197-199, размеры/качества hf_browser:229-231, 324-326, 385-387, прогресс 461-463 |

---

## Top 3 «Погасить немедленно»

1. **Форма настроек рисует сфабрикованные дефолты при провале/загрузке fetch** (`settings_screen.dart:282-283`: `_replySnapshot ?? providerSettings ?? const SettingsReply()`; нет ветки `AsyncError`/`AsyncLoading`) — при мёртвом демоне пользователь видит полностью заполненную форму из нулей/дефолтов как «текущие» значения; ModelsScreen обрабатывает `AsyncError` карточкой (models_screen.dart:128-152), а экран настроек — нет. Fix: `AsyncLoading` → spinner; `AsyncError` → error-карточка с retry по образцу models_screen + запрет `_commit` при отсутствии живого снапшота.
2. **Отказ ENGINE при выборе модели — тишина** (`models_screen.dart:50`: `if (reply.error != null) return;`) — диалог закрыт, демон отказал (main.rs:752, напр. во время диктовки), маркер не двинулся, НИЧЕГО не показано; SetReply-канон «note/error — данные» реализован только на settings-экране. Fix: показывать `reply.error` вербатим snackbar'ом (или liveRegion-строкой), успех — snack «выбрана» или至少 маркер+note.
3. **Таймауты/сокет-ошибки поиска и дерева не пойманы** (`hf_browser.dart:81,103`: только `on HfHttpException`; `fetchText` кидает `TimeoutException`/`SocketException` — hf_client.dart:26-31 `.timeout(30s)`) — необработанная async-ошибка, UI молчит, кнопка не disabled; попутно нет никакого in-flight на сетевых кнопках. Fix: `catch (e)` → честный `_fetchError` (код/локализованный ключ); disabled+спиннер на время fetch.

Cheap wins в том же прогоне фиксера: `foregroundColor: colors.onError` на delete-confirm (models_screen.dart:78); `MergeSemantics` на VRAM-строке и `_slider`/`_segmented` (см. Pillar 5).

---

## Detailed Findings

### Pillar 1: Иерархия/типографика (3/4)

**Хорошо:** статус-карточка — канон закрыт: заголовок `titleMedium` + состояние `titleLarge` с цветом по состоянию + liveRegion (`dashboard_screen.dart:64-80`), T8 доставлен. Новые экраны — секционные заголовки `titleMedium` («установленные модели», «каталог», «прямая ссылка») поверх того же токена; ноль литеральных размеров/весов в новых файлах; tabular figures на всех обновляющихся числах (readout слайдеров, размеры, прогресс); иерархия карточки каталога имя→muted-мета корректна.

- **WARNING — hint-ключ как заголовок секции.** Заголовок секции поиска — `l10n.modelSearchHint` («поиск на hugging face», .arb:450), он же hintText поля (`hf_browser.dart:258-261` и :270) — на экране одна и та же строка дважды, и заголовок читается как плейсхолдер, а не как существительное-заголовок как у соседей. Fix: отдельный ключ-существительное (`modelSearchTitle`), hint оставить полю.
- **MINOR — активная модель без визуального веса.** Помечена только muted-строкой «активна» (`models_screen.dart:177-181`); она главный факт экрана — можно дать `titleMedium`/primary маркер.

### Pillar 2: Цвет/тон (3/4)

**Хорошо:** error повсюду семантичен — статус ошибок (`settings_screen.dart:523-524`, `models_screen.dart:138`, `hf_browser.dart:314,241`), деструктивные кнопки строго по канону Outlined+error-пара (`models_screen.dart:196-199`, `hf_browser.dart:469-472`, `history_screen.dart:218-221`); баннер errorContainer/onErrorContainer; refusals красным, note — muted (`settings_screen.dart:518-526`); VRAM-over в error — обоснован (объясняет disabled). Греп hex: 0 в новых файлах.

- **WARNING — confirm-delete без onError-пары.** `FilledButton.styleFrom(backgroundColor: colors.error)` (`models_screen.dart:78`) — foreground остаётся onPrimary поверх error-красного: неверная тон-пара M3 (нужен `onError`), контраст совпадение, а не контракт. Fix: `foregroundColor: colors.onError` (одна строка). Заодно решить канон: деструктивный confirm = Filled+error/onError — ок, но зафиксировать как паттерн.
- **MINOR — один факт, два тона.** VRAM-over в каталоге — отдельная красная строка (`hf_browser.dart:239-242`), в дереве — суффикс внутри muted-подписи (`:382-384`). Fix: единая упаковка (красный суффикс или отдельная строка в обоих местах).
- **MINOR — плотность primary.** 6 каталог-строк с filled «скачать» + filled «искать» + filled «скачать» прямой ссылки (`hf_browser.dart:169,299,427`) — на одном экране 8 filled-кнопок; 60/30/10 напряжён. Fix: каталог-строки → `FilledButton.tonal`, filled оставить прямой ссылке/поиску.

### Pillar 3: Раскладка/отступы (2/4)

**Хорошо:** фаза-4 часть shell — оба слота заняты, IndexedStack сохраняет состояние экранов; `SizedBox(height: 20)` между секциями браузера — читаемая секционность; bottom-padding через `padding: EdgeInsets.all(16)` на settings/models — наследие №7 не регресснуло; мин-окно 800×600, `SingleChildScrollView` без overflow.

- **WARNING — нулевой вертикальный ритм внутри групп формы.** `_group` = Column[12, title, 8, …children] (`settings_screen.dart:152-160`), а дети (слайдеры, сегменты, свитчи) не разделены НИЧЕМ: readout одного слайдера касается лейбла следующего (`:178-202` — Column без SizedBox внизу; vertical:4 есть только у `_field`, `:258-259`). 17 контролов одной кучей; группы отделены теми же 12px, что и всё остальное — группировка держится только на заголовках. Fix: `SizedBox(height: 12)` после каждого не-ListTile-ребёнка группы (или обёртка-паддинг), межгрупповой отступ — 20-24.
- **WARNING — дрейф карточного скелета.** Канон фаз 1–3: `Card > Padding(16)` + межкарточно 12 (`dashboard_screen.dart:58-113`). Фаза 4: карточки моделей/каталога `Padding(12)` (`models_screen.dart:169`, `hf_browser.dart:219`) и межкарточный зазор = дефолтная маржа Card 4px (`models_screen.dart:164-208`, каталог `:213-256`) — список визуально плотнее остального приложения без решения. Fix: принять «списочный» вариант осознанно (Padding 12 + явный 8-зазор) или вернуть 16/12.

### Pillar 4: Консистентность/паттерны (3/4)

**Хорошо:** кнопочная матрица по канону — confirm-диалоги везде TextButton(отказ)+FilledButton(подтверждение) (`settings_screen.dart:111-120`, `models_screen.dart:33-42`); «выбрать»=tonal vs «удалить»=Outlined+error (`models_screen.dart:186-203`); отмена загрузки Outlined+error, «показать ещё»/назад TextButton (`hf_browser.dart:333,352`); M3-формы: Slider/SwitchListTile/SegmentedButton/DropdownButtonFormField; wire vs display формат слайдеров честно разведён (`settings_screen.dart:171-192`); оба слота shell; кириллический гейт 0 — все новые строки в .arb с @-описаниями.

- **WARNING — класс «строка вне l10n» воссоздан в фазе 4.** Фаза 2 закрыла «команда не выполнена» → .arb; теперь: `'delete failed: …'` в snackbar пользователю (`model_store.dart:36` → `models_screen.dart:101`), `'download failed: invalid name'/'…: <runtimeType>'` (`downloads.dart:85,164,172` → snackbar `hf_browser.dart:153`), `'HTTP 404'` как тело ошибки (`hf_browser.dart:86,108`). Это не вербатим демона — это собственные строки приложения,英語. Fix: локализованные ключи (`modelDeleteFailed`, `modelHttpError(code)`, `modelDownloadInvalidName`), вербатим-деталь второй строкой.
- **MINOR — снек-упаковка ошибок разная.** История: error-snackbar из двух строк (`history_screen.dart:178-201`); браузер: одна строка «загрузка не удалась: <eng>» (`hf_browser.dart:153`); settings — liveRegion. Свести к одному скелету (заголовок+тело).

### Pillar 5: Доступность (3/4)

**Хорошо:** MergeSemantics+Semantics(label) на ВСЕХ текстовых полях, включая прямую ссылку (`hf_browser.dart:405-417`, `:264-276`, `settings_screen.dart:260-275`, `history_screen.dart:33-59`) — пункт оркестратора выполнен; настоящие Slider'ы = клавиатура/AT из коробки (класс blocker'а фазы 2 закрыт: calibration, вся форма — настоящие слайдеры); liveRegion ×4; AlertDialog — дефолтный focus trap, Esc = отказ (confirmed=null → decline); «активна» — текст, не только цвет; ensureSemantics-пробы в тестах фазы 4 (history_screen_test.dart:142,365); модельный dropdown — labelText внутри декорации.

- **WARNING — слайдеры/сегменты анонсируют значение без имени.** `_slider` и `_segmented` (`settings_screen.dart:178-247`) и VRAM-строка (`hf_browser.dart:190-211`) не имеют MergeSemantics-связи с лейблом: tab-обход слышит «0.35», «1500 мс», «paste» — какой из 10 слайдеров регулирует что, AT-пользователю неясно. VRAM-слайдер тоже: label «больше лимита…»-строки рядом, но узел слайдера безымянный. Fix: обернуть Column в `MergeSemantics` с `Semantics(label: <имя настройки>)` на слайдере (значение Slider объявит сам), для сегментов — `Semantics(label: <имя поля>)`.
- **WARNING — ошибки fetch не в liveRegion.** `_fetchError` поиск/дерево (`hf_browser.dart:309-316`) — только визуальный красный; для AT провал поиска/открытия репозитория молчит (у download ошибок snackbar есть, у сетевых fetch — нет). Fix: `Semantics(liveRegion: true)` на строке ошибки.
- **MINOR — прогресс МБ внутри liveRegion.** Панель загрузки целиком liveRegion (`hf_browser.dart:440-441`), счётчик МБ обновляется каждый чанк — риск спама AT; канон фазы 2 — `excludeSemantics` на быстрых числах (dashboard делает, `dashboard_screen.dart:87-108`). Fix: вынести прогресс-текст из liveRegion, оставить liveRegion только на факте начала/завершения.

### Pillar 6: Состояние/отклик (2/4)

**Хорошо:** reload-спиннер + disabled всей ENGINE-группы во время перезагрузки (`settings_screen.dart:419-493`), включая повторную пометку language (IN-06); один set на жест, wire-значения без юнитов; download-панель: детерминированный/недетерминированный прогресс + отмена + snackbar done/cancelled/failed, «уже есть» без перезаписи и без лживого снека (`hf_browser.dart:146-155`, `downloads.dart:97`); delete: тройной guard по живому снапшоту + обязательный refetch + snackbar с честным телом (`models_screen.dart:86-115`); empty-состояния честные («моделей нет — скачай из каталога ниже» — с действием; «файлов .bin нет»; «ничего не нашлось»); «показать ещё» не дописывает дублей (`hf_browser.dart:66-74`).

- **WARNING (top-1) — форма из сфабрикованных дефолтов при провале fetch.** `settings_screen.dart:282-283` — нет веток AsyncError/AsyncLoading: при недоступном демоне экран показывает полностью «заполненную» форму (нули, ctrl+v, …) без единого индикатора; пользователь верит, что редактирует текущие значения. ModelsScreen тот же провал обрабатывает (error-карточка + retry, `models_screen.dart:128-152`) — асимметрия честности внутри одной фазы. Fix: как в top-1.
- **WARNING (top-2) — отказ ENGINE при выборе модели молчит.** `models_screen.dart:50` — `reply.error` (вербатимный отказ main.rs:752) выбрасывается; диалог закрылся, тишина. Fix: snackbar с `reply.error` по образцу настроек.
- **WARNING (top-3) — сетевые исключения поиска/дерева вне catch.** `hf_browser.dart:56-89,91-111` ловят только `HfHttpException`; `TimeoutException` (30s, `hf_client.dart:26-31`), `SocketException`, ошибки DNS уходят в unhandled async error — UI без отклика и без disabled. Fix: `on Exception`-ветка → локализованный `_fetchError`.
- **MINOR — set без in-flight.** `_commit` (`settings_screen.dart:103-150`) не гасит форму на время IPC: двойной жест в полёте = два set (мс на локальном IPC, но таймаут сокета даст «двойное»); сетевые search/retry — тем более (см. top-3).
- **MINOR — VRAM-readout заморожен на время drag.** `onChanged: (_) {}` (IN-05, осознанно) — бегунок движется, число и label рядом (`hf_browser.dart:205-210`) стоят на старом значении до отпускания. Fix: писать pending в локальное поле (не в notifier) в onChanged — жест всё равно один set.

---

## Registry Safety

Неприменимо: Flutter-проект, `components.json` отсутствует; сторонних UI-блоков из registries нет; зависимостей в фазе 4 не добавлялось (заморозка pubspec подтверждена 04-SUMMARY).

## Files Audited

- app/lib/features/settings/settings_screen.dart (17-ключевая форма, confirm, reload)
- app/lib/features/models/models_screen.dart (список, guard'ы, dialog'и)
- app/lib/features/models/hf_browser.dart (каталог/поиск/дерево/VRAM/ссылка/загрузка)
- app/lib/features/models/model_store.dart (имена, canDelete, deleteModel)
- app/lib/features/shell/shell.dart (4 слота, баннер)
- app/lib/features/dashboard/dashboard_screen.dart (T8: статус-заголовок — сверка наследия)
- app/lib/features/history/history_screen.dart (T8: bottom-padding, suffix, empty — сверка)
- app/lib/core/theme_ext.dart, main.dart, test/shell_test.dart, test/fake_dicta_engine.dart (3+1 регистраций)
- app/lib/core/hf/hf_client.dart, core/downloads.dart (ошибки/таймауты)
- app/lib/l10n/app_ru.arb (новые ключи фазы 4)
- .planning/phases/04-nastroyki-brauzer-modeley/04-SUMMARY.md, 04-01-SUMMARY.md (контракт)

## Вход планирования фазы 5 (наследие UI)

1. **MergeSemantics-связь лейбл→контрол** для слайдеров/сегментов формы настроек и VRAM-строки (13+ безымянных контролов для AT).
2. **Локализация собственных ошибок**: `delete failed:`/`download failed: <runtimeType>`/`HTTP NNN` → ключи .arb с вербатим-деталью второй строкой; единый скелет error-snackbar (заголовок+тело).
3. **Ритм формы настроек**: межконтрольные 12, межгрупповые 20–24; решение по карточному скелету списков (12+8 vs канон 16+12) — зафиксировать в PATTERNS.md.
4. **In-flight на сетевых кнопках** (search/tree/retry) + `catch-all` не-HTTP исключений; VRAM-readout live на drag через локальный pending.
5. **Заголовок секции поиска** — отдельный ключ-существительное (не hint); активная модель — визуальный вес (titleMedium/primary).
6. **liveRegion на fetch-ошибках браузера + excludeSemantics на МБ-счётчике** (AT-спам против AT-тишины — по образцу dashboard).
7. **Свёртка длинных записей истории** (перенос из фазы 3 №5, подтверждён CONTEXT:33).
8. **Канон деструктивного confirm** (Filled+error/onError) — записать в PATTERNS.md после гашения onError; density primary на экране моделей (каталог → tonal).

**Аудитор:** gsd-ui-auditor, 2026-11-22. Код не менялся, коммитов нет.

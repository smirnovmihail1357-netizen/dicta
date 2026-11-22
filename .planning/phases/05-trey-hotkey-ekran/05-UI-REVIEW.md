# Phase 5 — UI Review (retroactive, 6-pillar)

**Audited:** 2026-11-22
**Baseline:** abstract 6-pillar standards (UI-SPEC.md не существует) + каноны 02/03/04-UI-REVIEW.md (ThemeExtension mutedOf, snackbar-канон «заголовок+тело», tabular figures, destructive-confirm, MergeSemantics на полях, liveRegion на error-карточках, 3+1 регистраций AppTextStyles)
**Scope:** дифф b81178b..HEAD по app/ (коммиты 74dcce2..23dafbd, включая фикс-серию 331f79f..23dafbd): HK-карточка, свёртка истории, модели/браузер (наследие Task 5), TrayController snackbar (WR-01), shell selectedTabProvider, main/bootstrap; трей-меню и SNI-иконка — нативные поверхности вне flutter-дерева, оценены по SNI-контракту из SUMMARY/05-REVIEW-DISPOSITION
**Screenshots:** not captured — Flutter Linux desktop без web-таргета (прецедент 02-UI-REVIEW:6); кодовый аудит
**Interaction captures:** off (workflow.ui_interaction_capture не включён; живой DBusMenu-smoke зафиксирован в 05-01-SUMMARY:159-179 и WR-03-доказательстве tray.dart:107-118)
**Качество базы:** flutter analyze «No issues found!» (прогнан этим аудитом); батарея 178/178 и cyrillic 0 — по гейтам 05-REVIEW/05-REVIEW-DISPOSITION (не перегонялась по условию); 3+1 регистраций AppTextStyles целы (main.dart:65,70 + shell_test.dart:30 + fake_dicta_engine.dart:220)

---

## Pillar Scores

| Pillar | Score | Key Finding |
|--------|-------|-------------|
| 1. Иерархия/типографика | 3/4 | Заголовок HK-карточки — существительное titleMedium, вес активной модели и заголовок поиска закрыты; но под заголовком карточки плоский одно-весовой поток (5 текст-блоков одним тоном), свёртка режет текст без ellipsis |
| 2. Цвет/тон | 3/4 | error повсюду семантичен, ноль hex-литералов в lib; но рестарт-предупреждение — errorContainer-баннер для НЕ-error уведомления, delete-fail снек без error-тона (канон-расхождение с history/models-select) |
| 3. Раскладка/отступы | 2/4 | Межгрупповые 20 закрыты (_group :157); но внутригрупповой нулевой ритм формы (обещание 04-UI-REVIEW:126 «межконтрольные 12») так и не закрыт — readout слайдера прилипает к лейблу следующего |
| 4. Консистентность/паттерны | 3/4 | Класс «строка вне l10n» погашен полностью (все 3 литерал-гейта 0); но DE-команды карточки не копируемы, а tray-снек расходится с error-snackbar-скелетом (тело без заголовка, без тона) |
| 5. Доступность | 4/4 | MergeSemantics-покрытие полное (слайдеры/сегменты/VRAM/поля/текущая комбинация), liveRegion на _ConfigError (23dafbd, тест :94) и fetch-ошибках, excludeSemantics МБ, тач-таргеты 48dp (M3 padded) |
| 6. Состояние/отклик | 3/4 | WR-01/WR-02 гварды честные и протестированы; но retry-in-flight невидим (IN-04, отложено), свёртка без анимации, у скрытого окна ошибки тихие (задокументировано, но поверхность не решена) |

**Overall: 18/24** (фаза 2: 17/24, фаза 3: 19/24, фаза 4: 16/24).

Траектория 17 → 19 → 16 → 18: всё обещанное наследие фазы 4 закрыто, кроме одного пункта
(внутригрупповой ритм — см. таблицу ниже), и новые поверхности (HK-карточка, tray-канал
ошибок) в целом по канону; regression — узкий: три WARNING на новых поверхностях и
один незакрытый долг наследия. BLOCKER нет.

---

## Сверка наследия фазы 4 (04-UI-REVIEW:120-131)

| Обещание | Статус | Доказательство (живой код) |
|---|---|---|
| №2 Локализация `delete failed:` | CLOSED | model_store.dart:36-39 возвращает голый `error.message` (данные); models_screen.dart:134-135 заголовок `errDeleteFailed` + вербатим-деталь; тест models_screen_test.dart:195 |
| №2 `HTTP NNN` | CLOSED | hf_browser.dart:95,128 → `errHttp(code)` (.arb:625); оба call-site передают `statusCode.toString()` (принятая девиация gen-l10n) |
| №2 `} GB`/`} MB` литералы | CLOSED | hf_browser.dart:148-153 `_sizeLabel` → `modelSizeGb/modelSizeMb` placeholders (.arb:640,647); гейт-греп 0 |
| №4 Заголовок поиска (отдельный ключ-существительное) | CLOSED | hf_browser.dart:296-299 `modelSearchTitle` («поиск моделей», .arb:617); `modelSearchHint` остался только hintText (:308) |
| №4 Вес активной модели | CLOSED | models_screen.dart:213-224 `titleMedium` + `colorScheme.primary` на имени активной |
| №5 Свёртка длинных записей (>160) | CLOSED | history_screen.dart:266-302 `maxLines: 3` + histExpand/histCollapse с ключами; SelectableText сохранён в обоих режимах (HIST-03); undo не тронут (:225-247); тест history_screen_test.dart:456-487 |
| №6 liveRegion на fetch-ошибках браузера | CLOSED | hf_browser.dart:364-372 `Semantics(liveRegion: true)` на `modelFetchError` |
| №6 excludeSemantics МБ-счётчика | CLOSED | hf_browser.dart:519-531 счётчик вне liveRegion, `excludeSemantics: true`, tabular figures |
| №7 (з) Ритм межгрупповой 20-24 | CLOSED | settings_screen.dart:156-157 `_group` → `SizedBox(height: 20)` |
| №7 (з) Ритм МЕЖКОНТРОЛЬНЫЙ 12 | **NOT CLOSED** | settings_screen.dart:160 дети `_group` без разделителей: `_slider` (:186-210) кончается readout'ом без нижнего отступа → readout прилипает к лейблу следующего контроля (17 контролов, :382-442). 04-UI-REVIEW:126 обещал «межконтрольные 12, межгрупповые 20–24»; план Task 5 истолковал «12/8 уже канон» — но 12/8 в _group это отступы ЗАГОЛОВКА, не межконтрольные |
| №8 In-flight сетевых кнопок + VRAM-readout live | CLOSED (04-REVIEW-FIX, сверено) | `_fetchInFlight` disabled+спиннер (hf_browser.dart:41,66,106,112,137); `_pendingVram` локальный pending → живой readout на drag (hf_browser.dart:45,171,238-246) |
| №1 MergeSemantics слайдеров/сегментов/VRAM | CLOSED (04-REVIEW-FIX, сверено) | settings_screen.dart:183-212 (10 слайдеров), :240-263 (3 сегмента), hf_browser.dart:221-249 (VRAM); греп MergeSemantics в settings == 4 |
| №3 Отказ ENGINE при выборе модели | CLOSED (04-REVIEW-FIX, сверено) | models_screen.dart:52-74 snackbar «заголовок+вербатим», оба в error-тоне |
| onError-пара delete-confirm | CLOSED (04-REVIEW-FIX, сверено) | models_screen.dart:101-109 `backgroundColor: error` + `foregroundColor: onError` |

**Итог сверки:** 13 закрыто (9 — фазой 5, 4 — ранее 04-REVIEW-FIX и сверено живым кодом), 1 не закрыто (межконтрольный ритм).

---

## Top 3 «Погасить немедленно»

1. **Внутригрупповой нулевой ритм формы настроек** (`settings_screen.dart:160, :382-442` — обещание 04-UI-REVIEW:126) — 17 контролов одной кучей: readout «0.35» касается лейбла «автостоп» ниже; форма читается как стена. Fix: `SizedBox(height: 12)` после каждого не-ListTile-ребёнка `_group` (или `Padding(bottom: 12)` на обёртках `_slider`/`_segmented`) — один точечный прогон, rail-ассерты не затрагивает.
2. **DE-команды HK-карточки не копируемы** (`hotkey_card.dart:97-98` `_deLine` → `Text`) — единственный actionable-контент карточки («./systemd/install.sh && ./adapters/kwin/install.sh», `apply.sh '<Ctrl><Alt>v'`) нельзя выделить и скопировать. Канон «важные данные копируемы» уже жив (confConfigPathField readOnly+selectable, SelectableText истории). Fix: `SelectableText` в `_deLine` (или `SelectionArea` вокруг таблицы) — 1 строка.
3. **Tray-снек расходится с error-snackbar-каноном** (`tray_bootstrap.dart:148-152` против `history_screen.dart:181-205`, `models_screen.dart:56-70`) — снек показывает только тело, без заголовка `dashCommandFailed` и без error-тона. Fix: `Column[Text(l10n.dashCommandFailed, error-тон), Text(body, error-тон)]` — паритет скелета в третьем call-site.

---

## Detailed Findings

### Pillar 1: Иерархия/типографика (3/4)

**Хорошо:** заголовок HK-карточки — существительное `hkTitle` («глобальный хоткей», .arb:573) в titleMedium, тон соседних `_group`-заголовков; DE-таблица 4 статичных строк вербатим из README:223-234, ноль гадания о текущем DE; заголовок поиска теперь существительное «поиск моделей» (hf_browser.dart:296-299), hint остался полю (:308); активная модель получила titleMedium+primary (models_screen.dart:213-224); tabular figures на всех обновляющихся числах (readout'ы слайдеров settings:205-207, sizes/прогресс hf_browser:267-268,447-448,527-528, время истории history:111-115); ноль литеральных размеров/весов в новых файлах; rail-слоты не тронуты, нового пункта нет (settings_screen.dart:598-599 врезка внутрь группы «система»).

- **MINOR — плоский одно-весовой поток под заголовком карточки.** `hotkey_card.dart:73-90`: DE-строки, `hkInputGroupHint`, `hkEditHint` и лейбл «текущая комбинация» — все голый bodyMedium без дифференциации; канон приложения — вторичный текст в `AppTextStyles.mutedOf` (histEmptyHint history:79, engine-hints settings:500,579). Fix: `hkInputGroupHint`/`hkEditHint` → mutedOf.
- **MINOR — свёртка режет без ellipsis.** `history_screen.dart:289-292`: `SelectableText` не имеет overflow-параметра, maxLines:3 обрезает по строке без «…» — аффорданс несёт только кнопка ниже. Приемлемо; при желании fade-маска на третью строку.

### Pillar 2: Цвет/тон (3/4)

**Хорошо:** error повсюду семантичен — `_ConfigError` в `colors.error` (hotkey_card.dart:149), fetch-ошибка браузера в error (:370), VRAM-over (:279), статус-строка настроек по `_statusIsError` (settings:607-609); деструктивный канон цел: delete-confirm Filled+error/onError (models:101-109), delete/отмена загрузки Outlined+error-пара (models:241-247, hf_browser:534-539, history:217-224); греп hex в новых файлах lib/ = 0 (иконки — закоммиченные PNG-ассеты, детерминированы); отказы/notes разведены; темы Material You не тронуты (main.dart diff — только bootstrap/messengerKey).

- **MINOR — рестарт-предупреждение в error-тонах.** `hotkey_card.dart:80-88`: «изменение применится после рестарта демона» — штатная процедурная информация, а упакована как errorContainer/onErrorContainer-баннер — тот же тон, что у настоящих ошибок. Fix: `surfaceContainerHighest`-баннер или mutedOf-строка с info-иконкой; канон записать в PATTERNS.
- **MINOR — delete-fail снек без error-тона.** `models_screen.dart:128-138`: двухчастный снек собран по скелету, но обе строки без `TextStyle(color: colors.error)` — тогда как history inject-failure (history:188-201) и models select-failure (:60-67) красят обе. Fix: error-тон на обе строки.

### Pillar 3: Раскладка/отступы (2/4)

**Хорошо:** межгрупповой ритм 20 закрыт (`_group` :156-157); HK-карточка внутри группы имеет собственный внутренний ритм 8/12 (:53,70,77,79,89); Padding карточки 12 консистентен со «списочным» каноном (осознанное решение фазы 4); history bottom-padding и карточный скелет 16 не регресснули; IndexedStack/rail не тронуты; окно 800×600, SingleChildScrollView — без overflow.

- **WARNING (top-1) — внутригрупповой нулевой ритм НЕ закрыт.** `settings_screen.dart:160` — дети `_group` идут подряд без разделителей; `_slider` (:186-210) кончается muted-readout без нижнего отступа → readout прилипает к лейблу следующего слайдера; 17 контролов одной стеной. Fix: 12px после каждого не-ListTile-ребёнка.
- **MINOR — первая группа получает 36px сверху.** `_group` открывается `SizedBox(20)` (:157) и для ПЕРВОЙ группы тоже → 16 (паддинг скролла :378) + 20 до первого заголовка; остальные секции приложения (models :164) не имеют лишнего верхнего зазора. Fix: `SizedBox` только между группами.

### Pillar 4: Консистентность/паттерны (3/4)

**Хорошо:** класс «строка вне l10n» погашен полностью — все три литерал-гейта = 0, замены через .arb с вербатим-деталью второй строкой (models:134-136); 24 якорных ключа с @-описаниями, кириллический гейт 0; reuse-канон: `modelFetchRetry` переиспользован в `_ConfigError` (hotkey_card:154), `dashCommandFailedBody` — в tray-снеке, `appTitle` — tooltip трея (ноль новых ключей); свёртка — stateful-toggle без ExpansionTile; undo-канон истории цел; hotkey ∉ set-команд, writeAs/openWrite = 0; snackbar-упаковка «заголовок+тело» в delete/select/unified.

- **WARNING (top-2) — DE-команды не копируемы.** `hotkey_card.dart:97-98`: `_deLine` → обычный `Text`; четыре установочные команды — единственный actionable-контент карточки. Fix: `SelectableText` в `_deLine` или `SelectionArea` вокруг таблицы.
- **WARNING (top-3) — tray-снек вне error-snackbar-скелета.** `tray_bootstrap.dart:148-152`: только тело без заголовка/тона. Fix: как в top-3.

### Pillar 5: Доступность (4/4)

**Хорошо:** MergeSemantics+Semantics(label) на ВСЁМ: `_CurrentHotkey` объявляет «текущая комбинация: <значение>» целиком (hotkey_card:116-127); `_ConfigError` — liveRegion (23dafbd, тест hotkey_card_test.dart:94), fetch-ошибки браузера — liveRegion (hf_browser:364-372), МБ-счётчик — excludeSemantics (hf_browser:521-531); рестарт-предупреждение НЕ liveRegion — правильно (не transient, PATTERNS Analog 4); histExpand/histCollapse несут состояние в лейбле; ensureSemantics-пробы истории покрывают свёртку; тач-таргеты новых кнопок ≥48dp (M3 padded); tooltips трея через l10n.appTitle; ошибки — runtimeType/данные, не OS-строки.

- **INFO — IN-04 (retry-in-flight невидим) остаётся отложенным.** `hotkey_card.dart:57-61`: после ошибки tap «повторить» → AsyncLoading с сохранённым error → ошибка и retry-кнопка без индикатора. Disposition зафиксировал как косметику. Fix при желании: `if (config.hasError && !config.isLoading)`.

### Pillar 6: Состояние/отклик (3/4)

**Хорошо:** WR-01 закрыт честно (двойной гард, снек при видимом окне, скрытое окно — тишина с комментарием); WR-02: деградация в windowed-режим вместо краша старта (тест :110-127); HK-карточка: spinner на загрузке, честная ошибка чтения + retry, «не задан (off)» — штатное состояние (hotkey_card:110-114); свёртка — полный stateful-цикл с тестом; VRAM-readout живой на drag; undo intact; README ×2 документируют exit-при-записи (IN-02) и SNI-поведение; WR-03 закрыт живым DBus-замером IconPixmap (tray.dart:111-118).

- **WARNING-light/INFO — у скрытого окна ошибки тихие.** `tray_bootstrap.dart:141-146`: поверхность ошибки отсутствует в принципе при скрытом окне. Задокументировано, honest by design; UX-долг поверхности (иконка «ошибка»/tooltip-суффикс/auto-show при отказе) — кандидат в фазу 6+.
- **INFO — свёртка без анимации.** Plain-toggle легален (No Analog). Косметика.
- **INFO — порог 160 ≠ «не влезает в 3 строки».** Широкое окно: 200 символов влезают в 3 строки, кнопка уже показана. Безвредно, осознанное.

---

## Registry Safety

Неприменимо: Flutter-проект, `components.json` отсутствует; сторонних UI-блоков из registries нет; единственная новая зависимость — tray_manager 0.9.0 (плановая, легитимность верифицирована в 05-RESEARCH/05-REVIEW, lock-diff = только tray_manager-цепочка).

## Files Audited

- app/lib/features/settings/hotkey_card.dart (HK-01 карточка: провайдер, _CurrentHotkey, _ConfigError, DE-таблица, баннер)
- app/lib/features/history/history_screen.dart (_HistoryEntryText свёртка, undo, снеки)
- app/lib/features/models/hf_browser.dart (errHttp, _sizeLabel, liveRegion, excludeSemantics, VRAM pending)
- app/lib/features/models/models_screen.dart (вес активной, delete-fail снек, error-карточка)
- app/lib/features/models/model_store.dart (errDeleteFailed-рефактор)
- app/lib/features/settings/settings_screen.dart (_group ритм, врезка HotkeyCard, форма)
- app/lib/tray_bootstrap.dart (TrayController: гварды WR-01/WR-02, снек-канал)
- app/lib/core/tray.dart, core/window_shell.dart, core/config_path.dart, core/providers.dart, core/daemon_engine.dart
- app/lib/features/shell/shell.dart, app/lib/main.dart (selectedTabProvider, bootstrap, messengerKey)
- app/lib/l10n/app_ru.arb (24 новых ключа: tray 6 / hk 11 / наследие 7)
- app/test/tray_service_test.dart, hotkey_card_test.dart, history_screen_test.dart, models_screen_test.dart
- 05-01-PLAN.md Task 2-5, 05-REVIEW.md, 05-REVIEW-DISPOSITION.md, 04-UI-REVIEW.md, 05-01-SUMMARY.md

**Аудитор:** gsd-ui-auditor, 2026-11-22. Код не менялся, коммитов нет.

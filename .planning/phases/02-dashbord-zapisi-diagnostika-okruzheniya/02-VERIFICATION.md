---
phase: 02-dashbord-zapisi-diagnostika-okruzheniya
verified: 2026-11-22T11:55:06Z
status: human_needed
score: 9/11 must-haves verified
covered_files:
  - .planning/phases/02-dashbord-zapisi-diagnostika-okruzheniya/02-01-PLAN.md
  - .planning/phases/02-dashbord-zapisi-diagnostika-okruzheniya/02-01-SUMMARY.md
  - .planning/phases/02-dashbord-zapisi-diagnostika-okruzheniya/02-SUMMARY.md
  - app/lib/core/daemon_engine.dart
  - app/lib/core/engine.dart
  - app/lib/core/providers.dart
  - app/lib/features/dashboard/calibration_card.dart
  - app/lib/features/dashboard/dashboard_screen.dart
  - app/lib/features/dashboard/diagnostics_card.dart
  - app/lib/features/dashboard/dicta_control.dart
  - app/lib/features/dashboard/last_text_card.dart
  - app/lib/features/dashboard/level_history.dart
  - app/lib/features/shell/shell.dart
  - app/lib/l10n/app_ru.arb
  - app/lib/l10n/generated/app_localizations.dart
  - app/lib/l10n/generated/app_localizations_ru.dart
  - app/test/calibration_test.dart
  - app/test/daemon_engine_framing_test.dart
  - app/test/dashboard_buttons_test.dart
  - app/test/dashboard_state_test.dart
  - app/test/diagnostics_test.dart
  - app/test/fake_dicta_engine.dart
  - app/test/fake_dicta_engine_test.dart
  - app/test/last_text_test.dart
  - app/test/level_history_test.dart
  - app/test/shell_test.dart
  - app/test/widget_test.dart
  - app/tool/diag_smoke.dart
covered_digest: "v4:sha256:26e72a44cfbe1d56e6921505e54c3f9ed9836fc7694a50c432cd3cae26bc203b"
behavior_unverified: 2
behavior_unverified_items:
  - truth: "Живой микрофон: столбик уровня и гистограмма живые на каждом 10-Гц тике, тишина/речь визуально различимы (DASH-01/DASH-04 визуальная полировка)"
    test: "cd app && flutter run -d linux; говорить/молчать при записи"
    expected: "Столбик уровня и столбики гистограммы двигаются на каждом тике поллинга; тишина и речь визуально различимы; оттенки порогов читаемы"
    why_human: "Backstop-истина (verification: backstop): реальный аудиопоток и визуальная полировка — headless-тесты гоняют фейковые тики, живую акустику не видят"
  - truth: "Живая калибровка переживает перезапуск демона; inject вставляет в реальное активное окно; путь хоткея вставляет в чужое окно"
    test: "Запись → тишина → двигать отметку → отпустить → systemctl --user restart dictad; затем диктовка по хоткею и кнопка «вставить заново»"
    expected: "Порог применяется к живой записи сразу по отпусканию и остаётся после рестарта демона (config.toml); кнопка вставляет в активное окно (само приложение при клике); хоткейный путь вставляет в чужое окно"
    why_human: "Backstop-истина (verification: backstop): сохранение в config.toml на стороне демона, реальный фокус окна и глобальный хоткей — вне досягаемости widget-тестов и греп-проверок"
human_verification:
  - test: "cd app && flutter run -d linux; говорить/молчать — живой уровень и гистограмма (DASH-01/DASH-04)"
    expected: "Столбик живой ~10 Гц, гистограмма наполняется только при записи, тишина/речь различимы, отметка порога видна"
    why_human: "Backstop-истина: визуальная полировка на реальном аудиопотоке"
  - test: "Живая калибровка: запись → тишина → драг отметки → отпускание; затем systemctl --user restart dictad"
    expected: "Порог применяется по отпусканию (одна команда set, читается в config.toml/панели) и переживает рестарт демона"
    why_human: "Backstop-истина: реальная акустика + персистентность на стороне демона"
  - test: "Inject в реальное окно: фокус в стороннее текстовое поле, диктовка по хоткею; затем кнопка «вставить заново» из приложения"
    expected: "Кнопка вставляет в активное окно (при клике — само окно приложения, подпись честная); кросс-оконный путь — хоткей демона"
    why_human: "Backstop-истина: реальный фокус окна и системный clipboard-restore — вне headless-тестов"
  - test: "Ручные строки чеклиста (DIAG-01): dictactl toggle и живая диктовка вставки"
    expected: "Хоткеей-путь и вставка проверены руками; приложение само их не дёргает (закреплено тестом — строки шлют ноль команд)"
    why_human: "Backstop-истина: пункты помечены «проверьте руками» по протоколному замку (get hotkey не существует)"
  - test: "Human sign-off 8 judgment-tier prohibitions (протокол заморожен, нулевые зависимости, кириллица только в .arb, без транспорта в features/, jsonEncode-only, без хоткеев, без логирования payload, UI-честность)"
    expected: "Нарушений не найдено (LLM-вердикт ниже чист по всем восьми — но подпись человека обязательна по escalation-gate контракту)"
    why_human: "Judgment-tier prohibitions не могут быть авторитативно машинно-верифицированы"
---

# Phase 2: Дашборд записи + диагностика окружения Verification Report

**Phase Goal:** Пользователь видит и управляет записью из приложения, калибрует порог тишины и понимает здоровье своего окружения
**Verified:** 2026-11-22T11:55:06Z
**Status:** human_needed
**Re-verification:** No — initial verification

**Method note:** вся автоматика ниже ПЕРЕИСПОЛНЕНА верификатором в этой сессии против живого демона (сокет `/run/user/1000/dicta.sock`, `DIAG SMOKE state=idle connected=true`). SUMMARY.md/REVIEW.md рассматривались как утверждения, не как доказательства. Сверенная база: `git diff --quiet 2714e27 HEAD -- crates/` и `-- app/pubspec.yaml app/pubspec.lock` — оба пустые (переисполнено); все 10 коммитов фазы (603d410..3fcd6ac) — предки HEAD (merge-base проверен поштучно).

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence (produced this session) |
|---|-------|--------|----------------------------------|
| 1 | `cd app && flutter test` exits 0 (51 тест: 13 фазы 1 + 38 фазы 2, включая 3 ревью-фикса и UI-долг) и `flutter analyze` — «No issues found!» | ✓ VERIFIED | Переисполнено: `+51: All tests passed!` (2 c); analyze → «No issues found! (ran in 0.5s)» |
| 2 | DASH-01: `dashboard_state_test` ведёт РЕАЛЬНЫЙ `DictaStatusNotifier` (перекрыт только engineProvider, тики 100 мс): три метки, уровень бара, mm:ss из base.seconds; неизвестный state → idle-fallback; уровень-история пушится только тиками записи и чистится на повторном входе | ✓ VERIFIED | `test/dashboard_state_test.dart:24-47` — метки «состояние: ожидание/запись/распознавание», `LinearProgressIndicator.value`, `время 01:05` (65.3 с → `_formatSeconds`, dashboard_screen.dart:189-190 — локального счётчика нет); unknown-state fallback — framing_test:55; история — state_test:60-90 (clear при re-entry, нули вне записи не пишутся — ref.listen в dashboard_screen.dart:39-46); весь suite зелёный в этом прогоне |
| 3 | DASH-02: честная матрица кнопок × {idle, recording, transcribing, disconnected}; cancel недоступна при transcribing; «распознаю…»+спиннер гаснут следующим idle-тиком; команда в полёте дизейблит кнопку | ✓ VERIFIED | `test/dashboard_buttons_test.dart`: «start/stop/cancel matrix across all four states», «spinner dies on the next polled idle tick», «in-flight stop disables its button»; матрица в коде — dashboard_screen.dart:100-152 (canStart/canStop/canCancel от `dictaStatusProvider` + `commandInFlight`, cancel при transcribing = null); спиннер привязан к `status.state == transcribing` (поллинг), не к Future |
| 4 | DASH-03: текст ТОЛЬКО из ответов stop/inject; копирование через шов по клику; inject ровно текст пользователя через jsonEncode; пустой текст — кнопки недоступны; ошибки демона дословно | ✓ VERIFIED | `dicta_control.dart:94,136` — единственные записи `lastText` (из `snapshot.text` stop/inject; grep `status.text` по lib/features → 0); `last_text_test.dart`: «honest empty state, then text from the stop reply», «copy calls the seam once», «inject sends exact text once, disables while in flight», «empty captured text keeps both actions disabled»; `buildInjectLine` = jsonEncode (engine.dart:99-100), round-trip — framing_test:63-79; clipboardWriterProvider → Clipboard.setData — единственный call-site last_text_card.dart:55 за кликом |
| 5 | DASH-04: одна `set('min_level','0.42')` на отпускание (формат {:.2}), ноль во время драга; отметка из settings-ответа и канонична ПОСЛЕ set (CR-01 fix); калибровка не блокируется при записи; a11y-слайдер | ✓ VERIFIED | `test/calibration_test.dart`: «one set(0.42) on release, zero during the drag», «marker starts at the settings min_level», «marker follows the canonical snapshot AFTER a successful set» (тест ловит именно CR-01-класс бага), «calibration is not blocked while recording», «accessible slider previews locally, then commits exactly one»; код — calibration_card.dart:71-95 (`invalidate` + `await` вместо `.value` после refresh), toStringAsFixed(2), Slider с Semantics (UI-долг 3fcd6ac) |
| 6 | DIAG-01: зелёные/нейтральные/красные строки из status/settings; ручные строки без команд демону; «перепроверить» = один poll + один рефетч; подсказка запуска при disconnected | ✓ VERIFIED | `test/diagnostics_test.dart`: «connected and model ready: three green automatic rows», «reloading is a neutral state, not an error», «disconnected: red rows and the daemon start hint», «recheck performs one poll and one settings refetch»; diagnostics_card.dart:35-63 (connected→primary/error, reloading→нейтральная, модель из settingsSnapshotProvider), manual-строки diagHotkeyManual/diagInjectManual — только текст |
| 7 | `dart run tool/diag_smoke.dart` по живому демону: `DIAG SMOKE state=<…>` + `INJECT-REJECTED-AS-EXPECTED`, exit 0, ноль побочных эффектов | ✓ VERIFIED | Переисполнено в этой сессии: обе строки напечатаны, exit 0; smoke шлёт ровно `inject(text: '')` (diag_smoke.dart:16) — демон отвергает до вставки |
| 8 | Cyrillic gate печатает ровно 0 | ✓ VERIFIED | Переисполнено: `bash tool/cyrillic_gate.sh` → `0`, exit 0 |
| 9 | Единая фабрика фейков: `implements DictaEngine` в test/ == 1 | ✓ VERIFIED | Переисполнено: grep → `1`; обе инлайн-копии фазы 1 слиты в fake_dicta_engine.dart |
| 10 | Живой микрофон: столбик и гистограмма живые, тишина/речь визуально различимы | ⚠ HUMAN NEEDED (backstop) | Код и поллинг проверены (ref.listen → push только в recording), но это заявленная `verification: backstop`-истина — реальный аудиопоток headless не подтверждается |
| 11 | Живая калибровка применяется и переживает рестарт демона; inject в реальное окно; путь хоткея в чужое окно | ⚠ HUMAN NEEDED (backstop) | Механика set/config.toml на стороне демона и реальный фокус окна — `verification: backstop`; виджет-тесты подтверждают жест и команду, не живое окно |

**Score:** 9/11 truths verified (2 backstop-истины маршрутизированы человеку; см. Human Verification Required)

### Roadmap Success Criteria

| SC | Criterion | Status | Evidence |
|----|-----------|--------|---------|
| 1 | Состояние демона + живой уровень ~10 Гц + время записи | ✓ code-verified / ⚠ живой звук = human (truth 10) | Truths 2: 100-мс поллинг (providers.dart:45), три метки, бар уровня, mm:ss из base.seconds |
| 2 | Кнопки старт/стоп/отмена; отмена честно недоступна при transcribing («распознаю…»), спиннер до Idle | ✓ VERIFIED | Truth 3: матрица × 4 состояний + спиннер до idle-тика закреплены widget-тестами |
| 3 | Последний текст виден; копировать или вставить заново (`inject`) | ✓ VERIFIED / ⚠ живое окно = human (truth 11) | Truth 4: текст только из action-ответов; copy-шов; inject jsonEncode 120 с |
| 4 | Графическая калибровка min_level: гистограмма живого уровня, подвижная отметка, применение по отпусканию | ✓ VERIFIED / ⚠ живая акустика = human (truths 10-11) | Truth 5: одна команда на жест, {:.2}, канонический маркер после set, не блокируется при записи, a11y-слайдер |
| 5 | Onboarding-чеклист: зелёные/красные пункты (демон/сокет/хоткей-путь/модель/вставка) с подсказкой починки | ✓ VERIFIED / ⚠ ручные строки = human | Truth 6 + живой diag_smoke: два авто-пункта из поллинга, модель из settings, хоткей/вставка — нейтральные ручные с подсказками (`dictactl toggle`), подсказка `systemctl --user start dictad` при disconnected |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|------------|-------------|---------|---------|
| DASH-01 | 02-01-PLAN.md | Состояние/живой уровень/время записи | ✓ SATISFIED (автоматика) / ⚠ живой звук backstop | Truths 2, 10 |
| DASH-02 | 02-01-PLAN.md | Старт/стоп/отмена; честная отмена при transcribing; спиннер до Idle | ✓ SATISFIED | Truth 3 |
| DASH-03 | 02-01-PLAN.md | Последний текст, копировать, вставить заново | ✓ SATISFIED (автоматика) / ⚠ реальное окно backstop | Truths 4, 11 |
| DASH-04 | 02-01-PLAN.md | Графическая калибровка порога тишины | ✓ SATISFIED (автоматика) / ⚠ живая акустика backstop | Truths 5, 10, 11 |
| DIAG-01 | 02-01-PLAN.md | Onboarding-чеклист диагностики окружения | ✓ SATISFIED (автоматика) / ⚠ ручные пункты backstop | Truth 6 + diag_smoke |

Orphaned requirements: нет — REQUIREMENTS.md маппит на фазу 2 ровно DASH-01..04 + DIAG-01, план декларирует те же пять.

### Required Artifacts

| Artifact | Status | Details |
|----------|--------|---------|
| app/lib/features/dashboard/dashboard_screen.dart | ✓ VERIFIED | Статус-карточка (liveRegion, tabular figures), матрица кнопок M3, спиннер, ref.listen истории; ноль dart:io |
| app/lib/features/dashboard/level_history.dart | ✓ VERIFIED | Ring buffer ёмкость 200, push/clear, in-memory; юнит-тест порядка/обрезки |
| app/lib/features/dashboard/dicta_control.dart | ✓ VERIFIED | DictaControlState/Notifier; lastText только из stop/inject; clipboardWriterProvider; lastErrorCommand (WR-01) |
| app/lib/features/dashboard/last_text_card.dart | ✓ VERIFIED | SelectableText / честное пустое состояние; copy по клику; inject; ошибки дословно; снекбар успеха inject |
| app/lib/features/dashboard/calibration_card.dart | ✓ VERIFIED | CustomPainter-гистограмма + мышиный жест-шорткат + доступный Slider (3fcd6ac); CR-01 fix: invalidate + await |
| app/lib/features/dashboard/diagnostics_card.dart | ✓ VERIFIED | 5 строк чеклиста + «перепроверить» = poll + рефетч; AsyncLoading/AsyncError честно нейтральны/красны (IN-01/02) |
| app/lib/core/engine.dart | ✓ VERIFIED | stop → Future&lt;DictaStatus&gt;, inject({required text, String? app}), buildInjectLine (jsonEncode) |
| app/lib/core/daemon_engine.dart | ✓ VERIFIED | _snapshotCommand с _injectTimeout 120 с; status 2 с / settings 30 с не тронуты; on TimeoutException — честные ветки (WR-03) |
| app/lib/core/providers.dart | ✓ VERIFIED | settingsSnapshotProvider → SettingsReply.fromReply (единственный парсер); поллинг 100 мс |
| app/lib/features/shell/shell.dart | ✓ VERIFIED | IndexedStack index 0 = DashboardScreen(); _DashboardPlaceholder удалён (grep — 0 хитов) |
| app/tool/diag_smoke.dart | ✓ VERIFIED | Живой дым переисполнен: exit 0, только пустой inject |
| app/test/fake_dicta_engine.dart | ✓ VERIFIED | Единственная фабрика (implements == 1); крюки status/setCalls/injectTexts/stopGate/fetchSettingsReply/fetchCount + onSet/cancelReply/statusError/injectGate/setReply (rule-2 расширения) |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|----|--------|---------|
| shell.dart | dashboard_screen.dart | IndexedStack index 0 — экран смонтирован всегда | ✓ WIRED | shell.dart:82 `DashboardScreen()`; слушатель поллинга живёт при переключении вкладок |
| dashboard_screen.dart | level_history.dart | ref.listen: вход в recording → clear(), тик записи → push(level) | ✓ WIRED | dashboard_screen.dart:39-46; пуш вне записи запрещён |
| dicta_control.dart | engine.dart | start/stop/cancel/inject через engineProvider | ✓ WIRED | Транспорт только в core/; grep dart:io по lib/features → 0 |
| engine.dart | crates/dictad (замороженный протокол) | stop/inject → _snapshotCommand 120 с; inject только jsonEncode | ✓ WIRED | Протокол не тронут: git diff 2714e27..HEAD -- crates/ пуст (переисполнено) |
| providers.dart | settings.dart | settingsSnapshotProvider → SettingsReply.fromReply | ✓ WIRED | Единственный парсер; пятой копии нет |
| app_ru.arb | dashboard/* | gen-l10n → AppLocalizations, 37 ключей dash*/calibration*/diag* | ✓ WIRED | Все видимые строки; l10n-дыра закрыта — dashCommandFailedBody в .arb (3fcd6ac), читается на месте показа (last_text_card.dart:107) |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
|---------|--------------|--------|-------------------|--------|
| dashboard_state_test → DictaStatusNotifier | state/level/seconds | 100-мс поллинг движка (скриптуемый fake) | Yes — тест ведёт экран тиками | ✓ FLOWING |
| tool/diag_smoke.dart → DaemonEngine.status() | DictaStatus | живой UDS /run/user/1000/dicta.sock | Yes — `DIAG SMOKE state=idle connected=true` в этой сессии | ✓ FLOWING |
| last_text_card → DictaControlState.lastText | lastText | только ответы stop/inject (dicta_control.dart:94,136) | Yes; статус-текст никогда не читается (grep status.text в features → 0) | ✓ FLOWING |
| calibration_card → settingsSnapshotProvider | marker | fetchSettings демона → SettingsReply.fromReply | Yes — инициализация из minLevel + канонический refresh после set | ✓ FLOWING |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| Полный набор тестов | `cd app && flutter test` | `+51: All tests passed!` | ✓ PASS |
| Статический анализ | `cd app && flutter analyze` | «No issues found! (ran in 0.5s)» | ✓ PASS |
| Живой дым диагностики | `cd app && dart run tool/diag_smoke.dart` | `DIAG SMOKE state=idle connected=true` + `INJECT-REJECTED-AS-EXPECTED`, exit 0 | ✓ PASS |
| Кириллический гейт | `cd app && bash tool/cyrillic_gate.sh` | `0`, exit 0 | ✓ PASS |
| Единственная фабрика фейков | `grep -rn "implements DictaEngine" test/*.dart \| wc -l` | `1` | ✓ PASS |
| Неизменность протокола | `git diff --quiet 2714e27 HEAD -- crates/` | пусто | ✓ PASS |
| Нулевые зависимости | `git diff --quiet 2714e27 HEAD -- app/pubspec.yaml app/pubspec.lock` | пусто | ✓ PASS |

### Probe Execution

| Probe | Command | Result | Status |
|-------|---------|--------|-------|
| app/tool/diag_smoke.dart (живой демон) | `dart run tool/diag_smoke.dart` | обе строки, exit 0 | PASS |
| app/tool/cyrillic_gate.sh | `bash tool/cyrillic_gate.sh` | 0 | PASS |

### Review / Security / UI-Review Disposition Cross-Check

- **Код-ревью (02-REVIEW.md, 1 BLOCKER + 4 WARNING + 7 INFO):** фикс-коммит `b953547` — предок HEAD. CR-01 (блокер) подтверждён в коде: calibration_card.dart:87-95 — `invalidate` + `await` вместо синхронного `.value` после refresh, тест «marker follows the canonical snapshot AFTER a successful set» существует и ловит этот класс бага. WR-01 (lastErrorCommand — last_text_card.dart:97-107), WR-02 (`if (!mounted) return` в calibration_card), WR-03 (ветки on TimeoutException в daemon_engine), WR-04 (тесты cancel «запись отменена» байт-в-байт main.rs:947 и backoff через statusError) — всё спот-чекнуто в исходнике. IN-03/05/07 приняты осознанно (disposition зафиксирован).
- **Security (02-SECURITY.md, SECURED 4/4):** все четыре mitigation перепроверены независимо в этой сессии grep'ами: лог-поверхность пуста (`print\|debugPrint` в lib/+tool/ → 0), Clipboard.setData единственный за кликом, клиентской перевалидации длины нет, crates+pubspec байт-в-байт. Информационные остаточные риски (stderr-печать объекта ошибки в diag_smoke; in-memory lastText как фича) зафиксированы в отчёте — согласен с классификацией.
- **UI-review (17/24, BLOCKER a11y):** UI-долг-коммит `3fcd6ac` — предок HEAD; блокер закрыт в коде: настоящий `Slider` + readout (calibration_card.dart:150-161, Semantics), live regions (shell.dart:32-33, dashboard_screen.dart:64), M3-иерархия кнопок (ElevatedButton/FilledButton.tonal/OutlinedButton — dashboard_screen.dart:106/119/131), tabular figures (dashboard_screen.dart:82,96), «команда не выполнена» → .arb. Отложенное (theme-extension muted, заголовок статус-карточки, тон «модель: …», дубль-строки диагностики) — осознанный долг фаз 3–5 по disposition, не гэп фазы 2.

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| (none) | — | Zero TBD/FIXME/XXX/TODO/HACK/PLACEHOLDER во всех 25 файлах фазы | — | — |
| lib/core/daemon_engine.dart + dicta_control.dart | 13-15 | Русский runtime-текст в \uXXXX-эскейпах | ℹ️ Info | IN-05 из ревью, принят: плата за кириллический гейт; UI-текст весь через .arb, дыра dashCommandFailedBody закрыта |
| lib/features/dashboard/dicta_control.dart | — | Общий `commandInFlight` морозит все кнопки на время команды | ℹ️ Info | IN-07, принят (defensive-superset); при мёртвом демоне поллер показывает баннер |
| app/test/widget_test.dart | — | «placeholder» слово | ℹ️ Info | Унаследованный smoke бут DictaApp — ассерт фазы 1, не заглушка |

### Prohibition Review (judgment-tier — non-authoritative LLM verdict, human sign-off required)

| # | Prohibition | LLM verdict | Evidence |
|---|-------------|-------------|----------|
| 1 | Протокол заморожен: ни одной правки crates/* | clean | `git diff --quiet 2714e27 HEAD -- crates/` пуст (переисполнено); команды только status/start/stop/cancel/inject/set/settings |
| 2 | Нулевые новые зависимости | clean | pubspec.yaml+lock байт-в-байт (переисполнено); гистограмма — CustomPainter/LayoutBuilder, chart-пакетов нет |
| 3 | Кириллица в lib/ вне .arb/generated запрещена | clean | gate → 0 (переисполнено) |
| 4 | Ни одного dart:io в lib/features/* | clean | grep → 0 хитов |
| 5 | JSON только через jsonEncode | clean | buildInjectLine/buildSetLine — jsonEncode; replaceAll/ручное экранирование в lib/ → 0 |
| 6 | Приложение не регистрирует хоткей и не дёргает хоткей демона; config.toml не читается | clean | grep hotkey/RegisterHotKey/config.toml по lib/ → только l10n-подсказки; хиты config.toml — 0 |
| 7 | Диктованный текст не логируется | clean | grep print/debugPrint по lib/+tool/ → 0; diag_smoke шлёт только `''` |
| 8 | UI-честность: cancel недоступна при transcribing; спиннер до idle-тика; четвёртого состояния нет; текст не выдумывается из status | clean | Все четыре пункта закреплены тестами (truths 3-4); четвёртое состояние не введено (switch по трём значениям) |

Все восемь остаются `flagged: true` по escalation-gate контракту — записаны для энд-фазового human-checkpoint, не silently passed.

### Human Verification Required

**1. Живой уровень/гистограмма на реальном микрофоне (DASH-01/DASH-04, backstop truth 10)**
**Test:** `cd app && flutter run -d linux`; запись кнопкой — говорить/молчать.
**Expected:** Столбик уровня и столбики гистограммы живые на каждом тике (~10 Гц); тишина/речь визуально различимы; оттенки порогов и отметка min_level читаемы.
**Why human:** Backstop-истина: реальный аудиопоток и визуальная полировка — тесты гоняют скриптованные тики.

**2. Живая калибровка с переживанием рестарта демона (DASH-04, backstop truth 11)**
**Test:** Запись → тишина → двигать отметку → отпустить; затем `systemctl --user restart dictad`.
**Expected:** Порог применяется к живой записи сразу по отпусканию (одна команда set) и остаётся после рестарта демона (сохранение в config.toml на стороне демона).
**Why human:** Backstop-истина: реальная акустика + серверная персистентность.

**3. Inject в реальное окно + путь хоткея (DASH-03, backstop truth 11)**
**Test:** Диктовка по хоткею в стороннем поле; затем кнопка «вставить заново» из приложения.
**Expected:** Кнопка вставляет в активное окно (при клике — само окно приложения, подпись честная); хоткейный путь вставляет в чужое окно; clipboard восстанавливается демоном.
**Why human:** Backstop-истина: реальный фокус окна и системный clipboard.

**4. Ручные строки чеклиста (DIAG-01)**
**Test:** `dictactl toggle` (хоткей-путь) и живая диктовка со вставкой.
**Expected:** Оба пути работают руками; строки в чеклисте — нейтральные подсказки, приложение само их не проверяет (это закреплено тестом: строки шлют ноль команд).
**Why human:** Backstop-истина: глобальный хоткей и реальная вставка.

**5. Prohibition sign-off (8 judgment-tier must-NOTs)**
**Test:** Просмотр таблицы выше.
**Expected:** Нарушений нет (LLM-вердикт чист по всем восьми на кодовых доказательствах — но подпись ваша по escalation gate).
**Why human:** Judgment-tier prohibitions не могут быть авторитативно машинно-верифицированы.

### Gaps Summary

Автоматических гэпов нет. Все 9 не-backstop истин подтверждены доказательствами, произведёнными в этой сессии: 51/51 тестов (переисполнено), analyze чист, cyrillic gate 0, живой diag_smoke exit 0, единственная фабрика фейков, crates/ и pubspec байт-в-байт к базе 2714e27. Каждая из пяти success-критерий ROADMAP закреплена поведенческим тестом против проверенного поведения демона (не на надежду): матрица кнопок × 4 состояния, спиннер до idle-тика, текст только из action-ответов, одна команда set на жест с каноническим маркером после применения (CR-01), чеклист из существующих команд. Блокер ревью (CR-01) и a11y-блокер UI-аудита закрыты в коде и прикрыты тестами.

Статус `human_needed`, а не `passed`: две заявленные `verification: backstop`-истины (живой микрофон; живая калибровка/inject в реальное окно с переживанием рестарта демона) плюс ручные строки диагностики требуют человека за живым рабочим столом, и восемь judgment-tier prohibitions требуют человеческой подписи по контракту escalation gate. Эти пункты — повестка `/gsd-verify-work`, точно как в фазе 1.

---

_Verified: 2026-11-22T11:55:06Z_
_Verifier: the agent (gsd-verifier)_

---
phase: 04-nastroyki-brauzer-modeley
verified: 2026-11-22T15:01:00Z
status: human_needed
score: 15/15 must-haves verified
covered_files:
  - .planning/phases/04-nastroyki-brauzer-modeley/04-01-PLAN.md
  - .planning/phases/04-nastroyki-brauzer-modeley/04-01-SUMMARY.md
  - .planning/phases/04-nastroyki-brauzer-modeley/04-SUMMARY.md
  - app/lib/core/daemon_engine.dart
  - app/lib/core/downloads.dart
  - app/lib/core/engine.dart
  - app/lib/core/hf/download_engine.dart
  - app/lib/core/hf/hf_catalog.dart
  - app/lib/core/hf/hf_client.dart
  - app/lib/core/hf/hf_parse.dart
  - app/lib/core/hf/hf_urls.dart
  - app/lib/core/providers.dart
  - app/lib/core/settings.dart
  - app/lib/features/dashboard/calibration_card.dart
  - app/lib/features/dashboard/dashboard_screen.dart
  - app/lib/features/history/history_screen.dart
  - app/lib/features/models/hf_browser.dart
  - app/lib/features/models/model_store.dart
  - app/lib/features/models/models_screen.dart
  - app/lib/features/settings/settings_screen.dart
  - app/lib/features/shell/shell.dart
  - app/lib/l10n/app_ru.arb
  - app/lib/l10n/generated/app_localizations.dart
  - app/lib/l10n/generated/app_localizations_ru.dart
  - app/test/calibration_test.dart
  - app/test/dashboard_state_test.dart
  - app/test/download_state_test.dart
  - app/test/fake_dicta_engine.dart
  - app/test/fake_download_engine.dart
  - app/test/fake_hf_client.dart
  - app/test/fixtures/hf_search.json
  - app/test/fixtures/hf_tree.json
  - app/test/hf_parse_test.dart
  - app/test/hf_urls_test.dart
  - app/test/history_screen_test.dart
  - app/test/model_delete_test.dart
  - app/test/models_browser_test.dart
  - app/test/models_screen_test.dart
  - app/test/settings_model_test.dart
  - app/test/settings_screen_test.dart
  - app/tool/status_smoke.dart
covered_digest: "v4:sha256:646dee6aea1d92a95ae7e5218cdb7630106ac0ca11bf5e401b795e571975dd7c"
behavior_unverified: 0
overrides_applied: 0
human_verification:
  - test: "Живая смена настроек на живом демоне (min_level и др.)"
    expected: "форма обновляется из снимка SetReply без дополнительного fetch; живой демон принимает raw-ms значения"
    why_human: "widget-батарея идёт на фейках; поведение живого демона и live-обновление формы глазами не проверяются grep'ом"
  - test: "Живой отказ reload: начать диктовку (Alt+V), затем сменить модель в настройках"
    expected: "вербатим-строка main.rs:752 в строке статуса формы; слайдер остаётся на месте"
    why_human: "требует живой записи через демон; вербатим-паритет строки уже сверен побайтово, живую доставку не проверить прогоном"
  - test: "Живая загрузка маленькой модели из каталога (ggml-tiny-q5_1.bin)"
    expected: "полоса прогресса, .part в dest_dir, rename, модель появляется в списке установленых, выбор её работает"
    why_human: "реальная сеть/диск-мутация; фейковый движок доказал контракт, живой путь — человеку"
  - test: "Прерывание загрузки (отмена или обрыв) + повторный старт"
    expected: "докачка 206 продолжилась с длины .part; 200-при-resume перезаписал с нуля"
    why_human: "живой HTTP-сервер с Range-поддержкой недоступен в тестовом контейнере"
  - test: "Битая прямая ссылка (404/закрытый репозиторий) в поле прямой ссылки"
    expected: "честная ошибка текстом (HTTP 404 / «репозиторий закрыт или не найден»), без исключения в UI"
    why_human: "живой HF-ответ; фейковый клиент уже покрыл оба кода в тестах"
  - test: "Удаление неактивной модели с confirm-диалогом"
    expected: "файл удалён только внутри dest_dir, активная заблокирована, snackbar «удалено», список обновился"
    why_human: "живая ФС-мутация; guards доказаны тестами, живой путь — человеку"
  - test: "Подпись must-NOT (визуальный тон)"
    expected: "VRAM-блокировка, confirm-диалоги и reloading-спиннер выглядят честно и не читаются как ошибки"
    why_human: "визуальный тон/UX — только глазами владельца (подпись запрета, конвенция фаз 01-03)"
---

# Phase 4: Настройки + браузер моделей — Verification Report

**Phase Goal:** Полный паритет с dicta-settings по настройкам и моделям — критерий
закрытия вехи (CONF-01/02, MODEL-01/02 в app-слое поверх замороженного протокола dictad).
**Verified:** 2026-11-22T15:01:00Z (live re-run, не по SUMMARY)
**Status:** human_needed — все 15 must-haves подтверждены автодоказательствами (батарея
144/144 запущена верификатором заново); живой UAT (7 пунктов, включая подпись must-NOT)
остаётся человеку — конвенция фаз 01-03: никогда не PASSED молча.
**Re-verification:** No — initial verification (предыдущего VERIFICATION.md не было).

## Goal Achievement

### Live Battery (re-run by verifier, not trusted from SUMMARY)

| Check | Command | Result | Status |
| --- | --- | --- | --- |
| Батарея | `cd app && flutter test` | **144/144, All tests passed!** | ✓ PASS |
| Статический анализ | `flutter analyze` | **No issues found! (ran in 0.5s)** | ✓ PASS |
| Кириллиц-гейт | `bash tool/cyrillic_gate.sh` | **0**, exit 0 | ✓ PASS |
| Контракт заморозки | `git diff 9aee17e HEAD -- crates/ app/pubspec.yaml app/pubspec.lock` | **пусто** (byte-clean) | ✓ PASS |
| Implements-гейты | grep `implements X` в test/ | DictaEngine **1**, HfClient **1**, DownloadEngine **1** | ✓ PASS |
| Commit-генеалогия | `git merge-base --is-ancestor` для всех 21 коммитов d646e3a..180aa27 | все — предки HEAD | ✓ PASS |

### Observable Truths (PLAN must_haves, live evidence)

| # | Truth | Status | Evidence |
| --- | --- | --- | --- |
| 1 | Set-ответ несёт снимок: форма обновляется из reply.settings, fetchCount НЕ растёт | ✓ VERIFIED | settings_screen_test.dart:89-130 — `expect(fake.fetchCount, fetchCountBeforeSet)`, readout/slider = 900 из ответа; тест зелёный в живой батарее |
| 2 | Отказ reload — вербатим-строка в note при error:None | ✓ VERIFIED | test:347-371 содержит строку «идёт запись или распознавание — смену модели отложи; заверши диктовку (Alt+V) и повтори»; сверена **побайтово** с crates/dictad/src/main.rs:752 |
| 3 | Confirm перед каждым ENGINE-set (вкл. language); без подтверждения setCalls пуст | ✓ VERIFIED | test:282 «ENGINE switch waits for confirm; decline sends nothing», test:321 «language is an ENGINE key and needs confirm too» — `expect(fake.setCalls, isEmpty)` до «продолжить»; kEngineKeys вкл. language (settings.dart:30-37) |
| 4 | Один set на завершённый жест | ✓ VERIFIED | test:65 «timed drag commits exactly one set», test:444 — «no set during drag» + «one set per release» на всех 4 слайдерах |
| 5 | reloading:true → спиннер + ENGINE disabled | ✓ VERIFIED | test:373 — confReloadSpinner, «модель загружается…», confUseGpuSwitch.onChanged == null |
| 6 | Все 17 ключей редактируемы; bool 'on'/'off'; диапазоны демона | ✓ VERIFIED | kEditableKeys = 17 ключей (settings.dart:10-28), все 17 найдены в settings_screen.dart (grep 17/17); test:161 все группы/контролы, test:230 on/off, test:394 threads 1..64 / beam 1..10 |
| 7 | VRAM 2..32 (default 24) UI-only, set-команд не порождает | ✓ VERIFIED | models_browser_test.dart:84-138 — default 24, `expect(fake.setCalls, isEmpty)`, синтетический файл 5 000 000 000 байт заблокирован «больше лимита vram», кнопка disabled |
| 8 | HF-URL ровно по формулам hf.rs; данные только через HfClient-шов; fetch только по действию | ✓ VERIFIED | hf_urls_test.dart (browseUrl/treeUrlAt/resolveUrl точные), models_browser_test:139 «search fetches exactly browseUrl», :520/:554 таймауты; ноль Timer/retry в hf-коде (04-SECURITY T-4-03) |
| 9 | Загрузка: одна в момент; .part Range докачка; 200-on-resume overwrite; отмена стирает .part; ошибка сохраняет; rename → settings.models | ✓ VERIFIED | download_state_test: :60 ok200+rename+без Range, :93 resume206 **fromByte=40 от длины .part**, hf_urls_test:59 resumePlan append-on-206/overwrite-on-200, :128 cancel erases, :153 HTTP-ошибка keeps .part, :172 one-at-a-time, :189 «already installed» не перезаписывается, :207 success → invalidate+await settings refetch |
| 10 | Удаление: активная blocked, путь вне dest_dir отклонён, confirm, refetch | ✓ VERIFIED | model_delete_test:16-34 (active + `../x.bin` + абсолютный чужой путь — blocked, in-dir allowed); models_screen_test:108 confirm AlertDialog → файл удалён → fetchCount растёт; download_state_test:279 `..` отвергнут ДО ФС (`fake.calls` пуст) — WR-05 |
| 11 | Payload-privacy: set-значения/тела ошибок не логируются | ✓ VERIFIED | grep print/debugPrint/log по app/lib — 0; status_smoke.dart:13 `error.runtimeType`; SECURITY T-4-04 |
| 12 | Только dart:io HttpClient, ноль curl, ноль новых зависимостей | ✓ VERIFIED | grep `Process.run` в lib/ (кроме канонического runSync('id') T-2-01) — 0 сетевых; `curl` — 0; pubspec-дифф пуст |
| 13 | Список моделей — только settings.models, без скана ФС | ✓ VERIFIED | models_screen.dart:24 `settings?.models`; в features/models нет Directory/listSync-скана (grep) |
| 14 | Батарея ~135-145 зелёная, analyze 0, cyrillic 0, implements 1/1/1 | ✓ VERIFIED | Живой прогон верификатора: **144/144**, analyze «No issues found!», gate 0, implements 1/1/1 |
| 15 | Протокол/манифест заморожены к базе 9aee17e | ✓ VERIFIED | `git diff 9aee17e HEAD -- crates/ app/pubspec.yaml app/pubspec.lock` — **пусто** (живая команда) |

**Score:** 15/15 truths verified (0 present-but-behavior-unverified — каждый
behavior-dependent инвариант покрыт зелёным тестом, запущенным верификатором живьём).

### Review-fix regressions (spot-checked, live tests green)

| Finding | Fix commit | Regression test | Status |
| --- | --- | --- | --- |
| CR-01/CR-02 note-is-data (success ≠ red error) | d867622 | settings_screen_test:89 — `style?.color, isNot(errorColor)` | ✓ VERIFIED |
| CR-03 raw-ms slider wire values | ecee344 | settings_screen_test:444 — 4 ключа, `matches(r'^\d+$')`, raw wire | ✓ VERIFIED |
| WR-01 pagination offset reset + append | 8fd9e9d | models_browser_test:460 «show more appends page 2; a new search resets the offset» | ✓ VERIFIED |
| WR-02 one-at-a-time await gap | 851eea2 | download_state_test:229 «two starts during the settings await never run concurrently» | ✓ VERIFIED |
| WR-03 cancel during connect | b659c7a | download_state_test:258 «cancel during connect is held until the request exists» | ✓ VERIFIED |
| WR-05 traversal-like dest names | 01460fe | download_state_test:279 (reject before FS) + hf_parse_test:77-96 (`..`, `.`, `a\..`, `..?download=1` → null) | ✓ VERIFIED |
| WR-06 honest models empty state | cc4ef19 | models_screen_test:184 «a failed settings fetch is not shown as an empty model list» | ✓ VERIFIED |
| UI top-3 (honest fetch states, refusal feedback, timeout surfaces) | 3a296bc | settings_screen_test:512/554/581/609; models_browser_test:520/554 | ✓ VERIFIED |

### Required Artifacts

| Artifact | Status | Details |
| --- | --- | --- |
| `app/lib/core/settings.dart` — SetReply{note,error,settings} + fromReply | ✓ VERIFIED | SetReply:159, fromReply:164 переиспользует SettingsReply.fromReply (пятого парсера нет) |
| `app/lib/core/hf/` — 5 файлов | ✓ VERIFIED | hf_urls(28)/hf_parse(125)/hf_catalog(51)/hf_client(41)/download_engine(149) строк, все wired |
| `app/lib/core/downloads.dart` — DownloadManager | ✓ VERIFIED | 193 строк; one-at-a-time sync-claim :78,88; traversal-reject :81-87; rename :131 |
| `app/lib/features/settings/settings_screen.dart` — 17 ключей | ✓ VERIFIED | 612 строк; 17/17 ключей grep; 4 группы; confirm; honest states |
| `app/lib/features/models/model_store.dart` | ✓ VERIFIED | modelName/canDelete/deleteModel; lexical confinement :22-27 |
| `app/lib/features/models/models_screen.dart` + `hf_browser.dart` | ✓ VERIFIED | 242 + 537 строк; wired в shell.dart:87-88 |
| `app/test/fake_hf_client.dart`, `fake_download_engine.dart` | ✓ VERIFIED | по одному фейку на шов; implements 1/1 (grep) |
| `app/test/hf_urls_test.dart` + `hf_parse_test.dart` | ✓ VERIFIED | паритет hf.rs дословно; зелёные |
| `download_state/model_delete/settings_screen/models_screen/models_browser`_test | ✓ VERIFIED | все зелёные в батарее 144/144 |
| `app/test/fixtures/hf_search.json`, `hf_tree.json` | ✓ VERIFIED | используются hf_parse_test:135 |
| conf*/model* в app_ru.arb + generated | ✓ VERIFIED | cyrillic gate 0; l10n сгенерированы |
| `app/tool/status_smoke.dart:12-13` — error.runtimeType | ✓ VERIFIED | долг 03-SECURITY закрыт |
| `app/lib/features/shell/shell.dart` — оба слота, без _SectionPlaceholder | ✓ VERIFIED | grep `_SectionPlaceholder` — 0; ModelsScreen():87, SettingsScreen():88 |

### Key Link Verification

| From | To | Via | Status | Evidence |
| --- | --- | --- | --- | --- |
| engine.dart | settings.dart | `Future<SetReply> set(key, value)` | ✓ WIRED | engine.dart:90 + settings_screen_test:89 (set → SetReply → форма) |
| settings_screen.dart | providers.dart | ref.watch(settingsSnapshotProvider) | ✓ WIRED | тесты: спиннер in-flight (:609), retry-card (:554/:581) |
| hf_browser.dart | hf_urls + hf_parse | fetchText(browseUrl/treeUrlAt) → parse* | ✓ WIRED | models_browser_test:139/:208 — «fetches exactly browseUrl», treeUrlAt root+subdirs |
| downloads.dart | download_engine.dart | download() → Stream; resumePlan append/write | ✓ WIRED | download_state_test:60/:93; hf_urls_test:59 |
| downloads.dart | providers.dart | по done: invalidate + await refetch | ✓ WIRED | download_state_test:207 + WR-02 :229 |
| shell.dart | models_screen + settings_screen | IndexedStack слоты 2/3 | ✓ WIRED | shell.dart:87-88 |
| model_store.dart | dart:io File | canDelete guards → File.delete() | ✓ WIRED | models_screen_test:108 — файл реально удалён; активная blocked |

### Behavioral Spot-Checks (run live by verifier)

| Behavior | Command | Result | Status |
| --- | --- | --- | --- |
| Полная батарея | `cd app && flutter test` | 00:07 +144 All tests passed! | ✓ PASS |
| Каталог 6 позиций | в батарее (models_browser_test:51) | 6 имён + «уже есть»×2 | ✓ PASS |
| 401/404 честные | в батарее (:429) | «репозиторий закрыт или не найден» / «HTTP 404» | ✓ PASS |
| 206-докачка от длины .part | в батарее (download_state_test:93) | fromByte=40, прогресс 40→100 | ✓ PASS |
| Traversal '..' до ФС | в батарее (download_state_test:279) | DownloadFailed, fake.calls пуст | ✓ PASS |
| WR-02 await-gap | в батарее (download_state_test:229) | один слот, второй start не прошёл | ✓ PASS |

### Probe Execution

| Probe | Command | Result | Status |
| --- | --- | --- | --- |
| Батарея/analyze/gate | см. Live Battery выше | 144/144, No issues, 0 | ✓ PASS |
| Живой daemon smoke (`tool/status_smoke.dart`) | требует живого dictad | не запускался верификатором (read-only сессия; UAT-пункт живого демона) | ? SKIP → human |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
| --- | --- | --- | --- | --- |
| CONF-01 | 04-01 | 17 ключей формами, живой set, без спама | ✓ SATISFIED | Truths 1,4,6 + тесты :65/:89/:444 |
| CONF-02 | 04-01 | Честные состояния reload/confirm | ✓ SATISFIED | Truths 2,3,5 + тесты :282/:321/:347/:373 |
| MODEL-01 | 04-01 | HF-браузер: каталог/поиск/VRAM/ссылка/докачка | ✓ SATISFIED | Truths 7,8,9 + models_browser_test весь |
| MODEL-02 | 04-01 | Установленные модели, активная пометка, удаление неактивной | ✓ SATISFIED | Truths 10,13 + model_delete/models_screen tests |

Orphaned requirements: нет — REQUIREMENTS.md мапит на Phase 4 ровно CONF-01/02, MODEL-01/02,
ровно те, что заявлены в PLAN. Чекбоксы REQUIREMENTS/ROADMAP фазового уровня остаются
`[ ]` по конвенции проекта (фазы 1-3 аналогично; план-чекбокс `[x] 04-01-PLAN.md` стоит).

### Security Verification

04-SECURITY.md существует, **SECURED 7/7, threats_open: 0**. Оба review-фикса сверены
живыми тестами: WR-05 (traversal) — download_state_test:279 + hf_parse_test:77-96;
WR-02 (await gap) — download_state_test:229. Контракт заморозки — git diff пуст.

### Anti-Patterns Found

| File | Pattern | Severity | Impact |
| --- | --- | --- | --- |
| — | TBD/FIXME/XXX в фазовых файлах: **0** | — | — |
| — | TODO/HACK/PLACEHOLDER в фазовых файлах: **0** | — | — |
| — | Пропущенные тесты (skip/@Skip): **0** в test/ | — | — |

Долговых маркеров нет; skipped-тестов нет; батарею 144/144 верификатор запускал живьём.

### Test Quality Audit

Отключённых тестов на требования — 0. Циркулярных фикстур — нет: ожидаемые строки
(вердатим main.rs:752, URL-формулы hf.rs) взяты из **замороженного Rust-эталона**, не из
Dart-кода. Уровень ассертов — value/behavioral (`fromByte == 40`, `setCalls.length == 1`,
`fetchCount` не растёт, стиль не error-color). Требуемых количеств хватает (6 позиций
каталога — тест перечисляет все 6).

### Decision Coverage

`query check.decision-coverage-verify` → skipped: «no trackable decisions»
(CONTEXT.md без trackable `<decisions>`). Gate не блокирует.

### Human Verification Required

Живой UAT остаётся — 7 пунктов в `04-UAT.md` (статус testing): живая смена настроек на
живом демоне; живой отказ reload при диктовке; живая загрузка маленькой модели; прерывание
и докачка 206 живьём; битая прямая ссылка; удаление неактивной модели; **подпись must-NOT**
(визуальный тон VRAM/confirm/спиннеров). Автодоказательства (15/15) не заменяют живой
подписи — конвенция фаз 01-03.

### Gaps Summary

Gaps не найдены: все 15 must-haves подтверждены живыми прогонами (не по SUMMARY), все
контракты заморозки целы, оба review-фикса имеют зелёные regression-тесты, долговых
маркеров нет, security SECURED 7/7. Статус **human_needed**, не passed: остаётся живой
UAT (7 пунктов) — включая обязательную подпись must-NOT.

---

_Verified: 2026-11-22T15:01:00Z_
_Verifier: the agent (gsd-verifier)_

---
status: draft
phase: 4
requirements: [CONF-01, CONF-02, MODEL-01, MODEL-02]
---

# Phase 4 VALIDATION — Настройки + браузер моделей

## Edge Probe Resolutions

| Req | Facet | Probe | Resolution | Verification |
|-----|-------|-------|------------|--------------|
| CONF-01 | adjacency | equal/touching things: merge, collide, separate? | **explicit**: каждый контрол 1:1 свой ключ; равные значения двух ключей НЕ сливаются — это независимые set; set-значение не зависит от соседей | widget-тест: два слайдера с равными значениями → два независимых set-вызова, по одному на отпускание |
| CONF-01 | empty | empty/single/null input | **explicit**: пустые/отсутствующие поля снимка → дефолты SettingsReply (settings.dart:90-115, tolerant fromJson); пустой set-value → ошибка демона показывается вербатим (валидация на демоне — контракт) | существующий poisoned-JSON тест + новый: set с пустым value → вербатим-ошибка в дереве |
| CONF-01 | ordering | equal compare — stable order? | **explicit**: порядок между разными ключами не специфицирован и не нужен — каждая команда = свежее соединение (engine.dart library doc); внутри ключа сериализация через commandInFlight-гейт; повторный set равного значения идемпотентен на демоне | тест «one set per drag»: timedDrag → fake.setCalls.length == 1 |
| CONF-02 | unclassified | — | **explicit**: отказ reload демона приходит в note при error:None (main.rs:723-739) → SetReply{note,error,settings} (finding C1, Wave-0 слот 1); вербатим-строка main.rs:752 в дереве; confirm перед ENGINE_KEYS: без подтверждения setCalls пуст | fake setReplyMap note=отказ при error:null → текст в дереве; ENGINE-кнопка → диалог → setCalls пуст до confirm |
| MODEL-01 | unclassified | — | **explicit** для URL/парсеров/VRAM/имён (чистые функции, паритетные тесты hf.rs:172-341); **backstop** для живой сети (полоса, докачка 206, отмена) — manual UAT: живая загрузка маленькой модели, прерывание+повтор | unit: browse_url комбинации, parse_search/parse_tree/lfs.size, vramBlocked, filename_of; manual UAT-лист ниже |
| MODEL-02 | unclassified | — | **explicit**: guards удаления — активная blocked, путь вне dest_dir отклонён (нормализация + префикс-тест), confirm до delete, после — re-fetch; temp-dir в тестах, сеть не нужна | unit на temp-dir: delete неактивной ок; delete активной rejected; path traversal rejected; widget: confirm-диалог |

## Wave 0 Requirements (ДО экранов)

1. `DictaEngine.set` → `SetReply{note, error, settings}` + миграция fake/calibration/тестов; батарея 71/71 зелёная после.
2. Порт hf.rs → core/hf (чистые функции: urls + parse); паритетные тесты (browse_url комбинации, битый JSON → пусто, lfs.size, dest_name, filename_of, %20).
3. Швы `HfClient` (GET→String, фейк с картой url→body) и `DownloadEngine` (download→Stream<DownloadProgress>, cancel; фейк-сценарии 200/206/отказ/abort); implements==1 на каждый.
4. HF-фикстуры (поиск id/downloads/likes; tree .bin+lfs+dirs) с живых проб 04-RESEARCH.md.

## Per-Task Verification Map

| Task ID (TBD → bind after plan) | Requirement | Command | Fails when |
|---|---|---|---|
| TBD | Wave 0: SetReply + миграция | `cd app && flutter test && flutter analyze && bash tool/cyrillic_gate.sh` | батарея < 71 + новых; analyze ≠ 0; cyrillic > 0 |
| TBD | Wave 0: hf-порт | `cd app && flutter test test/hf_urls_test.dart` | паритетные строки не совпали |
| TBD | Wave 0: швы+фейки | `cd app && flutter test` + `grep implements` == ожидание | implements != 1 на шов |
| TBD | CONF-01 форма | `cd app && flutter test test/settings_screen_test.dart` | set-спам (>1 на жест); fetchCount растёт после set |
| TBD | CONF-02 честные состояния | тот же файл | note-отказ не в дереве; confirm отсутствует; reloading не дизейблит |
| TBD | MODEL-01 браузер | `cd app && flutter test test/models_screen_test.dart test/download_state_test.dart` | URL не из browseUrl; VRAM-порог не блокирует; прогресс из content-length 302-ловушка |
| TBD | MODEL-02 модели+удаление | `cd app && flutter test test/model_delete_test.dart` | активная удалена; путь вне dest_dir прошёл; delete без confirm |
| TBD | Security residual | `cd app && ! grep -n '\$error' tool/status_smoke.dart` | найден $error |
| TBD | Финальный гейт | `cd app && flutter test && flutter analyze && bash tool/cyrillic_gate.sh` + `git diff --quiet <phase-base> HEAD -- crates/ app/pubspec.yaml app/pubspec.lock` | любой не-ноль; контракт продырявлен |

## Sampling Rate

- Per-task commit: `flutter test` + `flutter analyze` + `bash tool/cyrillic_gate.sh`.
- Per-wave merge: полный suite + живой smoke (демон запущен → экраны открываются).
- Phase gate: вся батарея зелёная + гейты до `/gsd-verify-work 04`.

## Manual UAT (backstop, network/visual)

- Живая загрузка маленькой модели из каталога: полоса, `.part`, rename, «выбери в списке».
- Отмена + повтор → докачка 206 продолжилась.
- Живой отказ reload: диктовка (Alt+V) активна → смена модели → вербатим-сообщение.
- Битая прямая ссылка → честная ошибка 404/401.
- VRAM-слайдер/confirm-диалоги глазами (наследие UI-аудита фазы 3, CONTEXT пп.1-7).

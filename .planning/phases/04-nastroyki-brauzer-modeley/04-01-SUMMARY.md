# Phase 4 Plan 01: Настройки + браузер моделей — Summary

**Plan:** 04-01 · **Status:** complete · **Autonomous:** yes (checkpoints: none in plan)
**Objective:** полный паритет с dicta-settings в app-слое: 17 ключей с живым set через
SetReply-снимок, HF-браузер (каталог/поиск/дерево/VRAM/прямая ссылка/загрузка),
установленные модели с пометкой активной и guards-удалением. Протокол и Rust заморожены.

## Per-task execution

| Task | Wave | What was done | Key files | Commit | Battery |
|------|------|---------------|-----------|--------|---------|
| 1 | 0 | SetReply{note,error,settings} + fromReply через единый SettingsReply.fromReply; DaemonEngine.set парсит полный ответ с errors-as-data; CalibrationCard consume snapshot без invalidate+refetch; fake hook setReplyMap; status_smoke → error.runtimeType | core/settings.dart, core/engine.dart, core/daemon_engine.dart, calibration_card.dart, fake_dicta_engine.dart, settings_model_test.dart, status_smoke.dart | d646e3a | 71→74 |
| 2 | 0 | Порт hf.rs/catalog.rs чистыми функциями (паритет строк дословно); HfClient/DownloadEngine швы + HttpClient-реализации; resumePlan/totalFromContentRange; по одному фейку на шов; фикстуры hf_search/hf_tree | core/hf/{hf_urls,hf_parse,hf_catalog,hf_client,download_engine}.dart, providers.dart, fake_{hf_client,download_engine}.dart, hf_{urls,parse}_test.dart | ed876a0 | 74→91 |
| 3 | 1 (TRACER) | Один ключ paste_delay_ms end-to-end: slider → set → SetReply → форма, без re-fetch; слот 3 shell → SettingsScreen; conf* l10n | features/settings/settings_screen.dart, shell.dart, app_ru.arb + generated, settings_screen_test.dart | f7cf8f6 | 91→95 |
| 4 | 2 | Все 17 ключей в 4 группах; один set на отпускание; confirm перед каждым ENGINE-ключом (включая language); отказ вербатим в liveRegion; reloading-спиннер + disabled ENGINE; таймер re-fetch | settings_screen.dart, app_ru.arb + generated, settings_screen_test.dart | c09ac93 | 95→105 |
| 5 | 3 | Список только из settings.models; активная помечена/неудаляема; три guard'а (active/outside destDir/confirm); File.delete + snackbar + обязательный refetch; оба слота shell заняты | features/models/{model_store,models_screen}.dart, shell.dart, app_ru.arb + generated, model_delete_test.dart, models_screen_test.dart | c874887 | 105→112 |
| 6 | 4 | DownloadManager: one-at-a-time, Range от длины .part, rename на success + invalidate/await snapshot, cancel стирает .part, ошибка сохраняет, «уже есть» без перезаписи | core/downloads.dart, fake_download_engine.dart, download_state_test.dart | 1173ecd | 112→120 |
| 7 | 5 | HF-браузер: каталог 6 с «уже есть», UI-only VRAM 2..32, поиск с sort/ASR/пагинацией, дерево с подпапками, прямая ссылка http/https, панель загрузки с прогрессом/отменой/честными ошибками | features/models/hf_browser.dart, models_screen.dart, buildFakeApp-Overrides, app_ru.arb + generated, models_browser_test.dart | 4858b79 | 120→130 |
| 8 | 6 | Наследие UI: заголовок статус-карточки (titleMedium + titleLarge с цветом), SnackBar-AT-прогон через ensureSemantics, bottom-padding, clear-search suffix, единая empty-упаковка, честный тон «модель: …» | dashboard_screen.dart, history_screen.dart, app_ru.arb + generated, dashboard_state_test.dart, history_screen_test.dart | e2ffbbd | 130→131 |
| 9 | 7 | Финальный гейт: батарея, контракты заморозки, implements 1/1/1, живой smoke | SUMMARY, STATE/ROADMAP (если включены) | (this commit) | 131 |

**Траектория батареи:** 71 (база) → 74 → 91 → 95 → 105 → 112 → 120 → 130 → **131/131 зелёная**.

## Final gate numbers

- `flutter test`: **131/131, All tests passed** (71 унаследованный + 60 новых).
- `flutter analyze`: **No issues found!**
- `bash tool/cyrillic_gate.sh`: **0**
- `dart run tool/status_smoke.dart`: **SMOKE OK state=idle, exit 0** (живой демон, precondition Task 9 выполнен).
- Заморозка: `git diff base(9aee17e) HEAD -- crates/ app/pubspec.yaml app/pubspec.lock` — **чисто**.
- implements-гейты в test/: DictaEngine **1**, HfClient **1**, DownloadEngine **1**.
- Анти-curl в lib/**.dart: **0**; `$error`-печать в status_smoke: **0**;
  `_SectionPlaceholder`: **0**; `placeholderScreen`: **0**.
- liveRegion в features/settings + features/models: **2** (confStatus, modelDownloadPanel).

## Requirement coverage

- **CONF-01** → Task 3 (tracer: один ключ через реальный шов) + Task 4 (17 ключей, 4 группы, one-set-per-release).
- **CONF-02** → Task 1 (SetReply несёт note+снимок) + Task 4 (confirm ENGINE incl. language, отказ main.rs:752 вербатим, reloading-спиннер/disabled).
- **MODEL-01** → Task 2 (порт hf.rs + швы) + Task 6 (DownloadManager) + Task 7 (браузер: каталог/поиск/VRAM/ссылка/прогресс).
- **MODEL-02** → Task 5 (список, активная, тройной guard удаления + refetch).
- Долг 03-SECURITY (status_smoke `$error`) → Task 1.

## UI-наследие фазы 3

- Закрыто: №1 (заголовок статус-карточки, Task 8), №2 (destructive confirm, Task 5),
  №3 (единый механизм ошибок: SetReply.liveRegion / snackbar-панель, Task 4/7),
  №4 (reloading-спиннер, Task 4), №6 (liveRegion + SnackBar-AT-прогон, Task 4/7/8),
  №7 (bottom-padding, тон «модель: …», clear-search suffix, empty-упаковка, Task 8).
- **№5 (свёртка длинных записей) — осознанно перенесена в фазу 5** (CONTEXT:33: «опционально сюда, иначе фаза 5»).
- **Рекомендация 03-UI-REVIEW:43 «убрать busy-gate на дашборде» ОТКЛОНЕНА** планировщиком: гейт
  защищает от двойного stop (гонка фазы 2) и закреплён `dashboard_buttons_test.dart:83-104`. Копия не менялась.

## Deviations from plan

1. **[Rule 1 — Bug, Task 2]** `as num?` в parseSearch бросал TypeError на строковых полях
   (`"likes":"12"`); заменено на безопасную проверку `is num` — паритет Rust `as_u64()` → None → 0.
2. **[Implementation detail, Task 6/7]** FakeDownloadEngine пишет `.part` **синхронно** (RandomAccessFile),
   а DownloadManager использует `renameSync`: асинхронныеdart:io-операции не завершаются в fake-async zone
   widget-тестов. Контракт симуляции сохранён полностью (прогресс/Range/отмена/ошибки).
3. **[Implementation detail, Task 2/7]** `fromByte` фейка выводится из длины `.part` по `partPath`
   (в интерфейсе DownloadEngine нет отдельного параметра offset) — ровно так же, как реальный движок.
4. **[Test-only, Task 7]** Для проверки VRAM-блокировки использован синтетический tree-файл 5 ГБ:
   максимальный каталожный файл 1.1 ГБ всегда меньше минимального лимита 2 ГБ.
5. **[Additive l10n]** Добавлены служебные ключи сверх списка плана: `modelDeleteBlocked`,
   `confSeconds`, `confMilliseconds`, `modelNoteSmall` (для 6 позиций каталога), `modelSearchSemantics`.
   Все с @-описаниями, генерированы через `flutter gen-l10n`.
6. **[Naming, Task 9]** Созданы оба имени summary: `04-01-SUMMARY.md` (`<output>` плана) и
   `04-SUMMARY.md` (имя из контракта оркестратора) — identical content.

Auth gates: отсутствуют. Checkpoint'ов план не содержит; autonomous-режим подтверждён оркестратором.

## Manual UAT backstop (для /gsd-verify-work 04)

1. Живая загрузка `ggml-tiny-q5_1.bin` из каталога до конца: полоса → `.part` → rename → «выбери её в списке».
2. Отмена загрузки + повтор → докачка 206 продолжилась с длины `.part`.
3. Живой отказ reload: начать диктовку (Alt+V), сменить модель → вербатим main.rs:752 в строке статуса.
4. Битая прямая ссылка → честный 404/401 без исключения в UI.
5. VRAM-слайдер и confirm-диалоги глазами (UI-аудит наследия).

## Self-check

- Все созданные файлы существуют на диске; каждый task-коммит достижим из HEAD.
- `git status` перед финальным коммитом: только SUMMARY-файлы (без незакоммиченного кода).

## Self-Check: PASSED

- All created files verified present (corrected app/-prefixed test paths).
- All 8 task commits are ancestors of HEAD; final gate green.

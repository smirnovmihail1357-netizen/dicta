# Phase 4 CONTEXT — Настройки + браузер моделей

**Phase:** 4 · **Reqs:** CONF-01, CONF-02, MODEL-01, MODEL-02
**Goal (ROADMAP):** Полный паритет с dicta-settings в dicta-app — критерий закрытия вехи; 17 ключей формами с живым `set`, HF-браузер (каталог/поиск/VRAM/прямая ссылка/прогресс/докачка .part), установленные модели с пометкой активной и удалением неактивной.

## Locked decisions (не пересматривать)

- Протокол демона и Rust-код заморожены: `set`/`settings` — единственный путь изменения конфига. Прямого редактирования config.toml из приложения НЕТ.
- `crates/dicta-settings` — эталон паритета, НЕ трогать (декомиссия — фаза 8).
- Новые pubspec-зависимости — только если research докажет, что std/dart:io недостаточен. Стартовая позиция: `dart:io` HttpClient + `dart:convert` (dio из STACK-исследования НЕ добавлять без нужды).
- Кириллица в lib/ только через .arb (гейт `bash tool/cyrillic_gate.sh`).
- UI-hint: у фазы 4 наибольшая UI-нагрузка. ui-plan-gate = block:false (без UI-SPEC); решение оркестратора: продолжать без UI-SPEC, UI-долг гасить немедленно по результату gsd-ui-auditor post-phase (модель фаз 2-3 работает).

## Проверенные факты (живой код, file:line)

- Шов готов: `DictaEngine.fetchSettings() Future<Map>` (engine.dart:84), `set(key,value)` (:85); фабрика фейков уже имеет крюки `fetchSettingsReply`/`fetchCount`/`onSet`/`setReply` (fake_dicta_engine.dart:21,40-41,57,68,114-116) — расширяй, не клонируй.
- `SettingsReply` — 20 полей, толерантный fromJson (settings.dart:65-115); `reloading: bool` уже есть (:152).
- `ENGINE_KEYS` = model, language, use_gpu, gpu_device, n_threads, beam_size (dicta-core config.rs:163-170). Их set → `schedule_engine_reload`; ОТКАЗ при state != Idle: «идёт запись или распознавание — смену модели отложи; заверши диктовку (Alt+V) и повтори» (dictad main.rs:744-762). UI обязан честно показывать этот отказ и подсвечивать, какие ключи ENGINE.
- set-ответ: text = note («перезагружаю модель …» или накопленная ошибка через ' · '), error: None, и ВСЕГДА свежий settings-снимок в reply (main.rs:731,737-741) — обновляй форму из ответа, без лишнего fetchSettings.
- `reloading` в settings-снимке (settings_view main.rs:382-403) — честный флаг «модель загружается…»: polling уже существует (status-поллер 100 мс; для формы настроек можно пере-fetchSettings по таймеру/после set).
- Эталон паритета (READ-ONLY, Rust): catalog.rs (CATALOG 6: tiny 32МБ … large-v3 1.1ГБ, note-подсказки); hf.rs — browse_url(search/filter=automatic-speech-recognition/sort=downloads|likes|lastModified/limit=20/offset), tree_url_at(root+subpath), resolve_url, parse_search(id/downloads/likes), parse_tree(.bin, size||lfs.size), parse_tree_dirs, dest_name; VRAM-слайдер 2..32 ГБ (default 24), файл > лимита — помечен, кнопка неактивна; прямая ссылка http/https + filename; загрузка в .part с докачкой (`-C -`), отмена; dest_dir = parent(текущей модели), fallback ~/.local/share/whisper-cpp (учёт XDG_DATA_HOME).
- Загрузка в Dart: HttpClient даёт стриминг + Range-заголовок для докачки .part; НИКАКИХ Process.run('curl') — приложение кросс-платформенно (Windows-фаза 6), curl-shell-out запрещён.
- MODEL-02 (удаление неактивной модели): файловая операция dart:io delete; блокировать удаление активной модели (сравнение с settings.model), подтверждение по канону destructive (OutlinedButton error-foreground + undo невозможен → честный confirm-диалог). Путь удаления = dest_dir модели; НИКОГДА не удалять вне dest_dir.
- Существующие каноны UI: ThemeExtension muted (3 точки регистрации: main×2 темы, buildFakeApp, suite-local buildTestApp в shell_test — grep-комментарий theme_ext.dart:3-6); snackbar-канон (dashInjected/dashInjectFailed/histUndo); канон кнопок (Filled=primary, Elevated=copy/neutral, Outlined+error=destructive, Text=tertiary); tabular figures; MergeSemantics для полей ввода (урок WR-01).

## Наследие UI-аудита фазы 3 — распределить в этой фазе (вход планирования)

1. Заголовочный паттерн статус-карточки дашборда — ПОСЛЕДНЕЕ незакрытое наследие фазы 2 (существительное-заголовок, состояние titleLarge с цветом).
2. Деструктивный канон закрепить (error-foreground; до сбросов конфигурации undo нет → confirm-диалог).
3. Единый механизм ошибок cross-screen команд.
4. In-flight индикаторы async-кнопок — загрузка модели первое ДОЛГОЕ действие (settings.reloading → спиннер в форме).
5. Свёртка длинных записей (maxLines/превью) — опционально сюда, иначе фаза 5.
6. liveRegion области результатов + semanticsEnabled-прогон SnackBar-AT.
7. Мелочи одним коммитом: bottom-padding ListView, busy-gating copy (рекомендация: убрать gate на дашборде), тон «модель: …» в .arb (arb :205,213), clear-search suffix, empty-упаковка унификация.

## Security residual (погасить в этой фазе)

- `app/tool/status_smoke.dart:12` печатает `$error` целиком → перевести на `error.runtimeType`.

## Must-NOT (запреты)

- НЕ трогать: crates/ (весь Rust), протокол, app/pubspec.yaml/lock (без решения research), оверлей dicta-ui.
- НЕ писать конфиг напрямую, НЕ логировать значения set-команд (payload privacy, канон T-2-01/T-3-01).
- НЕ регистрировать глобальные хоткеи (фаза 5 отдельно), НЕ лезть в трей.
- НЕ удалять/перезаписывать файлы вне dest_dir(модели); активную модель не удалять.
- НЕ добавлять cyrillic в lib вне .arb; НЕ клонировать фабрику фейков (implements==1 гейт).
- История/дашборд не ломать: батарея 71/71 обязана остаться зелёной.

## Open questions → research

- DownloadManager в Dart: HttpClient + Range докачка, прогресс по content-length, отмена (закрыть client), лимит параллельных загрузок = 1 (как dicta-settings).
- HF API из Dart: те же эндпоинты, JSON через dart:convert; схема ответов уже известна (парсеры-аналоги hf.rs).
- Форма 17 ключей: группировка (Запись/Вставка/Движок/Система?), Slider-onChangeEnd-паттерн против спама set, confirm перед ENGINE-ключами.
- Где хранить VRAM-лимит: UI-only state (как dicta-settings) — в конфиг демона НЕ писать (ключа нет).

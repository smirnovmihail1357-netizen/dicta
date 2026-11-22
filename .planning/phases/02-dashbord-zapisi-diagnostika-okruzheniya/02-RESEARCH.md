# Phase 2: Дашборд записи + диагностика окружения - Research

**Researched:** 2026-11-22
**Domain:** Flutter UI поверх замороженного протокола dictad (поллинг status, команды start/stop/cancel/inject/set/settings)
**Confidence:** HIGH — все факты протокола прочитаны из исходников демона и приложения в этой сессии; новых пакетов нет, весь стек уже стоит.

## User Constraints (from CONTEXT.md)

### Locked Decisions (верbatim, CONTEXT.md §«Замеченные решения»)
1. Протокол демона заморожен: только существующие команды status/start/stop/cancel/inject/set/settings; никаких правок `crates/*`.
2. IPC-клиент уже существует: `app/lib/core/daemon_engine.dart` + `providers.dart` (поллинг 100 мс, backoff 200 мс→2 с).
3. Диагностика — только через существующие команды демона и локальные проверки; новых команд НЕ добавлять. Хоткей-путь — совет «проверь `dictactl toggle`», без дёргания хоткея.
4. UI-слой: Material You shell из фазы 1; новые экраны — файлы в `app/lib/features/dashboard/`, состояние — Riverpod поверх DictaEngine.
5. Русский UI только через gen-l10n; кириллица в lib/ вне generated/comments запрещена гейтом `app/tool/cyrillic_gate.sh`.
6. `inject` — команда демона; «скопировать» — Clipboard Flutter, локально, без демона.
7. Честность кнопок: отображаемое состояние — `base.state` из status-поллинга; во время `transcribing` отмена недоступна; после stop — ждать следующего status-тика, спиннер «распознаю…».
8. Калибровка min_level: гистограмма живого уровня + отметка порога; отправка `set min_level` по отпусканию ползунка (без спама).
9. `base.level` — float 0..1, `base.seconds` уже в status-ответе — время не считаем в UI локально.

### Verified facts (CONTEXT.md, сверено с кодом в этой сессии — подтверждено)
- Состояния демона — только три: `idle` / `recording` / `transcribing` [VERIFIED: crates/dictad/src/main.rs:91-98, `State::Idle => "idle"`, `State::Recording => "recording"`, `State::Transcribing => "transcribing"`]. «Паузы» нет.
- `inject` требует явный непустой текст ≤5000 символов [VERIFIED: crates/dictad/src/main.rs:886, `if text.is_empty() || text.chars().count() > 5000`]; ответ: `text: Some(text)` при успехе, `error: Some("inject: текст пустой или длиннее 5000 символов")` при пустом/длинном, `error: Some("вставка не удалась: {err}")` при ошибке вставки [VERIFIED: main.rs:887-933].

### Scope fences (OUT OF SCOPE)
История — фаза 3; настройки/модели — фаза 4; трей/хоткеи — фаза 5. Стриминг текста, JSONL-журнал, правки демона — вне вехи. Новые зависимости — нулевые; гистограмма рисуется стандартными виджетами (CustomPainter/LayoutBuilder).

## Summary

Фаза 2 — UI-контент поверх зелёного каркаса фазы 1: ни одного нового пакета, ни одной правки демона. Основная работа — (а) расширить шов `DictaEngine` так, чтобы дашборд получал последний текст и результаты команд, (б) построить экран `features/dashboard/` (состояние/уровень/секунды, кнопки, последний текст, калибровка) и карточку диагностики, (в) добавить ~25 новых ключей в `app_ru.arb`.

**Главное открытие, меняющее дизайн DASH-03:** ответ `status` **никогда не несёт текст** — ветка статуса жёстко возвращает `text: None, error: None` [VERIFIED: crates/dictad/src/main.rs:828-833]. Последний текст приходит только в ответе на команды `start|stop|toggle|cancel` (`text: h.last_text.clone()`, main.rs:978-984) и `inject`. Значит «последний распознанный текст» на дашборде — это текст, захваченный из ответа **собственного stop-а приложения**; диктовки, начатые/остановленные хоткеем демона, текст в приложение не приносят (протокол не имеет read-only команды для last_text). UI обязан честно показывать «нет распознанного текста» вместо выдумки.

**Второе открытие (против предположения в скоупе):** `set min_level` **разрешён во время записи** и применяется к живому конфигу немедленно — гейтинг `state != Idle` существует только для ENGINE_KEYS (model/language/use_gpu/gpu_device/n_threads/beam_size) в `schedule_engine_reload` [VERIFIED: main.rs:723-728, 750-754]; `min_level` в ENGINE_KEYS не входит, а воркер демона перечитывает пороги из живого конфига каждый тик [VERIFIED: main.rs:222-231, комментарий «Пороги берутся из живого конфиг каждый тик: панель настроек может сменить их прямо во время записи»]. Калибровка прямо во время записи — легальный и правильный UX.

**Primary recommendation:** расширить `DictaEngine` (stop возвращает снапшот `DictaStatus` вместо `DictaReply`, добавить `inject`), весь новый UI — в `app/lib/features/dashboard/` поверх Riverpod-провайдеров, вся русская строка — только новые ключи `app_ru.arb` + `flutter gen-l10n`.

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Состояние/уровень/секунды (DASH-01) | Flutter UI (watch `dictaStatusProvider`) | — | Поллинг уже в core; UI только читает |
| Кнопки записи (DASH-02) | Flutter UI → `DictaControlNotifier` | daemon (валидация) | UI решает доступность, демон — источник истинности состояния |
| Последний текст (DASH-03) | Flutter notifier, захват из stop/inject-ответа | daemon `last_text` | status текст не несёт — захват на стороне команд |
| Гистограмма/калибровка (DASH-04) | Flutter UI (CustomPainter + ring buffer) | daemon `set` | Данные — level из поллинга; порог — одна команда set |
| Диагностика (DIAG-01) | Flutter UI поверх status+settings | — | Ничего кроме существующих команд |

## 1. DASH-01 — состояние, живой уровень, секунды

**Состояние.** Уже готово: `ref.watch(dictaStatusProvider)` даёт `DictaStatus` ~10 Гц; русские метки `daemonStateIdle/Recording/Transcribing` уже в .arb. Ничего менять в core не нужно. Заменить `_DashboardPlaceholder` в `shell.dart` (IndexedStack, индекс 0) на `DashboardScreen` из `features/dashboard/dashboard_screen.dart`.

**Живой уровень.** `base.level` — float 0..1 [VERIFIED: doc-комментарий main.rs:9 `"level":0..1`; fixture]. Ключевой факт честности: **level ненулевой только во время записи** — воркер пишет `level.store(...)` лишь в состоянии Recording и обнуляет иначе (`level.store(0)` при не-записи) [VERIFIED: main.rs:232-234, 250-251, 266-268]. Оверлей dicta-ui делает так же (POLL_MS=100, окно WAVE_LEN=18, `cur.levels.clear()` вне записи) [VERIFIED: crates/dicta-ui/src/main.rs:59, 483-488; render.rs:31]. Внутренний тик сэмплирования демона — 80 мс, но клиент видит уровень только через status-ответы, т.е. ~10 Гц [VERIFIED: main.rs:215 `const TICK_MS: u64 = 80`].

Рекомендация: один переиспользуемый подход для DASH-01 и DASH-04 —
- **Live-бар (DASH-01):** тонкая полоса `CustomPaint` (или `LinearProgressIndicator(value: status.level.clamp(0.0, 1.0))`) — перерисовка 10 Гц дешёвая, StateNotifier уже триггерит rebuild. Перерисовку ограничить `AnimatedContainer`/репейнтом только при изменении значения.
- **Ring buffer (DASH-04):** `LevelHistoryNotifier` в `features/dashboard/level_history.dart` — ring buffer на `List<double>` с фиксированной ёмкостью `kLevelHistoryCapacity = 200` (≈20 с при 10 Гц; CONTEXT открытый вопрос №3 — in-memory, НЕ persist). Питание — `ref.listen(dictaStatusProvider, ...)` в `DashboardScreen` (ConsumerStatefulWidget): пуш только при `state == DaemonState.recording`, иначе буфер засоряется нулями. `IndexedStack` держит дашборд смонтированным при переключении вкладок — слушатель живёт всегда.

```dart
// features/dashboard/level_history.dart (скелет)
class LevelHistoryNotifier extends Notifier<List<double>> {
  static const capacity = 200; // ~20 s at 10 Hz; in-memory only
  @override
  List<double> build() => const <double>[];
  void push(double level) {
    final next = [...state, level];
    state = next.length > capacity ? next.sublist(next.length - capacity) : next;
  }
  void clear() => state = const <double>[];
}
final levelHistoryProvider =
    NotifierProvider<LevelHistoryNotifier, List<double>>(LevelHistoryNotifier.new);
```

Копия списка из 200 double 10 раз в секунду — пренебрежимо; зато стандартный Riverpod-паттерн без ручных ChangeNotifier.

**Секунды.** `base.seconds` — double, приходит в каждом status-ответе [VERIFIED: main.rs:828-831]; демон отдаёт 0.0 вне записи [VERIFIED: main.rs:799-808 `if State != Recording return 0.0`]. Не считать локально. Формат `mm:ss` из `Duration(milliseconds: (status.seconds * 1000).round())` — 3 строки, без intl-зависимостей для duration.

## 2. DASH-02 — честные кнопки

**Матрица доступности** (источник истинности — `status.state` из поллинга; плюс состояние «демон недоступен» из `status.connected`):

| status.state | «запись» (start) | «стоп» (stop) | «отмена» (cancel) | дополнительно |
|---|---|---|---|---|
| idle | активна (primary) | disabled | disabled | — |
| recording | disabled | активна (primary) | активна | счётчик секунд живой |
| transcribing | disabled | disabled | **disabled + «распознаю…»** | спиннер до Idle |
| connected == false | disabled (все) | disabled | disabled | баннер из фазы 1 уже есть |

Обоснование по коду демона:
- `start` при уже идущей записи — молчаливый no-op (`if State == Recording return Ok(())`, main.rs:1015-1016) — кнопка «запись» в recording не нужна;
- `stop` вне Recording — no-op (`run_stop` возвращает `Ok(None)` при `State != Recording`, main.rs:1036-1038);
- `cancel` срабатывает всегда, но **во время transcribing это ложная отмена**: ветка ставит state=Idle, однако воркер inference уже держит сэмплы и всё равно закончит распознавание и вставку [VERIFIED: main.rs:940-948 — cancel не трогает воркер; run_stop спавнит воркер раньше]. Поэтому отмена при transcribing честно недоступна (CONTEXT lock №7), в отличие от recording, где cancel реально выбрасывает аудио без распознавания.

**«Распознаю…» — семантика спиннера.** Ответ stop демона держит соединение до конца вставки [VERIFIED: main.rs:965-969, комментарий «Соединение со "stop" держим до конца вставки; воркер шлёт сигнал сразу после клавиш»]. Правильный UX: спиннер и подпись «распознаю…» привязаны к `state == transcribing` из **поллинга**, а не к pending-Future кнопки — тогда честно покрывается и случай, когда стоп нажали хоткеем демона (приложение стоп не слало, но состояние видит). Следующий status-тик с `idle` гасит спиннер. Ответ stop (Future) — отдельный источник: он приносит текст (см. §3). Текущий таймаут stop в `daemon_engine.dart` — 120 c (`_injectTimeout`, daemon_engine.dart:53) — достаточен для inference+вставки; не трогать.

**Куда кладём команды.** Новый `DictaControlNotifier` (`features/dashboard/dicta_control.dart`): методы `start()/stop()/cancel()`, состояние — `AsyncValue`-подобный флаг «команда в полёте» + `lastText`/`lastError` из ответов. Кнопки вызывают его; доступность считается от `dictaStatusProvider`, «в полёте» дополнительно дизейблит кнопку.

## 3. DASH-03 — последний текст, копирование, inject

**Источник текста — ответ команды, НЕ status** (см. Summary). Таблица текста/ошибки по командам [VERIFIED: main.rs:828-834, 886-933, 978-984]:

| Команда | text в ответе | error в ответе |
|---|---|---|
| status | всегда `None` | всегда `None` |
| start | `None` (start очищает `h.last_text`, main.rs:1018-1019) | `None` |
| stop / toggle(→stop) | распознанный текст | напр. `"слишком тихо (уровень … < …) — распознавание пропущено"` (main.rs:1080-1084) |
| cancel | `None` | `"запись отменена"` (main.rs:947) |
| inject (успех) | эхо вставленного текста (main.rs:915-921) | `None` |
| inject (пустой/длинный) | `None` | `"inject: текст пустой или длиннее 5000 символов"` |

**Изменение шва (app-side, протокол не трогаем):** сейчас `DictaEngine.stop()` возвращает `DictaReply` и **выбрасывает text из ответа** (`_commandLine` читает только `error`, daemon_engine.dart:83-97). Рекомендация:

```dart
// engine.dart — сигнатуры фазы 2
abstract interface class DictaEngine {
  // ...существующее...
  Future<DictaReply> start({String? app});   // ответ text не нужен (start его чистит)
  Future<DictaStatus> stop({String? app});   // ИЗМЕНЁН: ответ несёт последний текст
  Future<DictaReply> cancel();               // error "запись отменена" уже покрывает
  Future<DictaStatus> inject({required String text, String? app}); // НОВЫЙ
}
```

`DictaStatus.fromReplyLine` уже парсит text/error/state — переиспользовать для разбора ответа stop/inject. Реализация в `DaemonEngine`: `_snapshotCommand(cmd, app, timeout: _injectTimeout)` — тот же `_call`, но `DictaStatus.fromReplyLine` на выходе. **Таймаут inject: 120 c, НЕ 30 с** — вставка выполняется синхронно в `serve_conn` и включает ожидание восстановления буфера, а `clipboard_restore_ms` конфигурируем до 60000 мс [VERIFIED: main.rs:548 `count(200, 60_000)`]; 30-секундный таймаут ложно провалил бы легальную вставку. Константа `_injectTimeout = Duration(seconds: 120)` уже существует (daemon_engine.dart:53).

**Copy — без демона:** `Clipboard.setData(ClipboardData(text: ...))` из `package:flutter/services.dart` (SDK, нулевая зависимость). Для тестируемости обернуть в провайдер-шов:

```dart
final clipboardWriterProvider =
    Provider<void Function(String)>((ref) => (text) =>
        Clipboard.setData(ClipboardData(text: text)));
```

**Inject — честная подпись.** `inject` вставляет в **активное на момент команды окно**, которое при клике по кнопке приложения — само окно dicta-app (демон резолвит фокус на своей стороне: `dicta_core::focus::active_app(req.app.as_deref())`, main.rs:895-899; `app` в запросе — только подсказка клавиш, не указатель окна). Практические следствия для UI:
- кнопка «вставить заново» полезна как проверка механизма вставки и для вставки из чужой диктовки; подпись честная: «вставка идёт в активное окно»;
- для реальной вставки в другое окно совет (l10n-строкой): пользуйся хоткеем демона.
- Кнопка disabled при `lastText == null || lastText.isEmpty` (демон вернёт ошибку на пустой — не слать заведомо невалидное).

**Хоткейные диктовки не видны приложению** — известное ограничение протокола: last_text читается только ответами action-команд. Отображать честный placeholder «нет распознанного текста». (Хак «опрос через cancel в idle» возвращает last_text, но затирает `last_error` значением `"запись отменена"` — отвергнут; заметить для фазы 3, где HIST-01 сформулирован как «текст из status-потока», что с кодом демона не сходится — план фазы 3 обязан это учесть.)

## 4. DASH-04 — гистограмма и калибровка min_level

**Данные.** Ring buffer §1: 200 сэмплов `base.level`, пуш только при recording. Гистограмма пустая вне записи — честная подпись «начните запись, чтобы увидеть уровень».

**Рисование.** `CustomPainter` поверх `LayoutBuilder` (CONTEXT lock: без пакетов): столбики уровня (высота = level, ширина = width/200), вертикальная линия-отметка min_level. Цвет-подсказка по образцу пилюли dicta-ui: shades порогов 0.6 / 0.3 [VERIFIED: render.rs:718-721 `let shade = if level > 0.6 … else if level > 0.3`] — опционально.

**Текущее значение min_level.** Из `settings`-ответа: `SettingsReply.minLevel` уже есть (settings.dart:93, fixture `"min_level":0.35`). Добавить `settingsSnapshotProvider` (FutureProvider на `engine.fetchSettings()`, обновлять после каждого успешного `set` — ответ `set` уже несёт свежий `settings`-снапшот в поле `settings` [VERIFIED: main.rs:730-732 `*extra = Some(settings_view(&h, reloading))`], можно не рефетчить).

**Маркер и применение.** StatefulWidget-состояние `markerValue` (инициализация из `settingsSnapshotProvider.minLevel`); `GestureDetector.onHorizontalDragUpdate` двигает маркер локально (0..1), `onHorizontalDragEnd` шлёт **одну** команду:

```dart
await engine.set('min_level', markerValue.toStringAsFixed(2)); // формат как у панели: {:.2}
```

[VERIFIED: формат панели — `format!("{:.2}", …)`, crates/dicta-ui/src/panel.rs:290-297; диапазон демона 0.0..=1.0, вне — ошибка `"min_level: {value} вне диапазона 0..1"`, main.rs:523-531, 543.]

**set во время записи разрешён** (см. Summary): `set_setting` не гейтируется по состоянию вовсе; `schedule_engine_reload` с гейтом `state != Idle` вызывается только для `change.needs_engine()` [VERIFIED: main.rs:718-728], а `needs_engine()` = членство в `ENGINE_KEYS` = `"model", "language", "use_gpu", "gpu_device", "n_threads", "beam_size"` [VERIFIED: main.rs:438-442 + settings.dart kEngineKeys:30-37]; `min_level` там нет. Следствие: калибровка работает прямо во время записи — рекомендованный флоу: «запись → тишина → смотришь гистограмму → двигаешь маркер → отпускаешь». **Внимание к auto_stop:** если у пользователя включён auto_stop (по умолчанию off, fixture), тишина сама остановит запись и запустит распознавание — в подсказке калибровки предложить пользоваться «отмена» (cancel выбрасывает аудио без распознавания/вставки, main.rs:940-948).

**Честный ответ set.** Успех: `error: None`, а `text` = заметка вида `"min_level = 0.5 — сохранено в /path/config.toml"` или `"… — уже так"` [VERIFIED: main.rs:687-716]. Отказ парсинга: `error: Some(...)`, значение НЕ применено. UI: показать заметку/ошибку из ответа как есть (строки демона уже русские — это данные, не литералы, гейт не касается).

## 5. DIAG-01 — чеклист диагностики

**Автоматически проверяемое** (только существующие команды — CONTEXT lock №3):

| Пункт | Сигнал | Цвет |
|---|---|---|
| Демон жив / сокет отвечает | `status.connected == true` из поллинга (фаза 1) | зелёный / красный + подсказка `"systemctl --user start dictad"` (хинт уже в daemon_engine.dart:26-29) |
| Модель найдена и загружена | `settings.model` непуст + `reloading == false` | зелёный; `reloading == true` → нейтральный «модель загружается…» (reloading стартует true при загрузке демона [VERIFIED: main.rs:179 `let reloading = Arc::new(AtomicBool::new(true));`]) |
| Хоткей-путь | НЕ проверяется автоматически | нейтральный, ручной: «проверь хоткеем: `dictactl toggle`» |
| Вставка работает | НЕ проверяется автоматически (вставка идёт в активное окно) | нейтральный, ручной: «продиктуй что-нибудь в поле ввода» |

Оговорка честности для «модель загружена»: `reloading == false` означает «демон не грузит модель сейчас», но провал прошлой загрузки (`h.last_error = "модель …: {err}"`) читается только из ответов action-команд, не из settings — полную гарантию даёт только живая диктовка. Отразить в подписи пункта (данные демона, не обещание).

**Рекомендация по размещению:** карточка «Диагностика» секцией внизу дашборда (shell не трогаем), строки со state-цветом (success/error/neutral из colorScheme), кнопка «перепроверить» = принудительный `poll()` + рефетч `settingsSnapshotProvider`. Все строки, кроме двух автоматических, — ручные с подсказкой починки; это закрывает CONTEXT открытый вопрос №2 тем же способом, который он сам предлагает.

## 6. L10n и кириллический гейт

- Все новые русские строки — только новые ключи `app/lib/l10n/app_ru.arb` + `flutter gen-l10n` (генерат в `lib/l10n/generated/` коммитится, гейтом исключён — cyrillic_gate.sh:8-10).
- Семантика гейта проверена по скрипту: он считает **не-комментные** строки с кириллицей в `lib/**/*.dart`, исключая `lib/l10n/generated/` и строки-комментарии (`grep -vE ':[0-9]+:[[:space:]]*(//|/\*|\*)'`) [VERIFIED: app/tool/cyrillic_gate.sh:1-19]. Значит: русские комментарии формально проходят, но стиль фазы 1 — английские комментарии в app/lib; **рекомендация: сохранять английские комментарии**, runtime-строки — только l10n. Тестовые файлы гейт не сканирует (только lib/), русские литералы в `find.text(...)` — легальны (уже так в shell_test.dart).
- Ошибки демона (русские) отображаем как данные — гейт не нарушается.
- Новые ключи (~25): `dashStartButton` «начать запись», `dashStopButton` «остановить», `dashCancelButton` «отмена», `dashTranscribing` «распознаю…», `dashRecordingTime` «время {time}», `dashLastTextTitle` «последний текст», `dashNoText` «нет распознанного текста», `dashNoTextHint` «текст появится после остановки записи из приложения», `dashCopyButton` «скопировать», `dashCopied` «скопировано», `dashInjectButton` «вставить заново», `dashInjectHint` «вставка идёт в активное окно», `dashInjectFailed` «вставка не удалась», `calibrationTitle` «порог тишины», `calibrationEmptyHint` «начните запись, чтобы увидеть уровень», `calibrationApplyHint` «отпусти ползунок — порог применится», `calibrationAutoStopHint` «включён авто-стоп — для калибровки нажимай отмена», `diagTitle` «диагностика», `diagDaemonAlive` «демон жив», `diagSocketAnswers` «сокет отвечает», `diagModelReady` «модель готова», `diagModelLoading` «модель загружается…», `diagHotkeyManual` «хоткей — проверь вручную: dictactl toggle», `diagInjectManual` «вставку проверь живой диктовкой», `diagRecheck` «перепроверить», `diagStartHint` «запусти: systemctl --user start dictad». Метки состояний (`daemonState*`) уже существуют.

## Don't Hand-Roll

| Проблема | Не строить | Использовать | Почему |
|---|---|---|---|
| Русские строки | литералы/escapes в dart | .arb + gen-l10n | гейт + APP-05 |
| Кольцевая история уровня | свой ChangeNotifier-фреймворк | Riverpod Notifier (§1 скелет) | единый паттерн фазы 1 |
| Буфер обмена | каналы платформы руками | `Clipboard.setData` (flutter/services) | SDK |
| JSON наружу | ручное экранирование | `jsonEncode` (уже в engine.dart) | APP-06 |
| Диалоги ошибок | свой toast-механизм | `SnackBar` Material 3 | нулевая зависимость |

## Common Pitfalls

1. **Ждать текст из status** — status всегда `text: None` [VERIFIED main.rs:828-833]; текст только в ответах action-команд. Проверка плана: нигде нет чтения `status.text` как «последнего текста».
2. **Отмена при transcribing** — демон ставит Idle, но воркер вставит текст всё равно (main.rs:940-948); UI обязан дизейблить отмену в transcribing.
3. **Таймаут inject 30 с** — вставка синхронна в соединении и включает clipboard_restore_ms ≤ 60000 [VERIFIED main.rs:548]; ставить 120 с (`_injectTimeout`).
4. **Спам set при драге** — слать `set` на каждый onDragUpdate захлебнёт демон записями конфига; только onHorizontalDragEnd.
5. **Пушить level вне записи** — нули вне записи (main.rs:250-267) утянут гистограмму к полу и обесценят калибровку; гейтить по `state == recording`.
6. **Поломка компиляции фейков** — изменение сигнатуры stop/добавление inject ломает `_FakeEngine` в `widget_test.dart` и `FakeDictaEngine` в `shell_test.dart`; править в той же задаче, что и интерфейс.
7. **«Вставить заново» в чужое окно** — демон резолвит активное окно на своей стороне (main.rs:895-899); кнопка приложения вставит в само приложение — подпись честности обязательна.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | flutter_test (SDK, Flutter 3.50.1 — установлен и проверен в этой сессии) |
| Config file | `app/analysis_options.yaml` (flutter_lints 6); тестовой конфигурации не нужно |
| Quick run command | `cd app && flutter test` |
| Full suite command | `cd app && flutter test && ./tool/cyrillic_gate.sh` |

### Phase Requirements → Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| DASH-01 | Метка состояния/уровень/секунды на каждом из 3 состояний | widget (FakeDictaEngine) | `flutter test test/dashboard_state_test.dart -x` | ❌ Wave 0 |
| DASH-01 | Ring buffer: ёмкость 200, порядок, clear | unit | `flutter test test/level_history_test.dart -x` | ❌ Wave 0 |
| DASH-02 | Матрица доступности кнопок × {idle, recording, transcribing, disconnected} | widget | `flutter test test/dashboard_buttons_test.dart -x` | ❌ Wave 0 |
| DASH-02 | «распознаю…» + спиннер видны при transcribing; гаснут при idle-тике | widget (pump 100 мс тики) | там же | ❌ Wave 0 |
| DASH-03 | stop-ответ с text → карточка текста; копирование через фейковый clipboard-шов | widget | `flutter test test/last_text_test.dart -x` | ❌ Wave 0 |
| DASH-03 | inject: шлёт `{"cmd":"inject","text":…}` (jsonEncode); disabled при пустом; error из ответа показан | widget (fake отвечает ok:false c error) | там же | ❌ Wave 0 |
| DASH-04 | Драг маркера → ровно один `set('min_level','0.42')` на отпускание; формат `{:.2}` | widget (fake считает вызовы) | `flutter test test/calibration_test.dart -x` | ❌ Wave 0 |
| DIAG-01 | Зелёный при connected+reloading=false; нейтральный при reloading=true; красный при disconnected; «перепроверить» рефетчит | widget | `flutter test test/diagnostics_test.dart -x` | ❌ Wave 0 |
| smoke | Живой демон: status-тик + безвредный `inject ""` (ответ с error «текст пустой…», ноль побочных эффектов — вставка не вызывается, main.rs:886-893) | integration (ручной, tool-скрипт) | `dart run tool/diag_smoke.dart` | ❌ Wave 0 |

### Sampling Rate
- **Пер тик:** поллинг 100 мс уже в `DictaStatusNotifier` — в widget-тестах тики сжимаются `tester.pump(const Duration(milliseconds: 100))` (паттерн фазы 1: pump 300 мс в shell_test.dart).
- **Per task commit:** `cd app && flutter test && ./tool/cyrillic_gate.sh`
- **Phase gate:** full suite + UAT-сценарии ниже.

### Manual UAT (не автоматизируется честно)
- Живой уровень и цвет столбиков на реальном микрофоне (визуальная полировка).
- Калибровка на живой записи: тишина/речь различимы, порог применяется и переживает перезапуск демона (сохранение в config.toml).
- Inject в реальное окно (хоткей-путь) и поведение кнопки «вставить заново» (вставка в активное окно).

### Wave 0 Gaps
- [ ] `app/test/fake_dicta_engine.dart` — вынести единую фабрику фейков (сейчас 2 инлайн-копии в widget_test.dart/shell_test.dart ломаются от изменения интерфейса) с сценариями: статус-снапшот, счётчик `set`, скриптуемые ответы stop/inject
- [ ] Все новые тест-файлы из таблицы выше
- [ ] `app/tool/diag_smoke.dart` — по образцу `tool/status_smoke.dart` (фаза 1)
- [ ] Новые ключи .arb + `flutter gen-l10n` — до первого виджета

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Flutter SDK | всё | ✓ | 3.50.1 (проверено `flutter --version` в этой сессии) | — |
| dictad (живой демон) | smoke/UAT | ✓ | сокет `/run/user/1000/dicta.sock` присутствует (проверено) | тесты идут на фейках |
| flutter_riverpod 3.5.0 | состояние | ✓ | pubspec.lock фазы 1 | — |
| Новые пакеты | — | не требуются | — | — |

**Missing dependencies with no fallback:** нет.

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | Кнопка «вставить заново» вставит текст в само окно приложения (фокус на нём в момент клика) | §3, §5 | Низкий — прямое следствие verified-кода main.rs:895-899; подпись честности покрывает любой исход |
| A2 | `reloading == false && model != ""` достаточно как «модель готова» для чеклиста | §5 | Низкий — подпись пункта честно говорит «по данным демона» |
| A3 | 200 сэмплов (~20 с) — достаточно для калибровочной гистограммы | §1 | Нет — константа, меняется одной строкой |
| A4 | HIST-01 (фаза 3) сформулирован вопреки протоколу («текст из status-потока») | §3 | Средний — план фазы 3 спроектирует неверный захват истории; заметка внесена, планер фазы 3 обязан перечитать |

## Sources

### Primary (HIGH confidence, прочитано в этой сессии)
- `crates/dictad/src/main.rs` — states 91-98, status 828-833, inject 878-935, start/stop/toggle/cancel 936-984, cancel-семантика 940-948, run_start 1014-1024, run_stop 1030-1068, level-воркер 215-272, elapsed_seconds 799-808, set_setting 641-740, schedule_engine_reload 744-783, parse_change 497-549, Reply/serde(flatten) 351-359, reloading-init 179
- `app/lib/core/{engine,daemon_engine,providers,settings}.dart`, `app/lib/features/shell/shell.dart`, `app/lib/main.dart`, `app/lib/l10n/app_ru.arb`, `app/l10n.yaml`, `app/tool/{cyrillic_gate.sh,status_smoke.dart}`, `app/test/*`, `app/test/fixtures/settings_reply.json`
- `crates/dicta-ui/src/{main.rs,render.rs,panel.rs}` — POLL_MS/WAVE_LEN/уровень/формат min-кнопок
- `.planning/phases/01-karkas-dicta-app-ipc-klient/01-RESEARCH.md` — fixture, ENGINE_KEYS, история решений

### Metadata
**Confidence breakdown:** протокол/состояния демона — HIGH (всё прочитано из исходников с цитатами); архитектура Riverpod — HIGH (паттерны фазы 1 живы в коде); UX-детали inject/фокуса — MEDIUM (следствие кода, живое поведение проверить на UAT).
**Research date:** 2026-11-22. **Valid until:** протокол заморожен на всю веху — устаревание связано только с правками app/, которых фаза не планирует.

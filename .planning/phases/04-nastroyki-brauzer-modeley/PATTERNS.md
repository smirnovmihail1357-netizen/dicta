# Phase 4: Настройки + браузер моделей - Pattern Map

**Mapped:** 2026-11-22
**Files analyzed:** 12 (7 новых, 5 модифицируемых)
**Analogs found:** 12 / 12 (из них 3 — «behavior-only», паритет по Rust-эталону)

Все названные аналоги — git-tracked (сверено `git ls-files`, 5/5 выборочно;
весь `app/lib` и `crates/dicta-settings/src` — tracked, зеркал нет).
Кириллица в `app/lib/**/*.dart` вне `lib/l10n/generated/` запрещена гейтом
`app/tool/cyrillic_gate.sh` — весь русский UI только через `.arb`; русский
текст в core/features при необходимости — Unicode-escape (канон
`dicta_control.dart:12-17`, `daemon_engine.dart:24-31`).
Гейт единственной фабрики фейков живой: `grep -c "implements DictaEngine"`
→ только `fake_dicta_engine.dart` (== 1). Фейк РАСШИРЯТЬ, не клонировать.
Rust-файлы `crates/dicta-settings/*` — эталон ПОВЕДЕНИЯ (READ-ONLY), не кода:
их curl-shell-out и egui-структура в Dart НЕ копируются (CONTEXT:22,42).

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|---|---|---|---|---|
| `app/lib/features/settings/settings_screen.dart` (new) | component (экран, 17 ключей, группировка) | request-response (set) | `calibration_card.dart:62-98,156-164` (commit-sliders) + `history_screen.dart` (scaffold/поля/snackbar) | exact (роль+поток) |
| `app/lib/features/models/models_screen.dart` (new) | component (каталог + HF + VRAM + прямая ссылка) | request-response + streaming (прогресс) | `history_screen.dart` (screen scaffold, поле+MergeSemantics, список, кнопки) + `dashboard_screen.dart:72-75` (прогресс-бар) | exact (роль; стриминг-часть — см. No Analog) |
| `app/lib/core/downloads.dart` (new: DownloadManager) | service | streaming + file-I/O (.part, resume, cancel) | структурный: `daemon_engine.dart:44-93` (бюджеты, errors-as-data); поведение: `dicta-settings main.rs:223-262,366-416` | role-match; кодовой формы НЕТ (No Analog: streaming) |
| `app/lib/core/hf_client.dart` (new: HF seam: URL/парсеры + HTTP GET) | service + utility | request-response | `engine.dart:88-100` (чистые build*Line/decodeReplyMap) + `daemon_engine.dart:49-93` (транспорт, таймауты); поведение: `hf.rs` (точные URL/парсеры + юнит-тесты) | exact (роль+поток) |
| `app/lib/features/models/model_store.dart` (new: список моделей + удаление, riverpod) | store + file-I/O (delete) | CRUD (list/delete локальных файлов) | `history.dart:27-58` (Notifier) + `history_screen.dart:199-231` (destructive-канон) | role-match (delete — No Analog: dart:io) |
| `app/lib/core/providers.dart` (modify: провайдеры загрузки/HF/VRAM-лимита) | store | event-driven (прогресс) | сам файл: `DictaStatusNotifier` :36-73 (inFlight bool + backoff + ref.onDispose) + `dicta_control.dart:23-62` (commandInFlight state) | exact |
| `app/lib/features/shell/shell.dart` (modify: 2 плейсхолдера → экраны) | component | — | сам файл: destinations :56-77, IndexedStack-слоты :81-89, `_SectionPlaceholder` :100-113 | exact |
| `app/lib/core/engine.dart` + `daemon_engine.dart` (modify, ЕСЛИ seam отдаёт снимок из set) | service (seam) | request-response | сам файл: `fetchSettings` :156-159 (полная карта ответа) | exact — см. «Дилемма set-снимка» ниже |
| `app/test/fake_dicta_engine.dart` (modify: крюки под новые сценарии) | test | request-response | сам файл: hooks :16-28, gates :34-38, counters :68-72, `onSet` :57-63 | exact |
| новые сьюты: `settings_screen_test.dart`, `models_screen_test.dart`, `downloads_test.dart`, `hf_client_test.dart` | test | все потоки | `settings_model_test.dart` (golden-фикстуры), `calibration_test.dart` (set-flow), `history_screen_test.dart` (widget-сьют) | exact |
| `app/lib/l10n/app_ru.arb` + generated (modify: ключи `conf*`/`model*`) | config (l10n) | — | сам файл: префиксные семейства `dash*` :59-125, `hist*` :126-157, `diag*` :185-240 | exact |
| `app/tool/status_smoke.dart` (modify: `$error` → `error.runtimeType`) | tool | request-response | `03-…/03-SECURITY` residual-паттерн; сам файл :11-13 | exact |

## Pattern Assignments

### `app/lib/features/settings/settings_screen.dart` (component, request-response)

**Analog 1 (commit-контролы: один set на отпускание):**
`app/lib/features/dashboard/calibration_card.dart:62-98`

```dart
class _CalibrationCardState extends ConsumerState<CalibrationCard> {
  double? _marker;
  String? _error;

  /// Local-only move: no I/O until the gesture or slider drag is released.
  void _previewMarker(double value) {
    setState(() => _marker = value.clamp(0.0, 1.0).toDouble());
  }

  /// One set command per completed interaction.
  Future<void> _commitMarker(double value) async {
    final next = value.clamp(0.0, 1.0).toDouble();
    setState(() => _marker = next);
    final reply = await ref
        .read(engineProvider)
        .set('min_level', next.toStringAsFixed(2));
    // No ref use after await until the element is known alive.
    if (!mounted) return;
    if (reply.error != null) {
      // Keep the locally chosen marker on failure: no stale snapshot read
      // must overwrite the value we just sent.
      setState(() => _error = reply.error);
      return;
    }
    ref.invalidate(settingsSnapshotProvider);
    // A FutureProvider's snapshot is the PREVIOUS AsyncData right after
    // refresh/invalidate in Riverpod 3.5 - await the refetch (CR-01).
    final fresh = await ref.read(settingsSnapshotProvider.future);
    if (!mounted) return;
    setState(() {
      _error = null;
      _marker = fresh.minLevel; // daemon's canonical value
    });
  }
```

Копировать для ВСЕХ слайдеров формы (`min_level`, `auto_stop_level`,
`auto_stop_after_ms`, `auto_stop_grace_ms`, `max_record_ms`,
`clipboard_restore_ms`, `paste_delay_ms`, `n_threads`, `beam_size`):
`onChanged` → локальный preview (состояние стейта, НЕ set),
`onChangeEnd` → один `set` (calibration_card.dart:156-164):

```dart
Slider(
  key: const Key('calibrationSlider'),
  value: marker.clamp(0.0, 1.0).toDouble(),
  divisions: 100,
  label: marker.toStringAsFixed(2),
  onChanged: _previewMarker,
  onChangeEnd: _commitMarker,
)
```

Формат значения — как у демона: `toStringAsFixed(2)` для float-ключей,
`toString()` для int/bool/string. Отказ ENGINE-ключей (dictad main.rs:744-762
через CONTEXT:18) рендерить через `_error` + подсветку группы ENGINE
(`kEngineKeys`, settings.dart:30-37), честно, без吞吞 retry.

**Analog 2 (каркас экрана, поле ввода, empty, snackbar):**
`app/lib/features/history/history_screen.dart`
- скаффолд экрана без собственного Scaffold (Column, :27-50);
- поле текстового ключа (`paste_keys`): `MergeSemantics` + `Semantics(label:)`
  вокруг `TextField` (:33-47), ключ `Key('settingsPasteKeysField')` по канону
  `Key('historySearchField')` :37; без TextEditingController-двойного состояния
  (:43-46);
- empty/подсказки: `AppTextStyles.mutedOf(context)` (:69,79);
- `titleMedium`-заголовки секций (= заголовок карточки calibration :114-117;
  наследие UI-аудита №1: существительное-заголовок);
- busy-gating кнопок: `busy ? null : ...` (:134-136).

**Analog 3 (17 ключей и их типы):** `app/lib/core/settings.dart`
- `kEditableKeys` :10-28 — ровно 17 редактируемых ключей;
- `kEngineKeys` :30-37 — 6 ключей с reload и возможным отказом;
- толерантные геттеры `_string/_double/_int/_bool/_strings` :39-63 — НЕ
  клонировать: `SettingsReply` уже распарсен, форма читает
  `ref.watch(settingsSnapshotProvider).value`.

**Дилемма set-снимка (решение за планировщиком):** демон возвращает в
set-ответе свежий settings-снимок (dictad main.rs:731,737-741 через
CONTEXT:19), но текущий шов `set()` возвращает `DictaReply{ok,error}`
(engine.dart:85) и снимок ТЕРЯЕТ. Два пути, оба с аналогом:
1. Расширить шов: `set()` → полная карта ответа (как `fetchSettings()`
   :84, daemon_engine.dart:156-159), парсить `SettingsReply.fromReply`.
   Трогает interface + fake + calibration — оценить цену.
2. Оставить `DictaReply` и следовать рабочему прецеденту calibration
   (invalidate + await refetch, CR-01, :88-92). Дешевле, но +1 fetchSettings
   на каждый set.

---

### `app/lib/features/models/models_screen.dart` (component, request-response + streaming-отображение)

**Analog 1 (каркас, поиск, список, кнопки):**
`app/lib/features/history/history_screen.dart`
- экран = `Column` c полем поиска сверху (`MergeSemantics`, :29-50);
- список = `ListView.builder` c `Card`-строками (:82-237);
- пустые состояния — двухуровневые, честные (`histEmpty`/`histSearchNoMatches`,
  :52-81): для HF — «ничего не нашлось» (паритет `main.rs:338`), для дерева —
  «нет .bin файлов» (паритет `main.rs:347`);
- активная модель = `SelectableLabel`-аналог → Flutter `RadioListTile`/
  выделенная строка с «✓» и `set('model', path)` (паритет `main.rs:501-515`);
- кнопка скачивания — `FilledButton` (primary-канон), `Elevated` не для
  скачивания (канон кнопок CONTEXT:24).

**Analog 2 (прогресс-бар):** `dashboard_screen.dart:72-75`

```dart
LinearProgressIndicator(
  value: status.level.clamp(0.0, 1.0).toDouble(),
  minHeight: 8,
),
```

Паритет `main.rs:529-546`: если `expected > 0` — `value: downloaded/expected`
+ проценты; иначе — indeterminate (`LinearProgressIndicator()` без value) +
живой счётчик МБ (`tabularFigures`, как history_screen.dart:96-100).
Отмена — `OutlinedButton` error-foreground (см. Shared: destructive).

**Analog 3 (VRAM-слайдер):** `calibration_card.dart:156-164` (Slider +
divisions) — НО int-диапазон 2..32 с шагом 1 и UI-only состоянием (в конфиг
демона НЕ писать, CONTEXT:53; как `vram_limit_gb` в `main.rs:157` — память
приложения, не конфиг). Хранить вriverpod-нотификаторе, как
`HistoryQueryNotifier` (history.dart:60-68).

**Analog 4 (прямая ссылка):** поле = history search field (:33-47);
валидация `http://`/`https://` + имя файла — чистая функция (поведенческий
паритет `hf.rs:155-165 filename_of`), кнопка неактивна при `downloading`
(паритет `main.rs:759`).

---

### `app/lib/core/downloads.dart` (service, streaming + file-I/O)

**Структурный код-аналог:** `app/lib/core/daemon_engine.dart:38-93`
Единственный файл, знающий транспорт (зеркало правила APP-03):
- именованные таймауты-бюджеты константами (:44-47) — для HttpClient свои
  (connect / отсутствие данных);
- «один вызов = один клиент» вместо «одна команда = один сокет», но тот же
  принцип: без долгоживущего соединения, кроме активной загрузки;
- errors-as-data: НИКОГДА не бросать исключение в UI — обёртка
  `try { ... } on SocketException catch (error) { return ...; } on
  TimeoutException { ... } catch (_) { ... }` (:77-93, :103-126);
- payload-privacy: в сообщениях об ошибках — тип и путь, не тело
  (комментарий :57-58).

**Поведенческий паритет (Rust, READ-ONLY, НЕ код):**
- `main.rs:224-262` `start_download`: ОДНА загрузка одновременно
  (`if self.download.is_some() { return; }`, паритет «лимит = 1» CONTEXT:51);
  dest = `catalog::dest_dir(settings.model).join(file)`; «уже есть» →
  честная пометка, не перезапись (:230-233); `create_dir_all` (:234-239);
  `.part`-файл рядом с dest.
- `main.rs:366-375` `cancel_download`: убить процесс, СТЕРЕТЬ `.part`
  (следующий старт с нуля), flash «загрузка прервана».
- `main.rs:377-416` `poll_download`: успех → rename `.part`→dest,
  `refresh_snapshot()` (модель сразу видна в `settings.models`), flash
  «выбери её в списке»; падение → `.part` СОХРАНЁН для докачки.
- `catalog.rs:54-58` `part_of`: dest + `.part`.
- `catalog.rs:62-88` `dest_dir`: parent(текущей модели), иначе
  `$XDG_DATA_HOME | ~/.local/share` + `/whisper-cpp` (учёт XDG_DATA_HOME).
- В Dart: `dart:io HttpClient` + `Range: bytes=<part.length>-` для докачки
  (аналог curl `-C -`), прогресс по `Content-Length` из ответа 206/200,
  отмена = `client.close(force: true)` / destroy. НИКАКИХ
  `Process.run('curl')` (CONTEXT:22).

**Кодовой формы НЕТ** — см. No Analog Found: в app/ нет стриминга
(grep `HttpClient|Process.run` по lib/ → только `Process.runSync('id')` в
daemon_engine.dart:20). Планировщик опирается на RESEARCH + Rust-поведение.

---

### `app/lib/core/hf_client.dart` (service + utility, request-response)

**Analog 1 (чистые функции шва):** `app/lib/core/engine.dart:88-100`

```dart
Map<String, dynamic> decodeReplyMap(String line) { ... }

String buildStatusLine() => '${jsonEncode({'cmd': 'status'})}\n';

String buildSetLine(String key, String value) =>
    '${jsonEncode({'cmd': 'set', 'key': key, 'value': value})}\n';
```

Копировать СТИЛЬ: URL-билдеры и парсеры — чистые top-level функции без
класса и без сети, юнит-тестируемые без интернета. Поведение — построчный
паритет с `hf.rs` (у каждого парсера уже есть готовые юнит-тесты с битым
JSON — переносить кейсы дословно):

| Dart-функция | Паритет (hf.rs) | Кейсы в hf.rs tests |
|---|---|---|
| `urlEncode` | :5-15 | :172-177 |
| `browseUrl(query, sort, asrOnly, offset)` | :21-35 | :179-207 |
| `treeUrlAt(repo, subpath)` | :38-44 | :209-219 |
| `resolveUrl(repo, path)` | :47-49 | :221-227 |
| `parseSearch(json) → List<HfRepo>` (id/modelId, downloads, likes, 0 при мусоре) | :60-83 | :229-258 |
| `parseTree(json)` (type=file, `.bin`, `lfs.size` при size=0) | :93-127 | :260-293 |
| `parseTreeDirs(json)` | :130-147 | :295-309 |
| `destName(path)` | :150-152 | :319-325 |
| `filenameOf(url)` (strip `?…`, None без имени) | :155-165 | :327-341 |

Толерантность парсеров = канон `DictaStatus.fromReplyLine`
(engine.dart:33-52): битый JSON → пустой список/снимок, не исключение.

**Analog 2 (транспортный слой):** `daemon_engine.dart:49-93` — один класс
`HfClient` (или `httpGetJson(url)`), свежий `HttpClient` на запрос или один
переиспользуемый, errors-as-data, таймаут-бюджет константой. Каталог 6
моделей — `const`-список в этом же слое (паритет `catalog.rs:15-46`:
tiny 32МБ … large-v3 1.1ГБ + note-подсказки; тексты note — в .arb).

**Analog 3 (интерфейс-шов для тестов):** `engine.dart:69-86` — если сети
нужен шов `abstract interface class HfClient`, фейк строится по канону
`fake_dicta_engine.dart` (hooks в конструкторе, Completer-гейты для
«запрос в полёте», счётчики вызовов). Это ВТОРОЙ implements — гейт
«== 1» касается только `DictaEngine`, но правило «одна фабрика на шов»
сохранить.

---

### `app/lib/features/models/model_store.dart` (store + file-I/O)

**Analog 1 (notifier с in-memory state):** `history.dart:27-58`
— `class … extends Notifier<…>`, `build()` возвращает пустое/начальное,
мутаторы пересобирают state, `NotifierProvider` в том же файле (:57-58).

Содержимое: список моделей из `SettingsReply.models` (settings.dart:151) +
пометка активной (`== settings.model`) — активную НЕ удалять (CONTEXT:23,45);
путь удаления = `dest_dir` модели, НИКОГДА не вне dest_dir (CONTEXT:45);
`File(...).delete()` через dart:io, ошибка — errors-as-data.

**Analog 2 (destructive-кнопка):** `history_screen.dart:199-231`

```dart
OutlinedButton(
  key: Key('histDeleteButton-${entry.id}'),
  // Destructive intent: error foreground on an outlined shell
  // (cancel-button canon).
  style: OutlinedButton.styleFrom(
    foregroundColor: colors.error,
    side: BorderSide(color: colors.error),
  ),
  ...
```

Undo у истории есть (:216-227), у удаления файла НЕТ → перед удалением
confirm-диалог (наследие UI-аудита №2; формы диалога в кодовой базе НЕТ —
см. No Analog).

---

### `app/lib/core/providers.dart` (modify, store)

**Analog: сам файл** :36-73 (`DictaStatusNotifier`) и
`dicta_control.dart:23-62` (`DictaControlState` + copyWith).

- Нотификатор загрузки: состояние = `null | active(file, url, expected) |
  done/failed`; обновление прогресса — event-driven, без таймера (стрим
  HttpClient рапортует байты; таймер-поллинг status-поллера НЕ копировать);
  `_inFlight`-bool-гвард (:38,53-54) — чтобы cancel/progress-события не
  реентерили;
- `ref.onDispose` + `_disposed` (:46-49) — для загрузки ОБЯЗАТЕЛЬНО:
  dispose отменяет HttpClient;
- VRAM-лимит: `Notifier<double>` по образцу `HistoryQueryNotifier`
  (history.dart:60-68), default 24, UI-only;
- бэкофф :60-69 — НЕ нужен (сетевые ошибки HF показываются пользователю,
  не поллятся).

### `app/lib/features/shell/shell.dart` (modify)

Слоты готовы: destinations c `modelsTitle`/`settingsTitle` уже объявлены
(:67-76), IndexedStack :81-89 — заменить два `_SectionPlaceholder()`
(:86-87) на `ModelsScreen()`/`SettingsScreen()`, удалить
`_SectionPlaceholder` (:100-113) и ключ `placeholderScreen` из arb.

### `app/test/fake_dicta_engine.dart` (modify)

Крюки уже есть: `fetchSettingsReply` :40-41, `setReply` :47-48, `onSet`
:57-60 (меняет `fetchSettingsReply` между set и refetch — CR-01),
`fetchCount` :68, `setCalls` :62-63. Расширять сценарии через ЭТИ крюки
(например: `setReply` c error для отказа ENGINE-ключа; `onSet`, выставляющий
`reloading: true`). Новый фейк для HF/загрузок — по тому же рецепту
(constructor-hooks + gates), отдельным классом в том же файле или рядом.

### Тесты

- golden-фикстуры ответов демона: `app/test/settings_model_test.dart`
  (канон «drift tripwire», settings.dart:3-5);
- set-flow с await-refetch: `calibration_test.dart`;
- widget-сьют с `buildFakeApp` :131-166 (локаль ru, AppTextStyles
  обязательны — иначе `mutedOf` assert);
- сьют загрузки: гейты по канону `stopGate`/`injectGate` (:34-38, 86-91) —
  `Completer<void>? downloadGate` держит «загрузку» в полёте до resolve;
- юнит-тесты hf-функций: кейсы из таблицы выше (у Rust уже 100% покрытие
  формул — переносить дословно).

### `app/lib/l10n/app_ru.arb` (modify)

Канон имён: семейства с префиксом фичи — `dash*`, `hist*`, `calibration*`,
`diag*`. Новые семейства: `conf*` (форма настроек: заголовки групп
Запись/Вставка/Движок/Система, подписи полей, hints, refusal-текст
ENGINE-ключей), `model*` (браузер: каталог, поиск, VRAM, скачивание,
прогресс, удаление, confirm-тексты). Каждый ключ с `@`-описанием;
placeholders-объявление как `calibrationThreshold` :174-180. Тон «модель:
…» из наследия №7 (arb :205,213 — `diagModelLoading`/`diagModelUnavailable`)
— переформулировать в `model*`-семейство, не дублировать.

### `app/tool/status_smoke.dart` (modify)

Строка 12: `stderr.writeln('SMOKE FAIL $error')` → печатать только
`error.runtimeType` (payload-privacy, канон T-2-01/T-3-01; CONTEXT:38).

## Shared Patterns

### Registration: ThemeExtension (3+1 точки)
**Source:** `core/theme_ext.dart:3-6` (grep-комментарий — ВЕРХНИЙ док-коммент,
не удалять), `main.dart:46-54` (обе темы), `fake_dicta_engine.dart:151-157`
(buildFakeApp), suite-local `buildTestApp` в shell_test.
**Apply to:** каждый новый тест-скаффолд обязан регистрировать
`AppTextStyles.from(colors)`, иначе `mutedOf(context)` assert (:31-38).

### Канон кнопок
**Source:** CONTEXT:24 + `history_screen.dart:109-191`.
Filled = primary; Elevated = copy/neutral; Outlined + error-foreground +
error-side = destructive (histDeleteButton :199-206); Text = tertiary.
**Apply to:** скачать (Filled), выбрать модель (Radio/выделение), отмена
загрузки и удалить модель (destructive), показать ещё (Text/Elevated).

### Snackbar-канон
**Source:** `history_screen.dart:113-121` (успех), :160-187 (двухстрочная
ошибка: локализованный заголовок + вербатим-текст демона, оба `colors.error`).
**Apply to:** «модель скачана — выбери её» (паритет `main.rs:400-403`),
«загрузка прервана» (:374), отказ set. Гейт: только явный клик; undo-действие
только там, где восстановление реально.

### Errors-as-data (никаких исключений в UI)
**Source:** `daemon_engine.dart:77-93` + `dicta_control.dart:80-86`
(`catch (_) → genericFailure`).
**Apply to:** hf_client, downloads, set-поток формы. В отличие от
status-поллера, HF/загрузка — разовые действия: ошибка хранится в state
как `lastError`-аналог, не поллится.

### Payload privacy
**Source:** `daemon_engine.dart:57-58` (тип ошибки и путь, не содержимое);
`status_smoke.dart:12` — единственное нарушение, гасим в этой фазе.
**Apply to:** НИКОГДА не логировать значения set-команд и URL с
токенами; сообщение об ошибке HF — код статуса/тип, не тело ответа.

### One-set-per-release (Slider)
**Source:** `calibration_card.dart:156-164` + `:66-98`.
**Apply to:** все 9 слайдеров формы + VRAM-слайдер (VRAM — локальный, без set).

### CR-01: invalidate + await refetch
**Source:** `calibration_card.dart:88-97`.
**Apply to:** после каждого успешного set — НЕ читать `.value`
синхронно после invalidate.

### mounted-guards после await
**Source:** `calibration_card.dart:81,93`; `history_screen.dart:143`.
**Apply to:** все async-обработчики новых экранов (скачивание — долго:
guard обязателен перед каждым snackbar).

### MergeSemantics для полей ввода
**Source:** `history_screen.dart:33-47` (урок WR-01: bare Semantics даёт
второй focus-stop). **Apply to:** paste_keys, HF-поиск, прямая ссылка.

### Cyrillic gate
**Source:** `.arb` для UI; `dicta_control.dart:12-17` /
`daemon_engine.dart:24-31` (Unicode-escape) для протокольных констант в коде.
**Apply to:** все новые файлы; проверить `bash tool/cyrillic_gate.sh`.

## No Analog Found

Честный список ФОРМ, которых в кодовой базе нет (планировщик берёт из
RESEARCH/Flutter-канонов, поведение — из Rust-эталона):

| Форма | Почему нет аналога | поведенческий источник |
|---|---|---|
| Стриминг-загрузка с прогрессом, докачкой (Range) и отменой | grep `HttpClient` по lib/ пуст; единственный транспорт — однострочный request-response сокет | `main.rs:223-262,366-416`, `catalog.rs:54-88` |
| Confirm-диалог (AlertDialog/showDialog) | grep `showDialog` по lib/ пуст — ни одного диалога | наследие UI-аудита №2 (CONTEXT:29); тексты — .arb |
| dart:io delete файла модели | в lib/ нет файловых операций (единственный dart:io writer — transport/`Process.runSync` :20) | CONTEXT:23,45 (dest_dir-гейт, активную не удалять) |
| Switch/Checkbox/выбор из 2-3 вариантов (use_gpu, inject_mode, gpu_device, language, asr_only) | grep `Switch\|Checkbox\|Dropdown\|SegmentedButton` пуст; bool-тумблеров в UI ещё не было | `main.rs:778-790` (SelectableLabel-ряд); accessibility — MergeSemantics-канон |
| Отмена in-flight сетевой операции (client.destroy) | в фейке есть гейт-паттерн, в продукте отмены сетевых операций нет | `main.rs:366-375`; гейт-тест-паттерн `fake:34-38` |

Родственные, но НЕ идентичные формы (частичный перенос, пометить в планах):
`LinearProgressIndicator` есть (`dashboard_screen.dart:72-75`), но без
индетерминированного варианта и cancel-кнопки; `Slider` есть, но только
double 0..1 (`calibration_card.dart`), int-диапазона 2..32 нет.

## Metadata

**Analog search scope:** `app/lib/**`, `app/test/**`, `app/tool/**`,
`crates/dicta-settings/src/*` (READ-ONLY паритет), `.planning/phases/0{1,2,3}-*/PATTERNS.md` (формат).
**Grep-проверки:** `showDialog|Switch|Checkbox|HttpClient|DropdownButton|SegmentedButton|LinearProgressIndicator|Process.run` по `app/lib` — вывод учтён в No Analog.
**Tracked-source gate:** все кодовые аналоги сверены `git ls-files` — зеркал/ignored-путей нет; Rust-эталон cited как behavior-analog.
**Pattern extraction date:** 2026-11-22.

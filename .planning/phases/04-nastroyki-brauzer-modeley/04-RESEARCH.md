# Phase 4: Настройки + браузер моделей — Research

**Researched:** 2026-11-22
**Domain:** Flutter/Dart UI поверх заморожённого демона dictad: HTTP-загрузчик на dart:io (HF API, Range-докачка, отмена), форма 17 ключей через `set`, управление локальными .bin-моделями
**Confidence:** HIGH для фактов живого кода (file:line ниже) и dart:io API (локальный SDK + api.dart.dev); SDK-поведение HF проверено живыми GET-пробами сегодня

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked decisions (копия, не пересматривать)
- Протокол демона и Rust-код заморожены: `set`/`settings` — единственный путь изменения конфига. Прямого редактирования config.toml из приложения НЕТ.
- `crates/dicta-settings` — эталон паритета, НЕ трогать (декомиссия — фаза 8).
- Новые pubspec-зависимости — только если research докажет, что std/dart:io недостаточен. Стартовая позиция: `dart:io` HttpClient + `dart:convert` (dio из STACK-исследования НЕ добавлять без нужды).
- Кириллица в lib/ только через .arb (гейт `bash tool/cyrillic_gate.sh`).
- UI-hint: ui-plan-gate = block:false (без UI-SPEC); UI-долг гасить немедленно по результату gsd-ui-auditor post-phase.

### Must-NOT (запреты, копия)
- НЕ трогать: crates/ (весь Rust), протокол, app/pubspec.yaml/lock (без решения research), оверлей dicta-ui.
- НЕ писать конфиг напрямую, НЕ логировать значения set-команд (payload privacy, канон T-2-01/T-3-01).
- НЕ регистрировать глобальные хоткеи (фаза 5 отдельно), НЕ лезть в трей.
- НЕ удалять/перезаписывать файлы вне dest_dir(модели); активную модель не удалять.
- НЕ добавлять cyrillic в lib вне .arb; НЕ клонировать фабрику фейков (implements==1 гейт).
- История/дашборд не ломать: батарея 71/71 обязана остаться зелёной.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description (REQUIREMENTS.md:36-39) | Research Support |
|----|-------------|------------------|
| CONF-01 | Пользователь меняет все 17 изменяемых ключей формами (слайдеры/чекбоксы/селекты) с живым `set` и без спама запросами | R3: карта «ключ → виджет → диапазон» из живого `parse_change` демона; Slider-onChangeEnd паттерн уже в коде (calibration_card.dart:156-164); спам-гейт = один set на жест |
| CONF-02 | UI честно отражает состояния: отказ reload при записи («модель загружается…»), предупреждение перед ключами, требующими перезагрузки движка | R3: КРИТИЧНО — текущий шов `set()` выбрасывает note и settings-снимок; отказ живёт в `text` при `error:None`; нужен поводок шва (см. Finding C1) |
| MODEL-01 | HF-браузер: каталог, поиск с фильтрами (ASR/сортировки/пагинация/подпапки), VRAM-лимит, прямая ссылка, прогресс и докачка (.part) | R1/R2/R5: HttpClient покрывает всё (стриминг, redirect, Range, abort); эндпоинты и парсеры — копия hf.rs; redirect-цепочка HF→Xet CDN проверена живой пробой, Range-докачка через redirect работает |
| MODEL-02 | Пользователь видит установленные модели, активную пометку и может удалить неактивную | R4: список уже приходит в `settings.models` (демон сканирует parent текущей модели, truncate(12)); удаление — dart:io + тройной guard; паритета в dicta-settings НЕТ (новая возможность) |
</phase_requirements>

## Summary

Phase 4 не добавляет ни одной зависимости: `dart:io` HttpClient + `dart:convert` закрывают весь download-manager (стриминг ответа в файл, автоматические redirect'ы с сохранением Range-заголовка, `Range`-докачка `.part`, отмена через `request.abort()`), а HF API — это три GET-эндпоинта, чьи URL и JSON-парсеры уже реализованы в Rust-эталоне `crates/dicta-settings/src/hf.rs` и переносятся в Dart один-в-один как чистые функции. Живые пробы сегодня (2026-11-22) подтвердили фактическое поведение HF: `resolve` отдаёт 302 на подписанный URL Xet-CDN (`us.aws.cdn.hf.co/xet-bridge`), `Range: bytes=N-` проходит сквозь redirect и возвращает 206, полный размер файла приходит в `content-range`, quota — 3000 запросов / 300 с.

Главная ловушка фазы — не сеть, а шов: `DictaEngine.set()` сегодня возвращает `DictaReply(ok, error)` и **выбрасывает note и settings-снимок из ответа** (daemon_engine.dart:77-93), а отказ перезагрузки движка демона кладёт **в note при `error: None`** (dictad main.rs:723-739). CONF-02 невозможно сделать честно без расширения шва — как в фазе 2 расширяли `stop() → DictaStatus`. Форма настроек обновляется из set-ответа без лишнего fetchSettings, ровно как диктует CONTEXT.

Вторая ловушка — тесты: `dart:io` доступен во flutter test (VM), но сетевые тесты запрещены политикой; значит до экранов нужны два новых шва — `HfClient` (GET URL → строка) и `DownloadEngine` (загрузка со стримом прогресса/отменой), по одному фейку на шов в фабрике, и чистые функции URL/парсинга, тестируемые вообще без швов.

**Primary recommendation:** ноль новых зависимостей; первый план-слот — Wave 0: расширение `set()`-шва до ответа с note+снимком, порт hf.rs как чистых функций, два новых шва с фейками; затем экран настроек (CONF-01/02), затем браузер (MODEL-01) и список моделей с удалением (MODEL-02).

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Хранение и валидация 17 ключей | Daemon (dictad) | — | Протокол заморожен; `parse_change` (dictad main.rs:497-609) — единственный валидатор; UI только шлёт `set` |
| Применение/сброс set-ответа в форму | App UI (Dart) | — | Ответ `set` несёт свежий settings-снимок (main.rs:730-731); форма — единственный потребитель |
| Список установленных моделей | Daemon (`settings.models`) | App UI (отображение) | Демон сканирует parent текущей модели (main.rs:362-380); приложение НЕ делает свой собственный обход ФС для списка |
| Удаление модели | App (dart:io) | — | Команды удаления в протоколе НЕТ; файловая операция в клиенте с guards |
| HF поиск/дерево (JSON API) | App (dart:io HttpClient) | — | Не касается демона вообще; чистые функции + тонкий сетевой шов |
| Скачивание .bin, докачка .part | App (dart:io HttpClient) | — | curl shell-out запрещён (кросс-платформенность, фаза 6); стриминг ответа → файл |
| VRAM-лимит | App UI (только локальное состояние) | — | Ключа в конфиге демона нет (kEditableKeys settings.dart:10-28); эталон — dicta-settings main.rs:683-684 |
| Перезагрузка модели | Daemon (schedule_engine_reload) | — | Поток-в-фоне в демоне (main.rs:744-783); UI только показывает `reloading` и отказ |

## Standard Stack

### Core
| Библиотека | Версия | Purpose | Why Standard |
|---------|---------|---------|--------------|
| dart:io HttpClient | Dart SDK 3.14 (Flutter 3.50.1, локально) | HTTP GET к HF, стриминг тела в файл, redirect, Range, abort | Есть всё нужное: `HttpClientResponse implements Stream<List<int>>` [VERIFIED: локальный SDK _http/http.dart:1916], `followRedirects` default true для GET [CITED: api.dart.dev/dart-io/HttpClientRequest/followRedirects.html], `request.abort()` [CITED: api.dart.dev/dart-io/HttpClientRequest-class.html] |
| dart:convert | Dart SDK | jsonDecode ответов HF / парсинг | Уже используется (engine.dart:9, daemon_engine.dart:6) |
| flutter_riverpod 3.5.0 | pubspec.yaml:33 | Провайдеры экранов | Уже в проекте; settingsSnapshotProvider существует (providers.dart:20-32) |

### Supporting
| Библиотека | Версия | Purpose | When to Use |
|---------|---------|---------|-------------|
| — | — | НИЧЕГО не добавляем | pubspec.yaml/lock — в Must-NOT; research ниже доказывает достаточность std |

**Installation:** нет — зависимостей не добавляется. Версия SDK подтверждена локально: Flutter 3.50.1, Dart 3.14 [VERIFIED: `flutter --version` в этой сессии; локальные исходники SDK в /home/wardreg/development/flutter/bin/cache/dart-sdk].

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| dart:io HttpClient | dio (пакет) | dio дал бы interceptor'ы/удобные ApiError из коробки, но стриминг+Range+abort нативно поддержаны HttpClient; dio — +1 зависимость против решения CONTEXT «dio НЕ добавлять без нужды». Нужды нет: все требования R1 закрыты std. **Рекомендация: HttpClient, ноль депов** [верифицировано покрытием ниже] |
| Свой percent-encode | Uri.encodeComponent | Использовать Uri.encodeComponent (пробел → %20, как url_encode в hf.rs:5-15). НЕ Uri.encodeQueryComponent — он кодирует пробел как `+`, что разошлось бы с эталонными тестами hf.rs:183-206 [VERIFIED: hf.rs:172-177 `url_encode("whisper ggml") == "whisper%20ggml"`] |

## Package Legitimacy Audit

Фаза **не устанавливает ни одного пакета** — рекомендация «ноль новых зависимостей» зафиксирована выше, pubspec.yaml/pubspec.lock не трогаются (Must-NOT). Audit-гейт протокола неприменим: нет пакетов для проверки. Существующие зависимости не меняются.

## Architecture Patterns

### System Architecture Diagram

```
                    ┌──────────────────────────────────────────────────────┐
                    │ dicta-app (Flutter, Linux)                            │
                    │                                                      │
  UDS-сокет ────────┤ DictaEngine (шов, РАСШИРИТЬ set→SetReply)            │
  {"cmd":"settings"}│   ├─ fetchSettings() → Map                           │
  {"cmd":"set",…}   │   └─ set(key,value) → note + error + СНИМОК (C1)     │
                    │        │                            ▲                │
                    │        ▼                            │ обновление     │
                    │  Экран настроек (17 ключей) ────────┤ формы без     │
                    │   Slider-onChangeEnd → 1 set/жест    │ fetchSettings │
                    │   confirm-dialog → ENGINE_KEYS (6)   │                │
                    │   refusal в UI + reloading-спиннер   │                │
                    │                                      │                │
                    │  Экран моделей ──────────────────────┘                │
                    │   ├─ список: settings.models (демон) ── активная ✓    │
                    │   ├─ delete: dart:io + guards (C2) → re-fetch         │
                    │   ├─ VRAM SliderState 2..32 (UI-only)                 │
                    │   ├─ каталог CATALOG (6 позиций, порт catalog.rs)     │
                    │   └─ HF-браузер ──┐                                   │
                    │                   │                                   │
                    │      HfClient (шов: GET url → String)                │
                    │       ├─ browse_url → parse_search → репо+страницы    │
                    │       └─ tree_url_at → parse_tree/parse_tree_dirs    │
                    │                   │                                   │
                    │      DownloadEngine (шов: url+Range → поток)         │
                    │       ├─ 302 → Xet CDN (followRedirects=true)         │
                    │       ├─ .part append + Range: bytes=<len>-           │
                    │       ├─ progress: offset+received / total           │
                    │       ├─ abort() → отмена, .part удалить              │
                    │       └─ rename .part → dest → re-fetch models         │
                    └──────────────────────────────────────────────────────┘
                                       │ HTTPS
                                       ▼
                    huggingface.co/api/models…  (302)  us.aws.cdn.hf.co/xet-bridge…
                    [VERIFIED: живые пробы 2026-11-22]
```

Читатель трассирует главный сценарий MODEL-01: список `settings.models` → HF-браузер (HfClient) → выбор файла → DownloadEngine (Range/redirect) → `.part` → rename → re-fetch снимка (новая модель в списке) → `set('model', …)` → отказ при записи → честная ошибка.

### Recommended Project Structure
```
app/lib/
├── core/
│   ├── engine.dart            # +SetReply тип ответа set (расширение шва)
│   ├── settings.dart          # без изменений; kEngineKeys уже есть (:30-37)
│   └── hf/                    # НОВОЕ — чистая логика без UI и без сети
│       ├── hf_urls.dart       #    порт browse_url/tree_url_at/resolve_url/url_encode (hf.rs)
│       ├── hf_parse.dart      #    порт parse_search/parse_tree/parse_tree_dirs/dest_name/filename_of
│       ├── hf_client.dart     #    абстрактный шов: Future<String> get(String url)
│       └── download_engine.dart #  абстрактный шов загрузки + DownloadState
└── features/
    ├── settings/              # экран настроек (CONF-01/02)
    └── models/                # экран моделей: список+delete, каталог, HF-браузер, VRAM
app/test/
├── fake_dicta_engine.dart     # РАСШИРИТЬ (setReply со снимком) — не клонировать
├── fake_hf_client.dart        # НОВОЕ — единственный фейк HfClient-шва
├── fake_download_engine.dart  # НОВОЕ — единственный фейк DownloadEngine-шва
├── hf_urls_test.dart          # паритет со строками-тестами hf.rs:172-341
└── …existing suites (71/71 не ломать)
```

### Pattern 1: Чистые функции + тонкий шов (как engine.dart)
**What:** Всю логику HF (URL-строительство, парсинг JSON, dest_name, VRAM-сравнение, guard'ы удаления) писать чистыми функциями Dart — точными портами hf.rs; сеть и файлы прятать за минимальными интерфейсами `HfClient`/`DownloadEngine` по образцу DictaEngine (engine.dart:69-86: интерфейс; engine.dart:88-100: чистые функции рядом).
**When to use:** везде в этой фазе. Тесты чистых функций не требуют ни фейков, ни сети — паритетные утверждения копируются из Rust-тестов hf.rs:172-341.
**Why:** в эталоне та же декомпозиция — «Никакой работы с сетью и сокетом — только строки и JSON, всё тестируемо без интернета» (hf.rs:1-2).

### Pattern 2: Slider-onChangeEnd — один set на жест (уже в коде)
**What:** локальный pending-value при `onChanged`, коммит `set` только в `onChangeEnd` (calibration_card.dart:156-164: `onChanged: _previewMarker, onChangeEnd: _commitMarker`); после await — `if (!mounted) return;` (:80-81, :92-93).
**When to use:** все слайдеры формы (min_level, auto_stop_level, все ms-ключи, n_threads, beam_size, VRAM-слайдер).
**Why:** «один set на завершённое взаимодействие» — уже принятая конвенция дашборда (calibration_card.dart:71-72 комментарий).

### Pattern 3: Форма обновляется из set-ответа, не из re-fetch
**What:** расширить шов `set()` до ответа `{note, error, settings}`; после коммита обновлять состояние формы из `settings` ответа (main.rs:730-739: `*extra = Some(settings_view(...))`, `text: Some(note)`, `error: None`). `SettingsReply.fromReply` (settings.dart:115-132) уже парсит такую форму (`reply['settings']` + верхний `error`) — тот же парсер годится и для set-ответа.
**When to use:** каждый set в экране настроек; для ENGINE-ключей note = «перезагружаю модель …» или накопленная ошибка через ' · ' (main.rs:723-728).
**Проверка «без спама»:** fetchCount фейка НЕ растёт после set (анти-паттерн — invalidate+refetch как в calibration_card.dart:88-92 там, где ответ уже несёт снимок).

### Pattern 4: Деструктивный канон + confirm
**What:** удаление модели — confirm-диалог (undo невозможен: файл физически удаляется) + кнопка OutlinedButton с `foregroundColor: colors.error` и стороной error (точный паттерн history_screen.dart:199-206). Confirm перед ENGINE_KEYS-set — обычный AlertDialog (FilledButton=подтвердить, TextButton=отмена).
**When to use:** MODEL-02 delete; каждый первый set ENGINE-ключа за сессию (или на каждый — планировщик решает; эталон предупреждение НЕ показывает, требование CONF-02 новее эталона).

### Anti-Patterns to Avoid
- **Process.run('curl')** — запрещён (CONTEXT.md:22): кросс-платформенность фазы 6; весь download — HttpClient.
- **Свой процент-энкодер «на глазок»** — только Uri.encodeComponent; расхождение с hf.rs ломает паритетные тесты.
- **Считать progress из content-length первого ответа** — первый ответ 302, его content-length = 1139 байт redirect-страницы [VERIFIED: живая проба]; размер брать из финального ответа (206 `content-range: bytes 0-0/77691713`) или из expected+offset.
- **Сетевые вызовы в тестах** — flutter test выполняется в VM, но сеть в тестах запрещена (см. R-SDK); только швы+фейки.
- **Клонирование фабрики фейков** — implements==1 гейт; расширять fake_dicta_engine.dart.
- **DRY-нарушение по settings-парсеру** — SettingsReply.fromReply один; НЕ писать второй парсер для set-ответа.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| HTTP-redirects HF→CDN | Ручной цикл по Location (это и делает docs-пример) | `followRedirects=true` (default) + `maxRedirects=5` | SDK сам переносит ВСЕ заголовки (включая Range) на redirect-цепочку и сам следит за лимитом; живая проба: Range через redirect → 206 [VERIFIED] |
| TLS, сертификаты, прокси | — | HttpClient из коробки | «certificate signed by a root CA … will work automatically» [CITED: api.dart.dev/dart-io/HttpClient-class.html]; findProxyFromEnvironment default |
| JSON-парсинг HF | RegExp | dart:convert jsonDecode + порты parse_search/parse_tree | Толерантность уже спроектирована в hf.rs (битый JSON → пустой список); RegExp сломается на lfs-полях |
| Стриминг файла в диск | Накопление List<int> в памяти | `response.pipe(file.openWrite(mode: FileMode.append))` или listen+flush | Модели до ~1.1 ГБ (catalog.rs:45) — в памяти нельзя; IOSink стримит |
| Отмена загрузки | Флаг «остановись» в цикле копирования | `request.abort([exception])` | Документированный механизм: рвёт соединение, future завершается ошибкой [CITED: api.dart.dev/dart-io/HttpClientRequest-class.html] |

**Key insight:** Rust-эталон решает всё это внешним curl (`-L --fail --retry 3 -C -`, main.rs:241-247). В Dart curl'у соответствует ровно один вызов `HttpClient.getUrl` с `Range`-заголовком — стандартный SDK покрывает флаги -L (followRedirects), -C (Range+append) и --fail (statusCode) сам.

## Findings: Research Questions R1-R5

### R1: Download manager на dart:io HttpClient — НОЛЬ новых депов ✅

**Рекомендация: HttpClient. dio не нужен** — все четыре требования (стриминг в файл, Range-докачка, прогресс, отмена) закрыты SDK.

Покрытие требований по пунктам, с верификацией:

1. **Стриминг ответа в файл.** `HttpClientResponse` — это поток: `abstract interface class HttpClientResponse implements Stream<List<int>>` [VERIFIED: локальный SDK `_http/http.dart:1916`]. Запись — `File.openWrite({FileMode mode = FileMode.write, Encoding encoding = utf8})` [VERIFIED: SDK `io/file.dart:576`], `FileMode.append` ставит позицию записи в конец [VERIFIED: SDK `io/file.dart:533`] — ровно то, что нужно для докачки `.part`.

2. **Redirect HF resolve → CDN.** `followRedirects` default `true`; автоматика только для GET/HEAD и кодов 301/302/303/307/308; «All headers added to the request will be added to the redirection request(s)» кроме чувствительных (Authorization/Cookie и т.п.) cross-domain — **Range НЕ чувствительный и переносится** [CITED: api.dart.dev/dart-io/HttpClientRequest/followRedirects.html]. `maxRedirects` default `5` [VERIFIED: SDK `_http/http.dart:1823`], превышение → `RedirectException` [CITED: там же]. Живая проба (2026-11-22): `resolve/main/ggml-tiny.bin` → `HTTP/2 302` с `location: https://us.aws.cdn.hf.co/xet-bridge-us/...&Expires=...` (Xet-CDN, подписанный URL); `ratelimit-policy: "fixed window";"resolvers";q=3000;w=300` [VERIFIED: живая проба]. Подписанный URL имеет `Expires` — **не кэшировать резолвнутый URL между загрузками**, каждый старт заново resolve (эталон так и делает: curl запускается с исходной ссылкой, main.rs:241-247).

3. **Range-докачка `.part` работает сквозь redirect.** Живая проба: `Range: bytes=1000000-1000999` через `resolve`-ссылку с -L → `final_code=206 size_dl=1000` — ровно запрошенный кусок [VERIFIED: живая проба 2026-11-22]. Ответ несёт `accept-ranges: bytes`, а полный размер — в `content-range: bytes 0-0/77691713` [VERIFIED: живая проба]. Схема докачки: если `.part` существует и `FileStat.length > 0` → `request.headers.set(HttpHeaders.rangeHeader, 'bytes=${len}-')` [константа `rangeHeader = "range"` VERIFIED: SDK `_http/http.dart:393`], писать в append; успех → `File.rename` .part → dest (как fs::rename в main.rs:397); ошибка → `.part` НЕ удалять (main.rs:408-413: «повтор продолжит загрузку, часть файла сохранена»); отмена пользователем → удалить `.part` (main.rs:366-375: cancel удаляет .part, чтобы следующий старт с нуля).

4. **Прогресс из content-length — с одной ловушкой.** У свежей загрузки (200) `contentLength` = полный размер; у докачки (206) `contentLength` = ОСТАТОК, полный размер лежит в `content-range` (`bytes start-end/total`) [VERIFIED: живые пробы — 206-ответ на bytes=0-0 имел `content-length: 1`, полный размер только в content-range]. Правильная математика: `received = offset + chunkCount`, `total = expected > 0 ? expected : totalFromContentRange`, где offset = длина .part на старте. `expected` приходит из parse_tree (`size || lfs.size`, hf.rs:110-120) или CATALOG.approx_bytes (catalog.rs:19-45); `expected == 0` → вместо полосы живой счётчик МБ, как эталон (main.rs:529-545: expected>0 → ProgressBar, иначе spinner+МБ).

5. **Отмена.** `HttpClientRequest.abort([Object? exception, StackTrace? stackTrace])` — «Aborts the client connection» [CITED: api.dart.dev/dart-io/HttpClientRequest-class.html]. Вызов рвёт соединение; стрим ответа завершается ошибкой → ловим, удаляем .part, сообщаем «загрузка прервана» (паритет main.rs:373-374).

6. **Одна загрузка в момент времени.** Слот `download: Option<Download>` у эталона (main.rs:110-111, :225-227 — start_download молча return если уже идёт). В Dart — поле состояния загрузки в Notifier/State: если идёт — кнопки «скачать» disabled (main.rs:583, :696, :759) или ранний return.

**Pitfalls R1:**
- **429 rate limit.** HF resolvers: квота 3000 req/300 s [VERIFIED: заголовок `ratelimit-policy` живой пробы]. 429-ответ не проверен живьём [ASSUMED: тело/Retry-After не пробовались] — обрабатывать как общий сетевой отказ с честным сообщением; агент не должен сам поллить HF в цикле.
- **`Accept-Encoding: gzip` ставится HttpClient по умолчанию** [CITED: api.dart.dev/dart-io/HttpClient-class.html]; Xet-CDN отдаёт `response-content-type=application/octet-stream` и не сжимает [VERIFIED: живая проба URL-параметров], так что content-length остаётся честным. Если параноить: `request.headers.removeAll(HttpHeaders.acceptEncodingHeader)` перед close.
- **206 vs 200.** После докачки сервер обязан вернуть 206; принять оба, но при ответе 200 (сервер игнорировал Range — старт заново) обнулить offset и перезаписать .part c нуля (FileMode.write), иначе файл склеится.
- **Первая проба — не последняя.** `curl --retry 3` эталона соответствует: авто-retry N=3 на SocketException/5xx с сохранённым .part; ниже — скелет.

### R2: HF API из Dart — те же эндпоинты, тот же парсинг

Эндпоинты — копия hf.rs (READ-ONLY эталон, файл не трогаем, переносим в Dart):

| Функция Rust | Формула | VERIFIED |
|---|---|---|
| `browse_url(query, sort, asr_only, offset)` (hf.rs:21-35) | `https://huggingface.co/api/models?` + `search=<enc>` если непусто + `filter=automatic-speech-recognition&` если asr_only + `sort={sort}&direction=-1&limit=20&offset={offset}` | hf.rs:180-207 тесты всех комбинаций |
| `tree_url_at(repo, subpath)` (hf.rs:38-44) | root: `.../api/models/{repo}/tree/main?recursive=false`; subpath: `.../tree/main/{subpath}` | hf.rs:210-219 |
| `resolve_url(repo, path)` (hf.rs:47-49) | `https://huggingface.co/{repo}/resolve/main/{path}` | hf.rs:222-227 |
| `parse_search` (hf.rs:60-83) | массив; `id` ?? `modelId` строкой, downloads/likes — u64 с default 0 (null/мусор → пропуск записи) | hf.rs:231-258 |
| `parse_tree` (hf.rs:93-127) | только `type=="file"` и путь с `.bin`; размер = `size` если >0, иначе `lfs.size`, иначе 0 | hf.rs:261-293 |
| `parse_tree_dirs` (hf.rs:130-147) | только `type=="directory"` → path | hf.rs:296-309 |
| `dest_name` (hf.rs:150-152) | последний сегмент `/`-разделённого пути | hf.rs:320-325 |
| `filename_of` (hf.rs:155-165) | последний сегмент URL без query; None если нет `/` или пустое имя | hf.rs:328-341 |
| `url_encode` (hf.rs:5-15) | `%XX` для всего вне `[A-Za-z0-9-._~]` | hf.rs:172-177 |

**JSON через dart:convert**: `jsonDecode(body)` → проверять `is List` → filter_map. Толерантность эталона: битый JSON и не-массив → пустой список (hf.rs:312-317) — тесты портировать дословно.

**Поверхности ошибок — живые пробы 2026-11-22 [VERIFIED]:**
- дерево несуществующего/закрытого репо без токена → **401**, тело `{"error":"Invalid username or password."}` — HF не раскрывает существование; честное сообщение «репозиторий закрыт или не найден».
- resolve несуществующего файла в публичном репо → **404**, тело `Entry not found` (plain text — НЕ JSON).
- browse-эндпоинт — 200, массив с `id`, `modelId`, `downloads`, `likes` (проба вернула `{"_id":...,"id":"argmaxinc/whisperkit-coreml","likes":277,"downloads":95140913,...}`) — форма ровно та, что парсит parse_search.
- Любой не-200/206 в Dart: `response.statusCode` [VERIFIED: SDK `_http/http.dart:1922`] → исключение шва → красная строка/снекбар. `--fail` curl'а = `statusCode >= 400 → throw`.

**HfClient-шов:** интерфейс из одного метода `Future<String> fetchText(String url)` (и возможно `Stream<List<int>> fetchBytes(String url, {int? fromByte})` для загрузки — либо отдельный DownloadEngine-шов; рекомендация ниже в Validation). Реализация: `HttpClient()`, `getUrl(Uri.parse(url))`, `request.close()`, `await response.transform(utf8.decoder).join()` для текстовых; стрим для байтовых. `client.close()` после каждого запроса или один клиент на приложение [CITED: HttpClient-class «Use HttpClient.close() to force shut down»].

### R3: Форма 17 ключей — группы, виджеты, confirm, честные состояния

**C1 — КРИТИЧЕСКИЙ FINDING: шов `set()` не проносит отказ и снимок.**
Сегодня `DaemonEngine.set()` идёт через `_commandLine` (daemon_engine.dart:162-165 → :77-93), который читает ТОЛЬКО `map['error']` и выбрасывает `text` и `settings`. А демон при отказе перезагрузки кладёт сообщение **в note при `error: None`**: `schedule_engine_reload` возвращает Err → `note = format!("{note} · {err}")` (dictad main.rs:723-728), а итоговый `Res { text: Some(note), error: None }` (main.rs:733-739) плюс свежий снимок `*extra = Some(settings_view(...))` (main.rs:730-731). То есть при записи/распознавании UI увидит `ok:true` и промолчит. **CONF-02 требует расширения шва** — как в фазе 2 расширяли `stop() → DictaStatus` (fake_dicta_engine.dart:2-4 комментарий «Task 2 interface extension»). Рекомендуемый тип:

```dart
// Source: спроектировано по живому протоколу dictad main.rs:730-739
class SetReply {
  const SetReply({this.note, this.error, required this.settings});
  final String? note;      // text ответа: «перезагружаю модель …» или «… · отказ»
  final String? error;     // top-level error (транспорт/валидация)
  final SettingsReply settings; // парсится тем же SettingsReply.fromReply
}
```

с парсером `SetReply.fromReply(Map reply)` на основе существующего `SettingsReply.fromReply` (settings.dart:115-132 — уже понимает `reply['settings']` + верхний `error`; добавить чтение `reply['text']`). `FakeDictaEngine` получает хук `setReplyMap` вместо голого `setReply` (существующие setReply:48 / setCalls:63 / onSet:60 — расширять, не клонировать). Калли-сайт calibration_card.dart:77-79 мигрирует механически.

**C2 — Группировка 17+1 ключей** (семантика демона, группы эталона main.rs:432-879; в app — те же группы, язык выделен как ENGINE):

| Группа | Ключ | Тип в parse_change | Виджет | Диапазон демона [VERIFIED main.rs:542-556,604-606] |
|---|---|---|---|---|
| Запись (6) | min_level | Level f32 | Slider 0..1, div 100, commit `{:.2}` | 0.0..=1.0 |
| | auto_stop | Flag | SwitchListTile, шлём `on`/`off` (как main.rs:1019) | bool (daemon принимает и true/false, main.rs:505-513) |
| | auto_stop_level | Level f32 | Slider | 0.0..=1.0 |
| | auto_stop_after_ms | Count | Slider (метка в секундах) | 200..=60000 |
| | auto_stop_grace_ms | Count | Slider | 0..=20000 |
| | max_record_ms | Count | Slider | 1000..=600000 |
| Вставка (5) | inject_mode | enum Text | SegmentedButton: `paste \| clipboard \| stdout \| atspi` (main.rs:550-553) | 4 значения |
| | paste_keys | Text | Presets SegmentedButton (`ctrl+v \| ctrl+shift+v \| shift+insert`, main.rs:818-824) + свободное TextField с коммитом по onSubmitted/потере фокуса (эталон: main.rs:826-832) | валидируется Combo::parse (main.rs:554-556) |
| | clipboard_restore_ms | Count | Slider | 200..=60000 |
| | paste_delay_ms | Count | Slider 0..3000 | 0..=3000 |
| | atspi_allow_any_role | Flag | Switch | bool |
| Движок (5, все ENGINE) | model | Text | Список `settings.models` (выбор = `set('model', path)`; демон принимает и stem — main.rs:569-603) | файл .bin или уникальный stem |
| | use_gpu | Flag (ENGINE) | Switch | bool |
| | gpu_device | Index (ENGINE) | SegmentedButton/Radio: 0..8 (эталон показывает 0 и 1 с замерами, main.rs:780-790) | 0..=8 |
| | n_threads | Index (ENGINE) | Slider | 1..=64 (эталон UI 1..=32 — брать диапазон ДЕМОНА, чтобы слайдер не мог породить ошибку демона) |
| | beam_size | Index (ENGINE) | Slider | 1..=10 (эталон UI 1..=8 — то же решение) |
| Система (1) | language | Text (ENGINE!) | SegmentedButton `ru \| en \| auto` (+возможно произвольное 2-8 букв) | 2-8 ascii-букв или `auto` (main.rs:558-568) |
| Только чтение | config_path | — | ReadOnly поле (паритет main.rs:873-878) | — |

⚠️ `language` входит в ENGINE_KEYS (dicta-core config.rs:163-170; дубль в settings.dart:30-37) — хоть эталон и кладёт его во вкладку «Система», пометка «перезагрузит движок» нужна и ему.

**Confirm перед ENGINE_KEYS:** диалог «эта настройка перезагрузит модель (~секунды) — продолжить?» перед первым set ENGINE-ключа; CONF-02 требует предупреждение, эталон его не показывает (новое поведение сверх паритета — зафиксировать в плане).

**Поверхность отказа:** отказ приходит в note при ok — показать note-строку в UI вербатим (демон пишет по-русски: «идёт запись или распознавание — смену модели отложи; заверши диктовку (Alt+V) и повтори», main.rs:752-753; и «модель уже загружается — подожди», main.rs:757). Значение set-команд НЕ логировать (Must-NOT, main.rs:716 eprintln пишет в лог ДЕМОНА — это его дело, не приложения).

**settings.reloading:** флаг в снимке (settings.dart:152, settings_view main.rs:403); после set ENGINE-ключа — спиннер «модель загружается…» в области движка (наследие UI-аудита №4), гасится следующим снимком с reloading:false; можно до-poll'ить fetchSettings по таймеру (CONTEXT:20 разрешает), или положиться на снимки последующих set. Паттерн in-flight индикатора уже есть: `dictaControlProvider.commandInFlight` (history_screen.dart:22,134).

### R4: Управление моделями — список, активная, удаление

**Список = settings.models — НЕ сканировать ФС самому.** Демон строит его в `available_models` (dictad main.rs:362-380): `dir = cfg.model.parent()`, все `*.bin`, `out.sort()`, `out.truncate(12)` [VERIFIED]. Снимок уже несёт полный список (settings_view main.rs:402), golden-фикстура подтверждает форму (test/fixtures/settings_reply.json — 3 модели). Имя для показа — порт `model_name` (main.rs:932-936: последний сегмент пути, минус `.bin`, минус `ggml-`).

**Активная = settings.model** — пометка ✓ (эталон main.rs:504-508 `label = "{name} ✓"`); выбор неактивной = `set('model', path)` с confirm (это ENGINE-ключ + честный отказ при записи).

**Удаление (MODEL-02) — новой возможности в эталоне НЕТ** (в dicta-settings main.rs нет delete для моделей — только скачивание). Это app-only файловая операция, дизайн с тройным guard:

1. **Активную не удалять:** `path == settings.model` → кнопка disabled (Must-NOT CONTEXT:45).
2. **Путь только внутри dest_dir:** dest_dir = `parent(settings.model)`; если модель без каталога — fallback `$XDG_DATA_HOME | ~/.local/share` + `/whisper-cpp` (порт dest_dir, catalog.rs:62-88 — с тестами :116-139). Guard: `File(path).parent.path == destDir` (нормализованные пути) — иначе отказ без обращения к ФС. Список и так приходит из этого каталога (демон сканирует parent), так что guard — защита от рассинхрона, не от атаки.
3. **Confirm-диалог** — undo после `File.delete()` невозможен → канон Pattern 4. `File.stat()` для размера (`FileSystemEntity.stat()` [VERIFIED: SDK io/file_system_entity.dart:392]).

После удаления — re-fetch снимка (`settings.models` сократится); fetchCount-гейт в тестах. Проявленный файл → `File.delete()` из dart:io [стандартный API].

### R5: VRAM-лимит — UI-only SliderState

- Слайдер `2.0..=32.0`, шаг 1.0, **default 24.0** [VERIFIED: dicta-settings main.rs:157 `vram_limit_gb: 24.0`; main.rs:744-747 слайдер `2.0..=32.0` step 1.0].
- Сравнение точное [VERIFIED: main.rs:683-684]: `file.size > 0 && file.size as f32 > self.vram_limit_gb * 1_000_000_000.0` → в Dart: `size > 0 && size > vramGb * 1e9` → пометка «больше лимита VRAM», кнопка «скачать» disabled (main.rs:687-694).
- В конфиг демона НЕ писать: ключа нет ни в kEditableKeys (settings.dart:10-28), ни в EDITABLE_KEYS демона (config.rs:142-160) — только локальное состояние экрана (State/StateProvider).

### SDK-факты (маркировка по контракту)

| Claim | Tag |
|---|---|
| followRedirects default true, GET/HEAD, 301/302/303/307/308, все заголовки переносятся (кроме чувствительных cross-domain) | [CITED: api.dart.dev/dart-io/HttpClientRequest/followRedirects.html] |
| `Range`-заголовок переживает автоматический redirect HF 302→Xet CDN | [VERIFIED: живая проба 2026-11-22 — 206 с ровно запрошенными байтами] + правило переноса заголовков [CITED] |
| maxRedirects default 5 → RedirectException | [VERIFIED: SDK _http/http.dart:1823] |
| `HttpClientRequest.abort()` — отмена | [CITED: api.dart.dev/dart-io/HttpClientRequest-class.html] |
| HttpClientResponse = Stream\<List\<int\>\>; statusCode/isRedirect/redirects/contentLength | [VERIFIED: SDK _http/http.dart:1916-1962] |
| `HttpHeaders.rangeHeader = "range"`, `locationHeader = "location"` | [VERIFIED: SDK _http/http.dart:393,388] |
| File.openWrite(FileMode.append) — стрим-синка со смещением в конец | [VERIFIED: SDK io/file.dart:576,533] |
| FileSystemEntity.stat() / FileStat.stat(path) | [VERIFIED: SDK io/file_system_entity.dart:392,153] |
| dart:io доступен в flutter test (VM) | [VERIFIED in-repo: app/test/settings_model_test.dart:3,13 уже читает File и батарея зелёная] — НО политика фазы: сеть в тестах запрещена → только шовы+фейки |
| Uri.encodeComponent кодирует пробел как %20 | [ASSUMED — из API-контракта encodeComponent; проверяется первым юнит-тестом hf_urls параллельно строкам hf.rs:172-177] |
| HF 429-тело/Retry-After | [ASSUMED — quota-заголовок VERIFIED пробой, но сам 429 не пойман] |
| Xet-подписанный URL протухает (Expires в Location) | [VERIFIED: параметр Expires в Location живой пробы; конкретный срок жизни ASSUMED — дизайн каждый раз re-resolve, так что не критично] |

## Common Pitfalls

### Pitfall 1: Молчаливый CONF-02 (шов съедает отказ)
**What goes wrong:** UI шлёт `set('model', …)` во время записи; демон отказывает в reload, но `error:None`, сообщение — в `text`. Текущий DictaReply скажет `ok:true` — пользователь решил, что применилось.
**Why it happens:** `_commandLine` читает только `error` (daemon_engine.dart:81-82).
**How to avoid:** расширение шва C1 до SetReply{note, error, settings}; рендер note всегда.
**Warning signs:** тест «refusal surfaces verbatim» отсутствует в плане.

### Pitfall 2: Прогресс от первого content-length
**What goes wrong:** первый ответ — 302 c content-length=1139 (страница редиректа) [VERIFIED: проба]; полоса уходит в 1139/77691713.
**Why:** чтение contentLength до прохождения redirect/финального ответа.
**How to avoid:** total = expected (из browse/каталога) или content-range финального ответа; счётчик = offset + полученное.

### Pitfall 3: 206-докачка склеивает файл при 200
**What goes wrong:** сервер игнорирует Range и отдаёт 200 с полным телом; код пишет в append → битый файл (старое+новое).
**How to avoid:** если статус 200 при resume-запросе — открыть .part заново в FileMode.write (offset=0).
**Warning signs:** юнит-тест фейка DownloadEngine не различает 200/206.

### Pitfall 4: Спам set при драге слайдера
**What goes wrong:** onChanged → set на каждый пиксель: сотни UDS-команд и ENGINE-ключи сотни раз триггерят reload-логику.
**How to avoid:** Pattern 2 (onChangeEnd; calibration_card.dart:156-164 уже эталон в коде).
**Warning signs:** тест «one set per drag» (setCalls.length==1 после simulated drag) отсутствует.

### Pitfall 5: %.20 vs + в query
**What goes wrong:** Uri-конструктор/queryParameters кодирует пробел как `+`; HF такое поймёт, но паритетные тесты hf.rs ждут `%20`.
**How to avoid:** собирать query строкой + Uri.encodeComponent; тест на строках из hf.rs:183-206.

### Pitfall 6: models-список руками
**What goes wrong:** приложение само листает каталог .bin и расходится с демоном (у демона sort + truncate(12), main.rs:377-378).
**How to avoid:** список только из settings.models; dart:io — только delete/stat.

### Pitfall 7: Поллинг HF-страниц в тестах/проdе
**What goes wrong:** «показать ещё» в цикле или автообновление списка упирается в resolvers-quota (3000/300с на IP) или лив-тестов касаться сети вообще.
**How to avoid:** fetch только по действию (как эталон: кнопка «найти»/«показать ещё», main.rs:630-639); тесты — фейки.

## Code Examples

### Загрузка со стримом, Range-докачкой и отменой (спроектировано по VERIFIED API)
```dart
// Source: api.dart.dev/dart-io/HttpClientRequest/followRedirects.html (redirect-пример)
// + локальный SDK _http/http.dart (rangeHeader:393, maxRedirects:1823, abort)
// + живые пробы HF 2026-11-22 (302→Xet, 206 через redirect)
class HttpDownloadEngine implements DownloadEngine {
  final HttpClient _client = HttpClient();

  @override
  Stream<DownloadProgress> download(Uri url, File part, {int? expectedBytes}) async* {
    final offset = await part.exists() ? part.length() : 0; // .part докачка
    final request = await _client.getUrl(url);
    if (offset > 0) {
      request.headers.set(HttpHeaders.rangeHeader, 'bytes=$offset-');
    }
    final response = await request.close(); // followRedirects=true: 302 → CDN
    if (response.statusCode == 206) {
      // докачка: contentLength = ОСТАТОК; total смотрим из content-range/expected
    } else if (response.statusCode == 200) {
      // сервер игнорировал Range — перезаписать с нуля (Pitfall 3)
    } else if (response.statusCode >= 400) {
      throw DownloadHttpException(response.statusCode); // 401 закрыт / 404 нет файла
    }
    var received = offset;
    final sink = part.openWrite(mode: response.statusCode == 206
        ? FileMode.append : FileMode.write);
    try {
      await for (final chunk in response) {
        sink.add(chunk);
        received += chunk.length;
        yield DownloadProgress(received: received, total: expectedBytes ?? 0);
      }
      await sink.flush();
      await sink.close();
    } catch (e) {
      await sink.close(); // .part НЕ удалять — повтор докачает (main.rs:408-413)
      rethrow;
    }
  }

  // Отмена: request.abort(...) из UI-диспатчера; .part удалить при
  // осознанной отмене пользователем (main.rs:366-375), оставить при обрыве сети.
}
```

### Чистый порт browse_url (паритет hf.rs:21-35)
```dart
// Source: crates/dicta-settings/src/hf.rs:21-35 (перенос, эталон не трогаем)
String browseUrl(String query, String sort, bool asrOnly, int offset) {
  var url = 'https://huggingface.co/api/models?';
  if (query.isNotEmpty) url += 'search=${Uri.encodeComponent(query)}&';
  if (asrOnly) url += 'filter=automatic-speech-recognition&';
  url += 'sort=$sort&direction=-1&limit=20&offset=$offset';
  return url;
}
// Паритетные тесты: строки из hf.rs:183-206 дословно.
```

### Форма из set-ответа без re-fetch (Pattern 3, по протоколу main.rs:730-739)
```dart
final reply = await ref.read(engineProvider).set(key, value); // SetReply (C1)
if (!mounted) return;
if (reply.error != null || reply.note != null && _isRefusal(reply.note!)) {
  setState(() => _error = reply.error ?? reply.note); // отказ reload вербатим
  return;
}
setState(() => _form = reply.settings); // снимок из ответа, БЕЗ fetchSettings
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| dicta-settings: curl subprocess (-L -C -) | dart:io HttpClient в app | эта фаза | кросс-платформенность (Windows фаза 6), без shell-out |
| EGUI drag_stopped коммит | Flutter Slider onChangeEnd | фаза 2 | уже принят в calibration_card.dart |

**Deprecated/outdated:** ничего в фазе; НЕ использовать dio (решение CONTEXT), НЕ Process.run('curl') (запрет).

## Don't Hand-Roll — Key insight
Сеть уже написана дважды до нас: в SDK (HttpClient: redirects, TLS, Range, abort) и в эталоне hf.rs (URL/парсеры с тестами). Писать надо только третью копию парсеров — и ту по образцу с готовыми тестовыми строками.

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | HF 429 возвращает читаемое тело / Retry-After | R1 pitfalls | низкий: обрабатываем как общий сетевой отказ; квота 3000/300с VERIFIED заголовком |
| A2 | Xet-подписанный URL живёт минуты (Expires) | R1 | нулевой в дизайне: каждый старт загрузки делает свежий resolve |
| A3 | Uri.encodeComponent даёт %20 для пробела | R2 | ловится первым же юнит-тестом паритета (hf.rs:172-177) — checkpoint Wave 0 |
| A4 | dio сравнение (что он «не нужен») | R1 | нулевой: recommendation — ноль депов; если HttpClient подведёт в исполнении, добавить dio может только план с новым решением |
| A5 | SegmentedButton умещает 4 варианта inject_mode на узком экране | R3 | UI-решение планировщика/аудитора; fallback DropdownButton |
| A6 | удаление неактивной модели безопасно для демона (файл не mmapped) | R4 | демон держит открытой только АКТИВНУЮ модель; неактивные файлы никто не держит [рассуждение по архитектуре, не проверено в коде демона]; guard п.1 уже блокирует активную |

**Все прочие утверждения помечены [VERIFIED …] с file:line или [CITED …] с URL; живые пробы HF от 2026-11-22.**

## Open Questions

1. **Один confirm на сессию или на каждый ENGINE-set?**
   - Знаем: CONF-02 требует предупреждение; эталон не предупреждает вовсе.
   - Неясно: не задолбает ли диалог при серии движковых правок.
   - Рекомендация: confirm на каждый ENGINE-set (безопасный дефолт), тон обсудить в UAT.
2. **Retry-политика сетевых срывов (3 раза как curl --retry?)**
   - Знаем: .part сохраняется, повтор докачает (VERIFIED Range).
   - Рекомендация: авто-retry N=3 только на транспортные ошибки соединения (SocketException до первого байта), ручной повтор пользователем в остальных случаях.
3. **Где живут .part при закрытии приложения** (загрузка прервана выходом)?
   - Рекомендация: оставить .part — следующий старт в этом же сеансе или новом докачает (архитектурно бесплатно).

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Flutter SDK | вся фаза | ✓ | 3.50.1 (channel stable, Dart 3.14) [VERIFIED: flutter --version] | — |
| dart:io HttpClient | MODEL-01 | ✓ | в составе SDK; исходники сверены локально | — |
| Демон dictad (живой) | UAT CONF/MODEL, как в фазах 1-3 | ✓ (инфраструктура стенда, см. память: dictad на CT/Core) | — | widget-тесты не требуют демона |
| Сеть до huggingface.co | MODEL-01 UAT | ✓ [VERIFIED: живые пробы сегодня] | — | браузер/загрузка честно сообщают об отказе сети |
| CATALOG-модели на диске | MODEL-02 UAT | ✓ (в models-папке стенда есть ≥3 .bin — fixture отражает реальность) | — | — |
| git/curl (для проб и коммитов) | research/CI | ✓ | — | — |

**Missing dependencies with no fallback:** нет.
**Missing with fallback:** нет.

## Validation Architecture

`workflow.nyquist_validation` отсутствует в .planning/config.json → считаем ВКЛЮЧЁННЫМ. Тест-инфраструктура существует: `flutter test` в app/, 71/71 зелёные (Must-NOT: не сломать). Конфиг — стандартный flutter_test; фабрика фейков app/test/fake_dicta_engine.dart со хуками (fetchSettingsReply:41, setReply:48, onSet:60, setCalls:63, fetchCount:69, statusError:57, gates:35-39).

### Wave 0 (ДО экранов — вход планирования, обязателен)
1. **Расширить `DictaEngine.set` → SetReply** (finding C1): тип + парсер из полного reply; миграция fake_dicta_engine (хук setReplyMap со снимком+note) + миграция calibration_card и его тестов; батарея 71/71 зелёная после миграции.
2. **Порт hf.rs → app/lib/core/hf/*_parse.dart + hf_urls.dart** (чистые функции, ноль швов): паритетные тесты строками из hf.rs:172-341 (browse_url комбинации, parse_search мусор, parse_tree lfs.size, dest_name, filename_of, битый JSON → пусто). Заодно закрывает A3 (%20).
3. **Два новых шва + по одному фейку:** `HfClient` (GET→String; фейк с картой url→body и записью вызовов) и `DownloadEngine` (download(url, part, expected) → Stream<DownloadProgress> + cancel(); фейк со сценариями: 200, 206, отказ, abort) в test-фабрике — implements==1 на каждый шов.
4. **Fixture-ответы HF** в app/test/fixtures/ (json с id/downloads/likes; tree с .bin+lfs+dirs) — снимки с живых проб выше.

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| CONF-01 | 17 ключей редактируемы, один set на жест | widget | `flutter test test/settings_screen_test.dart` | ❌ Wave 0/фаза |
| CONF-01 | Slider drag → ровно один set (спам-гейт) | widget | тот же файл, тест «one set per drag»: `fake.setCalls.length == 1` после `tester.timedDrag` | ❌ |
| CONF-01 | bool → on/off, enum → допустимые строки, диапазоны из таблицы R3 | widget+unit | `flutter test test/settings_screen_test.dart test/hf_urls_test.dart` | ❌ |
| CONF-01 | форма обновляется из set-снимка, fetchCount НЕ растёт | widget | тест: после set `fake.fetchCount` unchanged; fails_when: fetchCount инкрементнулся после set | ❌ |
| CONF-02 | confirm перед ENGINE-set (без подтверждения set НЕ уходит) | widget | тест: кнопка ENGINE-ключа → диалог → `setCalls` пуст до confirm | ❌ |
| CONF-02 | отказ reload показывается вербатим при ok-ответе | widget | fake setReplyMap с note=отказ (строка main.rs:752) при error:null; fails_when: текст отказа не найден в дереве | ❌ |
| CONF-02 | reloading → спиннер/дизейбл движковой группы | widget | fake-снимок reloading:true | ❌ |
| MODEL-01 | каталог 6 позиций, «уже есть ✓», download кнопка | widget | `flutter test test/models_screen_test.dart` | ❌ |
| MODEL-01 | поиск/сортировка/фильтр/пагинация/подпапки из FakeHfClient | widget | тот же файл; fails_when: вызван URL не из browseUrl | ❌ |
| MODEL-01 | VRAM: size>0 && size>gb*1e9 → пометка+disabled | unit+widget | чистая функция vramBlocked + рендер | ❌ |
| MODEL-01 | прямая ссылка: не-http отклонён, имя извлечено | unit | порт тестов filename_of (hf.rs:328-341) | ❌ |
| MODEL-01 | DownloadEngine: 200-загрузка, 206-докачка (Range от длины .part), cancel, one-at-a-time, 200-при-resume→перезапись | unit (фейк шва) | `flutter test test/download_state_test.dart` | ❌ |
| MODEL-01 | прогресс: received=offset+chunks, total из expected/content-range | unit | тот же файл | ❌ |
| MODEL-01 | ошибки сети/401/404 → честная строка, .part сохранён | unit+widget | fake сценариев | ❌ |
| MODEL-02 | список из settings.models, активная помечена | widget | fake-снимок с 3 моделями (fixture) | ❌ |
| MODEL-02 | удаление: активная blocked; путь вне dest_dir отклонён; confirm; после delete — re-fetch | unit (dart:io на temp-dir: сеть не нужна) + widget | `flutter test test/model_delete_test.dart` + models_screen suite | ❌ |
| Регресс | батарея истории/дашборда 71/71 зелёная | suite | `flutter test` | ✅ (не сломать) |
| Гейты | cyrillic_gate, flutter analyze | gate | `bash tool/cyrillic_gate.sh` (последняя строка — 0); `flutter analyze` | ✅ существующие |
| Security residual | status_smoke.dart:12 печатает `$error` целиком → `error.runtimeType` | gate/visual | `grep -n '\$error' tool/status_smoke.dart` пуст; fails_when: найдено | частично (файл есть, правка ❌ — задача фазы) |

### Sampling Rate
- **Per task commit:** `flutter test` (app/) + `flutter analyze` + `bash tool/cyrillic_gate.sh`
- **Per wave merge:** полный suite + ручной smoke демона (запущен → настройки открылись)
- **Phase gate:** 71/71 + новые сюиты зелёные до `/gsd-verify-work`

### Ручной UAT (визуальное/сетевое, автоматизировать честно нельзя)
- Живая загрузка маленькой модели из каталога до конца: полоса, .part появляется, rename, «выбери её в списке».
- Прерывание (кнопка отмена) + повтор → докачка продолжилась (206).
- Отказ reload: начать диктовку (Alt+V), сменить модель в app → вербатим-сообщение.
- Визуальный тон VRAM-слайдера/диалогов — по наследию UI-аудита фазы 3 (пп. 1-7 CONTEXT:26-34).
- Реальный 404/401 (битая прямая ссылка) — сообщение об ошибке.

## Security Domain

`security_enforcement` в config.json отсутствует → включён. Фаза — клиент поверх заморожённого протокола + сетевые загрузки + файловые удаления: релевантны V5 и файловая безопасность.

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no | публичные HF-эндпоинты, токенов нет |
| V3 Session Management | no | stateless GET-запросы |
| V4 Access Control | yes (файловая) | удаление только внутри dest_dir (guard R4 п.2), активная модель blocked; префикс-проверка нормализованных путей |
| V5 Input Validation | yes | прямая ссылка: только http/https (порт проверки main.rs:763-767), filename_of — последний сегмент без query (hf.rs:155-165) → имя файла не может содержать `/` или `..`; языковая/enum-валидация — на демоне |
| V6 Cryptography | no | TLS силами HttpClient (системные корни, CITED docs) |
| V12 File Handling | yes | .part в dest_dir; rename после полного успеха; подтверждение перед delete; НЕ удалять вне dest_dir (Must-NOT) |

### Known Threat Patterns for {Flutter/Dart + HF downloads}

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Path traversal через прямую ссылку (…/../../x) | Tampering/Elevation | filename_of берёт последний сегмент (hf.rs:155-165); dest = dest_dir.join(fileName); запрет абсолютных путей в fileName |
| Загрузка исполняемого контента под видом .bin | Tampering | только `.bin` из parse_tree/имени; rename после полного body; модель грузит ДЕМОН, не app |
| DoS/quota-исчерпание HF поллингом | DoS | fetch только по действию; одна загрузка в момент; retry ≤3 |
| Логирование приватных значений set | Information Disclosure | канон T-2-01/T-3-01: не логировать value (Must-NOT); копия запрета в UI-слое |
| Подписанный Xet URL за пределами срока | Repudiation (мягко) | каждый resolve заново, не кэшировать Location |

## Sources

### Primary (HIGH confidence)
- Живой код репо (Read в этой сессии): engine.dart, settings.dart, providers.dart, daemon_engine.dart, calibration_card.dart, history_screen.dart, shell.dart, fake_dicta_engine.dart, settings_model_test.dart, pubspec.yaml, app_ru.arb, status_smoke.dart, cyrillic_gate.sh; crates/dicta-settings/{hf.rs,catalog.rs,main.rs}; crates/dictad/src/main.rs:355-809; crates/dicta-core/src/config.rs:140-199.
- Локальный Dart SDK исходники (/home/wardreg/development/flutter/bin/cache/dart-sdk/lib/_http/http.dart, lib/io/*.dart): rangeHeader:393, locationHeader:388, maxRedirects=5:1823, HttpClientResponse:1916-1962, openWrite/append:576,533, FileStat:153,392.
- Живые пробы HF (2026-11-22, read-only GET): 302→us.aws.cdn.hf.co/xet-bridge-us (подпись+Expires), ratelimit-policy resolvers q=3000 w=300, Range bytes=1000000-1000999 → 206/1000 байт через redirect, content-range bytes 0-0/77691713, 401 {"error":"Invalid username or password."}, 404 "Entry not found", browse 200 форма id/modelId/downloads/likes.
- api.dart.dev/dart-io: HttpClientRequest-class.html, followRedirects.html, HttpClient-class.html.

### Secondary (MEDIUM confidence)
- — (не понадобилось: всё закрыто кодом/SDK/пробами)

### Tertiary (LOW confidence)
- A1-A6 из Assumptions Log.

## Metadata

**Confidence breakdown:**
- Standard stack (ноль депов, HttpClient): HIGH — покрытие каждой потребности VERIFIED/CITED; поведение HF проверено живыми пробами.
- Architecture (швы C1, группы, guards): HIGH — всё из живого кода с file:line; единственное проектное решение (SetReply) прямо диктуется протоколом main.rs:730-739.
- Pitfalls: HIGH — обе ловушки сети подтверждены пробами; ловушка шва — код daemon_engine.dart:77-93 против main.rs:723-739.
- SDK-специфика: HIGH для использованных API (сверено с исходниками SDK), MEDIUM для 429-деталей (A1).

**Research date:** 2026-11-22
**Valid until:** 2027-01-22 (стабильные протоколы; HF-эндпоинты менять не обещают, пробы следует перепроверить при UAT)



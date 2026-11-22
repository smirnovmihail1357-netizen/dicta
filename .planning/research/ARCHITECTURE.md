# Architecture Research — dicta-app (Flutter поверх существующего демона)

**Domain:** голосовой диктант; интеграция нового Flutter-приложения с существующей Rust-архитектурой «демон + тонкие клиенты»
**Researched:** 2026-11-22
**Confidence:** HIGH (FlClash-паттерн подтверждён 3 независимыми источниками; dart:io/FRB/interprocess — официальные доки; детали tray/hotkey — MEDIUM)

---

## System Overview (что добавляется к существующему)

```text
                       ┌────────────────────────────────────────────┐
                       │  dicta-app (НОВОЕ, Flutter)                │
                       │  app/ — Material You, Riverpod             │
                       │  ├─ lib/core/daemon_engine.dart   (Linux)  │
                       │  │    dart:io Socket → dicta.sock         │
                       │  │    + persistent subscribe-соединение   │
                       │  ├─ lib/core/embedded_engine.dart (Win)   │
                       │  │    flutter_rust_bridge → dicta-ffi     │
                       │  └─ lib/core/engine.dart — общий интерфейс│
                       ├──────────────┬─────────────────────────────┤
                       │              │                             │
        JSON-строки    │              │ FFI (in-process)            │
        req → reply    ▼              ▼                             │
┌──────────────────────────────┐   ┌──────────────────────────────┐
│ dictad (СУЩЕСТВУЕТ, +1 arm)  │   │ crates/dicta-ffi (НОВОЕ)     │
│  serve_conn: match req.cmd    │   │  FRB-обёртка dicta-core для  │
│  + "subscribe" → события      │   │  Windows: capture (WASAPI),  │
│    пушем в держаном соед.     │   │  stt (whisper-rs), inject    │
└──────────────┬───────────────┘   │  (SendInput)                  │
               │                   └──────────────┬───────────────┘
               ▼                                  ▼
        dicta-core (СУЩЕСТВУЕТ, +Windows-ветки capture/inject)
```

Существующие клиенты (dictactl, dicta-ui, dicta-settings) не меняются. Приложение — ещё один клиент демона, ровно как FlClash-desktop является клиентом своего Go-ядра.

---

## Component Responsibilities

| Компонент | Статус | Ответственность |
|-----------|--------|-----------------|
| dicta-app (`app/`) | НОВОЕ | Flutter-окно: дашборд записи, история, настройки, браузер моделей; трей. Клиент демона на Linux; host встроенного ядра на Windows |
| `app/lib/core/engine.dart` | НОВОЕ | Абстракция `DictaEngine`: `start/stop/toggle/status/setSettings` + `Stream<AppEvent>`. Две реализации (как `CoreController`+`interface.dart` у FlClash) |
| `app/lib/core/daemon_engine.dart` | НОВОЕ | Linux: dart:io Unix-сокет, короткие req/reply + подписка на события |
| `app/lib/core/embedded_engine.dart` | НОВОЕ | Windows: flutter_rust_bridge к `crates/dicta-ffi` |
| `crates/dicta-ffi` | НОВОЕ (crate) | FRB-api поверх dicta-core для Windows: start/stop, StreamSink уровня и текста |
| dictad `serve_conn` | МОДИФИЦИРУЕТСЯ (минимально) | Новая arm `"subscribe"`: держит соединение и пушит JSON-события; список подписчиков + broadcast в 4 точках |
| dicta-core `capture.rs` | МОДИФИЦИРУЕТСЯ (Windows-ветка) | WASAPI-захват (вместо pw-record) за `#[cfg(windows)]` |
| dicta-core `inject.rs` | МОДИФИЦИРУЕТСЯ (Windows-ветка) | SendInput-вставка (вместо uinput+wl-copy) за `#[cfg(windows)]` |
| dicta-ui / dictactl / dicta-settings | НЕ ТРОГАЮТСЯ | Работают поверх старого протокола; `subscribe` — аддитивная команда, старый демон для старых клиентов ничего не меняет |

---

## Decision 1 — PUSH-обновления Linux: `subscribe` на том же сокете

### Что делает FlClash (эталон, выбранный пользователем)

FlClash-desktop держит **один** транспорт (Unix-сокет Linux/macOS, named pipe Windows) с кадрированием «4-байтовый little-endian length-prefix, максимум 64 MiB», JSON-конверт `CoreMethodCall(id, method, args)` / `CoreMethodResponse(id, result, error)`. События (трафик/логи/состояние) идут **обратно по тому же каналу**: ядро держит три очереди (state — без потерь; priority/bulk — вытесняют только свои старейшие), батчер сливает каждые 16 мс или 32 сообщения, события никогда не блокируют ядро. То есть FlClash не поллит и не открывает второй сокет — события мультиплексированы в общий канал. [HIGH: docs/architecture.md репо, DeepWiki, opendeep.wiki — 3 источника]

### Сравнение вариантов для dicta

| Вариант | Плюсы | Минусы | Вердикт |
|---------|-------|--------|---------|
| **A. Поллинг 100 мс** (как dicta-ui) | ноль изменений демона; доказано в кодовой базе | 10 JSON/с на клиента всегда; состояние меняется мгновенно (хоткей из DE), а клиент узнаёт с задержкой до 100 мс; рассинхрон настроек (set из dictactl) ловится только тиком | ✅ как **fallback** и первая итерация |
| **B. `subscribe` на существующем сокете** | один сокет (права 0600 уже есть); `serve_conn` уже цикл `while read_line` и уже держит соединение (stop); аддитивно — старые клиенты/демон не ломаются; это и есть «framed messages» FlClash, но в dicta-стиле «строка JSON = кадр» | +1 arm в match, +список подписчиков в демоне | ✅ **РЕКОМЕНДУЕТСЯ** |
| **C. Второй сокет событий** | изоляция каналов | второй путь (права, уборка устаревшего файла, автостарт); два места валидации; клиенту всё равно нужен первый сокет для команд | ✗ |
| **D. WebSocket** | — | HTTP-сервер в демоне, async-рантайм, TCP-порт — против всех ограничений проекта (std-threads, локальный сокет 0600) | ✗ категорически |

dicta не нужны length-prefix-кадры FlClash: строки JSON с `\n` уже являются кадрированием, а полезные нагрузки < 1 КБ (у FlClash 64 MiB лимит из-за гигантских конфигов профилей — не наш случай). Перенимаем **семантику** (bidirectional event stream, неблокирующий пуш, батчинг), не синтаксис.

### Минимальное расширение протокола (конкретный дизайн)

Новая команда — держаное соединение, демон пишет строки-события:

```jsonc
// клиент: {"cmd":"subscribe"}
// демон отвечает снимком и далее пушит только изменения:
{"event":"hello","state":"idle","level":0.0,"panel":false,"settings":{...}}
{"event":"state","state":"recording","seconds":0.0}          // переходы Idle/Recording/Transcribing
{"event":"level","level":0.42}                                // каждый тик watchdog (80 мс ≈ 12.5 Гц), только в Recording
{"event":"text","text":"…","app":"konsole","mode":"paste"}    // после успешного распознавания+вставки
{"event":"error","error":"…"}                                 // ошибки записи/распознавания/вставки
{"event":"settings","settings":{...}}                         // после любого "set" (в т.ч. от dictactl/оверлея)
```

Правила реализации в демоне (все — std::thread, без async):
1. `Subscribers = Arc<Mutex<Vec<UnixStream>>>`; arm `"subscribe"` клонирует writer в список и отвечает hello.
2. **Broadcast никогда не блокирует владельца состояния**: пишем через `try_clone` + `try_lock`, либо один writer-поток на подписчика с `mpsc` (мёртвые/переполненные — отключаем по ошибке записи). Watchdog тикает каждые 80 мс под `try_lock` — пуш уровня встраивается туда же, без новых таймеров.
3. Точки broadcast (всего 4): переходы состояния в `run_start`/`run_stop`/`finish()`, тик уровня в watchdog, запись `last_error`, применение `Change::apply`.
4. Fallback: старый демон отвечает на `subscribe` ошибкой `"неизвестная команда"` (существующее поведение match) → Dart-клиент ловит её и откатывается на поллинг `status` 100 мс. Приложение пишется против `Stream<AppEvent>` — движок-абстракция прячет разницу.

### Питфолл «частичной транскрипции»

У whispер сейчас **один транскрипт после stop** — частичного текста в реальном времени в движке не существует. Живой «распознаваемый текст» в дашборде на самом деле = `event:"text"` после вставки. Настоящие стриминг-партиалы требуют чанковой/оконной схемы распознавания — отдельное исследование движка (флаг для будущей фазы). НЕ закладывать `event:"partial"` в MVP.

---

## Decision 2 — Windows-каркас: встраивание dicta-core в процесс (flutter_rust_bridge)

### Сравнение

| Критерий | A. FRB in-process (embed dicta-core) | B. Порт dictad на named pipe |
|----------|--------------------------------------|------------------------------|
| Объём порта | Windows-ветки только в `capture.rs` (WASAPI) и `inject.rs` (SendInput); `config/audio/stt/filter` платформонезависимы; whisper-rs/Vulkan собирается на Windows | Весь `dictad/main.rs`: сокет → pipe (interprocess crate), systemd-автостарт, evdev-хоткей, spawn_ui, uinput-инфраструктура |
| Жизненный цикл | Приложение = процесс; нечему «ставиться» отдельным сервисом | Второй фоновый процесс на Windows: автозапуск, трей для демона, обновление — всё дублируется |
| Живой уровень | `StreamSink<f32>` через FFI — ноль IPC | Тот же subscribe-механизм, но его ещё надо портировать |
| Отказоустойчивость | Паника whisper валит UI-процесс; FRB-кодген в CI на Windows-раннере | Изоляция процесса (как Linux) |
| Прецедент FlClash | FlClash так делает на Android (ядро in-process, либа) | FlClash так делает на desktop, потому что ядру нужны привилегии/фоновость |

### Рекомендация: **A — встраивание (FRB)** для MVP-каркаса

Обоснование:
- Цель вехи по PROJECT.md — «каркас, собирающийся под Windows, с рабочей диктовкой MVP (WASAPI + SendInput)» — это ровно встраивание, без демон-инфраструктуры.
- На Windows нет systemd; lifecycle второго процесса = чистая новая работа, не нужная MVP. Приложение в трее уже живёт фоном (tray_manager), диктовка работает, пока приложение запущено — приемлемо для каркаса; фоновый сервис — следующая веха.
- FRB даёт `StreamSink` для уровня/состояния без какого-либо IPC — меньше движущихся частей, чем pipe+протокол.
- Linux не задет вообще: сокет-клиент Dart используется только на Linux.

**Шов на будущее:** интерфейс `DictaEngine` в Dart проектируется по форме протокола демона (start/stop/toggle/set/settings + `Stream<AppEvent>`). Если позже Windows-версия переедет на демон с named pipe (interprocess crate — готовый путь), меняется только реализация `EmbeddedEngine → PipeEngine`.

Компромиссы принять осознанно: FRB-кодген — лишний шаг сборки (Windows-раннер CI ставит Rust + flutter_rust_bridge_codegen); крэш ядра = крэш приложения (whisper-rs на Windows годами стабилен, риск принят для MVP).

---

## Decision 3 — Аудио-конвейер и частота метра

Захват, уровень, порог тишины, автостоп — **остаются в Rust полностью** (обе платформы). Flutter не прикасается к сэмплам, только получает скаляр уровня.

| Конвейер | Где захват+RMS | Частота метра к UI | Обоснование |
|----------|----------------|--------------------|-------------|
| Linux | dictad: watchdog читает хвост WAV каждые 80 мс → `AtomicU32` (существующее) | **12.5 Гц** (`event:"level"` на каждом тике watchdog) | dicta-ui сегодня рисует волну на 100-мс опросе (~10 Гц) и этого достаточно; 12.5 Гц + плавная интерполяция `AnimatedContainer`/`Tween` во Flutter выглядит плавно; 60 Гц дало бы ×5 спама по сокету без воспринимаемой выгоды |
| Windows | dicta-ffi: WASAPI-поток в Rust, тот же расчёт уровня | **~15–20 Гц** через `StreamSink<f32>` | тот же компромисс; FFI-поток дешевле сокета, но выше 20 Гц не нужно |

Анти-паттерн: не тянуть PCM-сэмплы в Dart и не считать уровень на стороне UI — двойная пересылка и рассинхрон порогов с демонными `min_level`/автостопом. UI показывает то, что решает Rust.

---

## Recommended Project Structure

```text
dicta/
├── app/                          # НОВОЕ: Flutter-проект dicta-app (sibling crates/, как core/ у FlClash)
│   ├── lib/
│   │   ├── core/                 # мосты к ядру — зеркало lib/core/ FlClash
│   │   │   ├── engine.dart       #   абстракция DictaEngine + AppEvent
│   │   │   ├── daemon_engine.dart#   Linux: unix-сокет, req/reply + subscribe
│   │   │   ├── embedded_engine.dart # Windows: FRB
│   │   │   └── events.dart       #   парсеры строк событий (чистые, тестируемые)
│   │   ├── dashboard/            # запись: состояние, волна, старт/стоп, текст
│   │   ├── history/              # журнал диктовок (локальная персистентность Dart)
│   │   ├── settings/             # вкладки настроек (валидация остаётся в демоне)
│   │   ├── models/               # браузер HF (порт логики hf.rs на Dart http)
│   │   └── shell/                # трей, окно, тема, роутинг
│   ├── linux/  windows/          # раннеры Flutter
│   └── pubspec.yaml
├── crates/
│   ├── dicta-ffi/                # НОВОЕ: FRB-api (api.rs: start/stop/status/set + StreamSink)
│   ├── dicta-core/               # +Windows-ветки в capture.rs / inject.rs (cfg(windows))
│   └── dictad/                   # +subscribe (один файл main.rs, как всё в демоне)
└── .github/workflows/            # +windows-build.yml (windows-runner)
```

Rationale:
- Flutter-проект — **не cargo-crate**; он живёт каталогом `app/` верхнего уровня, sibling к `crates/` — так же, как FlClash держит `lib/` рядом с `core/`. Не прятать его в `crates/` — cargo-воркспейс остаётся чистым.
- FRB-клей — отдельный **workspace-member** `crates/dicta-ffi` (не внутри dicta-core): dicta-core остаётся свободным от FRB-зависимостей, а dicta-ffi — тонкая api-обёртка. CMake Windows-раннера линкует его в бинарь приложения.
- `app/lib/core/` — преднамеренное эхо структуры FlClash (`controller.dart`/`interface.dart`): одна точка, где решается платформа, остальное приложение не знает о транспорте.

---

## Data Flow

### Дашборд Linux (новый основной поток)

```text
[кнопка Запись / хоткей DE / dictactl toggle]
   → dictad: run_start (state→Recording, broadcast "state")
   → watchdog 80мс: RMS из хвоста WAV → AtomicU32 → broadcast "level" (12.5 Гц)
   → [стоп] → воркер: тишина-гейт → whisper → filter → focus → inject
   → finish(): state→Idle, broadcast "state" + "text" + "error"?
   → app: Stream<AppEvent> → Riverpod provider → виджеты (волна, текст, история)
   → история: app дописывает запись локально (демон истории не хранит)
```

Ключевое следствие: приложение видит **чужие** триггеры (хоткей из DE, dictactl, оверлей) мгновенно через события — не только собственные нажатия. Это главный выигрыш subscribe перед поллингом.

### Дашборд Windows

```text
[кнопка/hotkey_manager] → dicta-ffi.start() → WASAPI-поток (Rust)
   → StreamSink<LevelEvent> → Dart Stream (12–20 Гц)
[стоп] → whisper-rs → filter → SendInput-вставка → StreamSink<TextEvent>
```

### Настройки (обе платформы)

Linux: форма → `{"cmd":"set","key":..,"value":..}` → демон `parse_change` валидирует → `Change::apply` → `event:"settings"` всем подписчикам (в т.ч. обратно приложению — подтверждение). Валидация остаётся единственным местом в демоне, как сейчас. Windows: та же логика в dicta-ffi (валидация переносится с `parse_change` — переиспользовать `EDITABLE_KEYS`).

## Patterns to Follow

### Pattern 1: Engine-абстракция (как FlClash CoreController)

**Что:** `DictaEngine`-интерфейс в Dart; платформа выбирает реализацию на старте; всё выше (дашборд/настройки/история) пишет только к интерфейсу и `Stream<AppEvent>`.
**Когда:** всегда; именно это позволит позже заменить embedded на pipe-демон без касания UI.
**Trade-off:** чуть больше косвенности; окупается второй платформой.

### Pattern 2: Неблокирующий broadcast (FlClash «events never block the core»)

**Что:** подписчики демона пишутся из отдельного writer-потока/через try-клон; медленный или умерший подписчик (приложение подвисло/убито) не останавливает watchdog и запись.
**Когда:** любая точка пуша в dictad.
**Trade-off:** события могут потеряться у мёртвого подписчика — это нормально, `hello`-снимок восстанавливает истину при переподключении.

### Pattern 3: Снимок + дельты (hello + события)

**Что:** подписка начинается с полного снимка (state/settings), дальше — только изменения. UI никогда не реконструирует состояние из «пропущенного».
**Когда:** то же для FRB: первый StreamSink-кадр = снимок.

## Anti-Patterns

### AP-1: Схема Settings станет четвёртой копией
**Ошибка:** продублировать `Settings`-снимок четвёртым разом в Dart руками.
**Почему плохо:** рассинхрон трёх копий уже есть; четвёртая усугубляет.
**Вместо:** в Dart — одна модель, генерируемая из JSON демона c `#[serde(default)]`-толерантностью (как panel.rs), с тестом «поля демона ⊆ поля Dart». Считать копии осознанной развязкой версий, обновлять по чек-листу STRUCTURE.md «Новая настройка».

### AP-2: Поллинг как постоянное решение
**Ошибка:** оставить 100-мс опрос навсегда, потому что «работает».
**Почему плохо:** настройки, изменённые dictactl/оверлеем, приходят с задержкой; 10 Гц JSON впустую; история теряет события, произошедшие между тиками.
**Вместо:** поллинг — только fallback к старому демону;subscribe — цель первой фазы.

### AP-3: Захват аудио в Dart
**Ошибка:** тянуть сэмплы/уровень в Dart и там рисовать «настоящую» волну.
**Почему плохо:** рассинхрон с демонными порогами тишины, двойной транспорт, лишняя работа.
**Вместо:** скаляр уровня из Rust, плавность — анимация Flutter.

### AP-4: Один соединение на всё (Linux)
**Ошибка:** слать `stop` по тому же соединению, что и подписка.
**Почему плохо:** демон держит stop-соединение открытым до конца вставки — события и команды должны не мешать друг другу.
**Вместо:** два соединения: persistent subscribe + короткие командные (как dictactl-стиль «один запрос — одно соединение» для команд, persistent — для событий).

## Scaling Considerations

Не веб-сервис; «масштаб» здесь — число клиентов и платформ.

| Масштаб | Что меняется |
|---------|---------------|
| 1–3 клиента (оверлей, CLI, приложение) | Ничего: поток на соединение уже так живёт; broadcast — O(клиентов), копейки |
| +Windows-платформа | Только реализация `EmbeddedEngine`; протокол демона не трогается |
| Фоновый сервис Windows (будущее) | `DictaEngine`-шов: `PipeEngine` на named pipe (interprocess) без касания UI |

### Приоритет прочности
1. Первое узкое место — блокировка watchdog медленным подписчиком → Pattern 2 обязателен с первого дня.
2. Второе — переполнение буфера сокета при зависшем приложении → отключение подписчика по ошибке записи, `hello`-восстановление.

## Integration Points

### Изменения существующего кода (минимальные)

| Файл | Изменение | Риск |
|------|-----------|------|
| `crates/dictad/src/main.rs` | arm `"subscribe"`, `Subscribers`, 4 точки broadcast | Низкий: аддитивно, match по `req.cmd` уже расходуется по умолчанию в ошибку; юнит-тест парсера событий |
| `crates/dicta-core/src/capture.rs` | `#[cfg(windows)]` WASAPI-Recorder (тот же интерфейс start/level/stop) | Средний: новая платформа, но изолирована cfg-гейтом, Linux-код не меняется |
| `crates/dicta-core/src/inject.rs` | `#[cfg(windows)]` SendInput-путь | Средний: тот же характер |
| `.github/workflows/` | windows-раннер: Flutter+Rust+FRB-кодген | Низкий: новый файл CI |

### Новые компоненты

| Компонент | Зависит от |
|-----------|------------|
| `app/` скелет Flutter + engine-абстракция | ничего (поллинг-фallback) |
| `app/lib/core/daemon_engine.dart` | dart:io Unix-сокет (нативно, без плагинов) |
| `app/lib/core/embedded_engine.dart` | `crates/dicta-ffi` + FRB-кодген |
| `crates/dicta-ffi` | dicta-core (config/audio/stt/filter) |
| Дашборд | subscribe (демон) + engine |
| История | `event:"text"` + локальная персистентность |
| Настройки/браузер HF | engine.set / settings-снимок; Dart `http` вместо curl-сабпроцесса |
| Трей | tray_manager (StatusNotifierItem — KDE ок); хоткеи на Linux уже есть в демоне (evdev), дублировать в приложении НЕ надо |
| Windows-сборка CI | windows-раннер, FRB |

### Внешние сервисы

| Сервис | Паттерн | Заметки |
|--------|---------|---------|
| Hugging Face API | Dart `http` из браузера моделей (порт чистых функций `hf.rs`) | замена curl-сабпроцесса; докачку делать Range-запросами или спавном curl |
| tray_manager / hotkey_manager | pub.dev-пакеты (nativeapi) | Linux-хоткей НЕ через приложение (evdev в демоне уже есть); Windows-хоткей — hotkey_manager |

## Build Order (порядок фаз и зависимости)

```text
1. subscribe в dictad ──────────────┐
2. Flutter-скелет + DictaEngine +    │ (2 стартует на поллинг-фallbackе,
   daemon_engine (поллинг) ─────────┤  3 требует 1)
3. Дашборд: события, волна, старт/стоп ← (1+2)
4. История ← (3: event:text)
5. Настройки + браузер моделей ← (2; параллельно 3/4)
6. Трей + сворачивание ← (3; независим от 4/5)
7. dicta-ffi + WASAPI/SendInput (Windows-ветки dicta-core) ← (независим от 3–6,
   отдельный workstream; требует Flutter-Windows-раннера локально/в CI)
8. embedded_engine + Windows-каркас в CI ← (7)
9. Упразднение dicta-settings ← (5: паритет функций)
```

Правила порядка:
- **1 раньше 3**: дашборд без событий — это поллинг, т.е. не то, что заказано («живой уровень», «реагирование на чужие триггеры»).
- **2 можно начинать параллельно с 1** (fallback-поллинг), но `events.dart`/парсеры пишутся сразу под финальный формат событий.
- **7 не блокирует UI-фазы** — Windows-ветка трогает только crates/, её можно вести отдельным исполнителем; но CI (8) — последняя, ей нужен живой Flutter-Windows-тулчейн.
- **9 строго после 5** (паритет браузера моделей и настроек).

## Phase-Specific Research Flags

- **Фаза Windows-захвата (7):** WASAPI-захват в Rust (wasapi crate / windows-rs) — выбрать библиотеку; поведение SendInput с UAC-окнами и играми; русские раскладки. Нужен phase-research.
- **Фаза subscribe (1):** если события окажутся нужнее шире (progressbar загрузки модели) — держать конвейер событий расширяемым, но не раздувать MVP-набор (5 типов выше достаточно).
- **Фаза истории:** потеря событий при закрытом приложении — зафиксировать как известное ограничение MVP (демон-стороны ring-buffer в вехе нет); формат персистентности (JSONL vs sqflite) решить в фазе.
- **Стриминг-партиалы whispер:** вне вехи; отдельное исследование движка если заказчик захочет «текст печатается во время речи».

## Sources

- FlClash IPC/события: `github.com/chen08209/FlClash` docs/architecture.md + `.agents/architecture.md`, DeepWiki 3.4 «Flutter-Go Communication», opendeep.wiki «Core 控制与生命周期» — **HIGH** (3 независимых источника, перекрёстно сверены)
- flutter_rust_bridge v2 (StreamSink, кодgen): официальный сайт frb (logging.md, stream.md, integrate) — **HIGH**
- interprocess crate (named pipe server Windows, sync/tokio): официальный репо kotauskas/interprocess — **HIGH**
- dart:io Unix-сокеты: Dart SDK changelog + SO — Linux/macOS **HIGH**; Windows AF_UNIX только с Dart 3.11 — **MEDIUM**
- tray_manager 0.9 / hotkey_manager 0.4 (nativeapi): pub.dev/GitHub leanflutter — факты **HIGH**, зрелость **MEDIUM**

---
*Architecture research для вехи v1.1 «Dicta App»*
*Researched: 2026-11-22*

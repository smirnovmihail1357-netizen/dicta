# Project Research Summary

**Project:** dicta — веха v1.1 «Dicta App» (Flutter-приложение + Windows-каркас поверх существующего Rust-диктанта)
**Domain:** голосовой диктант (local-first STT); Flutter desktop control-center поверх существующего демона `dictad`
**Researched:** 2026-11-22
**Confidence:** HIGH (стек и фичи) / MEDIUM (Windows-инъекция, архитектурные развилки — см. Gaps)

## Executive Summary

dicta — системный голосовой диктант на Rust (демон `dictad` + unix-сокет с построчным JSON-протоколом + тонкие клиенты dictactl/dicta-ui/dicta-settings). Веха v1.1 добавляет два куска: **dicta-app** — Flutter-приложение в духе FlClash (Material You, сайдбар, трей), являющееся *ещё одним клиентом демона*, и **Windows-каркас** — минимально рабочий цикл диктовки на Windows (WASAPI + SendInput) с CI на виндовом раннере. Экспертный образец (Superwhisper, FlClash) единодушен: control-окно живёт в трее, дашборд минимален (состояние/уровень/кнопки/последний текст), история — центральная ценность окна, а ядро остаётся отдельным процессом, с которым UI говорит по IPC.

**Ключевая развилка — мост Flutter ↔ Rust — решается большинством исследований в пользу IPC поверх существующего протокола.** STACK.md и PITFALLS.md сходятся (2 из 3): на Linux приложение — UDS-клиент демона через `dart:io`/`dart_ipc` с поллингом `status` 100 мс, БЕЗ изменения протокола и БЕЗ flutter_rust_bridge (FRB отложен до мобильной вехи); на Windows MVP — порт того же JSON-line протокола на named pipe (`interprocess` в Rust, `dart_ipc` в Dart). ARCHITECTURE.md — исключение: предлагает FRB-встраивание ядра на Windows и аддитивную `subscribe`-команду в демоне. Обе его идеи конфликтуют с жёстким ограничением PROJECT.md («поверх существующего демона без изменения протокола») и с Pitfall 5/6; их ценность — в продуманном дизайне интерфейса `DictaEngine` и событий, который следует перенять как *шов на будущее* (включая fallback-поллинг), но не как решение этой вехи. Окончательный выбор на Windows (named-pipe порт демона vs FRB-встраивание) — **решение фазового уровня, подтвердить в plan-phase**; IPC-путь — рекомендация по умолчанию.

Главные риски: (1) GPL-контаминация от FlClash — guardrail «паттерны да, код нет» во все UI-фазы; (2) Windows-вставка — самая богатая на ошибки территория (гонка восстановления буфера, UIPI, зажатые модификаторы) — нужен спайк и 4 обязательных тест-кейса; (3) четвёртая копия схемы Settings в Dart — golden-тест до UI; (4) Windows-ветка сгниёт без CI — CI-воркфлоу обязан быть **первым deliverable** Windows-фазы, до любого кода фич. Живой стриминг текста во время распознавания — вне вехи (переделка inference движка).

## Key Findings

### Ключевое решение: мост Flutter ↔ Rust (конвергенция исследований)

| Вопрос | Рекомендация | Сходка источников |
|---|---|---|
| Linux-транспорт | UDS-клиент поверх существующего JSON-line протокола; поллинг `status` 100 мс через изолированный сервис-класс | STACK ✅, PITFALLS ✅, ARCHITECTURE (поллинг = fallback) |
| Изменение протокола демона | ❌ Никаких `subscribe`/event-пушей в этой вехе — жёсткое ограничение PROJECT.md | PITFALLS P6 ✅, STACK ✅; ARCHITECTURE — за `subscribe`, отклонить как нарушение ограничения, сохранить дизайн как будущий вариант |
| flutter_rust_bridge | ⛔ Не в этой вехе (дублирует IPC, ломает «приложение = клиент демона», кодоген/сборочная сложность) | STACK ✅, PITFALLS P5 ✅; ARCHITECTURE — за (только для Windows),.flag как решение plan-time |
| Windows MVP | Рекомендация по умолчанию: named pipe `\\.\pipe\dicta` + тот же JSON-протокол (`interprocess` 2.4.8 / `dart_ipc` 2.0.2), FlClash-десктоп-паттерн | STACK ✅, PITFALLS ✅ (большинство); ARCHITECTURE настаивает на FRB-embed — развилка,详见 Implications Phase 6 |
| Защитный шов | Интерфейс `DictaEngine` в Dart (start/stop/toggle/set/status + Stream/AppEvent) — платформа меняет реализацию, UI не знает о транспорте | Все три документа (паттерн CoreController FlClash) |

### Recommended Stack

Весь стек выровнен одной версией: **Flutter 3.50.0 / Dart 3.14.0** — на нём вышло поколение leanflutter-плагинов (tray_manager 0.9.0 / hotkey_manager 0.4.0 / window_manager 0.7.0, все требуют nativeapi 0.6.0 — ставить вместе, не смешивать со старыми). Версии сверены с pub.dev/crates.io 2026-11-22.

- **flutter_riverpod 3.5.0** — state management; дизайн-паритет с FlClash, `Notifier`/`AsyncNotifier` под состояние демона
- **dynamic_color 2.1.0** — Material You: XDG-portal/GTK на Linux, системный accent на Windows
- **tray_manager / window_manager / hotkey_manager** (leanflutter, единое nativeapi-ядро) — трей SNI, close-to-tray, RegisterHotKey на Windows
- **dart_ipc 2.0.2** — named pipe на Windows / UDS на Unix, socket-совместимый API, один `DictaClient` на обе платформы
- **dio 5.11.1** — браузер моделей HF (download-прогресс, CancelToken) — перенос `hf.rs`/`catalog.rs` в Dart
- **Rust (Windows-ветка):** `interprocess` 2.4.8 (named pipe), `cpal` 0.18.2 (WASAPI-захват, MSRV 1.85 — проверить тулчейн CI), `enigo` 0.6.1 (SendInput + KEYEVENTF_UNICODE)
- **l10n:** официальный `flutter gen-l10n` + ARB, русский-first (`app_ru.arb` шаблон)
- **CI:** `subosito/flutter-action@v2` на `windows-latest`, пинованные версии, rust-cache по Cargo.lock

Чего НЕ использовать: flutter_rust_bridge (веха), system_tray (протух), Dart-аудио-пакеты (аудио остаётся в Rust), Provider/Bloc (выбран FlClash-паритет), AF_UNIX на Windows (driver-зависимость).

### Expected Features

**Must have (table stakes, v1):** оболочка с сайдбаром (Дашборд/История/Модели/Настройки) + темы; дашборд состояния с живым уровнем (`status.level`), старт/стоп/отмена; последний текст + «Скопировать»/«Вставить заново» (`status.text` + `inject`); **история** — журнал/поиск/re-insert/удаление, пассивный захват через переходы `status` (Transcribing→Idle с новым текстом) без изменения протокола; настройки — все 17 EDITABLE_KEYS формами с живым `set` и честными состояниями (отказ reload при записи); браузер моделей HF + управление скачанными (удаление неактивной); трей (start/stop, toggle окна, выход; иконка состояния) + close-to-tray; hotkey-экран с определением рабочего пути (evdev vs DE-адаптер) и честным предупреждением «применится после рестарта демона».

**Should have (v1.x, после валидации):** быстрое переключение модели из дашборда/трея; визуальная калибровка порога тишины; диагностика окружения + onboarding-чеклист (killer-фича для Linux); таблица paste_keys по приложениям; словарь/initial prompt (требует лёгкого расширения демона — отложен).

**Defer (v2+):** потоковый показ текста во время распознавания (переделка inference — **вне вехи**); модели по языкам/маршрутизация; повторная обработка старых диктовок.

**Anti-features (не строить):** облачные STT, AI-постобработка LLM, запись встреч/файловая транскрипция, хранение аудио в истории, статистика WPM, свободный редактор config.toml, облачный backup настроек.

### Architecture Approach

Приложение — четвёртый клиент демона (как dictactl/панель/settings), НЕ второе ядро. Единая точка платформенной развилки — Dart-интерфейс DictaEngine в core-слое приложения (файл движка появится в Phase 1) с двумя реализациями (daemon_engine — Linux сокет; Windows-реализация — по итогам фазового решения). Структура `app/` — sibling к `crates/` (cargo-воркспейс не трогаем), `lib/core/` зеркалит назначение FlClash-структуры. Все записи настроек — только через `set` (валидация остаётся в демоне); аудио-конвейер (захват/RMS/пороги/автостоп) полностью в Rust — UI получает скаляр уровня. Критичные паттерны: поллинг через изолированный Dart-сервис (никаких сокет-вызовов из виджетов), snapshot-семантика статуса, jsonEncode для всех запросов (класс ручного экранирования — известный баг-паттерн dicta), reconnect-логика при смерти демона.

### Critical Pitfalls

1. **GPL-контаминация от FlClash** — guardrail в каждый UI-план: смотрим скриншоты/README/описание архитектуры, НЕ исходники; имена/структура файлов не калькировать.
2. **Windows-вставка — перенос Linux-паттерна «буфер → Ctrl+V → таймер → восстановить»** — воспроизведённая ошибка (Handy #502, OpenWhispr #829): правильный сигнал — delayed render `SetClipboardData(NULL)` + settle ~400 мс (Chromium probe-then-read), оптаут-форматы от Win+V/облака; плюс SendInput-ловушки: гонка фокуса, зажатые модификаторы, UIPI (тихий отказ в elevated-окна).
3. **FRB вместо IPC** — «выглядит проще сокета», но ломает схему «клиент демона», добавляет кодоген/версионные проверки/хрупкие build hooks. Решение «IPC, не FFI» — до первой строки Dart-кода.
4. **Четвёртая копия Settings** — Dart-модель из одного источника правды + golden-тест (fixture JSON демона → десериализация Dart ↔ Rust-тест), unknown-tolerance как `#[serde(default)]`; `set` по одному ключу, не объектом.
5. **Дублирование хоткея** — на Linux глобальный хоткей уже принадлежит демону (evdev); приложение его НЕ регистрирует (иначе двойной toggle = гонка двойного stop). Windows — RegisterHotKey в приложении, там демона в этой вехе нет.

## Implications for Roadmap

Предлагаемая структура — 8 фаз. Порядок следует зависимостям (ARCHITECTURE Build Order, скорректированный под ограничение «протокол не трогаем») и pitfall-to-phase mapping из PITFALLS.md.

### Phase 1: Каркас dicta-app + IPC-клиент
**Rationale:** архитектурное решение «IPC, не FFI» (Pitfall 5) фиксируется ДО первой строки кода; GPL-guardrail прописывается здесь и действует во всех UI-фазах.
**Delivers:** Flutter-скелет (linux/windows-платформы), `DictaEngine`-абстракция + daemon-клиент на поллинге (fallback-уровень всей вехи), Dart-модель Settings с golden-тестом, gen-l10n ru-first.
**Addresses:** оболочка/темы (частично), фундамент всего.
**Avoids:** P1 (GPL), P5 (FRB-ловушка), P7 (4-я копия Settings — схема до UI).

### Phase 2: Дашборд записи
**Rationale:** первая видимая ценность; всё нужное уже в `status` (state/level/seconds/text/error) + `toggle/start/stop/cancel`.
**Delivers:** индикатор состояния, живой уровень (10 Гц, CustomPaint на листовых виджетах), старт/стоп/отмена (cancel disabled в Transcribing с подписью «распознаю…» — честность вместо мнимой), последний текст + копировать/вставить заново, строка ошибки, spinner после stop.
**Uses:** Riverpod 3.5, dart-сервис поллинга.
**Avoids:** P6 (поллинг через сервис-класс; тест kill-dictad → reconnect), AP-3 (уровень из Rust, не сэмплы в Dart).

### Phase 3: История диктовок
**Rationale:** центральная ценность окна (Superwhisper-стандарт); зависима от дашборда (тот же status-поток).
**Delivers:** пассивный захват (переход Transcribing→Idle с новым `text` → локальный store приложения), журнал + поиск + копировать/re-insert (`inject`)/удалить. Известное ограничение MVP: приложение должно быть открыто/в трее — зафиксировать в docs.
**Avoids:** протокол демона не тронут (вариант B JSONL-в-демоне отклонён — против ограничения вехи; дверь не закрывать).

### Phase 4: Настройки + браузер моделей
**Rationale:** паритет с dicta-settings — критерий закрытия вехи; двигается после фундамента, независимо от истории.
**Delivers:** все 17 EDITABLE_KEYS формами, живой `set`, честные состояния (reload отказ при записи), предупреждение перед ENGINE-ключами; HF-браузер (порт hf.rs/catalog.rs на dio: каталог/поиск/VRAM/докачка .part), список установленных, активная помечена, удаление неактивной.
**Avoids:** P7-остатки (golden-тест уже зелёный), batch-применение ENGINE-ключей (не триггерить reload на каждый чекбокс).

### Phase 5: Трей + хоткей-экран (Linux)
**Rationale:** фоновость приложения; «запись без окна» на Linux уже обеспечена демон-хоткеем — приложение лишь дублирует действия кнопками.
**Delivers:** трей SNI (start/stop/отмена/настройки/выход, левый клик = toggle окна, иконка состояния), close-to-tray, hotkey-экран (показ комбинации, рекордер с Esc/Backspace, определение пути evdev vs DE-адаптер, предупреждение «рестарт демона»).
**Avoids:** P4 (приложение НЕ регистрирует глобальный хоткей на Linux — двойной toggle; GNOME-трей честно documented как degraded).

### Phase 6: Windows CI + каркас — CI ПЕРВЫЙ deliverable
**Rationale:** Flutter под Windows не кросс-компилируется с Linux; без CI ветка пишется вслепую и гниёт (Pitfall 8).
**Delivers:** (1) `windows-build.yml` — зелёный артефакт zip на ПУСТОМ скелете ДО кода фич (rust-cache, пинованные Flutter/Rust); (2) фазовое решение моста на Windows — named-pipe порт (по умолчанию, interprocess/dart_ipc) vs FRB-embed (аргумент ARCHITECTURE) — подтвердить в discuss/plan; (3) скелет Windows-движка + транспорт; пути `%APPDATA%`/`%LOCALAPPDATA%` (не хардкодить).
**Avoids:** P8, безопасность named pipe (DACL только текущий user-SID).
**⚠ Research flag:** решение моста + детали CI на виндовом раннере.

### Phase 7: Windows диктовка MVP (WASAPI + вставка)
**Rationale:** самое исследовательская фаза вехи — Windows-вставка это скопление воспроизведённых чужих багов.
**Delivers:** `#[cfg(windows)]`-ветки dicta-core: захват cpal/WASAPI (16 кГц mono s16 → существующий stt-путь), вставка SendInput + буфер с delayed render, RegisterHotKey (await + отказ-обработка), hotkey-триггер записи без окна.
**Avoids:** P2 (таймерное восстановление → delayed render + оптаут-форматы), P3 (гонка фокуса/модификаторы/UIPI — 4 обязательных кейса: обычное окно, elevated, зажатый модификатор, Chromium), cpal AUTOCONVERTPCM-тишина на Win11 24H2 (консольный/raw формат).
**⚠ Research flag: спайк delayed-render/WM_RENDERFORMAT обязателен — заложить время на эксперимент, не только план.**

### Phase 8: Декомиссия dicta-settings
**Rationale:** строго после полного паритета Phase 4 (+ трей/хоткеи).
**Delivers:** чек-лист паритета, подписанный пользователем (все поля egui + браузер моделей + валидация), затем удаление dicta-settings.
**Avoids:** регресс для пользователя — единственное полное окно настроек не убивать до равнозначной замены.

### Phase Ordering Rationale

- **1 → 2:** IPC-решение и Settings-схема до любого UI (Pitfalls P5/P7 — «самая первая фаза»).
- **2 → 3:** история питается тем же status-потоком, что дашборд; ничего не требует от демона.
- **4 параллельно 2/3:** настройки/модели зависят только от фундамента Phase 1.
- **5 после 2:** действия трея мапятся на команды, уже обкатанные дашбордом.
- **6–7 — отдельный workstream** (трогают только crates/ + CI), не блокирует UI-фазы; CI раньше кода (P8); вставка — последней как самая рискованная.
- **8 строго последней** — паритет подписан, потом удаляем.
- **Все фазы:** CI-гарантия «старые клиенты (dictactl) работают против демона без изменений протокола» — демон на Linux не пересобирается с изменениями протокола во всей вехе.

### Research Flags

Нужен `/gsd-plan-phase --research-phase`:
- **Phase 6 (Windows CI + каркас):** фазовое решение моста (named-pipe vs FRB-embed) + воркфлоу на windows-раннере (кеш, длинные пути, FRB-кодген если выбран embed).
- **Phase 7 (Windows диктовка):** спайк delayed-render/WM_RENDERFORMAT (hidden window + message pump), SendInput/UIPI-матрица, cpal-форматы на Win11 24H2. Самая исследовательская фаза вехи.

Стандартные паттерны (research не нужен):
- **Phase 1–2 (каркас, дашборд):** Flutter-скелет, Riverpod, dart-поллинг — документировано STACK/ARCHITECTURE, живые примеры в dicta-ui.
- **Phase 4 (настройки/браузер):** логика переносится из живого Rust/egui-кода (`hf.rs`, `catalog.rs`, EDITABLE_KEYS) — это порт, не исследование.
- **Phase 5 (трей):** стандартно при условии «хоткей остаётся в демоне» — фиксируется в плане до кода.

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | Все версии сверены с pub.dev/crates.io/docs.flutter.dev 2026-11-22; дайджесты в research-store |
| Features | HIGH | Конкурентный ландшафт (Superwhisper/FlClash) первоисточниками; MEDIUM только по стримингу/маршрутизации (вне вехи) |
| Architecture | MEDIUM | Паттерн FlClash подтверждён 3 источниками (HIGH), но ARCHITECTURE.md дважды отклоняется за нарушение ограничения вехи (subscribe, FRB-embed) — рекомендациям доверять выборочно |
| Pitfalls | MEDIUM | Windows-инъекция: 4+ независимых источника сходятся (формально web-класс, конвергенция сильная); локальный контекст HIGH (живой аудит CONCERNS.md) |

**Overall confidence:** HIGH для Linux-части вехи; MEDIUM для Windows-фаз (обе развилки — мост и вставка — решаются спайком/plan-time).

### Gaps to Address

- **Мост на Windows (named-pipe порт vs FRB-embed):** majority (STACK+PITFALLS) — named pipe; аргумент embed (ARCHITECTURE) — отсутствие демон-инфраструктуры на Windows. Решить в Phase 6 discuss/plan; `DictaEngine`-шов делает выбор обратимым.
- **dart:io AF_UNIX vs dart_ipc (противоречие в исследованиях):** STACK.md утверждает нативную поддержку AF_UNIX в `dart:io`, PITFALLS.md — что её нет и нужен `dart_ipc`. Мини-спайк в Phase 1 (одна строка `Socket.connect(type: Unix)`); dart_ipc — fallback, оба дают один API.
- **Надёжность пассивной истории:** события между тиками/при закрытом приложении теряются — принять как ограничение MVP (трей делает приложение почти всегда живым), JSONL-в-демоне — будущий вариант вне ограничения.
- **Hotkey-путь на Linux:** ключа нет в EDITABLE_KEYS (evdev читается демоном на старте) — UI обязан определять рабочий путь (evdev-конфиг vs DE-адаптер) и честно предупреждать о рестарте; уточнить семантику DE-адаптеров в Phase 5.
- **cpal на Linux-сборке:** даже с PipeWire-фичей требует libasound2-dev; MSRV 1.85 выше остального workspace — проверить тулчейн CI в Phase 6.
- **Репо-гигиена:** 22 незапушенных коммита в main на момент старта — запушить ДО создания Windows-ветки (Phase 6), иначе CI и ревью не увидят историю.

## Sources

### Primary (HIGH confidence)
- docs.flutter.dev; pub.dev (flutter_riverpod 3.5.0, dynamic_color 2.1.0, tray_manager 0.9.0, hotkey_manager 0.4.0, window_manager 0.7.0, dart_ipc 2.0.2, dio 5.11.1, FRB 2.13.0); crates.io/docs.rs (cpal 0.18.2, interprocess 2.4.8, enigo 0.6.1) — версии и совместимости, сверено 2026-11-22
- Протокол dictad: `crates/dictad/src/main.rs`, `crates/dicta-core/src/config.rs` (EDITABLE_KEYS/ENGINE_KEYS) — первоисточник
- `.planning/PROJECT.md`, `.planning/codebase/CONCERNS.md` — живой аудит кодовой базы
- Superwhisper официальная документация (History, Models, Advanced) — продуктовые решения лидера категории
- FlClash репо (README/CHANGELOG, описание архитектуры) — UI-паттерны и desktop-IPC (patterns only, GPL — код не читается)

### Secondary (MEDIUM confidence)
- FlClash DeepWiki/opendeep.wiki (архитектура: framed IPC, события, Riverpod) — 3 перекрёстных источника
- Whispering/Epicenter, stttui, WM Keyboard — фича-ландшафт local-диктантов
- hotkey_manager/tray_manager README + xdg-desktop-portal GlobalShortcuts docs — матрица DE-поддержки
- Трей-гайды (Wails/SkillDB/hashhackers) — конвергенция «5–8 пунктов, Show/Quit, click-toggle»
- win-text-inject (edfl.dev) — delayed render, оптаут-форматы, UIPI
- cpal issue #1200 (AUTOCONVERTPCM-тишина Win11 24H2), GitHub Actions rust-cache-практики

### Tertiary (LOW confidence, но конвергентные)
- Handy #502, OpenWhispr #829/#547, FluidVoice #940 — Windows-вставка/хоткеи: поодиночке LOW, 4+ независимых источника сходятся
- GPL clean-room консенсус (opensource.SE/softwareengineering.SE) — идеи свободны, код нет
- Дайджесты research-store (13 вопросов, тир MEDIUM) — кэш веб-поиска с кросс-проверкой

---
*Research completed: 2026-11-22*
*Ready for roadmap: yes*

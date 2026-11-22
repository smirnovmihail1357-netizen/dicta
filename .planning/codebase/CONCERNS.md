---
last_mapped_commit: 20e00408af8921966d7d02f862d064e71658eb28
last_mapped_at: 2026-11-22
---
# Codebase Concerns

**Analysis Date:** 2026-11-22

## Tech Debt

**Слепые sleep'ы в пути вставки (`inject.rs`):**
- Issue: вся последовательность paste построена на фиксированных паузах вместо подтверждённых событий: 300 мс после `UI_DEV_CREATE` (`crates/dicta-core/src/inject.rs:228`), 20 мс между нажатием и отпусканием клавиши (`inject.rs:287`), 15 мс flush после каждого `with_keyboard` (`inject.rs:148`), 600 мс после пересоздания uinput-девайса (`inject.rs:156`), 1500 мс в `test_keys` (`inject.rs:450`), 2 мс при EAGAIN (`inject.rs:258`).
- Files: `crates/dicta-core/src/inject.rs`
- Impact: на медленной машине (холодный udev, нагруженный KWin) 300 мс не хватает — «демон всё сделал, а вставки нет»; на быстрой — пауза добавляется к каждой вставке без пользы. Значения подобраны вручную под одну машину.
- Fix approach: заменить слепые паузы на события: чтение `/sys/class/input/event*/` до появления устройства после `UI_DEV_CREATE`; для буфера уже сделано правильно (`wait_clipboard_ready` — потолок ожидания вместо паузы, `inject.rs:330`); длительность press→release оставить минимальной, но измерять и логировать.

**Панель настроек в оверлее — кандидат на упрощение:**
- Issue: `dicta-ui` несёт полноценное подменю настроек (13 кнопок в 3 ряда, `crates/dicta-ui/src/panel.rs:165-263`) при том, что отдельное окно `dicta-settings` (`crates/dicta-settings/src/main.rs`) уже покрывает все настройки с полями ввода. Панель циклическая: «вставка» перелистывает paste→atspi→clipboard→stdout (`panel.rs:312-326`), «модель» перелистывает до 12 `.bin`-файлов по одному на клик (`panel.rs:354-370`), min± шагает по 0.05 (`panel.rs:288-299`). Прямой ввод невозможен: layer surface с `KeyboardInteractivity::None` (`crates/dicta-ui/src/main.rs:563`).
- Files: `crates/dicta-ui/src/panel.rs`, `crates/dicta-ui/src/render.rs` (`paint_panel`), `crates/dicta-ui/src/main.rs` (`draw_panel`, `press`, `panel_rest_sig`)
- Impact: каждый клик по «модель» инициирует полную перезагрузку whisper в VRAM (`schedule_engine_reload`, `crates/dictad/src/main.rs:744`); пользователь листает модель вслепую без превью; поддержка четырёх сторон × поворота кадра × кэша покоя `panel_rest_sig` (`dicta-ui/src/main.rs:1422-1435, 1491`) — большой объём хрупкого кода рендера ради дублирующего UI.
- Fix approach: сократить панель до управления записью (старт/стоп/отмена) + кнопка «настройки», открывающая `dicta-settings`; либо оставить только частные регуляторы (min±, авто-стоп), убрав циклы модель/режим/язык. Кэш `panel_rest_sig` при этом упрощается сам.

**Три независимые копии схемы настроек:**
- Issue: структура `Settings` продублирована с разным набором полей: серверная — `crates/dictad/src/main.rs:58-82`, панель оверлея — `crates/dicta-ui/src/panel.rs:17-38`, окно настроек — `crates/dicta-settings/src/state.rs:15-46`. Все десериализуют один и тот же JSON `{"cmd":"settings"}` с `#[serde(default)]`.
- Files: `crates/dictad/src/main.rs`, `crates/dicta-ui/src/panel.rs`, `crates/dicta-settings/src/state.rs`
- Impact: новое поле в ответе демона нужно добавлять в три места; забытое поле молча получает дефолт (тест `unknown_daemon_answers_do_not_break_the_panel` фиксирует допустимость расницы версий, но не полноту).
- Fix approach: вынести serde-структуру ответа `settings` в `dicta-core` (как уже сделано для `Config`), чтобы все три бинарника и `dictactl settings --json` делили её.

**Ручной разбор config.toml в `dicta-ui`:**
- Issue: путь сокета извлекается строковым сплитом `line.split('"').nth(1)` по первой строке, начинающейся с `socket` (`crates/dicta-ui/src/main.rs:1891-1904`), вместо `toml`-парсера, который есть в `dicta-core::Config::load`.
- Files: `crates/dicta-ui/src/main.rs:1891`
- Impact: `socket = 'одинарные кавычки'`, комментарий после значения или пробел до `=` ломают поиск — оверлей постучится не туда.
- Fix approach: `dicta-ui` уже зависит от `dicta-core` (модуль `edge`)? — если нет, использовать `dicta_core::config::default_socket_path`-логику или `DICTA_CONFIG`-парсер через `toml` в собственном коде.

**Четыре способа собрать JSON-команду вручную:**
- Issue: `dictactl` собирает payload конкатенацией строк и **вырезает** кавычки (`crates/dictactl/src/main.rs:140, 266, 275, 284`), панель оверлея экранирует только `\` и `"` (`crates/dicta-ui/src/panel.rs:87`), `net.rs` экранирует так же (`crates/dicta-settings/src/net.rs:51`), а демон парсит `serde_json`. Ни один клиент не использует `serde_json::to_string`.
- Files: `crates/dictactl/src/main.rs`, `crates/dicta-ui/src/panel.rs`, `crates/dicta-settings/src/net.rs`
- Impact: см. баг «dictactl inject теряет символы»; экранирование не покрывает управляющие символы (например `\n` в тексте поломает строковый протокол — демон читает построчно).
- Fix approach: строить команды через `serde_json::json!`/`to_string` во всех клиентах; протокол демона читает строку целиком, поэтому `\n` внутри JSON-строки (экранированный как `\\n`) безопасен только при корректном экранировании.

**Логирование через `eprintln!`:**
- Issue: весь демон и оверлей пишут в stderr без уровней и структуры; диагностика гонок и таймингов замешана в общий поток.
- Files: повсеместно — `crates/dictad/src/main.rs`, `crates/dicta-core/src/inject.rs`, `crates/dicta-ui/src/main.rs`
- Impact: под systemd `eprintln!` попадает в journal, но фильтровать (например, только вставки) нельзя.
- Fix approach: минимальный логгер с уровнями или `log`+`env_logger`; для протокола вставки — единый префикс.

**Артефакты правки в комментариях:**
- Issue: смешанные языки и задвоенные комментарии: «Убиваем上一个 демон» (`crates/dictad/src/main.rs:147`), продублированные абзацы о `draw_hint` (`crates/dicta-ui/src/main.rs:1503-1512`), о размещении (`main.rs:529-532`), о вертикальной пилюле (`main.rs:340-341`).
- Files: `crates/dictad/src/main.rs`, `crates/dicta-ui/src/main.rs`
- Impact: шум в чтении; признак активных зон правок, где комментарии уже разошлись с кодом.
- Fix approach: почистить при следующем проходе по файлу.

## Known Bugs

**Узкая гонка двойного stop в dictad:**
- Symptoms: два одновременных `stop` (горячая клавиша + `dictactl stop`, или watchdog тишины + клавиша) могут оба пройти проверку `state != Recording` (`crates/dictad/src/main.rs:1036`) до того, как первый сохранит `Transcribing` (`main.rs:1039`). Второй поток берёт мьютекс, находит `recorder == None`, вызывает `finish(state)` — и сбрасывает `Transcribing → Idle` (`main.rs:1197-1206`), пока воркер первого stop ещё распознаёт.
- Files: `crates/dictad/src/main.rs:1036-1039, 1046-1049, 1197-1206`
- Trigger: быстрые подряд идущие stop-команды из разных источников (hotkey-поток `do_toggle` `main.rs:1208`, watchdog `main.rs:260`, соединение сокета `main.rs:962`).
- Impact: `status` показывает `idle` во время работающего inference; пользователь может начать новую запись и сменить модель (`schedule_engine_reload` требует Idle, `main.rs:750`) во время распознавания — два воркера и, при смене модели, потенциально две модели.
- Workaround: нет; окно узкое (между `state.load` и `state.store`).
- Fix approach: заменить load-then-store на атомарный захват: `state.compare_exchange(Recording, Transcribing)` в начале `run_stop`; при неудаче — тихо вернуть `Ok(None)`. Это же гарантирует, что `finish()` второго потока не сработает.

**`cancel` во время распознавания не отменяет вставку:**
- Symptoms: Esc/`dictactl cancel` в состоянии `Transcribing` сбрасывает состояние в `Idle` (`crates/dictad/src/main.rs:940-948`), но воркер уже запущен: он распознает сэмплы и **вставит текст** в фокусное окно (`main.rs:1073-1192`). Документация обещает «отменить запись без распознавания».
- Files: `crates/dictad/src/main.rs:940-948` (ветка cancel), `main.rs:1073+` (воркер)
- Trigger: cancel, посланный после stop, пока идёт inference.
- Workaround: нет.
- Fix approach: флаг отмены (`Arc<AtomicBool>` на воркер), который воркер проверяет перед шагами 4-6 (пустой текст, вставка); cancel в `Transcribing` поднимает флаг, а не молча сбрасывает состояние.

**`dictactl inject` и `AtspiInsert` теряют кавычки и бэкслэши:**
- Symptoms: текст очищается удалением символов: `text.replace(['"', '\\'], "")` (`crates/dictactl/src/main.rs:266, 275, 279, 284`) — вместо экранирования. `dictactl inject 'он сказал "да"'` вставит `он сказал да`.
- Files: `crates/dictactl/src/main.rs:140` (та же потеря в `--app`), `main.rs:266-291`
- Trigger: любой inject/atspi-insert текст с `"` или `\`.
- Workaround: использовать протокол через `dictad` напрямую или панель (там экранирование корректное).
- Fix approach: `json_quote` уже есть в том же файле (`main.rs:154`) — применять её к text/keys/app вместо `replace`.

**`set_key` портит значения с `#`:**
- Symptoms: хвостовой комментарий извлекается как `line.find('#')` (`crates/dicta-core/src/config.rs:207`) без учёта кавычек. Строка `model = "/диск#папка/model.bin"` при смене любой настройки перепишется как `model = "<новое>"#папка/model.bin"` — TOML станет битым.
- Files: `crates/dicta-core/src/config.rs:185-231`
- Trigger: `#` внутри значения настройки (путь модели/сокета с `#`), затем любое изменение через панель/`dictactl settings`.
- Workaround: не использовать `#` в путях.
- Fix approach: искать `#` только вне кавычек (простой скан с учётом `"`), либо сначала парсить строку `toml`-значения и вычислять позицию комментария по ней.

**Разнобой комментариев `gpu_device`:**
- Symptoms: три несовместимых описания одного поля: `config.example.toml:14` — «0 = быстрое устройство (замер: ~200 мс), 1 = медленное (~4 с)»; тест-фикстура `crates/dicta-core/src/config.rs:336` — «0 = iGPU, 1 = XTX»; окно настроек `crates/dicta-settings/src/main.rs:780-784` повторяет замеры (0 — быстрое). `docs/performance.md:13` говорит, что GPU = Radeon 610M iGPU при `gpu_device = 0`. То есть «XTX» (дискретная) — как раз медленное устройство, но комментарий теста читается так, будто индексы — это стабильная семантика устройств.
- Files: `config.example.toml`, `crates/dicta-core/src/config.rs:336`, `crates/dicta-settings/src/main.rs:780-784`, `docs/performance.md`
- Trigger: пользователь читает комментарий «1 = XTX» и переключается на дискретную карту, ожидая ускорения, — получает ~4 с на фразу.
- Fix approach: описывать не «что за железо», а как выяснить: «индекс — это порядок перечисления бэкенда (уточняй по "распознано за" в journalctl, см. CONTRIBUTING.md)»; убрать «0 = iGPU, 1 = XTX» из тест-фикстуры.

**Гонки состояния Recording/Transcribing/Idle (сводка):**
- `run_start` (`crates/dictad/src/main.rs:1014`) проверяет `Recording` под мьютексом `Handler`, но **не** запрещает старт в состоянии `Transcribing` — новая запись может начаться, пока старая распознаётся (воркер это терпит: `finish()` понижает только `Transcribing`, `main.rs:1197`).
- `do_toggle` читает состояние вне мьютекса (`main.rs:1213`) — между чтением и входом в `run_start`/`run_stop` другой поток успевает сменить состояние; решается тем же CAS, что и двойной stop.
- `started_at` и `state` — отдельные атомики без общей синхронизации: `cancel` обнуляет `started_at` и ставит `Idle` (`main.rs:945-946`) вне транзакции с `run_start`; сейчас порядок не ломается из-за мьютекса `Handler`, но связку стоит держать в одном месте.
- Смягчение: `finish()` через `compare_exchange` (`main.rs:1197-1206`) — правильный паттерн; его надо распространить и на захват `Recording → Transcribing`.

## Security Considerations

**Системный буфер обмена как транспорт вставки:**
- Risk: при `inject_mode = "paste"` распознанный текст попадает в системный clipboard минимум на `clipboard_restore_ms` (2.5 с по умолчанию, `crates/dicta-core/src/config.rs:93`). Менеджеры буферов с историей (clipman, CopyQ, Klipper) и облачные синхронизаторы сохраняют диктовку надолго; за это время пользователь может вручную вставить её в другое окно.
- Files: `crates/dicta-core/src/inject.rs:386-412`
- Current mitigation: восстановление старого буфера в фоне (`schedule_restore`, `inject.rs:351`); альтернативный путь `atspi` без буфера (`crates/dicta-core/src/atspi.rs`).
- Recommendations: документировать риск в README/SECURITY.md рядом с `inject_mode`; предложить `clipboard_restore_ms` меньше только после подтверждения вставки, а не по таймеру.

**Восстановление буфера обмена не сохраняет типы:**
- Risk: `wl-paste --no-newline` читает только текстовый MIME (`inject.rs:393`), `String::from_utf8_lossy` (`inject.rs:322`) разрушает бинарные данные. Если в буфере лежала картинка, после диктовки она заменится на манглический текст (или не восстановится вовсе при пустом выводе).
- Files: `crates/dicta-core/src/inject.rs:307-326, 393-410`
- Current mitigation: нет.
- Recommendations: перед заменой проверять `wl-paste --list-types`; если среди типов нет `text/plain` — не трогать буфер и предупредить, либо восстанавливать через сохранение всех офферов (сложно; честнее задокументировать ограничение).

**Anyproces-инъекция через сокет без авторизации команд:**
- Risk: сокет `0o600` (`crates/dictad/src/main.rs:161`) ограничивает доступ пользователем, но любой процесс этого пользователя может послать `{"cmd":"inject","text":"..."}` — вставить произвольный текст в фокусное окно (в т.ч. команды шелла в открытом терминале).
- Files: `crates/dictad/src/main.rs:878-934`
- Current mitigation: право на сокет = право на сессию пользователя; это принято для десктоп-демонов.
- Recommendations: приемлемо по умолчанию; не расширять права на сокет (не выставлять группу) при инсталляции.

**Глобальный evdev-хоткей читает все устройства ввода:**
- Risk: режим `hotkey != "off"` требует чтения `/dev/input/event*` (`crates/dicta-core/src/hotkey.rs:106-128`) — демон получает поток **всех** клавиш и джойстиков системы, а не только своей комбинации.
- Files: `crates/dicta-core/src/hotkey.rs`, udev-подсказка `crates/dictad/src/main.rs:1284-1289`
- Current mitigation: по умолчанию `hotkey = "off"` (`crates/dicta-core/src/config.rs:90`); README предлагает клавишу DE.
- Recommendations: держать по умолчанию `off`; в SECURITY.md явно описать, что даёт группа `input`.

**Скачивание моделей без проверки целостности:**
- Risk: `dicta-settings` тянет `.bin` с Hugging Face через `curl -L` без контрольных сумм (`crates/dicta-settings/src/main.rs:241, 281`); повреждённый/подменённый файл скармливается whisper.cpp.
- Files: `crates/dicta-settings/src/main.rs`, `crates/dicta-settings/src/hf.rs`
- Current mitigation: whisper.cpp валидирует magic/заголовок модели (битый файл не загрузится), но злонамеренно собранный ggml-файл — доверенная кодогенерация на GPU.
- Recommendations: сверять `lfs.sha256` из ответа дерева (`hf.rs` уже парсит `lfs`, но sha256 отбрасывает) после загрузки.

## Performance Bottlenecks

**Перечитывание всего WAV каждые 80 мс:**
- Problem: `Recorder::level()` делает `std::fs::read` всего временного WAV-файла ради последних 1280 байт (`crates/dicta-core/src/capture.rs:79-93`); watchdog дёргает его каждые 80 мс (`crates/dictad/src/main.rs:215`). К 60-й секунде записи (`max_record_ms`) файл — 1.9 МБ, итого ~24 МБ/с лишнего I/O на каждый тик.
- Files: `crates/dicta-core/src/capture.rs:79-93`, `crates/dictad/src/main.rs:216-270`
- Cause: pw-record пишет в файл, уровня в памяти нет.
- Improvement path: читать хвост файла через `seek` (`File::seek(End - 1280)`), либо pipe-режим pw-record с кольцевым буфером в памяти вместо temp-файла.

**`with_keyboard` спит под глобальным мьютексом:**
- Problem: `KEYBOARD: Mutex` держится во время `sleep(15 мс)` после каждой отправки и `sleep(600 мс)` при пересоздании девайса (`crates/dicta-core/src/inject.rs:133-163`); параллельная вставка (воркер stop + команда `inject`) выстроится в очередь на секунду.
- Files: `crates/dicta-core/src/inject.rs:120, 133-163`
- Cause: мьютекс захвачен на всё действие, включая ожидание.
- Improvement path: держать `Keyboard` в `OnceLock`/без мьютекса (устройство не требует синхронизации — fd один), пересоздание делать в отдельном потоке.

**Каждый опрос статуса — новое соединение:**
- Problem: `dicta-ui` открывает Unix-сокет заново каждые 100 мс (`ask_status` → `ask_command` → `UnixStream::connect`, `crates/dicta-ui/src/main.rs:1914-1936`), демон порождает поток на соединение (`crates/dictad/src/main.rs:330-346`) и закрывает его после первого `status` (`main.rs:1007-1009`).
- Files: `crates/dicta-ui/src/main.rs:1914-1936`, `crates/dictad/src/main.rs:330-346`
- Cause: протокол спроектирован как request-response без постоянного соединения.
- Improvement path: 10 соединений/с по Unix-сокету дёшевы; при желании — постоянное соединение с push-ом статуса демоном.

**Блокировка мьютекса handler на время вставки через сокет-команду:**
- Problem: ветка `"inject"` держит мьютекс `Handler` на всё время `insert_text` (`crates/dictad/src/main.rs:878-934`) — включая ожидание буфера (до `paste_delay_ms`), uinput-отправку (~35 мс) и AT-SPI с дедлайном **8 с** (`crates/dicta-core/src/atspi.rs:529`). Все прочие команды (status/set/panel) стоят в очереди.
- Files: `crates/dictad/src/main.rs:878-934`, `crates/dicta-core/src/atspi.rs:529`
- Cause: путь вставки не вынесен в воркер, как это сделано для stop.
- Improvement path: зеркалить паттерн `run_stop`: забрать из-под мьютекса только снапшот cfg и уйти в отдельный поток; `h.last_app`/`last_text` записывать по завершении.

**Смена модели перезагружает whisper целиком:**
- Problem: каждый клик «модель» в панели оверлея (`crates/dicta-ui/src/panel.rs:354-370`) — полная выгрузка/загрузка ggml-файла в VRAM через `schedule_engine_reload` (`crates/dictad/src/main.rs:744-783`); «пролистать» 12 моделей — 12 перезагрузок.
- Files: `crates/dicta-ui/src/panel.rs`, `crates/dictad/src/main.rs:744`
- Cause: циклический UI поверх тяжёлой операции.
- Improvement path: в панели не менять модель вовсе (оставить окну настроек); в `dicta-settings` показывать предупреждение о перезагрузке до клика.

## Fragile Areas

**Unsafe-код uinput без тестов:**
- Files: `crates/dicta-core/src/inject.rs:14-66` (ручные IOCTL-константы и `#[repr(C)]` структуры), `inject.rs:165-305`
- Why fragile: размеры структур и номера ioctl захардкожены (`iow`-вычисления), запись события — сырой `libc::write`; изменение ABI uinput или ошибка в `InputEvent` проявляется как тихая потеря вставки.
- Safe modification: любые правки сопровождать `dictactl test-keys ctrl+v` на живой машине; не менять `#[repr(C)]` поля без сверки с `linux/uinput.h`.
- Test coverage: нет ни одного теста на `inject.rs` (проверяется только вручную).

**Протокольные танцы layer-shell в dicta-ui:**
- Files: `crates/dicta-ui/src/main.rs:1513-1602` (`draw_hint` — три ветки сокрытия с разными workaround'ами KWin/wlroots), `main.rs:1608-1710` (`draw_orb`), `main.rs:1199-1234` (`closed` — различение четырёх поверхностей по id)
- Why fragile: поведение отличается между композиторами (комментарии фиксируют: «KWin закрывал анмапнутый орб», «wlroots присылают configure на каждый commit»); каждое новое окно (пилюля, панель, ореол, орб) добавляет состояние `*_configured`/`*_mapped`, рассинхрон которых = protocol error и разрыв соединения.
- Safe modification: менять по одному surface за раз, проверять `DICTA_LAYER`/`DICTA_ANIM=0` на KWin и wlroots; тест `hidden_and_overshooting_frames_survive_on_every_side` (`main.rs:2259`) — единственная автоматическая защита рендера.
- Test coverage: рендер покрыт BMP-тестами; event-часть — только вручную.

**Граф доверия paste-клавиш:**
- Files: `crates/dicta-core/src/focus.rs:63-71` (`active_app`: подсказка адаптера → `DICTA_APP` → файл кэша), `crates/dicta-core/src/config.rs:107-138` (`default_paste_keys_by_app`), `crates/dictad/src/main.rs:1156-1160` (выбор combo), `crates/dictactl/src/main.rs:254-257` (`--app` из CLI)
- Why fragile: выбор клавиш вставки — цепочка из четырёх источников разной свежести, и ошибка на любом уровне вставляет **не те клавиши в чужое окно**: (а) кэш `~/.cache/dicta/focus` не имеет временной метки — протухший фокус подставит терминальные `ctrl+shift+v` в браузер и наоборот; (б) `normalize` (`focus.rs:53-60`) превращает «Mozilla Firefox» в «mozilla firefox» (с пробелом) — такой ключ в `paste_keys_by_app` никогда не совпадёт с дефолтным «firefox»; (в) при неподдерживаемой комбинации `Combo::parse(...).unwrap_or(paste_combo)` (`dictad/src/main.rs:1160`) молча подменяет на дефолт — пользователь не узнает, что его конфиг для этого окна битый.
- Safe modification: новые приложения добавлять в `paste_keys_by_app` только в нормализованном виде (тест `normalized` в `focus.rs:87` показывает правила); после правок запускать `dictactl paste-keys`.
- Test coverage: `paste_keys_for` покрыт тестом (`config.rs:307`), но цепочка «подсказка → кэш → выбор» в демоне не тестируется.

**Восстановление буфера — одна отложенная работа на процесс:**
- Files: `crates/dicta-core/src/inject.rs:343-383` (`PendingRestore`, `schedule_restore`)
- Why fragile: инвариант «первый prev и есть исходный буфер» верен только пока все вставки идут подряд без паузы > `clipboard_restore_ms`; если восстановление успело сработать, а потом пришла вторая вставка — она сохранит уже-восстановленный буфер (корректно), но если между ними пользователь скопировал своё — восстановление затрёт это по дедлайну, не зная о новом копировании (оконная гонка clipboard-менеджеров).
- Safe modification: не менять семантику продления дедлайна (`inject.rs:359-382`) без живого теста: две вставки подряд, пауза 2.6 с, проверка буфера.
- Test coverage: нет; логика воспроизводится только вручную (секция «Восстановление — в фоне», `inject.rs:407`).

**Кэш покоя панели (`panel_rest_sig`):**
- Files: `crates/dicta-ui/src/main.rs:1419-1435, 1489-1491`
- Why fragile: перерисовка панели подавляется по строковой подписи кадра (состояние|notice|подписи кнопок); любое новое меняющееся поле (таймер в заголовке при записи!) нужно внести в подпись, иначе панель «зависнет» со старым кадром.
- Safe modification: добавляя динамический элемент в панель — добавлять его в `sig`.
- Test coverage: частично (тесты рисуют кадры напрямую, минуя кэш).

**Restart-петля dicta-ui без роста паузы:**
- Files: `crates/dictad/src/main.rs:1242-1250` (`spawn_ui`: `loop { spawn; wait; sleep(2s) }`)
- Why fragile: если dicta-ui падает на старте (нет Wayland-сессии, нет layer shell), демон перезапускает его каждые 2 с бесконечно — лог journal забивается, CPU тратится; в отличие от hotkey-потока, где backoff растёт до 900 с (`main.rs:299-319`).
- Safe modification: при правках `spawn_ui` сохранять retry-петлю; отдельно — добавить экспоненциальную паузу.
- Test coverage: нет.

## Scaling Limits

**Одна модель в памяти, смена только в Idle:**
- Current capacity: один ggml-словарь в VRAM на процесс (`crates/dicta-core/src/stt.rs:37-39`), смена модели запрещена во время записи/распознавания (`crates/dictad/src/main.rs:750-754`).
- Limit: языковая модель не переключается на лету (language — та же модель, reload не нужен; но model — нужен); параллельные диктовки двух пользователей — нет по построению (один демон на сессию).
- Scaling path: кэш двух моделей в VRAM (бюджет 610M iGPU мал); сейчас честно задокументировано в `stt.rs:33-36`.

**Watchdog-уровни: `Vec::insert(0)` с `truncate(2048)`:**
- Current capacity: 2048 тиков (~164 с истории при 80 мс), вставка в голову — O(n) на тик (`crates/dictad/src/main.rs:236-237`).
- Limit: на практике незаметно; заметно только если TICK_MS уменьшат на порядок.
- Scaling path: `VecDeque` при следующей правке watchdog-а.

## Dependencies at Risk

**whisper-rs 0.16 + whisper-rs-sys 0.15:**
- Risk: wrapper и -sys разных минорных версий заданы вручную в workspace (`Cargo.toml`, `[workspace.dependencies]`); обновление whisper.cpp меняет поведение логов (`stt.rs:15-30` глушит лог по уровням ggml — при смене enum'а сломается тихо).
- Impact: сборка может взять несовместимые crate-версии при обновлении `whisper-rs` без синхронного `whisper-rs-sys`.
- Migration plan: держать версии в паре в одном PR; после обновления проверять `quiet_log` (в stderr не должен появиться шум whisper).

**Внешние бинарники как жёсткие зависимости рантайма:**
- Risk: `pw-record` (`crates/dicta-core/src/capture.rs:28`), `wl-copy`/`wl-paste` (`crates/dicta-core/src/inject.rs:333, 393, 398`), `curl` (`crates/dicta-settings/src/main.rs:241, 281`) вызываются как процессы без версионных требований; исчезновение/переименование утилиты в дистрибутиве ломает путь вставки.
- Impact: ошибка проявляется только в рантайме («не удалось запустить pw-record»).
- Migration plan: ошибки уже снабжены подсказками (например `capture.rs:40`); добавить проверку наличия утилит в `dictactl status`/docs.

## Missing Critical Features

**Отмена распознавания:**
- Problem: нет способа прервать начатый inference (`crates/dictad/src/main.rs:1073-1192`); `cancel` действует только до stop.
- Blocks: быстрый отказ от ошибочно запущенной длинной диктовки (в момент inference «Esc» ничего не делает).

**Проверка целостности скачанных моделей:**
- Problem: sha256 доступен в API HF (`crates/dicta-settings/src/hf.rs:266`), но не сверяется после `curl`.
- Blocks: безопасную установку моделей без ручной сверки.

## Test Coverage Gaps

**Путь вставки (`inject.rs`) — 0% покрытия:**
- What's not tested: `Combo::parse`, `Keyboard::new/send`, `paste_text`, `schedule_restore`, `wait_clipboard_ready` — весь unsafe-слой и логика буфера.
- Files: `crates/dicta-core/src/inject.rs`
- Risk: регресс в самой пользовательски-видимой операции (вставка) не ловится `cargo test`; ловится только руками (`tools/insert-test.sh`, `dictactl test-keys`).
- Priority: High. Минимум: юнит-тесты `Combo::parse` (чистая функция) и `schedule_restore`-переходов при выдуманных часах.

**Протокол сокета демона:**
- What's not tested: `serve_conn` — разбор команд, формат ответов, удержание соединения stop (`crates/dictad/src/main.rs:810-1012`); клиенты парсят ответ руками, и изменение поля ломает их молча.
- Files: `crates/dictad/src/main.rs:810-1012`, `crates/dictactl/src/main.rs:111-152`, `crates/dicta-ui/src/main.rs:1914-1965`
- Risk: несовместимость демон/клиент при обновлении (частично смягчена `#[serde(default)]` везде).
- Priority: Medium. Интеграционный тест: поднять `serve_conn` на temp-сокете, проверить round-trip статуса и set.

**Гонки состояния:**
- What's not tested: двойной stop, cancel-during-transcribing, start-during-transcribing — ничего из описанного в «Known Bugs» не воспроизводится тестом (потоковая структура `main.rs` не выделена в тестируемые функции).
- Files: `crates/dictad/src/main.rs:810-1225`
- Risk: любое «безобидное» переупорядочивание атомик может открыть/закрыть гонку незаметно.
- Priority: High (совместно с CAS-фикксом).

**`hotkey.rs`, `capture.rs`, `stt.rs`:**
- What's not tested: разбор evdev-событий и `keycode`/`parse_combo` (чистые функции, легко покрыть), поведение SIGINT-остановки pw-record, настройки whisper-параметров.
- Files: `crates/dicta-core/src/hotkey.rs`, `crates/dicta-core/src/capture.rs`, `crates/dicta-core/src/stt.rs`
- Risk: средняя; `keycode`-таблица — чистые данные, регресс ловится дешёвым тестом.
- Priority: Low (hotkey/capture), Medium (`keycode`-маппинг).

---

*Concerns audit: 2026-11-22*

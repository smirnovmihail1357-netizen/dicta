# Changelog

## 0.3.0 — авто-остановка записи (VAD) и вставка через AT-SPI2

### Added
- **Авто-остановка записи (VAD)**: `audio::silence_streak_ms()` — длина серии подряд
  субпороговых замеров уровня. Монитор уровня в `dictad` теперь одновременно watchdog:
  держит историю уровней, считает тишину и сам вызывает остановку, помня `last_app`,
  чтобы вставка ушла в то же окно, что начало запись.
- Конфиг авто-остановки: `auto_stop = true`, `auto_stop_level = 0.22`,
  `auto_stop_after_ms = 1500`, `auto_stop_grace_ms = 1200`, `max_record_ms = 60000`.
  Замерено вживую: запись в тишине закрылась сама через ~1.5 с; при проигрывании аудио
  запись держалась всё время звука и закрылась через ~1 с после его окончания.
- **Вставка через AT-SPI2** (`crates/dicta-core/src/atspi.rs`, `inject_mode = "atspi"`):
  поиск фокусного объекта в дереве доступности и запись через `org.a11y.atspi.EditableText`
  (`insert_text` в позицию caret, иначе `set_text_contents`). Ни буфера обмена, ни
  синтезированных клавиш.
- Cargo-фича `atspi` (по умолчанию выключена):
  `cargo build --release --features dictad/atspi,dictactl/atspi`.
- `dictactl atspi-insert <текст>` — проверка AT-SPI-вставки напрямую, минуя демон и
  `inject_mode`: видно и куда попало слово, и почему не попало.
- `dictactl atspi-dump [--app <окно>] [--depth N] [--tree]`: какие приложения отдают AT-SPI,
  где у них фокус, какие объекты действительно реализуют `EditableText`, и тайминги этапов.
- Дедлайны на AT-SPI: отдельный поток + `recv_timeout` (8 с на вставку, 20 с на дамп) —
  зависшее приложение не вешает демон.
- `atspi_allow_any_role` (default false): разрешает писать в фокусные редактируемые узлы
  нестандартной роли — только для диагностики.

### Fixed
- **Поиск приложений в AT-SPI был неверным**: `DBusProxy::list_names()` отдаёт только
  well-known имена, а приложения в a11y-бусе живут под уникальными именами соединений
  (`:1.13` = Firefox) и перечислены детьми корневого объекта реестра. Отсюда ложный вывод
  «в бусе только реестр, вставлять некуда». Теперь список берётся из
  `org.a11y.atspi.Accessible.GetChildren` на `/org/a11y/atspi/accessible/root`.
- **Скорость обхода**: последовательный обход дерева (сотни D-Bus round trip'ов) упирался в
  дедлайн за 12 с. Обход по уровням с 12 параллельными запросами: 389 объектов / 396
  D-Bus-вызовов дерева Firefox за ~60 мс, весь дамп — 85 мс.
- Фильтр по роли фокусного объекта (`entry`, `password text`, `combo box`, `text`): Firefox
  помечает `focused` узел `document web`, и тот объявляет `EditableText` — писать в документ
  вместо поля ввода нельзя.
- `dictactl atspi-dump` больше не зависает навсегда: дамп честно отрабатывает дедлайн.

## 0.2.0 — контейнер, CI, публичный релиз

### Added
- `dictactl inject <текст>` + команда сокета `inject`: проверка вставки в активное окно без
  микрофона и распознавания; опция `--keys` для принудительной комбинации.
- `dictactl calibrate --secs N`: замер пика речи / среднего / уровня первых 2 с — для подбора
  `min_level`.
- `audio::speech_level()`: индикатор по максимуму окон 300 мс вместо среднего RMS.
- `filter::looks_repetitive()` + русские артефакты whisper («по-русски», «subtitles by»,
  «подписал(ись) на канал»).
- `capture::record_samples()`; `dictactl --app <окно>` для прямого выбора окна.
- `Config::clipboard_restore_ms` (2500) и `paste_delay_ms` (150).
- Терминалы по умолчанию на `ctrl+shift+v` (Konsole/foot/alacritty/kitty/wezterm/ghostty),
  `shift+insert` остался у xterm/urxvt/st.
- Постоянная uinput-клавиатура (`inject::warm_up_keyboard()` при старте демона) с пересозданием
  при сбое.
- Контейнер: `docker/Dockerfile` (multi-stage, Vulkan/CPU), `docker/entrypoint.sh`,
  `docker/run-host.sh`, `compose.yaml`, `.env.example`, `docs/CONTAINERS.md`.
- `tools/insert-test.sh` (сквозная проверка вставки), `tools/fetch-model.sh`, `Makefile`.
- Cargo-фича `vulkan` (default) -> `cargo build --release --no-default-features` = CPU-сборка.
- CI (`build + tests`, Vulkan и CPU), релизный workflow (архивы + образ ghcr), LICENSE,
  CONTRIBUTING, SECURITY, шаблоны issues.

### Fixed
- **Вставка не работала**: `wl-copy --type text` объявлял MIME ровно `text`, который не
  принимает ни Konsole, ни Firefox. Теперь `--type text/plain`.
- **Клавиши не доходили до окон**: uinput-девайс создавался на ~350 мс и уничтожался раньше,
  чем libinput/KWin успевали его зарегистрировать.
- **Оверлей не появлялся**: в `idle` коммит шёл без буфера -> surface никогда не становился
  mapped -> frame callbacks не приходили -> `draw()` не вызывался. Теперь пустой буфер
  присоединяется и в `idle`.
- «Рябь» оверлея: точка REC больше не мигает, волна уровня рисуется по истории ~1.8 с вместо
  покадрового синусоидального дрожания.
- `wav_header` panic (8-байтный `to_le_bytes` в 4-байтный слот) в `dictactl record`.
- Гейтер тишины по среднему RMS резал реальную диктовку: теперь речевой пик, порог 0.35.
- `StartLimitIntervalSec=0` в oneshot-юнитах клавиш (клавиша «перставала работать» после
  нескольких нажатий).

## 0.1.0 — базовая версия

Демон (`dictad`), CLI (`dictactl`), оверлей (`dicta-ui`), запись через PipeWire, распознавание
whisper-rs (Vulkan/CPU), фильтр галлюцинаций, вставка через `wl-copy` + uinput с подбором клавиш
по фокусному окну, адаптеры горячих клавиш (KWin/Hyprland/Sway/Quickshell/GNOME), systemd-юниты.

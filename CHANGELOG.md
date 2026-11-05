# Changelog

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

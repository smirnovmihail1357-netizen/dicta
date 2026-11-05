# dicta

Системный голосовой ввод для Wayland/Linux: один демон вместо отдельной интеграции в каждое приложение.

`Alt+V` (или `dictactl toggle`) -> говоришь -> `Alt+V` ещё раз -> текст появляется там, где курсор:
Kate, Konsole, Firefox, Discord, чат в браузере, промпт pi/opencode.

Остановка — это **тот же Alt+V** (или клик по пилюле, или `dictactl stop`), не Enter.
Отмена без распознавания — `dictactl cancel` (клавишу для него назначить самому:
`DICTA_CANCEL_KEY='Meta+Shift+Escape' ./adapters/kwin/install.sh`).

## Состав

| crate | что делает |
|---|---|
| `dicta-core` | конфиг, запись (PipeWire), распознавание (whisper-rs, Vulkan/HIP/CPU), фильтр галлюцинаций |
| `dictad` | демон: Unix-сокет, запись, инференс, вставка текста в фокусное окно |
| `dictactl` | CLI: `toggle`, `start`, `stop`, `cancel`, `status`, `paste-keys`, `calibrate`, `inject <текст>`, `infer file.wav` (диагностика) |
| `adapters/` | горячие клавиши: KWin-скрипт (Plasma 6), Hyprland, sway, Quickshell, GNOME; все зовут `dictactl` |
| `systemd/` | user-сервис `dictad.service` + oneshot-юниты для клавиш DE |

## Статус

- [x] `dicta-core`: конфиг (TOML), WAV-чтение, whisper-rs (Vulkan-бэкенд собирается), фильтр галлюцинаций, тихий лог ggml/whisper
- [x] `dictactl infer --timing` — распознавание файла (проверено: 4.2 с русской речи -> 2.2 с на CPU 8 потоков, small-q5_1)
- [x] запись с микрофона (`pw-record` 16 kHz mono, SIGINT-остановка, RMS-уровень)
- [x] Unix-сокет + `dictactl start/stop/toggle/cancel/status` (проверено: status/start/level/cancel)
- [x] вставка текста: wl-copy + uinput-клавиши, режимы `paste` / `clipboard` / `stdout`;
      `dictactl inject "текст"` гоняет её без микрофона (проверено: `shift+insert` в konsole, `ctrl+v` в firefox)
- [x] layer-shell оверлей `dicta-ui` (пилюля 520x34 внизу по центру; проверено скриншотами KWin:
      idle — полоса mean=0.232, запись — mean=0.109 (пилюля видна), после stop — снова прозрачно).
      Важно: surface Wayland становится mapped только на commit с присоединённым буфером, поэтому
      и в idle attachается пустой буфер — иначе композитор не присылает frame callback и оверлей
      не появляется вообще (коммит «пустого» без буфера это и ломал)
- [x] горячие клавиши: `adapters/` (KWin-скрипт `registerShortcut` + `callDBus` -> systemd-юнит, Hyprland, sway, Quickshell, GNOME) и встроенный evdev-слушатель в демоне
      (проверено на живом Plasma 6: скрипт регистрирует `dicta-toggle`/`dicta-cancel`, цепочка
      «скрипт -> systemd1 -> юнит -> dictactl» отработала; evdev требует группу `input`)
- [x] `systemd/` — user-сервис демона + oneshot-юниты `dicta-toggle`/`dicta-cancel`
      (+ шаблонные `dicta-toggle@<окно>.service`), `dictad --install-autostart` для `.desktop`
- [x] выбор клавиш вставки по фокусному окну: `paste_keys_by_app`, `dictactl --app <окно>`,
      `dictactl paste-keys` (проверено: konsole -> shift+insert, firefox -> ctrl+v)
- [x] живая проверка: `Alt+V` на Plasma 6 поднимает `dicta-toggle@<окно>.service`, демон пишет
      `dictad: "текст" (записано 3.1s, распознано за 1.2s, вставка shift+insert в Some("org.kde.konsole"))`;
      гейтер тишины подобран (`min_level = 0.35`, speech_level по окнам 300 мс), артефакты шума
      («По-русски, по-русски, по-русски.») вырезает `filter::looks_repetitive`
- [ ] авто-остановка по тишине (VAD) — сейчас стоп только по повторному Alt+V/клику/`dictactl stop`
- [ ] AT-SPI2-вставка (zbus) как точный путь без буфера обмена

## Сборка

```bash
cd ~/projects/dicta && cargo build          # debug
cargo build --release                      # whisper.cpp собирается один раз, потом кэш
```

Нужны: `cmake`, `ninja`, `clang`, `glslc`/`shaderc`, `vulkan-headers` (для `-F vulkan`) — все уже стоят.

## Проверка распознавания

```bash
dictactl infer --timing ~/.local/share/opencode-dictation/demo/audio/ru.wav
```

На CPU (не трогает VRAM, безопасно при занятом GPU):

```bash
dictactl --config /tmp/dicta-cpu.toml infer --timing demo.wav
# в конфиге: use_gpu = false, model = ".../ggml-small-q5_1.bin"
```

## Диагностика и подбор порога

```bash
dictactl status                     # состояние и уровень
dictactl calibrate --secs 8         # 8 с говоришь: печатает пик речи, среднее и уровень первых 2 с
dictactl inject "проверка вставки"  # вставляет текст в активное окно без микрофона
dictactl inject --app konsole "…"   # принудительно Shift+Insert (тест терминала)
journalctl --user -u dictad.service -f   # что именно демон сделал и почему пропустил
```

Если диктовка постоянно «пропускается» — подними речь ближе к микрофону или убавь `min_level`
в `~/.config/dicta/config.toml` (0.35 по умолчанию; 0.25 = чувствительнее, 0.6 = агрессивнее).

## Оверлей

```bash
dicta-ui                 # рисует пилюлю внизу экрана, когда демон записывает/распознаёт
dicta-ui --probe         # проверить, понимает ли композитор zwlr_layer_shell_v1
```

Оверлей не перехватывает клавиатуру (`KeyboardInteractivity::None`) и не резервирует место
(`exclusive_zone = 0`); клик мышью по пилюле = «закончить и вставить». Когда демон в `idle`,
кадр полностью прозрачный.

Экспериментальные переменные (для подбора под конкретный композитор): `DICTA_LAYER`
(`overlay|top|bottom|background`), `DICTA_EXCLUSIVE`, `DICTA_DEBUG`, `DICTA_ALWAYS_DRAW`, `DICTA_FLASH`.

## Горячая клавиша

Полная инструкция — в [`adapters/README.md`](adapters/README.md). Варианты:

| Композитор | как |
|---|---|
| KDE Plasma 6 | `./systemd/install.sh && ./adapters/kwin/install.sh` (скрипт `dicta-hotkey`, `Alt+V` — Meta+V занят историей буфера обмена Plasma 6) или System Settings → Keyboard → Custom Shortcuts → `systemctl --user start dicta-toggle.service` |
| Hyprland | `source = ~/projects/dicta/adapters/hyprland/dicta.conf` |
| Sway | `include ~/projects/dicta/adapters/sway/dicta.conf` |
| GNOME | `./adapters/gnome/apply.sh '<Ctrl><Alt>v'` |
| любой DE/без DE | встроенный evdev-слушатель демона: `hotkey = "ctrl+alt+v"` + доступ к `/dev/input/event*` (группа `input` или udev-ACL) |

Демон сам стартует оверлей по `--ui` (`dictad --ui`), а `dictad --install-autostart` кладёт
`.desktop` в autostart и печатает подсказку по udev.

## Клавиши вставки под конкретное окно

Демон не умеет сам спрашивать у композитора фокусное окно (у KWin `queryWindowInfo`
интерактивный), поэтому окно передаёт адаптер: `dictactl toggle --app konsole`
(KWin-скрипт делает это сам через шаблонный юнит `dicta-toggle@konsole.service`).
Диагностика: `dictactl paste-keys --app org.kde.konsole`.

## Vulkan-устройства

В Rust-сборке перечисление Vulkan-устройств ggml такое (проверено логом):
`0 = AMD Radeon 610M (iGPU)`, `1 = AMD Radeon RX 7900 XTX`.
То есть в `~/.config/dicta/config.toml` для 7900 XTX нужно `gpu_device = 1`, либо внешний
`GGML_VK_VISIBLE_DEVICES=1`. Нумерация может отличаться от той, что видит opencode-плагин.

## VRAM

Модель держится в VRAM одним экземпляром. На этой машине GPU0 занято ~25.0/25.7 GiB
(модель llama-swap + whisper), поэтому второй экземпляр модели не допускается: демон
один, модель одна, приложения только клиенты.

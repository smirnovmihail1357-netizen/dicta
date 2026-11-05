# dicta

Системный голосовой ввод для Wayland/Linux: один демон вместо отдельной интеграции в каждое приложение.

`Alt/Meta+V` (или `dictactl start`) → говоришь → `Enter` → текст появляется в том месте, где сейчас курсор:
Kate, Konsole, Firefox, Discord, чат в браузере, промпт pi/opencode.

## Состав

| crate | что делает |
|---|---|
| `dicta-core` | конфиг, запись (PipeWire), распознавание (whisper-rs, Vulkan/HIP/CPU), фильтр галлюцинаций |
| `dictad` | демон: Unix-сокет, запись, инференс, вставка текста в фокусное окно |
| `dictactl` | CLI: `toggle`, `start`, `stop`, `status`, `infer file.wav` (диагностика) |
| `adapters/` | горячие клавиши: KWin/D-Bus, Hyprland, sway, Quickshell; все зовут `dictactl` |

## Статус

- [x] `dicta-core`: конфиг (TOML), WAV-чтение, whisper-rs (Vulkan-бэкенд собирается), фильтр галлюцинаций, тихий лог ggml/whisper
- [x] `dictactl infer --timing` — распознавание файла (проверено: 4.2 с русской речи -> 2.2 с на CPU 8 потоков, small-q5_1)
- [x] запись с микрофона (`pw-record` 16 kHz mono, SIGINT-остановка, RMS-уровень)
- [x] Unix-сокет + `dictactl start/stop/toggle/cancel/status` (проверено: status/start/level/cancel)
- [x] вставка текста: wl-copy + uinput-клавиши, режимы `paste` / `clipboard` / `stdout` (на живых окнах ещё не проверялась)
- [x] layer-shell оверлей `dicta-ui` (KWin проверялся: `zwlr_layer_shell_v1` есть, пилюля 520x34 рисуется и видна на снимке экрана)
- [ ] горячие клавиши по композиторам (KWin D-Bus, Hyprland bind, sway bindsym, Quickshell)
- [ ] выбор клавиш вставки по классу фокусного окна (Konsole -> Shift+Insert, Kate/Firefox -> Ctrl+V)
- [ ] AT-SPI2-вставка (zbus) как точный путь без буфера обмена
- [ ] systemd user-сервис + autostart

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

Где-то должен быть запуск `dictactl toggle`. Готовые сниппеты:

| Композитор | как |
|---|---|
| KDE Plasma | System Settings → Shortcuts → Custom Shortcuts → Command/URL → `dictactl toggle`, ключ `Meta+V` |
| Hyprland | `bind = META, V, exec, dictactl toggle` |
| Sway | `bindsym $mod+v exec dictactl toggle` |
| GNOME | `gsettings set org.gnome.settings-daemon.plugins.media-keys custom-keybindings "['/org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/custom0/']"` + name/command |
| любой (нужен доступ к /dev/input/event*) | встроенный evdev-слушатель демона (нужен udev-правкой доступ) |

## Vulkan-устройства

В Rust-сборке перечисление Vulkan-устройств ggml такое (проверено логом):
`0 = AMD Radeon 610M (iGPU)`, `1 = AMD Radeon RX 7900 XTX`.
То есть в `~/.config/dicta/config.toml` для 7900 XTX нужно `gpu_device = 1`, либо внешний
`GGML_VK_VISIBLE_DEVICES=1`. Нумерация может отличаться от той, что видит opencode-плагин.

## VRAM

Модель держится в VRAM одним экземпляром. На этой машине GPU0 занято ~25.0/25.7 GiB
(модель llama-swap + whisper), поэтому второй экземпляр модели не допускается: демон
один, модель одна, приложения только клиенты.

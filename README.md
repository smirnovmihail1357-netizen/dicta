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

- [x] `dicta-core`: конфиг, WAV-чтение, whisper-rs (Vulkan), фильтр галлюцинаций
- [x] `dictactl infer` — распознавание файла (проверка модели/качества)
- [ ] запись с микрофона (`pw-record`) в демоне
- [ ] Unix-сокет и `dictactl toggle/start/stop/status`
- [ ] вставка текста: wl-copy + uinput-клавиши (Ctrl+V / Ctrl+Shift+V / Shift+Insert по классу окна), AT-SPI как точный путь
- [ ] layer-shell оверлей (`● REC`, таймер, волна) / Quickshell-версия для wlroots
- [ ] горячие клавиши по композиторам
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

## VRAM

Модель держится в VRAM одним экземпляром. На этой машине GPU0 занято ~25.0/25.7 GiB
(модель llama-swap + whisper), поэтому второй экземпляр модели не допускается: демон
один, модель одна, приложения только клиенты.

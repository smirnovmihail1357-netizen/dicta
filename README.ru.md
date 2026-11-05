# dicta (Русский)

Системный голосовой ввод для Wayland/Linux: один демон вместо отдельной интеграции в каждое
приложение.

`Alt+V` (или `dictactl toggle`) -> говоришь -> `Alt+V` ещё раз -> текст появляется там, где
курсор: Kate, Konsole, Firefox, Discord, чат в браузере, промпт pi/opencode.

Остановка — это **тот же Alt+V** (или клик по пилюле, или `dictactl stop`), не Enter.
Отмена без распознавания — `dictactl cancel` (клавишу для него назначить самому:
`DICTA_CANCEL_KEY='Meta+Shift+Escape' ./adapters/kwin/install.sh`).

## Состав

| crate | что делает |
|---|---|
| `dicta-core` | конфиг (TOML), запись (PipeWire), распознавание (whisper-rs: Vulkan/CPU), фильтр галлюцинаций, вставка (wl-copy + uinput), клавиши по окнам |
| `dictad` | демон: Unix-сокет, запись, инференс, вставка текста в фокусное окно, порождает оверлей |
| `dictactl` | CLI: `toggle`, `start`, `stop`, `cancel`, `status`, `paste-keys`, `calibrate`, `inject <текст>`, `infer file.wav`, `atspi-dump`, `atspi-insert <текст>` |
| `dicta-ui` | layer-shell оверлей: пилюля REC + волна уровня + анимация распознавания |
| `adapters/` | горячие клавиши: KWin-скрипт (Plasma 6), Hyprland, sway, Quickshell, GNOME |
| `systemd/` | user-сервис `dictad.service` + oneshot-юниты, которые дёргает DE |

Цепочка: горячая клавиша DE -> oneshot-юнит systemd -> `dictactl toggle --app <окно>` -> демон
(`pw-record` -> whisper -> фильтр) -> `wl-copy` + синтез клавиш через `/dev/uinput`.

## Установка на хост

```bash
git clone https://github.com/<you>/dicta.git && cd dicta
make build                      # release-сборка с Vulkan
make install                    # -> ~/.local/bin/{dictad,dictactl,dicta-ui}
./tools/fetch-model.sh          # ggml-модель в ~/.local/share/whisper-cpp/
mkdir -p ~/.config/dicta && cp config.example.toml ~/.config/dicta/config.toml
./systemd/install.sh            # dictad.service + юниты toggle/cancel
./adapters/kwin/install.sh      # Plasma 6: Alt+V через KWin-скрипт (нужен выход/вход в сессию)
systemctl --user start dictad.service
```

Дальше: `Alt+V`, говорить 3–4 с, `Alt+V`. Что именно произошло — видно в журнале:
`journalctl --user -u dictad.service -f`.

Сборка без Vulkan (не нужен glslangValidator; в конфиге `use_gpu = false`):
`make build-cpu`.

## Диагностика и подбор порога

```bash
dictactl status                      # состояние и уровень
dictactl calibrate --secs 8          # 8 с говоришь: пик речи, среднее, уровень первых 2 с
dictactl inject "проверка вставки"   # вставляет текст без микрофона
dictactl inject --app konsole "…"    # принудительно клавиши терминала
dictactl atspi-dump --depth 8          # AT-SPI: приложения, роли, фокус, тайминги этапов
dictactl atspi-insert "текст"          # проверка AT-SPI-вставки напрямую, мимо демона
./tools/insert-test.sh               # сквозная проверка вставки в отдельное окно
journalctl --user -u dictad.service -f
```

Если диктовка «пропускается» — крути `min_level` (по умолчанию 0.35; 0.25 чувствительнее,
0.6 строже). Гейтер считает не среднее по всему аудио, а максимум по окнам 300 мс.

Запись заканчивается сама, когда в микрофоне тихо (`auto_stop = true` по умолчанию): монитор
уровня работает ещё и watchdog'ом и закрывает запись через `auto_stop_after_ms` тишины, с
передышкой `auto_stop_grace_ms` в начале и жёстким потолком `max_record_ms`.

## Вставка без буфера обмена (`inject_mode = "atspi"`)

Режим `paste` пишет буфер обмена и нажимает клавиши. Режим `atspi` пишет прямо в фокусный
виджет через AT-SPI2 (`org.a11y.atspi.EditableText`) — буфер обмена не трогается, клавиши не
синтезируются. Сборка с фичей:

```bash
cargo build --release --features dictad/atspi,dictactl/atspi
```

Поддерживает ли нужное окно такой путь — проверяется `dictactl atspi-dump`, ничего не
вставляя:

```bash
dictactl atspi-dump --depth 8          # приложения, роли, фокусные объекты, тайминги этапов
dictactl atspi-dump --app firefox --tree
```

Писать демон разрешает только объектам с ролью поля ввода (`entry`, `password text`,
`combo box`, `text`). Это важно: Firefox помечает `focused` узел `document web` (и он правда
реализует `EditableText`) — а это не «фокусное поле». `atspi_allow_any_role = true` имеет
смысл только при диагностике такого окна. Поддержка AT-SPI разная по тулкитам: Gecko/GTK —
да, Qt требует `QT_ACCESSIBILITY=1`, Chromium/Electron — `--force-renderer-accessibility`.

## Горячая клавиша

Полная инструкция — в [`adapters/README.md`](adapters/README.md).

| Композитор | как |
|---|---|
| KDE Plasma 6 | `./systemd/install.sh && ./adapters/kwin/install.sh` (`Alt+V`; `Meta+V` занят историей буфера обмена Plasma 6) или System Settings -> Keyboard -> Custom Shortcuts -> `systemctl --user start dicta-toggle.service` |
| Hyprland | `source = .../adapters/hyprland/dicta.conf` |
| Sway | `include .../adapters/sway/dicta.conf` |
| GNOME | `./adapters/gnome/apply.sh '<Ctrl><Alt>v'` |
| любой DE/без DE | evdev-слушатель демона: `hotkey = "ctrl+alt+v"` + доступ к `/dev/input/event*` |

## Статус

Работает: запись, распознавание (Vulkan и CPU), гейтер тишины, авто-остановка записи по
тишине (VAD), фильтр галлюцинаций, вставка буфером + клавишами с подбором под окно,
AT-SPI2-вставка без буфера обмена и без синтезированных клавиш, оверлей, цепочка клавиш
через KWin, systemd-юниты.

Не сделано: сквозная проверка AT-SPI-вставки в реальном фокусном поле ввода (на этой машине
тест упирался в то, что `focused` у Firefox — документ, а не поле), доступность Qt-окон без
`QT_ACCESSIBILITY=1`, пакеты для других дистрибутивов, macOS/Windows.

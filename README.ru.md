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
| `dictactl` | CLI: `toggle`, `start`, `stop`, `cancel`, `status`, `paste-keys`, `calibrate`, `inject <текст>`, `infer file.wav` |
| `dicta-ui` | layer-shell оверлей: пилюля REC + волна уровня + анимация распознавания |
| `adapters/` | горячие клавиши: KWin-скрипт (Plasma 6), Hyprland, sway, Quickshell, GNOME |
| `systemd/` | user-сервис `dictad.service` + oneshot-юниты, которые дёргает DE |
| `docker/`, `compose.yaml` | запуск демона в контейнере (Compose / Portainer stack) |

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

## Контейнер

```bash
docker compose up -d --build
docker exec dicta dictactl status
```

Portainer: Stacks -> New Stack -> Web editor, вставить `compose.yaml`, в environment steка
указать `XDG_RUNTIME_DIR=/run/user/1000` и `WAYLAND_DISPLAY=wayland-1` -> Deploy.
Подробности и подводные камни (uinput-ACL, PipeWire, Wayland-сокеты, GPU):
[docs/CONTAINERS.md](docs/CONTAINERS.md).

## Диагностика и подбор порога

```bash
dictactl status                      # состояние и уровень
dictactl calibrate --secs 8          # 8 с говоришь: пик речи, среднее, уровень первых 2 с
dictactl inject "проверка вставки"   # вставляет текст без микрофона
dictactl inject --app konsole "…"    # принудительно клавиши терминала
./tools/insert-test.sh               # сквозная проверка вставки в отдельное окно
journalctl --user -u dictad.service -f
```

Если диктовка «пропускается» — крути `min_level` (по умолчанию 0.35; 0.25 чувствительнее,
0.6 строже). Гейтер считает не среднее по всему аудио, а максимум по окнам 300 мс.

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

Работает: запись, распознавание (Vulkan и CPU), гейтер тишины, фильтр галлюцинаций, вставка
буфером + клавишами с подбором под окно, оверлей, цепочка клавиш через KWin, systemd-юниты,
образ контейнера.

Не сделано: авто-остановка по тишине (VAD), AT-SPI2-вставка без буфера обмена, пакеты для
других дистрибутивов, macOS/Windows.

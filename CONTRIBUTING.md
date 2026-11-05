# CONTRIBUTING

## Где что лежит

- `crates/dicta-core` — вся логика: конфиг, запись (`pw-record`), распознавание (whisper-rs),
  фильтр текста, гейтер тишины, вставка (wl-copy + uinput), подбор клавиш по окну.
- `crates/dictad` — демон и протокол Unix-сокета (одна JSON-строка в одну сторону).
  Команды: `start stop toggle cancel status inject calibrate record`.
- `crates/dictactl` — клиент; `crates/dicta-ui` — layer-shell оверлей.
- `adapters/` — привязка глобальной горячей клавиши к DE; `systemd/` — юниты.
- Контейнерный запуск (`docker/`, `compose.yaml`, `docs/CONTAINERS.md`) в релиз не входит:
  он живёт отдельной веткой `container`.

## Правила кода

- `cargo fmt` и `cargo test` должны проходить; тесты — unit, без микрофона и без GPU.
- Новые зависимости — только если они реально нужны (whisper-rs и так тащит whisper.cpp).
- Не добавлять молчаливых падений: сбой должен попадать в `eprintln!` демона (он идёт в
  `journalctl --user -u dictad.service`) и в поле `error` ответа сокета.
- GPU: модель одна и резидентная. Никаких «запасных» моделей и параллельных
  `llama-server`/whisper-экземпляров.
- Vulkan-нумерация устройств ggml не совпадает с `/sys/class/drm/cardN` — устройства задают
  `gpu_device` или `GGML_VK_VISIBLE_DEVICES`, не догадками.

## Как проверять изменение

```bash
cargo test --workspace
cargo build --release --locked
cargo build --release --locked --no-default-features   # CPU-вариант не должен ломаться
./tools/insert-test.sh                                 # вставка в окно (нужен foot)
dictactl calibrate --secs 8                            # пороги уровня
dictactl atspi-dump --depth 8                          # видит ли дерево доступности фокус
journalctl --user -u dictad.service -f
```

Injections и оверлей проверяются скриншотами: `spectacle -b -n -f -o /tmp/s.png` и сравнение
полосы, где рисуется пилюля. `dicta-ui` обязан присоединять буфер и в `idle` — Wayland surface
становится mapped только на commit с буфером, иначе frame-колбэков нет и оверлей не появляется.

## Протокол сокета

Одна JSON-строка: `{"cmd":"toggle","app":"konsole","text":"...","keys":"ctrl+shift+v"}` ->
`{"state":"recording","level":0.42,"seconds":1.8,"text":null,"error":null}`.
Обратная совместимость: неизвестные поля игнорируются, новые команды добавляются без смены версии.

## Релизы

См. [docs/RELEASE.md](docs/RELEASE.md): версия в `[workspace.package]`, запись в
`CHANGELOG.md`, тег `vMAJOR.MINOR.PATCH` -> CI собирает архивы (Vulkan и CPU).

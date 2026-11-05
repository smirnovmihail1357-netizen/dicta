# dicta in a container

dicta is *host-integrated by design*: it records from your PipeWire session, draws a Wayland
layer-shell overlay and synthesizes keystrokes through `/dev/uinput`. A container can do all of
that, but only if it is given the session's sockets and devices. `compose.yaml` already does it.

## What the container needs

| need | how it is provided |
|---|---|
| microphone (PipeWire) | `-v $XDG_RUNTIME_DIR:$XDG_RUNTIME_DIR` (PipeWire socket lives there) + `pipewire` package (provides `pw-record`) |
| overlay (Wayland) | same socket dir + `WAYLAND_DISPLAY` env |
| clipboard (`wl-copy`/`wl-paste`) | `wl-clipboard` in the image |
| keystrokes | `--device /dev/uinput` (in compose: `devices: [/dev/uinput:/dev/uinput]`) |
| Vulkan | `--device /dev/dri`, `--group-add video --group-add render`, `mesa-vulkan-drivers` |
| model file | volume `dicta-models:/models` (or a bind mount of an existing model dir) |
| control socket | `DICTA_SOCKET=/run/dicta/dicta.sock` (inside the container, so it cannot collide with a host daemon); the host controls it with `docker exec dicta dictactl ...` |

## Compose

```bash
docker compose up -d --build
docker logs -f dicta
docker exec dicta dictactl status        # управление идёт через docker exec
docker compose down
```

`compose.yaml` reads `XDG_RUNTIME_DIR` and `WAYLAND_DISPLAY` from the environment — copy
`.env.example` to `.env` or export them before `docker compose up`.

## Portainer (Stacks)

1. Portainer is at `https://127.0.0.1:9443` (or `http://127.0.0.1:9000`) in your setup.
2. **Stacks -> New Stack** -> *Web editor*, paste the contents of `compose.yaml`.
3. In *Environment variables* of the stack set:
   `XDG_RUNTIME_DIR=/run/user/1000`, `WAYLAND_DISPLAY=wayland-1` (KDE uses `wayland-1`,
   wlroots sessions usually `wayland-0`), optionally `GGML_VK_VISIBLE_DEVICES=0`.
4. **Deploy the stack**. Portainer builds the image from `docker/Dockerfile` in the repo
   (either point the stack at a Git repo, or paste the compose and let it build from the
   uploaded/clone context).
5. Check: `docker exec dicta dictactl status`, `docker logs dicta`.

## Hotkey while running in a container

Keep the hotkey on the host (a DE shortcut needs the compositor anyway) and let the unit call the
container instead of the host binary:

```ini
# ~/.config/systemd/user/dicta-toggle@.service  (ExecStart line)
ExecStart=/usr/bin/env docker exec dicta dictactl toggle --app %i
```

`adapters/kwin/install.sh` produces the host version; swap the `ExecStart` line for the container
one. The KWin script itself does not change.

## Permissions notes

- The stack runs as **root inside the container**, which is why `/dev/uinput` and the session
  sockets work without extra ACLs. If you set `user: "1000:1000"` in the compose file instead,
  the container user needs rw on `/dev/uinput` (udev rule or `setfacl`) and read access to the
  PipeWire/Wayland sockets.
- PipeWire authorizes clients by the socket directory owner; root connections normally work, but
  if you see `pw-record` failing, run the container as your own UID and mount the PipeWire socket
  dir explicitly.
- `/dev/shm` does **not** have to be shared: `wl_shm` buffers are passed as file descriptors.

## CPU-only container

Build with the CPU feature set, then keep `use_gpu = false` in the config:

```bash
docker build -f docker/Dockerfile -t dicta:cpu --build-arg VULKAN=0 .
```

`VULKAN=1` (default) builds ggml-vulkan and needs `glslangValidator`; `VULKAN=0` builds the
CPU-only feature set. Note that even a Vulkan-built binary runs on CPU when the config says
`use_gpu = false`.

## Verifying inside the container

```bash
docker exec dicta dictactl status
docker exec dicta dictactl inject "контейнерная вставка"
docker exec dicta dictactl calibrate --secs 8
docker exec -it dicta sh        # shell внутри образа
docker exec dicta dicta-ui --probe   # понимает ли композитор zwlr_layer_shell_v1
```

## Base image: trixie, not bookworm

`whisper-rs-sys 0.15` bundles a whisper.cpp whose `ggml-vulkan.cpp` uses `vk::LayerSettingEXT`
(Vulkan-Hpp from Vulkan-Headers 1.4). Debian bookworm ships 1.3.239 and the build fails with
`'LayerSettingEXT' is not a member of 'vk'`. That is why the Dockerfile uses `rust:1-trixie` +
`debian:trixie-slim`. If you must stay on bookworm, build with `--build-arg VULKAN=0` (CPU).

## Что проверено на живом железе (05.11.2026, KDE Plasma 6 + PipeWire)

Образ `dicta:local` (653 MB на диске):

```text
$ docker run -d --name dicta-test --entrypoint dictad \
    --device /dev/uinput -v /run/user/1000:/run/user/1000 \
    -v /tmp/dicta-ctest:/config -v ~/.local/share/whisper-cpp:/models:ro \
    -e DICTA_CONFIG=/config/config.toml -e DICTA_SOCKET=/run/dicta/dicta.sock dicta:local
dictad: сокет /run/dicta/dicta.sock, модель /models/ggml-small-q5_1.bin, язык ru
$ docker exec dicta-test dictactl status
{"state":"idle", ...}
$ docker exec dicta-test dictactl record /tmp/rec.wav --secs 3     # микрофон хоста через PipeWire
записано 3 с -> /tmp/rec.wav                                        # 95 KB
$ docker exec dicta-test dictactl infer --timing /config/ru.wav    # CPU-инференс в контейнере
infer=3.11 s
$ docker exec dicta-test dictactl paste-keys --app firefox
клавиши: ctrl+v
$ docker exec dicta-test dictactl inject "привет из контейнера"    # inject_mode=stdout
```

То есть из контейнера работают: PipeWire-запись, Unix-сокет, CLI, конфиг через `DICTA_CONFIG`,
модель из volume. Vulkan-путь в контейнере требует `/dev/dri` + `mesa-vulkan-drivers` (в образе
есть) и подтверждается только на конкретном железе.

## Known limits

- One daemon instance: the whisper model is kept resident; two daemons = two models in VRAM.
- The container cannot register a global shortcut itself — that stays with the DE adapter.
- X11-only apps under XWayland: paste keys still work (they are synthesized at the kernel level).

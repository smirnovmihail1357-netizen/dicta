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
| control socket | `DICTA_SOCKET=$XDG_RUNTIME_DIR/dicta.sock` — the host `dictactl` talks to the container daemon over the same path |

## Compose

```bash
docker compose up -d --build
docker logs -f dicta
docker exec dicta dictactl status
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

## Known limits

- One daemon instance: the whisper model is kept resident; two daemons = two models in VRAM.
- The container cannot register a global shortcut itself — that stays with the DE adapter.
- X11-only apps under XWayland: paste keys still work (they are synthesized at the kernel level).

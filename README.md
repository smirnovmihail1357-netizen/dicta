# dicta

System-wide voice dictation for Linux/Wayland: one daemon instead of a separate integration in
every app. Press a hotkey, speak, press it again — the text lands where your cursor is:
Kate, Konsole, Firefox, Discord, a browser chat, a terminal prompt.

```
Alt+V  ->  speak  ->  Alt+V  ->  text inserted into the focused window
```

Stopping is the **same Alt+V** (or a click on the overlay pill, or `dictactl stop`).
Abort without recognition: `dictactl cancel`.

Tested on KDE Plasma 6 (KWin, Wayland) + PipeWire + AMD RX 7900 XTX / Radeon 610M.
Hyprland, Sway, Quickshell and GNOME adapters are included but less exercised.

## How it works

| part | role |
|---|---|
| `crates/dicta-core` | config (TOML), PipeWire capture, recognition (whisper-rs / whisper.cpp: Vulkan or CPU), hallucination filter, clipboard + uinput injection, per-app paste keys |
| `crates/dictad` | the daemon: Unix socket, recording, inference, text injection, spawns the overlay |
| `crates/dictactl` | CLI client: `toggle start stop cancel status paste-keys calibrate inject infer` |
| `crates/dicta-ui` | layer-shell overlay (REC pill + level waveform + "transcribing" scanner) |
| `adapters/` | global hotkeys: KWin script (Plasma 6), Hyprland, Sway, Quickshell, GNOME |
| `systemd/` | `dictad.service` (user service) + oneshot units the DE hotkey calls |
| `docker/`, `compose.yaml` | running the daemon in a container (Docker Compose / Portainer stack) |

Chain: DE hotkey -> systemd oneshot unit -> `dictactl toggle --app <focused window>` ->
daemon (`pw-record` -> whisper -> filter) -> `wl-copy` + synthesized keystrokes via `/dev/uinput`.

## Quick start (host install)

```bash
git clone https://github.com/<you>/dicta.git && cd dicta
make build                      # release build with Vulkan
make install                    # -> ~/.local/bin/{dictad,dictactl,dicta-ui}
./tools/fetch-model.sh          # ggml model into ~/.local/share/whisper-cpp/
mkdir -p ~/.config/dicta && cp config.example.toml ~/.config/dicta/config.toml
./systemd/install.sh            # dictad.service + toggle/cancel units
./adapters/kwin/install.sh      # Plasma 6: Alt+V via KWin script (logout/login needed)
systemctl --user start dictad.service
```

Then: `Alt+V`, speak 3-4 seconds, `Alt+V`. Watch what happened:

```bash
journalctl --user -u dictad.service -f
```

CPU-only build (needs neither `glslc` nor `glslangValidator`; add `use_gpu = false` to the config):

```bash
make build-cpu        # cargo build --release --no-default-features
```

## Run in a container (Docker / Compose / Portainer)

```bash
docker compose up -d --build          # or: make docker-build && make docker-run
docker exec dicta dictactl status
```

Portainer: **Stacks -> New Stack -> Web editor**, paste `compose.yaml`, set stack environment
variables `XDG_RUNTIME_DIR=/run/user/1000` and `WAYLAND_DISPLAY=wayland-1`, then *Deploy the stack*.

The container needs the live session, so `compose.yaml` mounts the session socket dir,
`/dev/uinput` and `/dev/dri`. Details, permission notes and troubleshooting:
[docs/CONTAINERS.md](docs/CONTAINERS.md).

## Diagnostics

```bash
dictactl status                       # state + live input level
dictactl calibrate --secs 8           # speak for 8 s: prints speech peak / mean / first-2s level
dictactl inject "insertion test"      # inject text without a microphone
dictactl inject --app konsole "..."   # force the konsole key combo
dictactl paste-keys --app org.kde.konsole
dictactl infer --timing speech.wav    # recognition only, no injection
./tools/insert-test.sh                # end-to-end injection test into a dedicated window
```

If dictation keeps being skipped, tune `min_level` (0.35 default): 0.25 = more sensitive,
0.6 = stricter. The gate uses the loudest 300 ms window, not the average.

## Configuration

`~/.config/dicta/config.toml` (see [`config.example.toml`](config.example.toml)). Every key is
optional. Key ones: `model`, `language`, `use_gpu`, `gpu_device`, `mic`, `socket`,
`inject_mode` (`paste|clipboard|stdout`), `paste_keys`, `paste_keys_by_app`, `hotkey`,
`min_level`, `clipboard_restore_ms`, `paste_delay_ms`.

Environment: `DICTA_CONFIG`, `DICTA_UI`, `DICTA_HOTKEY`, `DICTA_APP`, and overlay knobs
`DICTA_LAYER`, `DICTA_EXCLUSIVE`, `DICTA_DEBUG`, `DICTA_ALWAYS_DRAW`.

## Hotkey wiring

| compositor | how |
|---|---|
| KDE Plasma 6 | `./systemd/install.sh && ./adapters/kwin/install.sh` (KWin script registers `Alt+V`; `Meta+V` is taken by Plasma clipboard history) |
| Hyprland | `source = .../adapters/hyprland/dicta.conf` |
| Sway | `include .../adapters/sway/dicta.conf` |
| GNOME | `./adapters/gnome/apply.sh '<Ctrl><Alt>v'` |
| no DE / any DE | daemon's built-in evdev listener: `hotkey = "ctrl+alt+v"` plus access to `/dev/input/event*` (`input` group or udev ACL) |

## Status

Working today: capture, recognition (Vulkan and CPU), silence gate, hallucination filter,
clipboard+uinput injection with per-app key combos, layer-shell overlay, KWin/hotkey chain,
systemd units, container image.

Not done: silence-based auto-stop (VAD), AT-SPI2 injection as an alternative to
clipboard+keystrokes, packaged releases for other distros, macOS/Windows.

## Contribute / release

See [CONTRIBUTING.md](CONTRIBUTING.md) and [docs/RELEASE.md](docs/RELEASE.md).
CI builds both the Vulkan and CPU-only variants; tagging `v*` produces release archives and a
container image. [CHANGELOG.md](CHANGELOG.md) · [SECURITY.md](SECURITY.md) · MIT license.

Русская версия: [README.ru.md](README.ru.md).

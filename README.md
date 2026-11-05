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
| `crates/dictactl` | CLI client: `toggle start stop cancel status paste-keys calibrate inject infer atspi-dump atspi-insert` |
| `crates/dicta-ui` | layer-shell overlay (REC pill + level waveform + "transcribing" scanner) |
| `adapters/` | global hotkeys: KWin script (Plasma 6), Hyprland, Sway, Quickshell, GNOME |
| `systemd/` | `dictad.service` (user service) + oneshot units the DE hotkey calls |

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

## Diagnostics

```bash
dictactl status                       # state + live input level
dictactl calibrate --secs 8           # speak for 8 s: prints speech peak / mean / first-2s level
dictactl inject "insertion test"      # inject text without a microphone
dictactl inject --app konsole "..."   # force the konsole key combo
dictactl paste-keys --app org.kde.konsole
dictactl atspi-dump --depth 8               # AT-SPI: apps, roles, focused objects, stage timings
dictactl atspi-insert "text"                # AT-SPI insertion probe (bypasses the daemon)
dictactl infer --timing speech.wav    # recognition only, no injection
./tools/insert-test.sh                # end-to-end injection test into a dedicated window
```

If dictation keeps being skipped, tune `min_level` (0.35 default): 0.25 = more sensitive,
0.6 = stricter. The gate uses the loudest 300 ms window, not the average.

Recording stops itself when the microphone goes quiet (`auto_stop = true`, default): the
level monitor doubles as a watchdog and closes the recording after `auto_stop_after_ms`
of silence, with a `auto_stop_grace_ms` grace period at the start and a `max_record_ms`
hard ceiling.

## Injection without a clipboard (`inject_mode = "atspi"`)

`paste` mode writes the clipboard and presses keys. `atspi` mode writes straight into the
focused widget through AT-SPI2 (`org.a11y.atspi.EditableText`) — no clipboard touched, no
synthesised keystrokes. Build it with the feature:

```bash
cargo build --release --features dictad/atspi,dictactl/atspi
```

Check whether the window you dictate into supports it, without inserting anything:

```bash
dictactl atspi-dump --depth 8          # apps, roles, focused objects, stage timings
dictactl atspi-dump --app firefox --tree
```

Only objects that report role `entry`, `password text`, `combo box` or `text` are written
to. That matters: Firefox marks its `document web` node as focused (and it does implement
`EditableText`), so writing there is not "the focused field" — set
`atspi_allow_any_role = true` only while diagnosing such an app. Note that AT-SPI support
per toolkit differs: Gecko/GTK yes, Qt needs `QT_ACCESSIBILITY=1`, Chromium/Electron needs
`--force-renderer-accessibility`.

## Configuration

`~/.config/dicta/config.toml` (see [`config.example.toml`](config.example.toml)). Every key is
optional. Key ones: `model`, `language`, `use_gpu`, `gpu_device`, `mic`, `socket`,
`inject_mode` (`paste|clipboard|stdout|atspi`), `paste_keys`, `paste_keys_by_app`, `hotkey`,
`min_level`, `clipboard_restore_ms`, `paste_delay_ms`, `auto_stop`, `auto_stop_level`,
`auto_stop_after_ms`, `auto_stop_grace_ms`, `max_record_ms`.

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

Working today: capture, recognition (Vulkan and CPU), silence gate, silence-based auto-stop
(VAD), hallucination filter, clipboard+uinput injection with per-app key combos, AT-SPI2
injection (no clipboard, no synthesised keystrokes), layer-shell overlay, KWin/hotkey chain,
systemd units.

Not done: an end-to-end AT-SPI insertion check against a real focused input field (on the
development machine Firefox reports `focused` on its document node, not on a field), Qt
windows without `QT_ACCESSIBILITY=1`, packaged releases for other distros, macOS/Windows.

## Contribute / release

See [CONTRIBUTING.md](CONTRIBUTING.md) and [docs/RELEASE.md](docs/RELEASE.md).
CI builds both the Vulkan and CPU-only variants; tagging `v*` produces release archives.
[CHANGELOG.md](CHANGELOG.md) · [SECURITY.md](SECURITY.md) · MIT license.

Русская версия: [README.ru.md](README.ru.md).

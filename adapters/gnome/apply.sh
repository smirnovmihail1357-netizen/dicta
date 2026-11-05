#!/usr/bin/env bash
# Custom keybinding в GNOME (Wayland) -> dictactl toggle.
#   ./apply.sh '<Ctrl><Alt>v'
# Отменить: ./apply.sh --remove
set -euo pipefail

SCHEMA=org.gnome.settings-daemon.plugins.media-keys
BASE=/org/gnome/settings-daemon/plugins/media-keys/custom-keybindings
KEY="${1:-<Ctrl><Alt>v}"

if [[ "$KEY" == "--remove" ]]; then
    gsettings set $SCHEMA custom-keybindings "[]"
    echo "custom keybindings очищены"
    exit 0
fi

existing=$(gsettings get $SCHEMA custom-keybindings)
chosen=""
for i in 0 1 2 3 4 5 6 7; do
    if ! grep -q "custom$i" <<<"$existing"; then
        chosen="$BASE/custom$i"
        break
    fi
done
if [[ -z "$chosen" ]]; then
    echo "все custom0..custom7 заняты — освободи один в GNOME Settings"
    exit 1
fi

gsettings set $SCHEMA custom-keybindings "$(sed "s/\]/, '$chosen']/" <<<"$existing" | sed "s/\[\]/['$chosen']/")"
gsettings set $SCHEMA.custom-keybinding:$chosen name "Dicta (голосовой ввод)"
gsettings set $SCHEMA.custom-keybinding:$chosen command "dictactl toggle"
gsettings set $SCHEMA.custom-keybinding:$chosen binding "$KEY"
echo "GNOME: $KEY -> dictactl toggle ($chosen)"

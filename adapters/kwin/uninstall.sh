#!/usr/bin/env bash
# Выгружает и удаляет KWin-скрипт dicta-hotkey.
set -euo pipefail
ID=dicta-hotkey
DEST="${XDG_DATA_HOME:-$HOME/.local/share}/kwin/scripts/$ID"

dbus-send --session --dest=org.kde.KWin /Scripting org.kde.kwin.Scripting.unloadScript string:"$ID" \
    >/dev/null 2>&1 || true
rm -rf "$DEST"
if command -v kwriteconfig6 >/dev/null; then
    kwriteconfig6 --file kwinrc --group Plugins --key "${ID}Enabled" --type bool false
    dbus-send --session --dest=org.kde.KWin /KWin org.kde.KWin.reconfigure >/dev/null 2>&1 || true
fi
echo "скрипт $ID выгружен и удалён ($DEST)"

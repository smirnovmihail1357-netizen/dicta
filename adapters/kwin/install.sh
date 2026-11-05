#!/usr/bin/env bash
# Устанавливает KWin-скрипт dicta-hotkey и загружает его в текущую сессию Plasma 6.
#
#   ./install.sh                                    # Meta+V = начать/закончить диктовку
#   DICTA_KEY='Ctrl+Alt+V' ./install.sh             # другой ключ
#   DICTA_CANCEL_KEY='Meta+Shift+Escape' ./install.sh
#   DICTA_SELFTEST=1 ./install.sh                   # скрипт сам дёргает юнит при загрузке:
#                                                   # проверка цепочки без нажатия клавиш
#
# Проверка после установки:
#   journalctl --user -b 0 | grep dicta-hotkey      # "toggle 'Meta+V' -> ok"
#   busctl --user call org.kde.kglobalaccel /kglobalaccel org.kde.KGlobalAccel \
#       allActionsForComponent as 1 kwin | tr ' ' '\n' | grep -i dicta
#
# Порядок важен: без флага в kwinrc loadScript в живом KWin часто не запускает скрипт,
# поэтому флаг ставится всегда, потом unload/load/start и reconfigure.
#
# Нужно, чтобы юниты dicta-toggle.service / dicta-cancel.service лежали в
# ~/.config/systemd/user (см. ../systemd/install.sh).
set -euo pipefail

ID=dicta-hotkey
HERE="$(cd "$(dirname "$0")" && pwd)"
SRC="$HERE/$ID"
DEST="${XDG_DATA_HOME:-$HOME/.local/share}/kwin/scripts/$ID"

KEY="${DICTA_KEY:-Meta+V}"
CANCEL_KEY="${DICTA_CANCEL_KEY:-}"
TOGGLE_UNIT="${DICTA_UNIT:-dicta-toggle.service}"
CANCEL_UNIT="${DICTA_CANCEL_UNIT:-dicta-cancel.service}"
SELFTEST=false
[[ "${DICTA_SELFTEST:-0}" == "1" ]] && SELFTEST=true

mkdir -p "$DEST/contents/code"
sed -e "s|__DICTA_KEY__|$KEY|g" \
    -e "s|__DICTA_CANCEL_KEY__|$CANCEL_KEY|g" \
    -e "s|dicta-toggle.service|$TOGGLE_UNIT|g" \
    -e "s|dicta-cancel.service|$CANCEL_UNIT|g" \
    -e "s|__DICTA_SELFTEST__|$SELFTEST|g" \
    "$SRC/contents/code/main.js" > "$DEST/contents/code/main.js"
cp "$SRC/metadata.json" "$DEST/metadata.json"
echo "скрипт: $DEST (toggle=$KEY unit=$TOGGLE_UNIT selftest=$SELFTEST)"

dbus-send --session --print-reply --dest=org.kde.KWin /Scripting \
    org.kde.kwin.Scripting.unloadScript string:"$ID" >/dev/null 2>&1 || true
dbus-send --session --print-reply --dest=org.kde.KWin /Scripting \
    org.kde.kwin.Scripting.loadScript string:"$ID" >/dev/null
dbus-send --session --dest=org.kde.KWin /Scripting org.kde.kwin.Scripting.start >/dev/null

if command -v kwriteconfig6 >/dev/null; then
    kwriteconfig6 --file kwinrc --group Plugins --key "${ID}Enabled" --type bool true
else
    echo "nужен kwriteconfig6, чтобы скрипт загружался сам (иначе он живёт до перезапуска KWin)"
fi
dbus-send --session --dest=org.kde.KWin /KWin org.kde.KWin.reconfigure >/dev/null 2>&1 || true
sleep 2

loaded=$(dbus-send --session --print-reply --dest=org.kde.KWin /Scripting \
    org.kde.kwin.Scripting.isScriptLoaded string:"$ID" | tail -1 | awk '{print $NF}')
actions=$(dbus-send --session --print-reply --dest=org.kde.kglobalaccel /kglobalaccel \
    org.kde.KGlobalAccel allActionsForComponent as 1 kwin 2>/dev/null | tr ' ' '\n' | grep -ci dicta || true)
echo "isScriptLoaded: $loaded, действий dicta в kglobalaccel: ${actions:-0}"

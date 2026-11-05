#!/usr/bin/env bash
# Кладёт юниты dicta в ~/.config/systemd/user и делает daemon-reload.
#
#   ./install.sh            # только положить юниты (демон не включается в автозапуск)
#   ./install.sh --enable   # + systemctl --user enable --now dictad.service
#
# Горячая клавиша тогда работает через dicta-toggle.service (KWin-скрипт,
# Hyprland/sway bind, GNOME custom shortcut -> systemctl --user start dicta-toggle.service).
set -euo pipefail
HERE="$(cd "$(dirname "$0")" && pwd)"
DEST="${XDG_CONFIG_HOME:-$HOME/.config}/systemd/user"
mkdir -p "$DEST"
for u in dictad.service dicta-toggle.service dicta-toggle@.service dicta-cancel.service \
         dicta-cancel@.service dicta-probe.service dicta-probe@.service; do
    install -m 644 "$HERE/$u" "$DEST/$u"
    echo "установлен $DEST/$u"
done
systemctl --user daemon-reload
echo
echo "проверка: systemctl --user status dicta-toggle.service"
echo "тест цепочки без диктовки:"
echo "  systemctl --user start 'dicta-probe@konsole.service' && tail /tmp/dicta-probe.log"
if [[ "${1:-}" == "--enable" ]]; then
    systemctl --user enable --now dictad.service
    echo "dictad.service включён в пользовательскую сессию"
fi

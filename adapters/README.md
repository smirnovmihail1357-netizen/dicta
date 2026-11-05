# Горячие клавиши (адаптеры)

Демон управляется одной командой: `dictactl toggle` (или `start`/`stop`/`cancel`).
Задача адаптера — вызвать её по клавише. Три способа, по возрастанию «магии»:

| способ | требует root | работает | что это |
|---|---|---|---|
| **systemd-user + клавиша DE** | нет | любой DE | клавиша DE → `systemctl --user start dicta-toggle.service` |
| **скрипт DE** (KWin `registerShortcut`, Hyprland `bind`, sway `bindsym`, Quickshell) | нет | соответствующий DE | самый быстрый отклик |
| **встроенный evdev-слушатель dictad** | да (группа `input` или udev-ACL) | любой DE, даже без DE | `hotkey = "ctrl+alt+v"` в конфиге |

## 1. Юниты systemd (нужны для KWin-адаптера)

```bash
cd ~/projects/dicta && ./systemd/install.sh          # положить юниты
./systemd/install.sh --enable                        # + включить демон при входе сессии
systemctl --user start 'dicta-probe@konsole.service' && tail /tmp/dicta-probe.log   # тест цепочки
```

Юниты: `dictad.service` (демон + оверлей), `dicta-toggle.service` и шаблонный
`dicta-toggle@<окно>.service`, `dicta-cancel.service` (`@`-вариант), `dicta-probe.service`
(`@`-вариант) — проверка без распознавания и вставки.

Шаблонный юнит нужен, чтобы демон знал, в какое окно вставлять: `dicta-toggle@konsole.service`
-> `dictactl toggle --app konsole` -> `paste_keys_by_app` -> `Shift+Insert`.

```bash
# проверка цепочки без диктовки (окно konsole -> клавиши shift+insert):
systemctl --user start 'dicta-probe@konsole.service' && tail /tmp/dicta-probe.log
```

## 2. KDE Plasma 6 (KWin, Wayland)

```bash
cd ~/projects/dicta && ./systemd/install.sh          # сначала юниты
./adapters/kwin/install.sh                          # затем скрипт клавиши
DICTA_SELFTEST=1 DICTA_UNIT=dicta-probe.service ./install.sh   # то же, но с самопроверкой
```

KWin 6 в скриптах не имеет `spawn()`, поэтому скрипт зовёт `callDBus` на
`org.freedesktop.systemd1 Manager.StartUnit("dicta-toggle@<окно>.service")`, а имя окна берёт
из `workspace.activeWindow` (`appId` / `resourceClass`). Ключ — `Alt+V`
(Meta+V в Plasma 6 занят историей буфера обмена; сменить: `DICTA_KEY='Ctrl+Alt+V' ./install.sh`),
отмена без клавиши по умолчанию
(`DICTA_CANCEL_KEY='Meta+Shift+Escape' ./install.sh`). Удаление: `./uninstall.sh`.

Проверка установки (без нажатия клавиш):

```bash
journalctl --user -b 0 | grep dicta-hotkey        # "toggle 'Alt+V' -> ok"
busctl --user call org.kde.kglobalaccel /component/kwin org.kde.kglobalaccel.Component shortcutNames \
    | tr ' ' '\n' | grep -i dicta                 # dicta-toggle, dicta-cancel
tail /tmp/dicta-probe.log                          # если ставил DICTA_SELFTEST=1
```

Замечено на живом Plasma 6: если скрипт ещё не был включён, одного `loadScript` иногда
недостаточно — `install.sh` всегда пишет `[Plugins] dicta-hotkeyEnabled=true` в `kwinrc` и
dёргает `KWin.reconfigure`; надёжнее всего это работает после перелогинования (или
`kwriteconfig6` + перезапуск сессии). Нажатия клавиш в самой сессии проверить автоматически
нельзя — это делает пользователь.

Вариант без скрипта: System Settings → Keyboard → Custom Shortcuts → New → Global →
Command/URL = `systemctl --user start dicta-toggle.service`, ключ `Alt+V`.

Смена клавиши: `DICTA_KEY='...' ./install.sh`, затем **перелогин** — на лету KWin скрипт не
перезагружается (`loadScript` в живой сессии не регистрирует клавишу заново), а старая
привязка живёт в `~/.config/kglobalshortcutsrc` (строки `dicta-toggle=...`; `./uninstall.sh` их
чистит). До перелогинов в этой сессии диктовку можно запускать руками:
`dictactl toggle` или `systemctl --user start dicta-toggle.service`.

Откат живой клавиши, не дожидаясь перелогинов (проверено — Plasma 6 сразу возвращает себе
Meta+V и историю буфера обмена):

```bash
# именно unregister (две строки), а не unRegister(array) — второй вариант KWin игнорирует
for a in dicta-toggle dicta-cancel; do
    dbus-send --session --dest=org.kde.kglobalaccel /kglobalaccel \
        org.kde.KGlobalAccel.unregister string:kwin string:$a
done
grep -v '^dicta-' ~/.config/kglobalshortcutsrc > /tmp/k.new && mv /tmp/k.new ~/.config/kglobalshortcutsrc
# посмотреть, кто держит клавишу:
busctl --user --json=short call org.kde.kglobalaccel /component/kwin \
    org.kde.kglobalaccel.Component allShortcutInfos |
    python3 -c 'import json,sys; [print(r[0], [hex(x) for x in r[6]]) for r in json.load(sys.stdin)["data"] if r[6]]'
# 268435542 = Meta+V, 33554518 = Alt+V
```

## 3. Hyprland

```
source = ~/projects/dicta/adapters/hyprland/dicta.conf
```

В файле уже есть `bind = SUPER, V, exec, dictactl toggle --app "$(hyprctl activewindow -j | jq -r .class)"`
— так демон знает, что фокусное окно терминал, и берёт `Shift+Insert`.

## 4. Sway / wlroots-подобные, у которых есть bindsym

`include ~/projects/dicta/adapters/sway/dicta.conf` — там `bindsym $mod+v` с
`swaymsg -t get_tree | jq` для имени фокусного окна.

## 5. Quickshell

`quickshell/dicta.qml` — `ShellShortcut` + `Process.exec`, имя окна берётся из
`Focus.window.active?.appId`; вставляется в существующую конфигурацию Quickshell.

## 6. GNOME (Wayland)

```bash
gnome/apply.sh '<Ctrl><Alt>v'    # custom keybinding -> dictactl toggle (отмена: ./apply.sh --remove)
```

## 7. Без DE и без скриптов: evdev прямо в демоне

```bash
# в ~/.config/dicta/config.toml:  hotkey = "ctrl+alt+v"   (по умолчанию hotkey = "off")
# доступ к /dev/input/event* один раз:
sudo usermod -aG input $USER         # и перелогин
# либо udev-правило с ACL:
printf 'KERNEL=="event*", SUBSYSTEM=="input", MODE="0660", GROUP="input"\n' | sudo tee /etc/udev/rules.d/99-dicta-input.rules
sudo udevadm control --reload-rules && sudo udevadm trigger
```
`dictad --install-autostart` печатает эти подсказки и кладёт `.desktop` в autostart.
Без доступа к /dev/input демон честно пишет «горячая клавиша недоступна» и ждёт клавишу DE.

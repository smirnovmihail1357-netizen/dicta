// Горячая клавиша диктовки dicta для Quickshell (https://quickshell.org).
// Вставить в свою конфигурацию (qs.config) как объект с Variants.
// Focus.window.active даёт appId фокусного окна -> dictactl --app -> Ctrl+V / Shift+Insert.

QtObject {
    property Variants dictaVariants: [
        Variants {
            name: "dicta"
            property ShellShortcut toggle: ShellShortcut {
                name: "dictaToggle"
                description: "Диктовка: начать/закончить"
                onActivate: {
                    const app = Focus.window.active?.appId ?? "";
                    const argv = app ? ["dictactl", "toggle", "--app", app] : ["dictactl", "toggle"];
                    Process.exec(argv, []);
                }
            }
            property ShellShortcut cancel: ShellShortcut {
                name: "dictaCancel"
                description: "Диктовка: отменить"
                onActivate: Process.exec(["dictactl", "cancel"], [])
            }
        }
    ]
}

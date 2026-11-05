/*
 * dicta-hotkey — глобальная горячая клавиша диктовки для KWin (Plasma 6, Wayland).
 *
 * KWin 6 в скриптах больше не имеет spawn(), зато есть callDBus(). Поэтому клавиша
 * не запускает процесс сама, а просит пользовательский systemd запустить oneshot-юнит,
 * который и вызывает `dictactl`:
 *   callDBus -> org.freedesktop.systemd1 Manager.StartUnit("dicta-toggle@konsole.service")
 *
 * Имя фокусного окна передаётся демоном-адаптером (instance юнита -> `dictactl --app`),
 * поэтому демон может выбрать Ctrl+V для браузеров и Shift+Insert для терминалов.
 *
 * Плюсы: не нужен root и доступ к /dev/input, работает на Wayland.
 * Юниты: systemd/dicta-toggle.service (+ dicta-toggle@.service), dicta-cancel.service.
 */

const TOGGLE_UNIT = "dicta-toggle.service";
const CANCEL_UNIT = "dicta-cancel.service";
const TOGGLE_KEY = "__DICTA_KEY__";
const CANCEL_KEY = "__DICTA_CANCEL_KEY__";

function activeApp() {
    try {
        const w = workspace.activeWindow;
        if (!w) {
            return "";
        }
        // appId есть у wayland-окон, resourceClass — у XWayland.
        const raw = w.appId || w.resourceClass || "";
        // имена инстансов systemd: только [A-Za-z0-9._-]
        return String(raw).toLowerCase().replace(/[^a-z0-9._-]/g, "-");
    } catch (e) {
        return "";
    }
}

function unitFor(base) {
    const app = activeApp();
    if (!app) {
        return base;
    }
    return base.replace(".service", "@" + app + ".service");
}

function startUnit(unit) {
    const reply = callDBus(
        "org.freedesktop.systemd1",
        "/org/freedesktop/systemd1",
        "org.freedesktop.systemd1.Manager",
        "StartUnit",
        unit,
        "replace"
    );
    if (reply && typeof reply.then === "function") {
        reply.then(
            (path) => print(`dicta: ${unit} запущен`),
            (err) => print(`dicta: не удалось запустить ${unit}: ${err}`)
        );
    }
}

const okToggle = registerShortcut("dicta-toggle", "Диктовка: начать/закончить", TOGGLE_KEY, () =>
    startUnit(unitFor(TOGGLE_UNIT))
);
const okCancel = registerShortcut("dicta-cancel", "Диктовка: отменить", CANCEL_KEY, () =>
    startUnit(unitFor(CANCEL_UNIT))
);

print(
    `dicta-hotkey: toggle '${TOGGLE_KEY}' -> ${okToggle ? "ok" : "ОШИБКА"}, ` +
    `cancel '${CANCEL_KEY}' -> ${okCancel ? "ok" : "ОШИБКА"}`
);

// __DICTA_SELFTEST__ — true только при установке с DICTA_SELFTEST=1: скрипт сам дёргает
// юнит при загрузке, чтобы по его логу проверить цепочку
// «скрипт -> callDBus -> systemd1 -> юнит -> dictactl» без нажатия клавиш.
if (__DICTA_SELFTEST__) {
    startUnit(TOGGLE_UNIT);
}

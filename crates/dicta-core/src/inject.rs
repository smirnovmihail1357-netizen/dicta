//! Вставка текста в фокусное окно.
//!
//! Путь по умолчанию: сохранить буфер обмена -> wl-copy -> послать клавиши вставки
//! через uinput (виртуальная клавиатура, работает на любом композиторе) -> вернуть буфер.
//!
//! Это композитор-независимо: KWin, Hyprland, sway, XWayland-окна.

use std::io::Write as _;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use anyhow::{Context, Result};

const UINPUT_IOCTL_BASE: u64 = b'U' as u64;

const fn iow(nr: u64, size: u64) -> u64 {
    (1 << 30) | (size << 16) | (UINPUT_IOCTL_BASE << 8) | nr
}
const fn io(nr: u64) -> u64 {
    (UINPUT_IOCTL_BASE << 8) | nr
}

const UI_DEV_CREATE: u64 = io(1);
const UI_DEV_DESTROY: u64 = io(2);
const UI_DEV_SETUP: u64 = iow(3, std::mem::size_of::<UinputSetup>() as u64);
const UI_SET_EVBIT: u64 = iow(100, std::mem::size_of::<i32>() as u64);
const UI_SET_KEYBIT: u64 = iow(101, std::mem::size_of::<i32>() as u64);

const EV_SYN: u16 = 0;
const EV_KEY: u16 = 1;
const SYN_REPORT: i32 = 0;

// Linux input key codes
const KEY_ENTER: u16 = 28;
const KEY_LEFTCTRL: u16 = 29;
const KEY_LEFTSHIFT: u16 = 42;
const KEY_LEFTALT: u16 = 56;
const KEY_V: u16 = 47;
const KEY_INSERT: u16 = 110;
const KEY_C: u16 = 46;

#[repr(C)]
#[derive(Default)]
struct InputId {
    bustype: u16,
    vendor: u16,
    product: u16,
    version: u16,
}

#[repr(C)]
struct UinputSetup {
    id: InputId,
    name: [u8; 80],
    ff_effects_max: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct InputEvent {
    tv_sec: i64,
    tv_usec: i64,
    kind: u16,
    code: u16,
    value: i32,
}

/// Комбинация клавиш, например Ctrl+V.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Combo {
    pub ctrl: bool,
    pub shift: bool,
    pub alt: bool,
    pub key: u16,
}

impl Combo {
    pub fn parse(s: &str) -> Result<Self> {
        let mut combo = Combo {
            ctrl: false,
            shift: false,
            alt: false,
            key: 0,
        };
        for part in s.to_lowercase().split('+').map(str::trim).filter(|p| !p.is_empty()) {
            match part {
                "ctrl" | "control" => combo.ctrl = true,
                "shift" => combo.shift = true,
                "alt" => combo.alt = true,
                "v" => combo.key = KEY_V,
                "c" => combo.key = KEY_C,
                "insert" | "ins" => combo.key = KEY_INSERT,
                "enter" | "return" => combo.key = KEY_ENTER,
                other => anyhow::bail!("неизвестная клавиша {other:?} в combos"),
            }
        }
        if combo.key == 0 {
            anyhow::bail!("в комбинации {s:?} нет основной клавиши");
        }
        Ok(combo)
    }
}

pub struct Keyboard {
    fd: i32,
}

impl Keyboard {
    /// Создаёт виртуальную клавиатуру через /dev/uinput.
    pub fn new() -> Result<Self> {
        let fd = unsafe {
            libc::open(
                b"/dev/uinput\0".as_ptr() as *const libc::c_char,
                libc::O_WRONLY | libc::O_NONBLOCK | libc::O_CLOEXEC,
            )
        };
        if fd < 0 {
            anyhow::bail!(
                "не открылся /dev/uinput (нужны права rw на устройство): {}",
                std::io::Error::last_os_error()
            );
        }

        let mut ok = true;
        ok &= unsafe { libc::ioctl(fd, UI_SET_EVBIT as libc::c_ulong, EV_KEY as libc::c_int) } >= 0;
        for code in [
            KEY_LEFTCTRL,
            KEY_LEFTSHIFT,
            KEY_LEFTALT,
            KEY_V,
            KEY_C,
            KEY_INSERT,
            KEY_ENTER,
        ] {
            ok &= unsafe { libc::ioctl(fd, UI_SET_KEYBIT as libc::c_ulong, code as libc::c_int) } >= 0;
        }
        if !ok {
            unsafe { libc::close(fd) };
            anyhow::bail!("UI_SET_EVBIT/UI_SET_KEYBIT отклонены");
        }

        let mut setup: UinputSetup = unsafe { std::mem::zeroed() };
        setup.id = InputId {
            bustype: 0x0003, // BUS_VIRTUAL
            vendor: 0x1234,
            product: 0x5678,
            version: 1,
        };
        let name = b"dicta-keyboard\0";
        for (i, b) in name.iter().enumerate().take(79) {
            setup.name[i] = *b;
        }

        if unsafe { libc::ioctl(fd, UI_DEV_SETUP as libc::c_ulong, &setup) } < 0 {
            unsafe { libc::close(fd) };
            anyhow::bail!(
                "UI_DEV_SETUP не прошёл: {}",
                std::io::Error::last_os_error()
            );
        }
        if unsafe { libc::ioctl(fd, UI_DEV_CREATE as libc::c_ulong) } < 0 {
            unsafe { libc::close(fd) };
            anyhow::bail!(
                "UI_DEV_CREATE не прошёл: {}",
                std::io::Error::last_os_error()
            );
        }

        // uinput-устройство появляется не мгновенно.
        std::thread::sleep(Duration::from_millis(300));
        Ok(Self { fd })
    }

    fn event(&self, kind: u16, code: u16, value: i32) -> Result<()> {
        let ev = InputEvent {
            tv_sec: 0,
            tv_usec: 0,
            kind,
            code,
            value,
        };
        let bytes = unsafe {
            std::slice::from_raw_parts(
                &ev as *const InputEvent as *const u8,
                std::mem::size_of::<InputEvent>(),
            )
        };
        let mut off = 0;
        while off < bytes.len() {
            let n = unsafe {
                libc::write(
                    self.fd,
                    bytes[off..].as_ptr() as *const libc::c_void,
                    bytes.len() - off,
                )
            };
            if n < 0 {
                let err = std::io::Error::last_os_error();
                if err.raw_os_error() == Some(libc::EAGAIN) {
                    std::thread::sleep(Duration::from_millis(2));
                    continue;
                }
                return Err(err).context("не удалось отправить событие uinput");
            }
            off += n as usize;
        }
        Ok(())
    }

    pub fn send(&mut self, combo: Combo) -> Result<()> {
        let mut mods = Vec::new();
        if combo.ctrl {
            mods.push(KEY_LEFTCTRL);
        }
        if combo.shift {
            mods.push(KEY_LEFTSHIFT);
        }
        if combo.alt {
            mods.push(KEY_LEFTALT);
        }

        for m in &mods {
            self.event(EV_KEY, *m, 1)?;
            self.event(EV_SYN, 0, SYN_REPORT)?;
        }
        self.event(EV_KEY, combo.key, 1)?;
        self.event(EV_SYN, 0, SYN_REPORT)?;
        // short delay so apps do not coalesce the sequence into an autorepeat
        std::thread::sleep(Duration::from_millis(20));
        self.event(EV_KEY, combo.key, 0)?;
        self.event(EV_SYN, 0, SYN_REPORT)?;
        for m in mods.iter().rev() {
            self.event(EV_KEY, *m, 0)?;
            self.event(EV_SYN, 0, SYN_REPORT)?;
        }
        Ok(())
    }
}

impl Drop for Keyboard {
    fn drop(&mut self) {
        unsafe {
            libc::ioctl(self.fd, UI_DEV_DESTROY as libc::c_ulong);
            libc::close(self.fd);
        }
    }
}

fn clipboard_cmd(cmd: &str, args: &[&str], input: Option<&str>) -> Option<String> {
    let mut child = Command::new(cmd)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;
    if let Some(text) = input {
        if let Some(mut stdin) = child.stdin.take() {
            let _ = stdin.write_all(text.as_bytes());
        }
    }
    let out = child.wait_with_output().ok()?;
    if out.status.success() {
        Some(String::from_utf8_lossy(&out.stdout).to_string())
    } else {
        Some(String::new())
    }
}

/// Кладёт текст в буфер обмена и посылает клавиши вставки; старый буфер возвращает.
pub fn paste_text(text: &str, combo: Combo, restore_after: Duration) -> Result<()> {    let previous = clipboard_cmd("wl-paste", &["--no-newline", "--type", "text"], None);
    clipboard_cmd("wl-copy", &["--type", "text"], Some(text));

    let mut kb = Keyboard::new().context("не создал виртуальную клавиатуру (uinput)")?;
    kb.send(combo)?;
    drop(kb);

    if let Some(prev) = previous {
        if !prev.is_empty() {
            std::thread::sleep(restore_after);
            clipboard_cmd("wl-copy", &["--type", "text"], Some(&prev));
        }
    }
    Ok(())
}

/// Вставить распознанный текст согласно inject_mode.
pub fn insert_text(text: &str, mode: &str, combo: Combo, restore_after: Duration) -> Result<()> {
    match mode {
        "clipboard" => {
            clipboard_cmd("wl-copy", &["--type", "text"], Some(text));
            Ok(())
        }
        "stdout" => {
            println!("{text}");
            Ok(())
        }
        _ => paste_text(text, combo, restore_after),
    }
}

/// Диагностика: `dictactl test-keys ctrl+v`
pub fn test_keys(combo_str: &str) -> Result<()> {
    let combo = Combo::parse(combo_str)?;
    let started = Instant::now();
    let mut kb = Keyboard::new()?;
    kb.send(combo)?;
    eprintln!("послано {combo_str} за {:?}", started.elapsed());
    Ok(())
}

//! Глобальная горячая клавиша через evdev (/dev/input/event*).
//!
//! Читает все доступные устройства ввода, слушает указанную комбинацию и
//! вызывает колбэк на нажатие. Работает на любом композиторе (KWin, Hyprland,
//! sway, GNOME), не требует X11/D-Bus/скриптов DE. Требует права на чтение
//! /dev/input/event*: группа `input` или udev-правило с ACL.
//!
//! Требует std (используется из dictad).

use std::collections::HashMap;
use std::os::unix::io::RawFd;

const EV_KEY: u16 = 1;

const MODS: &[(&str, u16)] = &[
    ("ctrl", 29),
    ("rctrl", 97),
    ("alt", 56),
    ("ralt", 100),
    ("shift", 42),
    ("rshift", 54),
    ("meta", 125),
    ("super", 125),
    ("win", 125),
    ("rmeta", 126),
];

/// имя клавиши -> linux keycode
pub fn keycode(name: &str) -> Option<u16> {
    let n = name.trim().to_ascii_lowercase();
    if let Some((_, code)) = MODS.iter().find(|(k, _)| *k == n) {
        return Some(*code);
    }
    let b = n.as_bytes();
    if b.len() == 1 {
        let c = b[0];
        if c.is_ascii_lowercase() {
            // KEY_A = 30 ... KEY_Z = 44
            return Some(30 + (c - b'a') as u16);
        }
        if c.is_ascii_digit() {
            // KEY_1 = 2 ... KEY_9 = 10, KEY_0 = 11
            return Some(if c == b'0' { 11 } else { 1 + (c - b'1') as u16 });
        }
    }
    match n.as_str() {
        "space" => Some(57),
        "enter" | "return" => Some(36),
        "tab" => Some(15),
        "esc" | "escape" => Some(1),
        "printscreen" | "prtsc" => Some(211),
        "f1" => Some(59),
        "f2" => Some(60),
        "f3" => Some(61),
        "f4" => Some(62),
        "f5" => Some(63),
        "f6" => Some(64),
        "f7" => Some(65),
        "f8" => Some(66),
        "f9" => Some(67),
        "f10" => Some(68),
        "f11" => Some(69),
        "f12" => Some(70),
        _ => None,
    }
}

pub struct Combo {
    pub key: u16,
    pub mods: Vec<u16>,
}

pub fn parse_combo(spec: &str) -> Option<Combo> {
    let mut key = None;
    let mut mods: Vec<u16> = Vec::new();
    for part in spec.split('+') {
        let code = keycode(part)?;
        if is_modifier(code) {
            mods.push(code);
        } else {
            key = Some(code);
        }
    }
    Some(Combo { key: key?, mods })
}

fn is_modifier(code: u16) -> bool {
    matches!(code, 29 | 97 | 56 | 100 | 42 | 54 | 125 | 126)
}

fn is_mod_held(code: u16, held: &HashMap<u16, i32>) -> bool {
    // учитываем левый/правый варианты одного модификатора
    let partners: &[u16] = match code {
        29 | 97 => &[29, 97],
        56 | 100 => &[56, 100],
        42 | 54 => &[42, 54],
        125 | 126 => &[125, 126],
        _ => &[code],
    };
    partners
        .iter()
        .any(|p| held.get(p).copied().unwrap_or(0) > 0)
}

/// Открыть все устройства ввода, которые реально можно читать.
fn open_devices() -> Vec<(RawFd, String)> {
    let mut out = Vec::new();
    let dir = match std::fs::read_dir("/dev/input") {
        Ok(d) => d,
        Err(_) => return out,
    };
    for entry in dir.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if !name.starts_with("event") {
            continue;
        }
        let path = format!("/dev/input/{name}");
        let c = match std::ffi::CString::new(path.clone()) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let fd = unsafe { libc::open(c.as_ptr(), libc::O_RDONLY | libc::O_NONBLOCK) };
        if fd >= 0 {
            out.push((fd, name));
        }
    }
    out
}

/// Блокирующий цикл: слушать combo, вызывать on_press при полном совпадении.
pub fn listen(spec: &str, mut on_press: impl FnMut()) -> std::io::Result<()> {
    let combo = parse_combo(spec).ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("не клавиша: {spec}"),
        )
    })?;
    let devices = open_devices();
    if devices.is_empty() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "нет доступа к /dev/input/event* (нужна группа input или udev-ACL)",
        ));
    }
    eprintln!(
        "dictad: горячая клавиша '{}' слушается на {} устройствах ввода",
        spec,
        devices.len()
    );

    let mut fds: Vec<libc::pollfd> = devices
        .iter()
        .map(|(fd, _)| libc::pollfd {
            fd: *fd,
            events: libc::POLLIN,
            revents: 0,
        })
        .collect();
    let mut held: HashMap<u16, i32> = HashMap::new();
    let mut fired = false;

    loop {
        let n = unsafe { libc::poll(fds.as_mut_ptr(), fds.len() as libc::nfds_t, -1) };
        if n < 0 {
            let err = std::io::Error::last_os_error();
            if err.raw_os_error() == Some(libc::EINTR) {
                continue;
            }
            return Err(err);
        }
        for pf in fds.iter_mut() {
            if pf.revents & (libc::POLLIN | libc::POLLERR | libc::POLLHUP) == 0 {
                continue;
            }
            pf.revents = 0;
            let mut buf = [0u8; 24 * 16];
            let got =
                unsafe { libc::read(pf.fd, buf.as_mut_ptr() as *mut libc::c_void, buf.len()) };
            if got <= 0 {
                continue;
            }
            for chunk in buf[..got as usize].chunks_exact(24) {
                // struct input_event: i64 sec, i64 usec, u16 type, u16 code, i32 value
                let kind = u16::from_ne_bytes([chunk[16], chunk[17]]);
                let code = u16::from_ne_bytes([chunk[18], chunk[19]]);
                let value = i32::from_ne_bytes([chunk[20], chunk[21], chunk[22], chunk[23]]);
                if kind != EV_KEY {
                    continue;
                }
                held.insert(code, value);
                if value == 1 && code == combo.key {
                    let mods_ok = combo.mods.iter().all(|m| is_mod_held(*m, &held));
                    if mods_ok && !fired {
                        fired = true;
                        on_press();
                    }
                }
                if code == combo.key && value == 0 {
                    fired = false;
                }
            }
        }
    }
}

//! Глобальная горячая клавиша Windows: RegisterHotKey с hWnd=NULL.
//!
//! WM_HOTKEY приходит в очередь ВЫДЕЛЕННОГО потока демона — окно не нужно
//! вовсе (MS-семантика; locked 8). Спека парсится чистой функцией, занятая
//! комбинация даёт честную ошибку с GetLastError, а не молчание.

use std::io::{Error, ErrorKind, Result};

use windows::Win32::Foundation::GetLastError;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    RegisterHotKey, HOT_KEY_MODIFIERS, VIRTUAL_KEY, MOD_ALT, MOD_CONTROL, MOD_NOREPEAT, MOD_SHIFT,
    MOD_WIN, VK_ESCAPE, VK_F1, VK_INSERT, VK_RETURN, VK_SPACE, VK_TAB,
};
use windows::Win32::UI::WindowsAndMessaging::{GetMessageW, MSG, WM_HOTKEY};

/// Парсер спеки hotkey.rs-формата → (MOD_* | MOD_NOREPEAT, VK-код).
/// Чистая функция: ни одного Win32-вызова.
fn parse_windows_combo(spec: &str) -> Option<(HOT_KEY_MODIFIERS, u32)> {
    let mut mods = HOT_KEY_MODIFIERS(0);
    let mut key: Option<VIRTUAL_KEY> = None;

    for raw in spec.split('+').map(str::trim).filter(|p| !p.is_empty()) {
        let part = raw.to_ascii_lowercase();
        match part.as_str() {
            "ctrl" | "control" => mods |= MOD_CONTROL,
            "shift" => mods |= MOD_SHIFT,
            "alt" => mods |= MOD_ALT,
            // Win-клавиша зарезервирована ОС: registration может не удаться —
            // это документированное поведение, не баг (Pitfall 6).
            "meta" | "super" | "win" => mods |= MOD_WIN,
            "space" => key = Some(VK_SPACE),
            "enter" | "return" => key = Some(VK_RETURN),
            "insert" | "ins" => key = Some(VK_INSERT),
            "tab" => key = Some(VK_TAB),
            "esc" | "escape" => key = Some(VK_ESCAPE),
            single if single.len() == 1 => {
                let b = single.as_bytes()[0];
                let vk = if b.is_ascii_lowercase() {
                    VIRTUAL_KEY(b.to_ascii_uppercase() as u16)
                } else if b.is_ascii_digit() {
                    VIRTUAL_KEY(b as u16)
                } else {
                    return None;
                };
                key = Some(vk);
            }
            function => {
                let n = function.strip_prefix('f')?.parse::<u16>().ok()?;
                if n == 0 || n > 12 {
                    return None;
                }
                key = Some(VIRTUAL_KEY(VK_F1.0 + n - 1));
            }
        }
    }

    let vk = key?;
    Some((mods | MOD_NOREPEAT, vk.0 as u32))
}

/// Блокирующий цикл: зарегистрировать комбинацию в ЭТОМ потоке и вызывать
/// on_press на WM_HOTKEY. Ни message-only, ни скрытого окна не создаём.
pub fn listen(spec: &str, mut on_press: impl FnMut() + Send + 'static) -> Result<()> {
    let (mods, vk) = parse_windows_combo(spec).ok_or_else(|| {
        Error::new(ErrorKind::InvalidInput, format!("не клавиша: {spec}"))
    })?;

    unsafe {
        RegisterHotKey(None, 1, mods, vk).map_err(|err| {
            Error::new(
                ErrorKind::PermissionDenied,
                format!(
                    "комбинация '{spec}' занята или недоступна (GetLastError={})",
                    err.code().0
                ),
            )
        })?;
    }

    let mut msg = MSG::default();
    loop {
        let outcome = unsafe { GetMessageW(&mut msg, None, 0, 0) };
        if outcome.0 == -1 {
            let code = unsafe { GetLastError() };
            return Err(Error::new(
                ErrorKind::Other,
                format!("GetMessageW не удался: {code:?}"),
            ));
        }
        if outcome.0 == 0 {
            break;
        }
        if msg.message == WM_HOTKEY && msg.wParam.0 as u32 == 1 {
            on_press();
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use windows::Win32::UI::Input::KeyboardAndMouse::{VK_M, VK_V};

    #[test]
    fn parses_alt_v_into_mod_and_vk() {
        let (mods, vk) = parse_windows_combo("alt+v").expect("alt+v");
        assert_eq!(mods.0, (MOD_ALT | MOD_NOREPEAT).0);
        assert_eq!(vk, VK_V.0 as u32);
    }

    #[test]
    fn parses_multiple_modifiers_and_function_key() {
        let (mods, vk) = parse_windows_combo("ctrl+shift+f2").expect("ctrl+shift+f2");
        assert_eq!(mods.0, (MOD_CONTROL | MOD_SHIFT | MOD_NOREPEAT).0);
        assert_eq!(vk, VK_F1.0 as u32 + 1);
    }

    #[test]
    fn parses_win_modifier_and_bare_key() {
        let (mods, vk) = parse_windows_combo("win+m").expect("win+m");
        assert_eq!(mods.0, (MOD_WIN | MOD_NOREPEAT).0);
        assert_eq!(vk, VK_M.0 as u32);
        let (bare_mods, bare_vk) = parse_windows_combo("v").expect("bare v");
        assert_eq!(bare_mods.0, MOD_NOREPEAT.0);
        assert_eq!(bare_vk, VK_V.0 as u32);
    }

    #[test]
    fn rejects_unknown_spec() {
        assert!(parse_windows_combo("xyz").is_none());
        assert!(parse_windows_combo("f13").is_none());
        assert!(parse_windows_combo("").is_none());
    }
}

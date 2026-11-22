//! Windows-транспорт демона: именованный канал `\\.\pipe\dicta`.
//!
//! DACL-контракт (locked 6): `D:P(A;;GA;;;SY)(A;;GA;;;<owner-SID>)` — SYSTEM
//! и ТОЛЬКО текущий пользователь. Пайп создаёт сервер, клиент (Flutter) только
//! подключается к полному имени без авто-префикса (PITFALL 7).

use std::io::BufReader;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, AtomicUsize};
use std::sync::{Arc, Mutex};
use std::thread;

use anyhow::{Context, Result};
use interprocess::os::windows::named_pipe::{pipe_mode, PipeListenerOptions, PipeStream};
use interprocess::os::windows::security_descriptor::SecurityDescriptor;
use interprocess::TryClone;
use widestring::U16CString;
use windows::core::PWSTR;
use windows::Win32::Foundation::{CloseHandle, GetLastError, HANDLE, HLOCAL, LocalFree};
use windows::Win32::Security::Authorization::ConvertSidToStringSidW;
use windows::Win32::Security::{
    GetTokenInformation, TokenUser, TOKEN_QUERY, TOKEN_USER,
};
use windows::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};

use crate::Handler;

/// env-шов теста (паритет DICTA_SOCK/DICTA_PIPE фазы 6): пустое значение
/// игнорируется, чтобы случайная пустая переменная не сломала дефолтный канал.
/// Цепочка: DICTA_PIPE → DEFAULT_PIPE_NAME. Настройка cfg.socket (unix-only)
/// в Windows-пути НЕ участвует — один источник истины (WR-02).
pub fn resolve_pipe_name() -> String {
    std::env::var_os("DICTA_PIPE")
        .map(|v| v.to_string_lossy().into_owned())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| dicta_core::config::DEFAULT_PIPE_NAME.to_string())
}

/// Паритет unix-пути (main.rs:161-171): если к пайпу уже можно подключиться,
/// другой dictad уже слушает. Именованные пайпы multi-instance — без пробы
/// клиенты попадали бы в произвольный инстанс с гонками состояния/watchdog.
fn pipe_already_listening(pipe_name: &str) -> bool {
    let Ok(path) = U16CString::from_str(pipe_name) else {
        return false;
    };
    PipeStream::<pipe_mode::Bytes, pipe_mode::Bytes>::connect_by_path(path.as_ucstr()).is_ok()
}

/// Строка SDDL: SYSTEM + владелец-пользователь, больше никого.
pub fn pipe_security_descriptor(sid: &str) -> String {
    format!("D:P(A;;GA;;;SY)(A;;GA;;;{sid})")
}

/// SID текущего процесса: OpenProcessToken → GetTokenInformation(TokenUser) →
/// ConvertSidToStringSidW. Ошибки — честные русские строки.
fn current_user_sid_string() -> Result<String> {
    unsafe {
        let mut token = HANDLE::default();
        OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token)
            .map_err(|e| anyhow::anyhow!("OpenProcessToken: {e}"))?;

        let mut needed = 0u32;
        // Первый вызов ожидаемо падает с insufficient buffer и возвращает размер.
        let _ = GetTokenInformation(token, TokenUser, None, 0, &mut needed);
        if needed == 0 {
            let code = GetLastError();
            anyhow::bail!("GetTokenInformation(TokenUser) не вернул размер: {code:?}");
        }
        let mut buf = vec![0u8; needed as usize];
        GetTokenInformation(
            token,
            TokenUser,
            Some(buf.as_mut_ptr() as *mut std::ffi::c_void),
            needed,
            &mut needed,
        )
        .map_err(|e| anyhow::anyhow!("GetTokenInformation(TokenUser): {e}"))?;
        // Токен больше не нужен: SID уже скопирован в buf (IN-04).
        let _ = CloseHandle(token);

        // Vec<u8> не гарантирует выравнивание под TOKEN_USER — читаем копию,
        // а не ссылку (иначе UB на не-x64, IN-04).
        let user = (buf.as_ptr() as *const TOKEN_USER).read_unaligned();
        let mut sid_wide = PWSTR::null();
        ConvertSidToStringSidW(user.User.Sid, &mut sid_wide)
            .map_err(|e| anyhow::anyhow!("ConvertSidToStringSidW: {e}"))?;
        let sid = sid_wide
            .to_string()
            .context("SID текущего пользователя не UTF-16")?;
        let _ = LocalFree(HLOCAL(sid_wide.0 as *mut std::ffi::c_void));
        Ok(sid)
    }
}

/// Accept-цикл поверх named pipe: одна команда = одно соединение = один тред.
pub fn serve(
    pipe_name: &str,
    handler: Arc<Mutex<Handler>>,
    state: Arc<AtomicUsize>,
    level: Arc<AtomicU32>,
    started_at: Arc<AtomicU64>,
    panel: Arc<AtomicBool>,
    reloading: Arc<AtomicBool>,
) -> Result<()> {
    let sid = current_user_sid_string()?;
    let sddl = pipe_security_descriptor(&sid);
    let sddl_wide = U16CString::from_str(&sddl)
        .map_err(|e| anyhow::anyhow!("SDDL содержит NUL: {e}"))?;
    let descriptor = SecurityDescriptor::deserialize(sddl_wide.as_ucstr())
        .context("не удалось разобрать SDDL пайпа (DACL)")?;

    // Полное имя: interprocess НЕ добавляет `\\.\pipe\` сам (PITFALL 7).
    let path_wide = U16CString::from_str(pipe_name)
        .map_err(|e| anyhow::anyhow!("имя пайпа содержит NUL: {e}"))?;
    // Второй инстанс не должен молча сосуществовать с первым (паритет unix).
    if pipe_already_listening(pipe_name) {
        anyhow::bail!("на {pipe_name} уже кто-то слушает — сначала останови старый dictad");
    }
    let listener = PipeListenerOptions::new()
        .path(path_wide.as_ucstr())
        .security_descriptor(Some(descriptor))
        .create_duplex::<pipe_mode::Bytes>()
        .with_context(|| format!("не могу создать пайп {pipe_name}"))?;

    for stream in listener.incoming() {
        let Ok(stream) = stream else { continue };
        let handler = handler.clone();
        let state = state.clone();
        let level = level.clone();
        let started_at = started_at.clone();
        let panel = panel.clone();
        let reloading = reloading.clone();
        thread::spawn(move || {
            let reader_stream = match stream.try_clone() {
                Ok(s) => s,
                Err(err) => {
                    eprintln!("dictad: {err}");
                    return;
                }
            };
            if let Err(err) = crate::serve_conn(
                BufReader::new(reader_stream),
                stream,
                handler,
                state,
                level,
                started_at,
                panel,
                reloading,
                PathBuf::new(),
            ) {
                eprintln!("dictad: {err}");
            }
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const FAKE_SID: &str = "S-1-5-21-111111111-222222222-333333333-1001";

    #[test]
    fn sddl_is_system_plus_owner_only() {
        let sddl = pipe_security_descriptor(FAKE_SID);
        assert!(sddl.starts_with("D:P"), "{sddl}");
        assert!(sddl.contains("A;;GA;;;SY"), "{sddl}");
        assert!(sddl.contains(FAKE_SID), "{sddl}");
        // Никаких общих ACE (everyone / все приложения) — locked 6, must-NOT 7.
        assert!(!sddl.contains("WD"), "{sddl}");
        assert!(!sddl.contains("S-1-15-2"), "{sddl}");
        assert!(!sddl.contains("AU"), "{sddl}");
    }

    #[test]
    fn occupied_pipe_is_detected_before_bind() {
        let name = format!(r"\\.\pipe\dicta-probe-{}", std::process::id());
        // Пока никто не слушает — проба тиха.
        assert!(!pipe_already_listening(&name));
        let wide = U16CString::from_str(&name).expect("pipe name");
        let listener = PipeListenerOptions::new()
            .path(wide.as_ucstr())
            .create_duplex::<pipe_mode::Bytes>()
            .expect("test listener");
        // Появился слушатель — проба обязана это увидеть: второй dictad упадёт.
        assert!(pipe_already_listening(&name));
        drop(listener);
    }
}

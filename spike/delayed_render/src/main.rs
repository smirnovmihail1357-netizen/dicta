//! ЭКСПЕРИМЕНТ (НЕ production): delayed render буфера обмена Win32.
//!
//! Цель — получить ЖИВЫЕ замеры для 07-SPIKE-RESULT.md ДО написания
//! production-вставки inject_win (Must-NOT 8 / locked 5). Бин печатает в stdout
//! построчно; пользователь копирует строки в таблицу result-файла.
//!
//! Никакого текста диктовки в логах: печатаем только длины, id форматов,
//! времена и флаги. Чистый Win32 (windows-rs 0.58) — без enigo/interprocess/cpal.

use std::env;
use std::ffi::c_void;
use std::mem::size_of;
use std::sync::Mutex;
use std::time::Instant;

use windows::core::*;
use windows::Win32::Foundation::{GetLastError, HANDLE, HGLOBAL, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::DataExchange::{
    CloseClipboard, EmptyClipboard, GetClipboardData, GetClipboardOwner,
    GetClipboardSequenceNumber, IsClipboardFormatAvailable, OpenClipboard,
    RegisterClipboardFormatW, SetClipboardData,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::System::Memory::{GlobalAlloc, GlobalLock, GlobalSize, GlobalUnlock, GMEM_MOVEABLE};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYBD_EVENT_FLAGS, KEYEVENTF_KEYUP,
    VIRTUAL_KEY, VK_CONTROL, VK_V,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW, SendMessageW, SetTimer,
    TranslateMessage, HWND_MESSAGE, MSG, PostQuitMessage, RegisterClassW, WINDOW_EX_STYLE,
    WM_DESTROY, WM_RENDERALLFORMATS, WM_RENDERFORMAT, WM_TIMER, WNDCLASSW, WS_OVERLAPPED,
};

/// CF_UNICODETEXT из Win32 (Ole-модуль windows-rs требует лишнего feature —
/// здесь числовой константы достаточно и она стабильна с Win3.1).
const CF_UNICODETEXT: u32 = 13;
/// Метка синтезированных SendInput-событий: свой хоткей не должен на них реагировать.
const SYNTH_TAG: usize = 0xD1C7A;
const CLASS_NAME: PCWSTR = w!("DictaDelayedRenderSpike");

/// Четыре оптаут-формата — ИМЕНА ТОЧНО по RESEARCH §5 / PITFALLS:233.
/// Need: Win+V история / облако MS / monitor processing / viewer ignore.
const OPT_OUT_NAMES: [PCWSTR; 4] = [
    w!("ExcludeClipboardContentFromMonitorProcessing"),
    w!("CanIncludeInClipboardHistory"),
    w!("CanUploadToCloudClipboard"),
    w!("Clipboard Viewer Ignore"),
];

#[derive(Default)]
struct Measurements {
    t0: Option<Instant>,
    render_count: u32,
    first_render_us: Option<u128>,
    first_render_t2_us: Option<u128>,
    all_formats_count: u32,
    events: Vec<String>,
}

static STATE: Mutex<Measurements> = Mutex::new(Measurements {
    t0: None,
    render_count: 0,
    first_render_us: None,
    first_render_t2_us: None,
    all_formats_count: 0,
    events: Vec::new(),
});
/// Текст только в памяти; в логи НЕ попадает.
static PAYLOAD: Mutex<Vec<u16>> = Mutex::new(Vec::new());
/// Все зарегистрированные форматы (для WM_RENDERALLFORMATS).
static FORMATS: Mutex<Vec<u32>> = Mutex::new(Vec::new());

fn to_utf16(text: &str) -> Vec<u16> {
    let mut wide: Vec<u16> = text.encode_utf16().collect();
    wide.push(0);
    wide
}

fn synth_key(vk: VIRTUAL_KEY, up: bool) -> INPUT {
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: vk,
                wScan: 0,
                dwFlags: if up {
                    KEYEVENTF_KEYUP
                } else {
                    KEYBD_EVENT_FLAGS::default()
                },
                time: 0,
                dwExtraInfo: SYNTH_TAG,
            },
        },
    }
}

/// Реальный рендер: выделить HGLOBAL, положить UTF-16, отдать SetClipboardData.
/// Возвращает строку-статус для лога (без содержимого текста).
unsafe fn render_format(fmt: u32) -> String {
    let payload = PAYLOAD.lock().unwrap().clone();
    let bytes = payload.len() * size_of::<u16>();
    let hg = match GlobalAlloc(GMEM_MOVEABLE, bytes) {
        Ok(h) => h,
        Err(e) => return format!("ALLOC-ERR:{e:?}"),
    };
    let ptr = GlobalLock(hg) as *mut u16;
    if !ptr.is_null() {
        std::ptr::copy_nonoverlapping(payload.as_ptr(), ptr, payload.len());
        let _ = GlobalUnlock(hg);
    }
    // Владение HGLOBAL переходит системе — НЕ освобождаем после успеха.
    match SetClipboardData(fmt, HANDLE(hg.0)) {
        Ok(_) => "OK".to_string(),
        // windows-rs считает NULL invalid → Err даже там, где Win32 вернул
        // NULL «на успех» promise. Для реального хендла Err = настоящий отказ.
        Err(e) => format!("ERR:{e:?}"),
    }
}

unsafe extern "system" fn clip_wndproc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_RENDERFORMAT => {
            let fmt = wparam.0 as u32;
            let t0 = STATE.lock().unwrap().t0;
            let started = Instant::now();
            let dt_us = t0
                .map(|t| started.duration_since(t).as_micros())
                .unwrap_or_default();
            let res = render_format(fmt);
            let t2_us = t0.map(|t| t.elapsed().as_micros()).unwrap_or_default();

            let mut st = STATE.lock().unwrap();
            st.render_count += 1;
            let ordinal = st.render_count;
            if st.first_render_us.is_none() {
                st.first_render_us = Some(dt_us);
                st.first_render_t2_us = Some(t2_us);
            }
            let line = format!(
                "WM_RENDERFORMAT fmt={fmt} render#{ordinal} t1-t0={dt_us}us t2={t2_us}us res={res}"
            );
            println!("SPIKE | {line}");
            st.events.push(line);
            LRESULT(0)
        }
        WM_RENDERALLFORMATS => {
            let fmts = FORMATS.lock().unwrap().clone();
            let mut rendered = 0u32;
            for fmt in fmts {
                let res = render_format(fmt);
                if res == "OK" {
                    rendered += 1;
                }
                let line = format!("WM_RENDERALLFORMATS fmt={fmt} res={res}");
                println!("SPIKE | {line}");
                STATE.lock().unwrap().events.push(line);
            }
            STATE.lock().unwrap().all_formats_count += rendered;
            LRESULT(0)
        }
        WM_TIMER if wparam.0 == 1 => {
            // Самотест замера 8: WM_RENDERALLFORMATS обязан отрендерить ВСЁ,
            // иначе данные теряются при потере владельца буфера.
            let _ = SendMessageW(hwnd, WM_RENDERALLFORMATS, WPARAM(0), LPARAM(0));
            print_summary();
            PostQuitMessage(0);
            LRESULT(0)
        }
        WM_DESTROY => {
            PostQuitMessage(0);
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

fn print_summary() {
    let st = STATE.lock().unwrap();
    println!("SPIKE | === 5s SETTLE LOG (замер 5) ===");
    for event in &st.events {
        println!("SPIKE | event: {event}");
    }
    let first_dt = st
        .first_render_us
        .map(|v| v.to_string())
        .unwrap_or_else(|| "НЕ БЫЛО".to_string());
    let first_t2 = st
        .first_render_t2_us
        .map(|v| v.to_string())
        .unwrap_or_else(|| "НЕ БЫЛО".to_string());
    println!(
        "SPIKE | summary: render_count={} first_t1_minus_t0_us={first_dt} first_t2_us={first_t2} all_formats_rendered={} sequence={}",
        st.render_count,
        st.all_formats_count,
        unsafe { GetClipboardSequenceNumber() },
    );
    println!("SPIKE | END");
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let flag = |name: &str, default: &str| -> String {
        args.iter()
            .find_map(|a| a.strip_prefix(name).map(str::to_string))
            .unwrap_or_else(|| default.to_string())
    };
    let mode = flag("--mode=", "user");
    let window_kind = flag("--window=", "message-only");
    let text = flag("--text=", "spike-probe");

    if mode != "user" && mode != "send" {
        println!("SPIKE | режим должен быть user или send");
        return Ok(());
    }
    if window_kind != "message-only" && window_kind != "hidden" {
        println!("SPIKE | окно должно быть message-only или hidden");
        return Ok(());
    }

    println!("SPIKE | === delayed render experiment (НЕ production) ===");
    println!("SPIKE | window={window_kind} mode={mode} (содержимое текста НЕ логируется)");

    let payload = to_utf16(&text);
    println!("SPIKE | payload: len_utf16={}", payload.len());
    *PAYLOAD.lock().unwrap() = payload;

    unsafe {
        let hinstance = GetModuleHandleW(None)?;
        let wc = WNDCLASSW {
            lpfnWndProc: Some(clip_wndproc),
            hInstance: hinstance.into(),
            lpszClassName: CLASS_NAME,
            ..Default::default()
        };
        if RegisterClassW(&wc) == 0 {
            println!("SPIKE | RegisterClassW failed");
            return Err(Error::from_win32());
        }
        // Замер 7: message-only (HWND_MESSAGE) vs скрытое WS_OVERLAPPED без WS_VISIBLE.
        let parent = if window_kind == "message-only" {
            HWND_MESSAGE
        } else {
            HWND::default()
        };
        let hwnd = CreateWindowExW(
            WINDOW_EX_STYLE(0),
            CLASS_NAME,
            w!("dicta delayed render spike"),
            WS_OVERLAPPED,
            0,
            0,
            0,
            0,
            parent,
            None,
            hinstance,
            None,
        )?;
        println!("SPIKE | owner window: {window_kind}, hwnd={hwnd:?}");

        OpenClipboard(hwnd)?;
        // Замер 1: baseline — только длина, НЕ содержимое (приватность даже в спике).
        if let Ok(handle) = GetClipboardData(CF_UNICODETEXT) {
            let hg = HGLOBAL(handle.0);
            let cap = GlobalSize(hg) / size_of::<u16>();
            let ptr = GlobalLock(hg) as *const u16;
            let mut len = 0usize;
            if !ptr.is_null() {
                while len < cap && *ptr.add(len) != 0 {
                    len += 1;
                }
                let _ = GlobalUnlock(hg);
            }
            println!("SPIKE | baseline 1: CF_UNICODETEXT present, len_utf16={len} (контент не печатаем)");
        } else {
            println!("SPIKE | baseline 1: CF_UNICODETEXT отсутствует");
        }
        EmptyClipboard()?;

        // Замер 2/3: promise + четыре оптаут-формата.
        let t0 = Instant::now();
        STATE.lock().unwrap().t0 = Some(t0);
        let promise = SetClipboardData(CF_UNICODETEXT, HANDLE::default());
        println!(
            "SPIKE | promise 2: SetClipboardData(CF_UNICODETEXT,NULL) -> {:?} — Err здесь ЭКВИВАЛЕНТ Win32-успеха (NULL на успехе), успех проверяем ниже",
            promise.as_ref().map(|h| h.0 as *const c_void)
        );
        let gle = GetLastError();
        println!("SPIKE | check 3: GetLastError={} (протухший — не индикатор успеха)", gle.0);
        println!(
            "SPIKE | check 3: IsClipboardFormatAvailable(CF_UNICODETEXT)={}",
            IsClipboardFormatAvailable(CF_UNICODETEXT).is_ok()
        );
        let owner = GetClipboardOwner().unwrap_or_default();
        println!(
            "SPIKE | check 3: GetClipboardOwner={owner:?} expected_hwnd={hwnd:?} match={}",
            owner == hwnd
        );

        for (i, name) in OPT_OUT_NAMES.iter().enumerate() {
            let fmt = RegisterClipboardFormatW(*name);
            let res = SetClipboardData(fmt, HANDLE::default());
            FORMATS.lock().unwrap().push(fmt);
            println!(
                "SPIKE | optout 2[{i}]: fmt={fmt} promise_null_ok={} res={:?}",
                res.is_err(),
                res.as_ref().map(|_| "handle").unwrap_or("null-as-success")
            );
        }
        FORMATS.lock().unwrap().push(CF_UNICODETEXT);
        CloseClipboard()?;

        if mode == "send" {
            let events = [
                synth_key(VK_CONTROL, false),
                synth_key(VK_V, false),
                synth_key(VK_V, true),
                synth_key(VK_CONTROL, true),
            ];
            let sent = SendInput(&events, size_of::<INPUT>() as i32);
            println!("SPIKE | insert 4: SendInput sent={sent}/4 dwExtraInfo={SYNTH_TAG:#x}");
        } else {
            println!("SPIKE | insert 4: режим user — нажмите Ctrl+V в целевом окне");
        }
        println!("SPIKE | замер 9: для второго способа дернуть рендер выполните в PowerShell: Get-Clipboard");

        SetTimer(hwnd, 1, 5000, None);
        println!("SPIKE | жду 5 с тишины после вставки (замер 5; потолок clipboard_restore_ms=2500)");

        let mut msg = MSG::default();
        loop {
            let outcome = GetMessageW(&mut msg, None, 0, 0);
            if outcome.0 <= 0 {
                break;
            }
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
    Ok(())
}

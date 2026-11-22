//! Живой end-to-end tracer: клиент → named pipe → serve_conn → JSON status.
//!
//! Тест гоняется ТОЛЬКО на windows-раннере CI (ветка windows, job rust): демон
//! живёт в headless-окружении, модель не нужна — `status` не грузит engine.

#![cfg(windows)]

use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use interprocess::os::windows::named_pipe::{pipe_mode, PipeStream};
use widestring::U16CString;

/// Демон и temp-конфиг обязательно умирают вместе с тестом, даже при panic.
struct DaemonGuard {
    child: Child,
    config: PathBuf,
}

impl Drop for DaemonGuard {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
        // Конфиг по pid мог остаться при падении теста — убираем всегда (IN-08).
        let _ = std::fs::remove_file(&self.config);
    }
}

#[test]
fn status_roundtrip_over_named_pipe() {
    let pipe_name = format!(r"\\.\pipe\dicta-test-{}", std::process::id());
    let config = std::env::temp_dir().join(format!("dicta-pipe-test-{}.toml", std::process::id()));
    std::fs::write(&config, "hotkey = \"off\"\n").expect("temp config");

    let child = Command::new(env!("CARGO_BIN_EXE_dictad"))
        .env("DICTA_PIPE", &pipe_name)
        .env("DICTA_CONFIG", &config)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        // Не piped(stderr) без читателя: при росте стартового шума демон
        // упрётся в буфер пайпа и зависнет (IN-08).
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn dictad.exe");
    let _daemon = DaemonGuard { child, config };

    let path = U16CString::from_str(&pipe_name).expect("pipe name");
    let deadline = Instant::now() + Duration::from_secs(2);
    let mut stream = loop {
        match PipeStream::<pipe_mode::Bytes, pipe_mode::Bytes>::connect_by_path(path.as_ucstr()) {
            Ok(stream) => break stream,
            Err(err) => {
                if Instant::now() >= deadline {
                    panic!("dictad не поднял пайп {pipe_name} за 2 с: {err}");
                }
                std::thread::sleep(Duration::from_millis(50));
            }
        }
    };

    stream
        .write_all(b"{\"cmd\":\"status\"}\n")
        .expect("write status request");
    stream.flush().expect("flush");
    // Сервер рвёт соединение после status (паритет main.rs:1004-1009).
    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    let got = reader.read_line(&mut line).expect("read status reply");
    assert!(got > 0, "пустой ответ пайпа");

    let reply: serde_json::Value = serde_json::from_str(line.trim()).expect("valid JSON reply");
    assert_eq!(reply["state"], "idle", "{reply}");
    assert!(reply["error"].is_null(), "{reply}");
    assert_eq!(reply["level"], 0.0, "{reply}");
}

//! dictad — единый демон голосового ввода.
//!
//! Слушает Unix-сокет, отвечает на JSON-команды по одной строке:
//!   {"cmd":"toggle"} {"cmd":"start"} {"cmd":"stop"} {"cmd":"status"}
//! Ответ: {"state":"idle|recording|transcribing","level":0..1,"seconds":..,"text":..,"error":..}

use std::io::{BufRead, BufReader, Write};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use dicta_core::inject::{self, Combo};
use dicta_core::{capture::Recorder, Config, Engine};

#[derive(serde::Deserialize)]
struct Req {
    cmd: String,
    /// Опциональная подсказка от адаптера: в какое окно вставлять (konsole, firefox...).
    #[serde(default)]
    app: Option<String>,
    /// Для {"cmd":"inject"}: текст, который нужно вставить в активное окно.
    #[serde(default)]
    text: Option<String>,
    /// Для {"cmd":"inject"}: принудительные клавиши вставки (диагностика раскладок).
    #[serde(default)]
    keys: Option<String>,
}

#[derive(serde::Serialize)]
struct Res<'a> {
    state: &'a str,
    level: f32,
    seconds: f32,
    text: Option<String>,
    error: Option<String>,
}

#[derive(Debug, PartialEq, Clone, Copy)]
enum State {
    Idle,
    Recording,
    Transcribing,
}

impl State {
    fn as_str(self) -> &'static str {
        match self {
            State::Idle => "idle",
            State::Recording => "recording",
            State::Transcribing => "transcribing",
        }
    }
    fn to_index(self) -> usize {
        match self {
            State::Idle => 0,
            State::Recording => 1,
            State::Transcribing => 2,
        }
    }
    fn from_index(v: usize) -> Self {
        match v {
            1 => State::Recording,
            2 => State::Transcribing,
            _ => State::Idle,
        }
    }
}

struct Handler {
    cfg: Config,
    engine: Option<Engine>,
    recorder: Option<Recorder>,
    last_text: Option<String>,
    last_error: Option<String>,
    paste_combo: Combo,
}

fn main() -> Result<()> {
    if std::env::args().any(|a| a == "--install-autostart") {
        return install_autostart();
    }
    let cfg = Config::load(
        std::env::var_os("DICTA_CONFIG")
            .as_deref()
            .map(std::path::Path::new),
    )?;
    let socket = cfg.socket.clone();
    let paste_combo = Combo::parse(&cfg.paste_keys)?;

    if let Some(parent) = socket.parent() {
        std::fs::create_dir_all(parent).ok();
    }
    if socket.exists() {
        // Убиваем上一个 демон, если он оставил сокет (или просим systemd сделать это).
        match std::os::unix::net::UnixStream::connect(&socket) {
            Ok(_) => anyhow::bail!(
                "на {} уже кто-то слушает — сначала останови старый dictad",
                socket.display()
            ),
            Err(_) => {
                std::fs::remove_file(&socket).ok();
            }
        }
    }

    let listener = UnixListener::bind(&socket)
        .with_context(|| format!("не могу создать {}", socket.display()))?;
    let _ = std::fs::set_permissions(&socket, std::fs::Permissions::from_mode(0o600));
    let handler = Arc::new(Mutex::new(Handler {
        cfg: cfg.clone(),
        engine: None,
        recorder: None,
        last_text: None,
        last_error: None,
        paste_combo,
    }));
    let state = Arc::new(AtomicUsize::new(State::Idle.to_index()));
    let level = Arc::new(AtomicU32::new(0));
    let started_at = Arc::new(AtomicU64::new(0));

    // Модель прогреваем сразу: первый диктофон не должен ждать загрузки.
    {
        let handler = handler.clone();
        let cfg = cfg.clone();
        thread::spawn(move || {
            match Engine::load(&cfg) {
                Ok(engine) => {
                    if let Ok(mut h) = handler.lock() {
                        h.engine = Some(engine);
                    }
                }
                Err(err) => {
                    eprintln!("dictad: модель не загрузилась: {err}");
                    if let Ok(mut h) = handler.lock() {
                        h.last_error = Some(err.to_string());
                    }
                }
            }
        });
    }

    // Индикатор уровня для UI (80 мс, как в opencode-плагине).
    {
        let handler = handler.clone();
        let state = state.clone();
        let level = level.clone();
        thread::spawn(move || loop {
            if State::from_index(state.load(Ordering::Relaxed)) == State::Recording {
                if let Ok(h) = handler.try_lock() {
                    if let Some(rec) = &h.recorder {
                        level.store(f32_to_bits(rec.level()), Ordering::Relaxed);
                    }
                }
            } else {
                level.store(0, Ordering::Relaxed);
            }
            thread::sleep(Duration::from_millis(80));
        });
    }

    // uinput-девайс регистрируется в KWin не мгновенно: создаём его сразу при старте,
    // чтобы первая же вставка дошла до окна.
    if cfg.inject_mode == "paste" {
        match inject::warm_up_keyboard() {
            Ok(()) => eprintln!("dictad: виртуальная клавиатура (uinput) готова"),
            Err(err) => eprintln!("dictad: не создал uinput-клавиатуру: {err}"),
        }
    }

    eprintln!(
        "dictad: сокет {}, модель {}, язык {}",
        socket.display(),
        cfg.model.display(),
        cfg.language
    );

    // Встроенная глобальная горячая клавиша (evdev): без скриптов DE,
    // работает в KWin/Hyprland/sway/GNOME одинаково.
    let hotkey_spec = std::env::var("DICTA_HOTKEY").unwrap_or_else(|_| cfg.hotkey.clone());
    if hotkey_spec != "off" {
        let handler = handler.clone();
        let state = state.clone();
        let started_at = started_at.clone();
        thread::spawn(move || {
            let mut backoff = Duration::from_secs(10);
            loop {
                let outcome = dicta_core::hotkey::listen(&hotkey_spec, || {
                    if let Err(err) = do_toggle(&handler, &state, &started_at) {
                        eprintln!("dictad: {err}");
                    }
                });
                match outcome {
                    Ok(()) => break,
                    Err(err) => {
                        eprintln!(
                            "dictad: горячая клавиша '{}' недоступна: {} (повтор через {:?})",
                            hotkey_spec, err, backoff
                        );
                        eprintln!(
                            "dictad: либо добавь пользователя в группу input (или udev-ACL на /dev/input/event*), \
                             либо hotkey = \"off\" и клавишу DE (см. adapters/README.md)"
                        );
                        thread::sleep(backoff);
                        backoff = (backoff * 2).min(Duration::from_secs(900));
                    }
                }
            }
        });
    }

    // По --ui поднимаем оверлей сами: демон + индикатор одной командой.
    if std::env::args().any(|a| a == "--ui") || std::env::var("DICTA_UI").is_ok() {
        thread::spawn(spawn_ui);
    }

    for stream in listener.incoming() {
        let Ok(stream) = stream else { continue };
        let handler = handler.clone();
        let state = state.clone();
        let level = level.clone();
        let started_at = started_at.clone();
        let socket2: PathBuf = socket.clone();
        thread::spawn(move || {
            if let Err(err) = serve_conn(stream, handler, state, level, started_at, socket2) {
                eprintln!("dictad: {err}");
            }
        });
    }

    Ok(())
}

fn f32_to_bits(v: f32) -> u32 {
    v.to_bits()
}
fn bits_to_f32(v: u32) -> f32 {
    f32::from_bits(v)
}

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

fn elapsed_seconds(started_at: &Arc<AtomicU64>, state: &Arc<AtomicUsize>) -> f32 {
    if State::from_index(state.load(Ordering::Relaxed)) != State::Recording {
        return 0.0;
    }
    let start = started_at.load(Ordering::Relaxed);
    if start == 0 {
        return 0.0;
    }
    now_ms().saturating_sub(start) as f32 / 1000.0
}

fn serve_conn(
    stream: UnixStream,
    handler: Arc<Mutex<Handler>>,
    state: Arc<AtomicUsize>,
    level: Arc<AtomicU32>,
    started_at: Arc<AtomicU64>,
    socket: PathBuf,
) -> Result<()> {
    let mut reader = BufReader::new(stream.try_clone()?);
    let mut writer = stream;
    let mut line = String::new();
    while reader.read_line(&mut line)? > 0 {
        let req: Req = serde_json::from_str(line.trim())?;
        let response = match req.cmd.as_str() {
            "status" => Res {
                state: State::from_index(state.load(Ordering::Relaxed)).as_str(),
                level: bits_to_f32(level.load(Ordering::Relaxed)),
                seconds: elapsed_seconds(&started_at, &state),
                text: None,
                error: None,
            },
            "inject" => {
                let text = req.text.clone().unwrap_or_default();
                let mut h = handler.lock().map_err(|_| anyhow::anyhow!("handler отравлен"))?;
                if text.is_empty() || text.chars().count() > 5000 {
                    Res {
                        state: State::Idle.as_str(),
                        level: 0.0,
                        seconds: 0.0,
                        text: None,
                        error: Some("inject: текст пустой или длиннее 5000 символов".into()),
                    }
                } else {
                    // Подсказка окна необязательна: demo сам спросит адаптер/кэш.
                    let app = dicta_core::focus::active_app(req.app.as_deref());
                    let keys = match req.keys.as_deref() {
                        Some(k) => k.to_string(),
                        None => h.cfg.paste_keys_for(app.as_deref()),
                    };
                    let combo = Combo::parse(&keys).unwrap_or_else(|_| h.paste_combo.clone());
                    h.last_text = Some(text.clone());
                    match inject::insert_text(
                        &text,
                        &h.cfg.inject_mode,
                        combo,
                        Duration::from_millis(h.cfg.clipboard_restore_ms),
                        Duration::from_millis(h.cfg.paste_delay_ms),
                    )
                    {
                        Ok(()) => {
                            eprintln!("dictad: inject {keys} в {app:?}: {text:?}");
                            h.last_error = None;
                            Res {
                                state: State::Idle.as_str(),
                                level: 0.0,
                                seconds: 0.0,
                                text: Some(text),
                                error: None,
                            }
                        }
                        Err(err) => {
                            h.last_error = Some(format!("вставка не удалась: {err}"));
                            Res {
                                state: State::Idle.as_str(),
                                level: 0.0,
                                seconds: 0.0,
                                text: None,
                                error: h.last_error.clone(),
                            }
                        }
                    }
                }
            }
            "start" | "stop" | "toggle" | "cancel" => {
                let mut h = handler.lock().map_err(|_| anyhow::anyhow!("handler отравлен"))?;
                let cmd = req.cmd.clone();
                if cmd == "cancel" {
                    h.recorder = None;
                    started_at.store(0, Ordering::Relaxed);
                    state.store(State::Idle.to_index(), Ordering::Relaxed);
                    h.last_error = Some("запись отменена".into());
                } else {
                let want_start = match cmd.as_str() {
                    "start" => true,
                    "stop" => false,
                    _ => State::from_index(state.load(Ordering::Relaxed)) != State::Recording,
                };
                if want_start {
                    run_start(&mut h, &state, &started_at)?;
                } else {
                    run_stop(&mut h, &state, &started_at, req.app.as_deref())?;
                }
                }
                let seconds = h
                    .recorder
                    .as_ref()
                    .map(|r| r.elapsed().as_secs_f32())
                    .unwrap_or(0.0);
                Res {
                    state: State::from_index(state.load(Ordering::Relaxed)).as_str(),
                    level: 0.0,
                    seconds,
                    text: h.last_text.clone(),
                    error: h.last_error.clone(),
                }
            }
            other => Res {
                state: State::from_index(state.load(Ordering::Relaxed)).as_str(),
                level: 0.0,
                seconds: 0.0,
                text: None,
                error: Some(format!("неизвестная команда {other:?}")),
            },
        };

        let json = serde_json::to_string(&response)?;
        writer.write_all(json.as_bytes())?;
        writer.write_all(b"\n")?;
        writer.flush()?;

        // Соединение с "stop" держим до конца вставки; статусные — сразу закрываем.
        line.clear();
        let _ = &socket;
        if req.cmd == "status" {
            break;
        }
    }
    Ok(())
}

fn run_start(h: &mut Handler, state: &Arc<AtomicUsize>, started_at: &Arc<AtomicU64>) -> Result<()> {
    if State::from_index(state.load(Ordering::Relaxed)) == State::Recording {
        return Ok(());
    }
    h.last_error = None;
    h.last_text = None;
    let rec = Recorder::start(&h.cfg)?;
    started_at.store(now_ms(), Ordering::Relaxed);
    h.recorder = Some(rec);
    state.store(State::Recording.to_index(), Ordering::Relaxed);
    Ok(())
}

fn run_stop(
    h: &mut Handler,
    state: &Arc<AtomicUsize>,
    started_at: &Arc<AtomicU64>,
    app_hint: Option<&str>,
) -> Result<()> {
    if State::from_index(state.load(Ordering::Relaxed)) != State::Recording {
        return Ok(());
    }

    state.store(State::Transcribing.to_index(), Ordering::Relaxed);
    let recorder = match h.recorder.take() {
        Some(rec) => rec,
        None => {
            state.store(State::Idle.to_index(), Ordering::Relaxed);
            return Ok(());
        }
    };
    let spoken = recorder.elapsed();
    started_at.store(0, Ordering::Relaxed);

    let samples = match recorder.stop() {
        Ok(s) => s,
        Err(err) => {
            h.last_error = Some(err.to_string());
            state.store(State::Idle.to_index(), Ordering::Relaxed);
            return Ok(());
        }
    };

    // Гейтер тишины: whisper на фоновом шуме сочиняет правдоподобные фразы,
    // и они вставятся в фокусное окно. Не запускаем распознавание на шуме.
    let level = dicta_core::speech_level(&samples);
    let overall = dicta_core::normalized_rms(&samples);
    if level < h.cfg.min_level {
        h.last_error = Some(format!(
            "слишком тихо (уровень {:.3} < {:.2}) — распознавание пропущено",
            level, h.cfg.min_level
        ));
        state.store(State::Idle.to_index(), Ordering::Relaxed);
        eprintln!(
            "dictad: пропуск: записано {:?}, речь {:.3} (в среднем {:.3}) ниже порога {:.2}",
            spoken, level, overall, h.cfg.min_level
        );
        return Ok(());
    }

    if h.engine.is_none() {
        // Модель ещё прогревалась — грузим синхронно.
        match Engine::load(&h.cfg) {
            Ok(engine) => h.engine = Some(engine),
            Err(err) => {
                h.last_error = Some(err.to_string());
                state.store(State::Idle.to_index(), Ordering::Relaxed);
                return Ok(());
            }
        }
    }

    let t0 = Instant::now();
    let engine = h.engine.as_ref().expect("engine должен быть");
    let text = match engine.transcribe(&samples) {
        Ok(text) => text,
        Err(err) => {
            h.last_error = Some(err.to_string());
            state.store(State::Idle.to_index(), Ordering::Relaxed);
            return Ok(());
        }
    };
    let inferred = t0.elapsed();

    if text.is_empty() {
        h.last_error = Some("ничего не распознано".into());
        state.store(State::Idle.to_index(), Ordering::Relaxed);
        return Ok(());
    }

    h.last_text = Some(text.clone());

    // Клавиши вставки подбираем под фокусное окно (konsole -> Shift+Insert,
    // браузер/Kate -> Ctrl+V).
    let app = dicta_core::focus::active_app(app_hint);
    let keys = h.cfg.paste_keys_for(app.as_deref());
    let combo = Combo::parse(&keys).unwrap_or_else(|_| h.paste_combo.clone());

    if let Err(err) = inject::insert_text(
        &text,
        &h.cfg.inject_mode,
        combo.clone(),
        Duration::from_millis(h.cfg.clipboard_restore_ms),
        Duration::from_millis(h.cfg.paste_delay_ms),
    ) {
        h.last_error = Some(format!("вставка не удалась: {err}"));
    }

    eprintln!(
        "dictad: \"{text}\" (записано {spoken:?}, распознано за {inferred:?}, вставка {keys} в {:?})",
        app
    );

    state.store(State::Idle.to_index(), Ordering::Relaxed);
    Ok(())
}

fn do_toggle(
    handler: &Arc<Mutex<Handler>>,
    state: &Arc<AtomicUsize>,
    started_at: &Arc<AtomicU64>,
) -> Result<()> {
    let mut h = handler.lock().map_err(|_| anyhow::anyhow!("handler отравлен"))?;
    let want_start = State::from_index(state.load(Ordering::Relaxed)) != State::Recording;
    if want_start {
        run_start(&mut h, state, started_at)
    } else {
        run_stop(&mut h, state, started_at, None)
    }
}

/// Держит оверлей dicta-ui живым, пока работает демон.
fn spawn_ui() {
    let exe = std::env::current_exe().ok().and_then(|path| {
        let sibling = path.parent()?.join("dicta-ui");
        if sibling.exists() {
            Some(sibling)
        } else {
            let local = home_dir().join(".local/bin/dicta-ui");
            local.exists().then_some(local)
        }
    });
    let Some(exe) = exe else {
        eprintln!("dictad: dicta-ui не найден рядом с демоном");
        return;
    };
    loop {
        match std::process::Command::new(&exe).spawn() {
            Ok(mut child) => {
                let _ = child.wait();
            }
            Err(err) => eprintln!("dictad: не смог запустить {}: {err}", exe.display()),
        }
        thread::sleep(Duration::from_secs(2));
    }
}

fn home_dir() -> PathBuf {
    std::env::var_os("HOME").map(PathBuf::from).unwrap_or_default()
}

/// Autostart-запись + подсказка про одноразовое разрешение на /dev/input.
fn install_autostart() -> Result<()> {
    let config_home = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| home_dir().join(".config"));
    let dir = config_home.join("autostart");
    std::fs::create_dir_all(&dir)?;

    let bin = {
        let local = home_dir().join(".local/bin/dictad");
        if local.exists() { local } else { std::env::current_exe()? }
    };
    let entry = format!(
        "[Desktop Entry]\nName=Dicta (голосовой ввод)\nComment=Диктовка в любое окно\nType=Application\nExec={} --ui\nTerminal=false\nX-GNOME-Autostart-enabled=true\nX-KDE-autostart-after=login\n",
        bin.display()
    );
    let file = dir.join("dicta.desktop");
    std::fs::write(&file, &entry)?;
    println!("автозапуск: {}", file.display());

    println!();
    println!("Один раз (нужен root), чтобы демон сам слушал клавишу:");
    println!(r#"  printf 'KERNEL=="event*", SUBSYSTEM=="input", MODE="0660", GROUP="input"\n' > /etc/udev/rules.d/99-dicta-input.rules"#);
    println!("  sudo udevadm control --reload-rules && sudo udevadm trigger");
    println!("либо: sudo usermod -aG input $USER и перелогин.");
    println!("Без этого диктовка запускается клавишей, назначенной в DE.");
    Ok(())
}

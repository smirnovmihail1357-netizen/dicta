//! dictad — единый демон голосового ввода.
//!
//! Слушает Unix-сокет, отвечает на JSON-команды по одной строке:
//!   {"cmd":"toggle"} {"cmd":"start"} {"cmd":"stop"} {"cmd":"status"} {"cmd":"cancel"}
//!   {"cmd":"inject","text":"…"}
//!   {"cmd":"panel"} {"cmd":"panel","on":true}      — показать/скрыть панель настроек в оверлее
//!   {"cmd":"settings"}                            — текущие настройки + список моделей
//!   {"cmd":"set","key":"min_level","value":"0.4"} — поменять настройку и сохранить в config.toml
//! Ответ: {"state":"idle|recording|transcribing","level":0..1,"seconds":..,"text":..,"error":..,
//!         "panel":bool,"settings":{…}}

use std::io::{BufRead, BufReader, Write};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, AtomicUsize, Ordering};
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
    /// Для {"cmd":"set"}: имя настройки (min_level, auto_stop, model, …).
    #[serde(default)]
    key: Option<String>,
    /// Для {"cmd":"set"}: новое значение (текстом; демон проверяет и котирует сам).
    #[serde(default)]
    value: Option<String>,
    /// Для {"cmd":"panel"}: true — показать, false — скрыть; нет — переключить.
    #[serde(default)]
    on: Option<bool>,
}

#[derive(serde::Serialize)]
struct Res<'a> {
    state: &'a str,
    level: f32,
    seconds: f32,
    text: Option<String>,
    error: Option<String>,
}

/// Снимок настроек для панели: то, что можно показать кнопкой.
#[derive(serde::Serialize)]
struct Settings {
    language: String,
    inject_mode: String,
    min_level: f32,
    auto_stop: bool,
    auto_stop_level: f32,
    auto_stop_after_ms: u64,
    auto_stop_grace_ms: u64,
    max_record_ms: u64,
    use_gpu: bool,
    gpu_device: i32,
    n_threads: i32,
    beam_size: i32,
    paste_keys: String,
    clipboard_restore_ms: u64,
    paste_delay_ms: u64,
    atspi_allow_any_role: bool,
    model: String,
    /// .bin-модели рядом с текущей: панель листает их кнопкой «модель».
    models: Vec<String>,
    /// Модель сейчас перегружается (смена model/use_gpu/…).
    reloading: bool,
    config_path: String,
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
    /// Куда писать изменения настроек (обычно ~/.config/dicta/config.toml).
    cfg_path: PathBuf,
    engine: Option<Engine>,
    recorder: Option<Recorder>,
    last_text: Option<String>,
    last_error: Option<String>,
    /// Какое окно считаем фокусным: подсказку адаптера запоминаем, чтобы авто-остановка
    /// вставляла в то же окно, где запись начиналась.
    last_app: Option<String>,
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
        cfg_path: dicta_core::config::config_path(),
        engine: None,
        recorder: None,
        last_text: None,
        last_error: None,
        last_app: None,
        paste_combo,
    }));
    let state = Arc::new(AtomicUsize::new(State::Idle.to_index()));
    let level = Arc::new(AtomicU32::new(0));
    let started_at = Arc::new(AtomicU64::new(0));
    // Панель настроек в оверлее: состояние живёт в демоне, чтобы её можно было
    // открыть и клавишей, и `dictactl panel`, и кнопкой самой панели.
    let panel = Arc::new(AtomicBool::new(false));
    // Идёт ли сейчас загрузка модели (показываем в панели, не даём грузить дважды).
    let reloading = Arc::new(AtomicBool::new(true));

    // Модель прогреваем сразу: первый диктофон не должен ждать загрузки.
    {
        let handler = handler.clone();
        let cfg = cfg.clone();
        let reloading = reloading.clone();
        thread::spawn(move || match Engine::load(&cfg) {
            Ok(engine) => {
                if let Ok(mut h) = handler.lock() {
                    h.engine = Some(engine);
                }
                reloading.store(false, Ordering::SeqCst);
            }
            Err(err) => {
                eprintln!("dictad: модель не загрузилась: {err}");
                if let Ok(mut h) = handler.lock() {
                    h.last_error = Some(err.to_string());
                }
                reloading.store(false, Ordering::SeqCst);
            }
        });
    }

    // Индикатор уровня для UI (80 мс) + watchdog остановки по тишине.
    //
    // По умолчанию watchdog по тишине выключен (auto_stop = false): запись закрывается
    // только горячей клавишей, тишина её не обрывает. Если auto_stop = true, демон сам
    // закрывает запись после N мс тишины, вторая клавиша не нужна. Речь при этом не
    // обрывается: речевой уровень (0.9-1.0) далеко от порога тишины (0.22).
    // Жёсткий потолок max_record_ms действует всегда — это страховка от «забыл выключить».
    {
        let handler = handler.clone();
        let state = state.clone();
        let level = level.clone();
        let started_at = started_at.clone();
        const TICK_MS: u64 = 80;
        thread::spawn(move || {
            let mut levels: Vec<f32> = Vec::new();
            loop {
                let recording =
                    State::from_index(state.load(Ordering::Relaxed)) == State::Recording;
                if recording {
                    if let Ok(h) = handler.try_lock() {
                        // Пороги берутся из живого конфига каждый тик: панель настроек
                        // может сменить их прямо во время записи.
                        let (stop_level, stop_max, stop_on, stop_grace, stop_after) = (
                            h.cfg.auto_stop_level,
                            h.cfg.max_record_ms,
                            h.cfg.auto_stop,
                            h.cfg.auto_stop_grace_ms,
                            h.cfg.auto_stop_after_ms,
                        );
                        let decision = if let Some(rec) = &h.recorder {
                            let l = rec.level();
                            level.store(f32_to_bits(l), Ordering::Relaxed);
                            levels.insert(0, l);
                            levels.truncate(2048);
                            let elapsed_ms = (rec.elapsed().as_secs_f64() * 1000.0) as u64;
                            let quiet_ms =
                                dicta_core::silence_streak_ms(&levels, stop_level, TICK_MS);
                            let reason = if elapsed_ms >= stop_max {
                                Some("жёсткий потолок длины записи")
                            } else if stop_on && elapsed_ms >= stop_grace && quiet_ms >= stop_after
                            {
                                Some("тишина")
                            } else {
                                None
                            };
                            (reason, h.last_app.clone())
                        } else {
                            level.store(0, Ordering::Relaxed);
                            levels.clear();
                            (None, None)
                        };
                        drop(h);

                        if let Some(reason) = decision.0 {
                            if let Ok(mut h) = handler.lock() {
                                eprintln!("dictad: авто-остановка записи: {reason}");
                                if let Err(err) =
                                    run_stop(&mut h, &state, &started_at, decision.1.as_deref())
                                {
                                    eprintln!("dictad: {err}");
                                }
                            }
                        }
                    }
                } else {
                    level.store(0, Ordering::Relaxed);
                    levels.clear();
                }
                thread::sleep(Duration::from_millis(TICK_MS));
            }
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
        let panel = panel.clone();
        let reloading = reloading.clone();
        let socket2: PathBuf = socket.clone();
        thread::spawn(move || {
            if let Err(err) = serve_conn(
                stream, handler, state, level, started_at, panel, reloading, socket2,
            ) {
                eprintln!("dictad: {err}");
            }
        });
    }

    Ok(())
}

/// Что отдаём клиенту: ответ демона плюс состояние панели и (иногда) настройки.
#[derive(serde::Serialize)]
struct Reply<'a> {
    #[serde(flatten)]
    base: Res<'a>,
    panel: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    settings: Option<Settings>,
}

/// .bin-модели рядом с текущей: панель листает их кнопкой «модель».
fn available_models(cfg: &Config) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let dir = match cfg.model.parent() {
        Some(dir) => dir,
        None => return out,
    };
    let Ok(entries) = std::fs::read_dir(dir) else {
        return out;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) == Some("bin") {
            out.push(path.display().to_string());
        }
    }
    out.sort();
    out.truncate(12);
    out
}

fn settings_view(h: &Handler, reloading: &AtomicBool) -> Settings {
    let cfg = &h.cfg;
    Settings {
        language: cfg.language.clone(),
        inject_mode: cfg.inject_mode.clone(),
        min_level: cfg.min_level,
        auto_stop: cfg.auto_stop,
        auto_stop_level: cfg.auto_stop_level,
        auto_stop_after_ms: cfg.auto_stop_after_ms,
        auto_stop_grace_ms: cfg.auto_stop_grace_ms,
        max_record_ms: cfg.max_record_ms,
        use_gpu: cfg.use_gpu,
        gpu_device: cfg.gpu_device,
        n_threads: cfg.n_threads,
        beam_size: cfg.beam_size,
        paste_keys: cfg.paste_keys.clone(),
        clipboard_restore_ms: cfg.clipboard_restore_ms,
        paste_delay_ms: cfg.paste_delay_ms,
        atspi_allow_any_role: cfg.atspi_allow_any_role,
        model: cfg.model.display().to_string(),
        models: available_models(cfg),
        reloading: reloading.load(Ordering::Relaxed),
        config_path: h.cfg_path.display().to_string(),
    }
}

/// Проверенное изменение одной настройки: знает, как примениться в памяти,
/// как записываться в TOML и нужна ли из-за него перезагрузка модели.
#[derive(Debug)]
enum Change {
    Level(&'static str, f32),
    Flag(&'static str, bool),
    Count(&'static str, u64),
    Index(&'static str, i32),
    Text(&'static str, String),
}

impl Change {
    fn key(&self) -> &'static str {
        match self {
            Change::Level(key, _) | Change::Flag(key, _) | Change::Count(key, _) => key,
            Change::Index(key, _) | Change::Text(key, _) => key,
        }
    }

    /// Как выглядит в config.toml.
    fn toml(&self) -> String {
        match self {
            Change::Level(_, v) => v.to_string(),
            Change::Flag(_, v) => v.to_string(),
            Change::Count(_, v) => v.to_string(),
            Change::Index(_, v) => v.to_string(),
            Change::Text(_, v) => format!("\"{}\"", v.replace('\\', "\\\\").replace('"', "\\\"")),
        }
    }

    fn needs_engine(&self) -> bool {
        dicta_core::config::ENGINE_KEYS
            .iter()
            .any(|k| *k == self.key())
    }

    fn apply(&self, cfg: &mut Config, combo: &mut Combo) -> Result<(), String> {
        match self {
            Change::Level(key, v) => match *key {
                "min_level" => cfg.min_level = *v,
                "auto_stop_level" => cfg.auto_stop_level = *v,
                other => return Err(format!("{other}: неожиданный ключ")),
            },
            Change::Flag(key, v) => match *key {
                "auto_stop" => cfg.auto_stop = *v,
                "atspi_allow_any_role" => cfg.atspi_allow_any_role = *v,
                "use_gpu" => cfg.use_gpu = *v,
                other => return Err(format!("{other}: неожиданный ключ")),
            },
            Change::Count(key, v) => match *key {
                "auto_stop_after_ms" => cfg.auto_stop_after_ms = *v,
                "auto_stop_grace_ms" => cfg.auto_stop_grace_ms = *v,
                "max_record_ms" => cfg.max_record_ms = *v,
                "clipboard_restore_ms" => cfg.clipboard_restore_ms = *v,
                "paste_delay_ms" => cfg.paste_delay_ms = *v,
                other => return Err(format!("{other}: неожиданный ключ")),
            },
            Change::Index(key, v) => match *key {
                "gpu_device" => cfg.gpu_device = *v,
                "n_threads" => cfg.n_threads = *v,
                "beam_size" => cfg.beam_size = *v,
                other => return Err(format!("{other}: неожиданный ключ")),
            },
            Change::Text(key, v) => match *key {
                "paste_keys" => {
                    cfg.paste_keys = v.clone();
                    *combo = Combo::parse(v).map_err(|err| format!("paste_keys: {err}"))?;
                }
                "inject_mode" => cfg.inject_mode = v.clone(),
                "language" => cfg.language = v.clone(),
                "model" => cfg.model = PathBuf::from(v),
                other => return Err(format!("{other}: неожиданный ключ")),
            },
        }
        Ok(())
    }
}

/// Проверка значения до применения: панель и CLI получают внятную ошибку вместо
/// битого конфига.
/// Имя настройки должно быть одним из EDITABLE_KEYS: тогда оно живёт всю жизнь
/// и `Change` не таскает за собой String.
fn static_key(key: &str) -> Option<&'static str> {
    dicta_core::config::EDITABLE_KEYS
        .iter()
        .find(|known| **known == key)
        .copied()
}

fn parse_change(name: &str, raw: &str, known_models: &[String]) -> Result<Change, String> {
    let Some(key) = static_key(name) else {
        return Err(format!(
            "неизвестная настройка {name:?}; доступны: {}",
            dicta_core::config::EDITABLE_KEYS.join(", ")
        ));
    };
    let raw = raw.trim();
    let bool_of = |value: &str| -> Result<bool, String> {
        match value.to_lowercase().as_str() {
            "1" | "true" | "yes" | "y" | "on" | "вкл" | "включено" | "да" => Ok(true),
            "0" | "false" | "no" | "n" | "off" | "выкл" | "выключено" | "нет" => {
                Ok(false)
            }
            _ => Err(format!("{key}: {value:?} — ожидается on/off")),
        }
    };
    let count = |min: u64, max: u64| -> Result<u64, String> {
        let value: u64 = raw
            .parse()
            .map_err(|_| format!("{key}: {raw:?} — ожидается целое {min}..{max}"))?;
        if !(min..=max).contains(&value) {
            return Err(format!("{key}: {value} вне диапазона {min}..{max}"));
        }
        Ok(value)
    };
    let level = || -> Result<f32, String> {
        let value: f32 = raw
            .parse()
            .map_err(|_| format!("{key}: {raw:?} — ожидается число 0..1"))?;
        if !(0.0..=1.0).contains(&value) {
            return Err(format!("{key}: {value} вне диапазона 0..1"));
        }
        Ok(value)
    };
    let index = |min: i32, max: i32| -> Result<i32, String> {
        let value: i32 = raw
            .parse()
            .map_err(|_| format!("{key}: {raw:?} — ожидается целое {min}..{max}"))?;
        if !(min..=max).contains(&value) {
            return Err(format!("{key}: {value} вне диапазона {min}..{max}"));
        }
        Ok(value)
    };

    match key {
        "min_level" | "auto_stop_level" => Ok(Change::Level(key, level()?)),
        "auto_stop" | "atspi_allow_any_role" | "use_gpu" => Ok(Change::Flag(key, bool_of(raw)?)),
        "auto_stop_after_ms" => Ok(Change::Count(key, count(200, 60_000)?)),
        "auto_stop_grace_ms" => Ok(Change::Count(key, count(0, 20_000)?)),
        "max_record_ms" => Ok(Change::Count(key, count(1_000, 600_000)?)),
        "clipboard_restore_ms" => Ok(Change::Count(key, count(200, 60_000)?)),
        "paste_delay_ms" => Ok(Change::Count(key, count(0, 3_000)?)),
        "inject_mode" => match raw {
            "paste" | "clipboard" | "stdout" | "atspi" => Ok(Change::Text(key, raw.to_string())),
            _ => Err("inject_mode: paste | clipboard | stdout | atspi".to_string()),
        },
        "paste_keys" => {
            Combo::parse(raw).map_err(|err| format!("paste_keys: {err}"))?;
            Ok(Change::Text(key, raw.to_string()))
        }
        "language" => {
            let lower = raw.to_ascii_lowercase();
            let ok = lower == "auto"
                || (lower.len() >= 2
                    && lower.len() <= 8
                    && lower.chars().all(|ch| ch.is_ascii_alphabetic()));
            if !ok {
                return Err("language: две-восемь букв (ru, en) или auto".to_string());
            }
            Ok(Change::Text(key, lower))
        }
        "model" => {
            // Принимается и полный путь, и имя из папки моделей: «ggml-medium-q5_0»
            // удобнее набирать с клавиатуры, чем шестьдесят символов пути.
            let path = PathBuf::from(raw);
            if path.is_file() {
                return Ok(Change::Text(key, path.display().to_string()));
            }
            let wanted = raw.trim();
            let stem = wanted.strip_suffix(".bin").unwrap_or(wanted);
            let matches: Vec<&String> = known_models
                .iter()
                .filter(|model| {
                    let name = PathBuf::from(model)
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or_default()
                        .to_string();
                    name == stem
                })
                .collect();
            match matches.len() {
                0 => Err(format!(
                    "model: файла {wanted} нет, и в папке моделей тако́го нет"
                )),
                1 => Ok(Change::Text(key, matches[0].clone())),
                _ => Err(format!(
                    "model: {wanted} подходит не одному файлу: {}",
                    matches
                        .iter()
                        .map(|m| m.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                )),
            }
        }
        "gpu_device" => Ok(Change::Index(key, index(0, 8)?)),
        "n_threads" => Ok(Change::Index(key, index(1, 64)?)),
        "beam_size" => Ok(Change::Index(key, index(1, 10)?)),
        #[allow(unreachable_patterns)]
        other => Err(format!("неизвестная настройка {other}")),
    }
}

/// Текущее значение настройки — в той же записи, что и `Change::toml()`.
/// Нужно, чтобы повторная кнопка (то же значение) не переписывала config.toml и,
/// главное, не перезагружала модель на GPU впустую.
fn current_value(cfg: &Config, key: &str) -> Option<String> {
    let text = |v: &str| format!("\"{}\"", v.replace('\\', "\\\\").replace('"', "\\\""));
    Some(match key {
        "min_level" => cfg.min_level.to_string(),
        "auto_stop" => cfg.auto_stop.to_string(),
        "auto_stop_level" => cfg.auto_stop_level.to_string(),
        "auto_stop_after_ms" => cfg.auto_stop_after_ms.to_string(),
        "auto_stop_grace_ms" => cfg.auto_stop_grace_ms.to_string(),
        "max_record_ms" => cfg.max_record_ms.to_string(),
        "inject_mode" => text(&cfg.inject_mode),
        "paste_keys" => text(&cfg.paste_keys),
        "clipboard_restore_ms" => cfg.clipboard_restore_ms.to_string(),
        "paste_delay_ms" => cfg.paste_delay_ms.to_string(),
        "atspi_allow_any_role" => cfg.atspi_allow_any_role.to_string(),
        "language" => text(&cfg.language),
        "model" => text(&cfg.model.display().to_string()),
        "use_gpu" => cfg.use_gpu.to_string(),
        "gpu_device" => cfg.gpu_device.to_string(),
        "n_threads" => cfg.n_threads.to_string(),
        "beam_size" => cfg.beam_size.to_string(),
        _ => return None,
    })
}

/// {"cmd":"set"}: применить настройку в живом конфиге, сохранить в config.toml и,
/// если настройка касается модели, заново загрузить её отдельным потоком.
fn set_setting(
    handler: &Arc<Mutex<Handler>>,
    state: &Arc<AtomicUsize>,
    reloading: &Arc<AtomicBool>,
    key: &str,
    value: &str,
    extra: &mut Option<Settings>,
) -> Res<'static> {
    let fail = |err: String| Res {
        state: State::from_index(state.load(Ordering::Relaxed)).as_str(),
        level: 0.0,
        seconds: 0.0,
        text: None,
        error: Some(err),
    };

    // Список моделей нужен, чтобы «model» можно было задать именем, а не путём.
    let known_models = match handler.lock() {
        Ok(h) => available_models(&h.cfg),
        Err(_) => Vec::new(),
    };
    let change = match parse_change(key, value, &known_models) {
        Ok(change) => change,
        Err(err) => {
            if let Ok(mut h) = handler.lock() {
                h.last_error = Some(err.clone());
            }
            return fail(err);
        }
    };

    let (mut note, changed) = {
        let Ok(mut h) = handler.lock() else {
            return fail("handler отравлен".to_string());
        };
        // Разбираем handler по полям: два &mut из MutexGuard в одном выражении
        // Rust не разрешает (deref_mut вызывается один раз на всю цепочку).
        let Handler {
            cfg, paste_combo, ..
        } = &mut *h;
        let before = current_value(cfg, change.key());
        if let Err(err) = change.apply(cfg, paste_combo) {
            h.last_error = Some(err.clone());
            return fail(err);
        }
        let after = current_value(cfg, change.key());
        if before.is_some() && before == after {
            // Значение уже такое: не пишем файл и не трогаем модель.
            (
                format!("{} = {} — уже так", change.key(), change.toml()),
                false,
            )
        } else {
            match dicta_core::config::set_key(&h.cfg_path, change.key(), &change.toml()) {
                Ok(()) => (
                    format!(
                        "{} = {} — сохранено в {}",
                        change.key(),
                        change.toml(),
                        h.cfg_path.display()
                    ),
                    true,
                ),
                Err(err) => (
                    format!(
                        "{} = {} применено, но сохранить в {} не вышло: {err}",
                        change.key(),
                        change.toml(),
                        h.cfg_path.display()
                    ),
                    true,
                ),
            }
        }
    };
    eprintln!("dictad: {note}");

    let needs_engine = changed && change.needs_engine();
    let cfg_snapshot = match handler.lock() {
        Ok(h) => h.cfg.clone(),
        Err(_) => return fail("handler отравлен".to_string()),
    };
    if needs_engine {
        match schedule_engine_reload(handler, state, reloading, cfg_snapshot) {
            Ok(msg) => note.push_str(&format!(" · {msg}")),
            Err(err) => note = format!("{note} · {err}"),
        }
    }

    if let Ok(h) = handler.lock() {
        *extra = Some(settings_view(&h, reloading));
    }
    Res {
        state: State::from_index(state.load(Ordering::Relaxed)).as_str(),
        level: 0.0,
        seconds: 0.0,
        text: Some(note),
        error: None,
    }
}

/// Перезагрузка модели вне потока сокета: medium-q5_0 грузится секунды, и держать
/// в это время мьютекс нельзя (запись бы встала).
fn schedule_engine_reload(
    handler: &Arc<Mutex<Handler>>,
    state: &Arc<AtomicUsize>,
    reloading: &Arc<AtomicBool>,
    cfg: Config,
) -> Result<String, String> {
    if State::from_index(state.load(Ordering::Relaxed)) != State::Idle {
        return Err(
            "идёт запись или распознавание — смену модели отложи; заверши диктовку (Alt+V) и повтори"
                .to_string(),
        );
    }
    if reloading.swap(true, Ordering::SeqCst) {
        return Err("модель уже загружается — подожди".to_string());
    }
    let handler = handler.clone();
    let reloading = reloading.clone();
    let name = cfg.model.display().to_string();
    let started = format!("модель {name} загружается…");
    thread::spawn(move || {
        let t0 = Instant::now();
        match Engine::load(&cfg) {
            Ok(engine) => {
                if let Ok(mut h) = handler.lock() {
                    h.engine = Some(engine);
                    h.last_error = None;
                }
                eprintln!("dictad: загружена модель {} за {:?}", name, t0.elapsed());
            }
            Err(err) => {
                eprintln!("dictad: модель {} не загрузилась: {err}", name);
                if let Ok(mut h) = handler.lock() {
                    h.last_error = Some(format!("модель {name}: {err}"));
                }
            }
        }
        reloading.store(false, Ordering::SeqCst);
    });
    Ok(started)
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
    panel: Arc<AtomicBool>,
    reloading: Arc<AtomicBool>,
    socket: PathBuf,
) -> Result<()> {
    let mut reader = BufReader::new(stream.try_clone()?);
    let mut writer = stream;
    let mut line = String::new();
    while reader.read_line(&mut line)? > 0 {
        let req: Req = serde_json::from_str(line.trim())?;
        // Настройки для панели: наполняются только в командах settings/set.
        let mut extra: Option<Settings> = None;
        let response = match req.cmd.as_str() {
            "status" => Res {
                state: State::from_index(state.load(Ordering::Relaxed)).as_str(),
                level: bits_to_f32(level.load(Ordering::Relaxed)),
                seconds: elapsed_seconds(&started_at, &state),
                text: None,
                error: None,
            },
            "panel" => {
                let want = match req.on {
                    Some(on) => on,
                    None => !panel.load(Ordering::Relaxed),
                };
                panel.store(want, Ordering::Relaxed);
                eprintln!(
                    "dictad: панель настроек {}",
                    if want {
                        "открыта"
                    } else {
                        "скрыта"
                    }
                );
                Res {
                    state: State::from_index(state.load(Ordering::Relaxed)).as_str(),
                    level: bits_to_f32(level.load(Ordering::Relaxed)),
                    seconds: elapsed_seconds(&started_at, &state),
                    text: None,
                    error: None,
                }
            }
            "settings" => {
                let h = handler
                    .lock()
                    .map_err(|_| anyhow::anyhow!("handler отравлен"))?;
                extra = Some(settings_view(&h, &reloading));
                Res {
                    state: State::from_index(state.load(Ordering::Relaxed)).as_str(),
                    level: bits_to_f32(level.load(Ordering::Relaxed)),
                    seconds: elapsed_seconds(&started_at, &state),
                    text: None,
                    error: None,
                }
            }
            "set" => set_setting(
                &handler,
                &state,
                &reloading,
                req.key.as_deref().unwrap_or(""),
                req.value.as_deref().unwrap_or(""),
                &mut extra,
            ),
            "inject" => {
                let text = req.text.clone().unwrap_or_default();
                let mut h = handler
                    .lock()
                    .map_err(|_| anyhow::anyhow!("handler отравлен"))?;
                if let Some(app) = req.app.clone() {
                    h.last_app = Some(app);
                }
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
                        h.cfg.atspi_allow_any_role,
                        app.as_deref(),
                    ) {
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
                let mut h = handler
                    .lock()
                    .map_err(|_| anyhow::anyhow!("handler отравлен"))?;
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

        let json = serde_json::to_string(&Reply {
            base: response,
            panel: panel.load(Ordering::Relaxed),
            settings: extra,
        })?;
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
        h.cfg.atspi_allow_any_role,
        app.as_deref(),
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
    let mut h = handler
        .lock()
        .map_err(|_| anyhow::anyhow!("handler отравлен"))?;
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
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_default()
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
        if local.exists() {
            local
        } else {
            std::env::current_exe()?
        }
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
    println!(
        r#"  printf 'KERNEL=="event*", SUBSYSTEM=="input", MODE="0660", GROUP="input"\n' > /etc/udev/rules.d/99-dicta-input.rules"#
    );
    println!("  sudo udevadm control --reload-rules && sudo udevadm trigger");
    println!("либо: sudo usermod -aG input $USER и перелогин.");
    println!("Без этого диктовка запускается клавишей, назначенной в DE.");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn models() -> Vec<String> {
        vec![
            "/m/ggml-medium-q5_0.bin".to_string(),
            "/m/ggml-large-v3-turbo-q5_0.bin".to_string(),
        ]
    }

    #[test]
    fn model_accepts_name_from_the_models_folder() {
        // «dictactl settings model ggml-small-q5_1» должно находиться по имени: путь
        // длиной в шестьдесят символов с клавиатуры не набрать
        let change = parse_change("model", "ggml-medium-q5_0", &models()).unwrap();
        match change {
            Change::Text(key, value) => {
                assert_eq!(key, "model");
                assert_eq!(value, "/m/ggml-medium-q5_0.bin");
            }
            other => panic!("ожидался Text, получил {other:?}"),
        }
    }

    #[test]
    fn model_rejects_what_is_not_there() {
        assert!(parse_change("model", "ggml-tiny", &models()).is_err());
    }

    #[test]
    fn values_are_validated_before_they_touch_the_config() {
        assert!(matches!(
            parse_change("min_level", "0.4", &models()).unwrap(),
            Change::Level(_, value) if (value - 0.4).abs() < 1e-6
        ));
        assert!(parse_change("min_level", "4", &models()).is_err());
        assert!(parse_change("inject_mode", "telepathy", &models()).is_err());
        assert!(parse_change("paste_keys", "ctrl+shift+v", &models()).is_ok());
        assert!(parse_change("weather", "sunny", &models()).is_err());
    }
}

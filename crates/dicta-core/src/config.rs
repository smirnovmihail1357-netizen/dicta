use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::{Context, Result};
use serde::Deserialize;

/// Конфигурация `dicta`. Всё опционально; значения по умолчанию ниже.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct Config {
    /// Путь к ggml-модели whisper.cpp.
    pub model: PathBuf,
    /// Код языка whisper ("ru", "en", ...) или "auto".
    pub language: String,
    /// Подсказка-вокабуляр для распознавания. None -> типовая по языку.
    pub prompt: Option<String>,
    /// Ширина beam search. 1 = greedy (быстрее, хуже).
    pub beam_size: i32,
    pub n_threads: i32,
    /// false -> только CPU (безопасно по VRAM).
    pub use_gpu: bool,
    pub gpu_device: i32,
    pub flash_attn: bool,
    /// PipeWire source для записи. None -> дефолтный микрофон системы.
    pub mic: Option<String>,
    /// Unix-сокет демона.
    pub socket: PathBuf,
    /// Клавиши вставки по умолчанию (uinput): ctrl+v, ctrl+shift+v, shift+insert.
    pub paste_keys: String,
    /// Как вставлять: "paste" (буфер + клавиши), "clipboard" (только буфер),
    /// "stdout" (напечатать в терминал — для проверки без влияния на окна).
    pub inject_mode: String,
    /// Минимальный уровень записи в той же шкале 0..1, что и индикатор в UI
    /// (`audio::normalized_rms`: -55 дБ -> 0.0, -15 дБ -> 1.0). Ниже порога
    /// распознавание не запускается: whisper на фоновом шуме уверенно сочиняет
    /// правдоподобные фразы, и они улетели бы в фокусное окно.
    /// Считается как speech_level(): максимум индикатора по окнам 300 мс — речь это
    /// всплески, а ровный фоновый шум по среднему RMS выглядит «громче», чем он есть.
    /// Измерено на этой машине: фон в комнате 0.34-0.35, русская речь 0.9-1.0.
    pub min_level: f32,
    /// Глобальная горячая клавиша демона (evdev, читает /dev/input/event*).
    /// По умолчанию "off": доступ к /dev/input обычно требует группы input или
    /// udev-ACL, а клавишу чаще навешивает DE (см. adapters/README.md).
    pub hotkey: String,
    /// Клавиши вставки по классу фокусного окна: { konsole = "shift+insert", ... }.
    /// Имя окна берётся у композитора (`focus::active_app`), ключ — после
    /// `focus::normalize` (без точек, тире и подчёркиваний, в нижнем регистре).
    pub paste_keys_by_app: HashMap<String, String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            model: default_model_path(),
            language: "ru".into(),
            prompt: None,
            beam_size: 5,
            n_threads: 4,
            use_gpu: true,
            gpu_device: 0,
            flash_attn: false,
            mic: None,
            socket: default_socket_path(),
            paste_keys: "ctrl+v".into(),
            inject_mode: "paste".into(),
            hotkey: "off".into(),
            min_level: 0.45,
            paste_keys_by_app: default_paste_keys_by_app(),
        }
    }
}

/// Терминалы и консоли обычно вставляют Shift+Insert или Ctrl+Shift+V,
/// текстовые поля — Ctrl+V.
fn default_paste_keys_by_app() -> HashMap<String, String> {
    let mut m = HashMap::new();
    for app in [
        "konsole",
        "yakuake",
        "alacritty",
        "xterm",
        "foot",
        "kitty",
        "wezterm",
        "ghostty",
        "terminal",
    ] {
        m.insert(app.to_string(), "shift+insert".to_string());
    }
    for app in [
        "firefox",
        "chromium",
        "chrome",
        "brave",
        "opera",
        "discord",
        "kate",
        "telegramdesktop",
    ] {
        m.insert(app.to_string(), "ctrl+v".to_string());
    }
    m
}

impl Config {
    /// Какие клавиши слать в это окно.
    pub fn paste_keys_for(&self, app: Option<&str>) -> String {
        match app {
            Some(app) => match self.paste_keys_by_app.get(&crate::focus::normalize(app)) {
                Some(keys) => keys.clone(),
                None => self.paste_keys.clone(),
            },
            None => self.paste_keys.clone(),
        }
    }
}

fn default_model_path() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("whisper-cpp")
        .join("ggml-medium-q5_0.bin")
}

fn default_socket_path() -> PathBuf {
    let runtime = std::env::var_os("XDG_RUNTIME_DIR").map(PathBuf::from);
    match runtime {
        Some(dir) if !dir.as_os_str().is_empty() => dir.join("dicta.sock"),
        _ => dirs::runtime_dir()
            .unwrap_or_else(|| std::env::temp_dir())
            .join("dicta.sock"),
    }
}

impl Config {
    /// $DICTA_CONFIG, затем XDG_CONFIG_HOME/dicta/config.toml, затем defaults.
    pub fn load(explicit: Option<&std::path::Path>) -> Result<Self> {
        let path = match explicit {
            Some(p) => Some(p.to_path_buf()),
            None => dirs::config_dir().map(|d| d.join("dicta").join("config.toml")),
        };

        let Some(path) = path else {
            return Ok(Config::default());
        };
        if !path.exists() {
            return Ok(Config::default());
        }
        let raw = std::fs::read_to_string(&path)
            .with_context(|| format!("не могу прочитать {}", path.display()))?;
        let cfg: Config =
            toml::from_str(&raw).with_context(|| format!("плохой TOML в {}", path.display()))?;
        Ok(cfg)
    }

    /// Вокабуляр, если пользователь его не задал.
    pub fn effective_prompt(&self) -> String {
        if let Some(p) = &self.prompt {
            return p.clone();
        }
        match self.language.as_str() {
            "ru" => "Разговор о программировании. Rust, TypeScript, JSON, API, git, PipeWire, Wayland."
                .to_string(),
            _ => "A conversation about programming. Rust, TypeScript, JSON, API, git, PipeWire, Wayland."
                .to_string(),
        }
    }
}

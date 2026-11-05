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

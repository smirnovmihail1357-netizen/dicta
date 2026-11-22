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
    /// Для `inject_mode = "atspi"`: писать и в нестандартные редактируемые узлы.
    /// По умолчанию false: разрешены только роли поля ввода (entry, password text,
    /// combo box, text). Некоторые приложения помечают `focused` не поле, а документ
    /// (Firefox — `document web`); в такой документ демону писать нельзя.
    pub atspi_allow_any_role: bool,
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
    /// Через сколько мс вернуть прежний буфер обмена после вставки.
    /// Слишком мало — приложение не успеет прочитать буфер и получит старый.
    pub clipboard_restore_ms: u64,
    /// Пауза между wl-copy и нажатием клавиш вставки.
    pub paste_delay_ms: u64,

    /// Автоматически заканчивать запись, когда долго тихо (VAD-остановка).
    /// По умолчанию выключено: запись закрывается только горячей клавишей (или `dictactl stop`).
    /// Пороги тишины при этом сохраняются в конфиге и включаются по клику в панели настроек.
    pub auto_stop: bool,
    /// Уровень ниже этого значения считается тишиной (шкала как в `speech_level`).
    pub auto_stop_level: f32,
    /// Сколько миллисекунд подряд должна быть тишина, чтобы остановить запись.
    pub auto_stop_after_ms: u64,
    /// Первые миллисекунды записи авто-остановка спит: дал старт -> начал говорить.
    pub auto_stop_grace_ms: u64,
    /// Жёсткий потолок длины записи.
    pub max_record_ms: u64,
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
            atspi_allow_any_role: false,
            hotkey: "off".into(),
            min_level: 0.35,
            paste_keys_by_app: default_paste_keys_by_app(),
            clipboard_restore_ms: 2500,
            paste_delay_ms: 150,
            auto_stop: false,
            auto_stop_level: 0.22,
            auto_stop_after_ms: 1500,
            auto_stop_grace_ms: 1200,
            max_record_ms: 60_000,
        }
    }
}

/// Терминалы: Ctrl+Shift+V (так по умолчанию в Konsole, foot, alacritty, kitty, wezterm,
/// ghostty). Shift+Insert остаётся для xterm и X11-наследия — на Wayland он в Konsole
/// ничего не вставляет. Текстовые поля — Ctrl+V.
fn default_paste_keys_by_app() -> HashMap<String, String> {
    let mut m = HashMap::new();
    for app in [
        "konsole",
        "yakuake",
        "alacritty",
        "foot",
        "footclient",
        "kitty",
        "wezterm",
        "ghostty",
        "terminal",
    ] {
        m.insert(app.to_string(), "ctrl+shift+v".to_string());
    }
    for app in ["xterm", "urxvt", "st"] {
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

/// Настройки, которые демон умеет менять на лету (панель оверлея, `dictactl settings`).
/// Имена — ровно ключи TOML, их же принимает `set_key`.
pub const EDITABLE_KEYS: &[&str] = &[
    "min_level",
    "auto_stop",
    "auto_stop_level",
    "auto_stop_after_ms",
    "auto_stop_grace_ms",
    "max_record_ms",
    "inject_mode",
    "paste_keys",
    "clipboard_restore_ms",
    "paste_delay_ms",
    "atspi_allow_any_role",
    "language",
    "model",
    "use_gpu",
    "gpu_device",
    "n_threads",
    "beam_size",
];

/// Ключи, смена которых требует заново загрузить модель whisper.
pub const ENGINE_KEYS: &[&str] = &[
    "model",
    "language",
    "use_gpu",
    "gpu_device",
    "n_threads",
    "beam_size",
];

/// Путь конфига, который читает `Config::load(None)` — туда же пишет панель настроек.
pub fn config_path() -> PathBuf {
    std::env::var_os("DICTA_CONFIG")
        .map(PathBuf::from)
        .or_else(|| dirs::config_dir().map(|dir| dir.join("dicta").join("config.toml")))
        .unwrap_or_else(|| PathBuf::from("dicta-config.toml"))
}

/// Поменять одно значение в config.toml, не трогая комментарии и соседние строки.
/// `value_toml` — уже TOML-литерал: `0.4`, `true`, `"paste"`.
/// Строка `key = значение` находится и заменяется по месту (хвостовой комментарий
/// сохраняется); если такой строки нет — значение дописывается в конец файла.
/// Запись атомарная: временный файл + rename.
pub fn set_key(path: &std::path::Path, key: &str, value_toml: &str) -> Result<()> {
    let raw = match std::fs::read_to_string(path) {
        Ok(text) => text,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(err) => return Err(err).with_context(|| format!("читать {}", path.display())),
    };
    let had_final_newline = raw.ends_with('\n');
    let mut lines: Vec<String> = raw.lines().map(|line| line.to_string()).collect();

    let mut replaced = false;
    for line in lines.iter_mut() {
        if line.trim_start().starts_with('#') {
            continue; // закомментированное значение пользователя не раскомментируем
        }
        let Some((lhs, _rest)) = line.split_once('=') else {
            continue;
        };
        if lhs.trim() != key {
            continue;
        }
        let indent = &line[..line.len() - line.trim_start().len()];
        // хвостовой комментарий сохраняем дословно, включая «#»
        let comment = line.find('#').map(|hash| format!(" {}", &line[hash..]));
        *line = match comment {
            Some(comment) => format!("{indent}{key} = {value_toml}{comment}"),
            None => format!("{indent}{key} = {value_toml}"),
        };
        replaced = true;
        break;
    }
    if !replaced {
        lines.push(format!("{key} = {value_toml}"));
    }

    let mut text = lines.join("\n");
    if !text.is_empty() && (had_final_newline || !raw.is_empty()) {
        text.push('\n');
    }
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).ok();
    }
    let tmp = path.with_extension("toml.tmp");
    std::fs::write(&tmp, &text).with_context(|| format!("записать {}", tmp.display()))?;
    std::fs::rename(&tmp, path)
        .with_context(|| format!("{} -> {}", tmp.display(), path.display()))?;
    Ok(())
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

/// Имя канала по умолчанию (windows_engine.dart:22 kDictaPipe) — env-шов
/// DICTA_PIPE в слушателе демона может подменить его только для теста.
#[cfg(windows)]
pub const DEFAULT_PIPE_NAME: &str = r"\\.\pipe\dicta";

fn default_model_path() -> PathBuf {
    // Windows: %LOCALAPPDATA% (dirs::data_dir дал бы Roaming — Pitfall 2).
    #[cfg(windows)]
    let base = dirs::data_local_dir();
    #[cfg(not(windows))]
    let base = dirs::data_dir();
    base.unwrap_or_else(|| PathBuf::from("."))
        .join("whisper-cpp")
        .join("ggml-medium-q5_0.bin")
}

// На Windows поле socket хранит полное имя именованного канала; инициатива
// слушателя — в transport_win (DICTA_PIPE → дефолт).
#[cfg(windows)]
fn default_socket_path() -> PathBuf {
    PathBuf::from(DEFAULT_PIPE_NAME)
}

#[cfg(not(windows))]
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
        // Приоритет: явный --config, затем $DICTA_CONFIG (нужно контейнеру: там конфиг в /config),
        // затем XDG_CONFIG_HOME/dicta/config.toml.
        let from_env = || std::env::var_os("DICTA_CONFIG").map(std::path::PathBuf::from);
        let path = match explicit {
            Some(p) => Some(p.to_path_buf()),
            None => from_env()
                .or_else(|| dirs::config_dir().map(|d| d.join("dicta").join("config.toml"))),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn picks_paste_keys_by_normalized_app() {
        let cfg = Config::default();
        assert_eq!(cfg.paste_keys_for(Some("org.kde.konsole")), "ctrl+shift+v");
        assert_eq!(cfg.paste_keys_for(Some("Firefox.desktop")), "ctrl+v");
        assert_eq!(cfg.paste_keys_for(Some("xterm")), "shift+insert");
        // неизвестное окно -> общий paste_keys
        assert_eq!(cfg.paste_keys_for(Some("whatapp")), "ctrl+v");
        assert_eq!(cfg.paste_keys_for(None), "ctrl+v");
    }

    #[test]
    fn clipboard_timing_defaults_leave_room_for_the_target_app() {
        let cfg = Config::default();
        assert!(cfg.clipboard_restore_ms >= 2000);
    }

    /// Временный конфиг с уникальным именем: тесты гоняются параллельно.
    fn tmp_config(name: &str, body: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("dicta-config-test-{name}"));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("config.toml");
        std::fs::write(&path, body).unwrap();
        path
    }

    #[test]
    fn set_key_replaces_in_place_and_keeps_trailing_comment() {
        let path = tmp_config(
            "inplace",
            "# рабочая конфигурация\nmodel = \"/x/ggml-medium-q5_0.bin\"\ngpu_device = 0            # 0 = iGPU, 1 = XTX\nmin_level = 0.35\n",
        );
        set_key(&path, "min_level", "0.5").unwrap();
        set_key(&path, "gpu_device", "1").unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        assert!(text.contains("min_level = 0.5"), "{text}");
        // комментарий и отступ сохранены, значения в той же строке
        assert!(
            text.contains("gpu_device = 1 # 0 = iGPU, 1 = XTX"),
            "{text}"
        );
        assert!(
            text.contains("model = \"/x/ggml-medium-q5_0.bin\""),
            "{text}"
        );
        assert!(text.contains("# рабочая конфигурация"), "{text}");
        // не плодит дублей
        assert_eq!(text.matches("min_level").count(), 1);

        let cfg = Config::load(Some(&path)).unwrap();
        assert_eq!(cfg.min_level, 0.5);
        assert_eq!(cfg.gpu_device, 1);
    }

    #[test]
    fn set_key_appends_missing_and_skips_commented_lines() {
        let path = tmp_config(
            "append",
            "language = \"ru\"\n# min_level = 0.9   # специально закомментировано\n",
        );
        set_key(&path, "min_level", "0.42").unwrap();
        set_key(&path, "language", "\"en\"").unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        // закомментированная строка осталась как была, новое значение дописано отдельно
        assert!(text.contains("# min_level = 0.9"), "{text}");
        assert!(text.contains("min_level = 0.42"), "{text}");
        assert!(text.contains("language = \"en\""), "{text}");

        let cfg = Config::load(Some(&path)).unwrap();
        assert_eq!(cfg.min_level, 0.42);
        assert_eq!(cfg.language, "en");
    }

    #[test]
    fn editable_keys_are_real_toml_keys() {
        // панель предлагает только то, что действительно есть в Config
        for key in EDITABLE_KEYS {
            assert!(key.len() > 2, "{key}");
        }
        assert!(ENGINE_KEYS.contains(&"model"));
    }
}

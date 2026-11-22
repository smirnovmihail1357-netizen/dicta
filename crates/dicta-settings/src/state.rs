//! Снимок настроек из ответа демона. Поля и дефолты — те же, что у панели оверлея
//! (`crates/dicta-ui/src/panel.rs`): старый демон, у которого нет `settings`,
//! не должен ломать окно.

use serde::Deserialize;

/// Ответ dictad на `{"cmd":"settings"}` / `{"cmd":"set", ...}`.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub language: String,
    pub inject_mode: String,
    pub min_level: f32,
    pub auto_stop: bool,
    pub auto_stop_level: f32,
    pub auto_stop_after_ms: u64,
    pub auto_stop_grace_ms: u64,
    pub max_record_ms: u64,
    pub use_gpu: bool,
    pub gpu_device: i32,
    pub n_threads: i32,
    pub beam_size: i32,
    pub paste_keys: String,
    pub clipboard_restore_ms: u64,
    pub paste_delay_ms: u64,
    pub atspi_allow_any_role: bool,
    pub model: String,
    pub models: Vec<String>,
    pub reloading: bool,
    pub config_path: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            language: "ru".into(),
            inject_mode: "paste".into(),
            min_level: 0.0,
            auto_stop: false,
            auto_stop_level: 0.22,
            auto_stop_after_ms: 1500,
            auto_stop_grace_ms: 1200,
            max_record_ms: 60_000,
            use_gpu: true,
            gpu_device: 0,
            n_threads: 4,
            beam_size: 5,
            paste_keys: "ctrl+v".into(),
            clipboard_restore_ms: 2500,
            paste_delay_ms: 150,
            atspi_allow_any_role: false,
            model: String::new(),
            models: Vec::new(),
            reloading: false,
            config_path: String::new(),
        }
    }
}

/// Разобрать ответ демона: настройки и строку ошибки (если демон отказал в изменении).
/// Отравленный JSON не паникует — получаем дефолты и пустую ошибку.
pub fn from_reply(v: &serde_json::Value) -> (Settings, Option<String>) {
    let settings = v
        .get("settings")
        .and_then(|value| serde_json::from_value::<Settings>(value.clone()).ok())
        .unwrap_or_default();
    let error = v
        .get("base")
        .and_then(|base| base.get("error"))
        .and_then(|value| value.as_str())
        .map(str::to_string);
    (settings, error)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn from_reply_reads_settings_and_error() {
        let reply = json!({
            "base": {
                "state": "Idle", "level": 0.0, "seconds": 0.0, "text": null,
                "error": "min_level: 2 вне диапазона 0..1"
            },
            "panel": false,
            "settings": {
                "min_level": 0.4, "auto_stop": true, "language": "en",
                "model": "/x/ggml-small-q5_1.bin",
                "models": ["/x/ggml-small-q5_1.bin"]
            }
        });
        let (settings, error) = from_reply(&reply);
        assert!((settings.min_level - 0.4).abs() < f32::EPSILON);
        assert!(settings.auto_stop);
        assert_eq!(settings.language, "en");
        assert_eq!(settings.model, "/x/ggml-small-q5_1.bin");
        assert_eq!(settings.models.len(), 1);
        assert_eq!(error.as_deref(), Some("min_level: 2 вне диапазона 0..1"));
    }

    #[test]
    fn reply_without_settings_gives_defaults() {
        // Демон старше окна: поля settings нет вовсе.
        let (settings, error) = from_reply(&json!({"base": {"error": null}}));
        assert_eq!(settings.language, "ru");
        assert_eq!(settings.inject_mode, "paste");
        assert_eq!(settings.beam_size, 5);
        assert!(settings.models.is_empty());
        assert!(error.is_none());
    }

    #[test]
    fn poisoned_settings_do_not_panic() {
        // Не объект, поле не того типа, null вместо числа — везде дефолты.
        let replies = [
            json!({"settings": "мусор"}),
            json!({"settings": {"min_level": "много"}}),
            json!({"settings": {"auto_stop": null}}),
        ];
        for reply in replies {
            let (settings, _) = from_reply(&reply);
            assert_eq!(settings.language, "ru");
            assert_eq!(settings.min_level, 0.0);
        }
    }
}

//! Панель настроек оверлея: какие кнопки показывать и какую команду демона послать по клику.
//!
//! Панель не хранит настройки сама: она рисует то, что ответил демон (`{"cmd":"settings"}`),
//! а по клику шлёт `{"cmd":"set", ...}` — демон проверяет значение, применяет к живому
//! конфига и сам пишет в config.toml. Так панель нельзя привести в состояние, которого
//! демон не придерживается.

use serde::Deserialize;

use crate::edge::Edge;
use crate::render::{Button, Tone, PANEL_GAP_X, PANEL_PAD, PANEL_W};

/// Ответ dictad на {"cmd":"settings"}. Недостающие поля — дефолты (старый демон
/// вообще не знает про settings: панель тогда покажет управление записью и всё).
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
            // 0 как «неизвестно»: панель тогда не показывает число
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

/// Тон кнопки и сама кнопка живут в `render` (это про внешний вид);
/// здесь — только их наполнение и команды.

/// Команда демона, которую отправляет кнопка.
#[derive(Debug, Clone)]
pub struct Request {
    pub cmd: &'static str,
    pub key: Option<&'static str>,
    pub value: Option<String>,
    pub on: Option<bool>,
}

impl Request {
    pub fn json(&self) -> String {
        let mut parts = vec![format!("\"cmd\":\"{}\"", self.cmd)];
        if let Some(key) = self.key {
            parts.push(format!("\"key\":\"{key}\""));
        }
        if let Some(value) = &self.value {
            let escaped = value.replace('\\', "\\\\").replace('"', "\\\"");
            parts.push(format!("\"value\":\"{escaped}\""));
        }
        if let Some(on) = self.on {
            parts.push(format!("\"on\":{on}"));
        }
        format!("{{{}}}\n", parts.join(","))
    }
}

/// Строки панели. X — от левого края корпуса, Y — вдоль него: для горизонтальной
/// панели это ряды сверху вниз, для вертикальной (сторона слева/справа) — колонки,
/// которые рендер разворачивает на 90°.
const ROWS: [u32; 3] = [34, 68, 102];

fn row(y: u32, specs: &[(&'static str, String, u32, Tone)]) -> Vec<Button> {
    let total: u32 = specs.iter().map(|(_, _, w, _)| *w).sum();
    let gaps = PANEL_GAP_X * (specs.len() as u32 - 1);
    let space = PANEL_W - 2 * PANEL_PAD;
    let avail = space.saturating_sub(gaps);
    // Заявленные ширины — пожелание. Влезает — делим остаток поровну (метки не липнут
    // друг к другу); не влезает — сжимаем пропорционально пожеланию. Без сжатия последние
    // кнопки ряда уезжают за край окна и в них нельзя попасть: ряд «модель/клавиши/край/
    // закрыть» просил 220 + 130 + 96 + 96 = 542 px при 476 доступных.
    let widths: Vec<u32> = if total > avail {
        let mut w: Vec<u32> = specs
            .iter()
            .map(|(_, _, width, _)| (*width as u64 * avail as u64 / total as u64) as u32)
            .collect();
        let mut rest = avail - w.iter().sum::<u32>();
        for width in w.iter_mut() {
            if rest == 0 {
                break;
            }
            *width += 1;
            rest -= 1;
        }
        w
    } else {
        let slack = (avail - total) / specs.len() as u32;
        specs.iter().map(|(_, _, width, _)| *width + slack).collect()
    };
    let mut x = PANEL_PAD;
    let mut out = Vec::new();
    for ((id, label, _, tone), width) in specs.iter().zip(widths.iter()) {
        out.push(Button {
            id,
            label: label.clone(),
            x,
            y,
            w: *width,
            tone: *tone,
        });
        x += *width + PANEL_GAP_X;
    }
    out
}

fn on_off(v: bool) -> &'static str {
    if v {
        "вкл"
    } else {
        "выкл"
    }
}

/// Имя модели без пути и без «ggml-»: «medium-q5_0» влезает в кнопку.
fn model_name(model: &str) -> String {
    let file = model.rsplit('/').next().unwrap_or(model);
    let stem = file.strip_suffix(".bin").unwrap_or(file);
    stem.strip_prefix("ggml-").unwrap_or(stem).to_string()
}

/// Кнопки панели по текущим настройкам. `edge` — сторона монитора, к которой прижат
/// оверлей: она показана на кнопке и та же кнопка переставляет оверлей на следующую.
pub fn buttons(settings: &Settings, recording: bool, edge: Edge) -> Vec<Button> {
    let mut out = Vec::new();

    // Ряд 1 — запись и самый частый регулятор: порог уровня.
    out.extend(row(
        ROWS[0],
        &[
            (
                "rec",
                if recording {
                    "■ стоп".to_string()
                } else {
                    "● запись".to_string()
                },
                118,
                if recording { Tone::Stop } else { Tone::Action },
            ),
            ("cancel", "отмена".to_string(), 92, Tone::Flat),
            (
                "min",
                if settings.min_level > 0.0 {
                    format!("min {}", (settings.min_level * 100.0).round() as i32)
                } else {
                    "min".to_string()
                },
                92,
                Tone::Value,
            ),
            ("min-", "−".to_string(), 54, Tone::Flat),
            ("min+", "+".to_string(), 54, Tone::Flat),
        ],
    ));

    // Ряд 2 — режимы работы.
    out.extend(row(
        ROWS[1],
        &[
            (
                "auto",
                format!("авто-стоп {}", on_off(settings.auto_stop)),
                140,
                if settings.auto_stop {
                    Tone::Value
                } else {
                    Tone::Flat
                },
            ),
            (
                "inject",
                format!("вставка {}", settings.inject_mode),
                130,
                Tone::Value,
            ),
            (
                "gpu",
                format!("gpu {}", on_off(settings.use_gpu)),
                100,
                if settings.use_gpu {
                    Tone::Value
                } else {
                    Tone::Flat
                },
            ),
            (
                "lang",
                format!("язык {}", settings.language),
                100,
                Tone::Value,
            ),
        ],
    ));

    // Ряд 3 — модель, клавиши, закрытие.
    out.extend(row(
        ROWS[2],
        &[
            (
                "model",
                if settings.reloading {
                    "модель загружается…".to_string()
                } else {
                    format!("модель {}", model_name(&settings.model))
                },
                220,
                Tone::Value,
            ),
            (
                "keys",
                format!("клавиши {}", settings.paste_keys),
                130,
                Tone::Value,
            ),
            ("edge", format!("край {}", edge.label()), 96, Tone::Flat),
            ("close", "закрыть ×".to_string(), 96, Tone::Flat),
        ],
    ));

    out
}

/// Что послать демону по кнопке. Значения для ± берём из текущих настроек,
/// чтобы «min +» был следующим шагом, а не случайным числом.
pub fn request_for(id: &str, settings: &Settings, _recording: bool) -> Option<Request> {
    let step = 0.05f32;
    match id {
        "rec" => Some(Request {
            cmd: "toggle",
            key: None,
            value: None,
            on: None,
        }),
        "cancel" => Some(Request {
            cmd: "cancel",
            key: None,
            value: None,
            on: None,
        }),
        "close" => Some(Request {
            cmd: "panel",
            key: None,
            value: None,
            on: Some(false),
        }),
        "min-" => Some(Request {
            cmd: "set",
            key: Some("min_level"),
            value: Some(format!("{:.2}", (settings.min_level - step).max(0.0))),
            on: None,
        }),
        "min+" => Some(Request {
            cmd: "set",
            key: Some("min_level"),
            value: Some(format!("{:.2}", (settings.min_level + step).min(1.0))),
            on: None,
        }),
        "auto" => Some(Request {
            cmd: "set",
            key: Some("auto_stop"),
            value: Some(if settings.auto_stop { "off" } else { "on" }.to_string()),
            on: None,
        }),
        "gpu" => Some(Request {
            cmd: "set",
            key: Some("use_gpu"),
            value: Some(if settings.use_gpu { "off" } else { "on" }.to_string()),
            on: None,
        }),
        "inject" => {
            // цикл по режимам, какие принимает демон
            let next = match settings.inject_mode.as_str() {
                "paste" => "atspi",
                "atspi" => "clipboard",
                "clipboard" => "stdout",
                _ => "paste",
            };
            Some(Request {
                cmd: "set",
                key: Some("inject_mode"),
                value: Some(next.to_string()),
                on: None,
            })
        }
        "lang" => {
            let next = match settings.language.as_str() {
                "ru" => "en",
                "en" => "auto",
                _ => "ru",
            };
            Some(Request {
                cmd: "set",
                key: Some("language"),
                value: Some(next.to_string()),
                on: None,
            })
        }
        "keys" => {
            let presets = ["ctrl+v", "ctrl+shift+v", "shift+insert"];
            let index = presets
                .iter()
                .position(|k| *k == settings.paste_keys)
                .unwrap_or(0);
            let next = presets[(index + 1) % presets.len()];
            Some(Request {
                cmd: "set",
                key: Some("paste_keys"),
                value: Some(next.to_string()),
                on: None,
            })
        }
        "model" => {
            if settings.models.is_empty() {
                return None;
            }
            let index = settings
                .models
                .iter()
                .position(|m| m == &settings.model)
                .unwrap_or(0);
            let next = settings.models[(index + 1) % settings.models.len()].clone();
            Some(Request {
                cmd: "set",
                key: Some("model"),
                value: Some(next),
                on: None,
            })
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::edge::{button_at, Placement};

    #[test]
    fn buttons_fit_the_panel_width() {
        let buttons = buttons(&Settings::default(), false, Edge::Bottom);
        // ряд записи (5) + ряд режимов (4) + ряд значений (5: модель, клавиши, край, закрыть…)
        assert_eq!(buttons.len(), 13);
        let space = PANEL_W - 2 * PANEL_PAD;
        // ряды: 5 кнопок, 4 кнопки, 4 кнопки (модель, клавиши, край, закрыть)
        for (row, len) in [(0usize, 5), (5, 4), (9, 4)] {
            let group = &buttons[row..row + len];
            let last = group.last().unwrap();
            assert!(
                last.x + last.w <= PANEL_PAD + space,
                "ряд не влезает: {:?}",
                group.iter().map(|b| (b.id, b.x, b.w)).collect::<Vec<_>>()
            );
            assert!(group
                .windows(2)
                .all(|w| w[0].x + w[0].w + PANEL_GAP_X <= w[1].x));
        }
    }

    #[test]
    fn the_edge_button_is_offered_on_every_side() {
        for edge in Edge::all() {
            let buttons = buttons(&Settings::default(), false, edge);
            assert!(
                buttons
                    .iter()
                    .any(|b| b.id == "edge" && b.label.ends_with(edge.label())),
                "ряд кнопок стороны {} не показывает её сторону",
                edge.as_str()
            );
        }
    }

    #[test]
    fn hit_test_uses_panel_offset() {
        let buttons = buttons(&Settings::default(), false, Edge::Bottom);
        let close = buttons.iter().find(|b| b.id == "close").unwrap();
        let cx = (close.x + close.w / 2) as f64;
        // панель поднята из «кармана»: координаты кнопок сдвигаются вместе с ней
        let shifted = Placement::new(Edge::Bottom).panel_coord(crate::render::PANEL_SURFACE_H, 0.0);
        assert_eq!(shifted, crate::render::PANEL_SURFACE_H as i32);
        assert_eq!(
            button_at(&buttons, cx, (close.y + 10) as f64 + 20.0, (0, 20), false)
                .map(|i| buttons[i].id),
            Some("close")
        );
        // мимо кнопок — ничего не выбираем
        assert_eq!(button_at(&buttons, 4.0, 4.0 + 20.0, (0, 20), false), None);
        // выше верха панели кликов нет: y меньше верха корпуса наружу не ловится
        assert_eq!(button_at(&buttons, cx, 10.0, (0, 20), false), None);
    }

    #[test]
    fn buttons_send_the_commands_the_daemon_understands() {
        let mut settings = Settings::default();
        settings.min_level = 0.35;
        settings.models = vec!["/m/a.bin".into(), "/m/b.bin".into()];
        settings.model = "/m/a.bin".into();

        let request = request_for("min+", &settings, false).unwrap();
        assert_eq!(request.cmd, "set");
        assert_eq!(request.key, Some("min_level"));
        assert_eq!(request.value.as_deref(), Some("0.40"));

        let request = request_for("min-", &settings, false).unwrap();
        assert_eq!(request.value.as_deref(), Some("0.30"));

        let request = request_for("model", &settings, false).unwrap();
        assert_eq!(request.value.as_deref(), Some("/m/b.bin"));

        let request = request_for("close", &settings, false).unwrap();
        assert_eq!(request.json().trim(), "{\"cmd\":\"panel\",\"on\":false}");

        let request = request_for("inject", &settings, false).unwrap();
        assert_eq!(request.value.as_deref(), Some("atspi"));

        assert!(request_for("rec", &settings, false).unwrap().cmd == "toggle");
        assert!(request_for("чепуха", &settings, false).is_none());
    }

    #[test]
    fn unknown_daemon_answers_do_not_break_the_panel() {
        // демон старше, чем панель: у него нет поля settings
        let settings: Settings = serde_json::from_str("{}").unwrap();
        assert_eq!(settings.language, "ru");
        assert!(buttons(&settings, true, Edge::Bottom)
            .iter()
            .any(|b| b.label == "■ стоп"));
    }
}

//! Отдельное окно настроек dicta (eframe/egui) вместо тесной панели в оверлее.
//!
//! Окно — только клиент сокета: читает снимок `{"cmd":"settings"}`, показывает его
//! вкладками и шлёт `{"cmd":"set","key":...,"value":...}` по каждому изменению.
//! Демон проверяет значение, применяет его к живому конфигу и пишет config.toml;
//! отказ демона показывается красной строкой внизу окна.

mod net;
mod state;

use std::path::PathBuf;

use eframe::egui;

use net::Client;
use state::{from_reply, Settings};

/// Вкладки окна.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Tab {
    Record,
    Model,
    Insert,
    System,
}

impl Tab {
    fn title(self) -> &'static str {
        match self {
            Tab::Record => "Запись",
            Tab::Model => "Модель",
            Tab::Insert => "Вставка",
            Tab::System => "Система",
        }
    }
}

struct App {
    socket: PathBuf,
    client: Option<Client>,
    settings: Settings,
    /// Ответ демона с ошибкой (или ошибка соединения) — красная строка внизу.
    error: Option<String>,
    tab: Tab,
    /// Произвольные клавиши вставки: шлём демону по потере фокуса, не на каждый символ.
    paste_keys_edit: String,
}

impl App {
    fn new(socket: PathBuf) -> Self {
        let mut app = Self {
            socket,
            client: None,
            settings: Settings::default(),
            error: None,
            tab: Tab::Record,
            paste_keys_edit: String::new(),
        };
        app.reconnect();
        app
    }

    /// (Пере)подключиться и взять снимок настроек.
    fn reconnect(&mut self) {
        self.client = None;
        match Client::connect(&self.socket).and_then(|mut client| {
            let reply = client.call(net::settings_json())?;
            Ok((client, reply))
        }) {
            Ok((client, reply)) => {
                self.client = Some(client);
                let (settings, error) = from_reply(&reply);
                self.paste_keys_edit = settings.paste_keys.clone();
                self.settings = settings;
                self.error = error;
            }
            Err(err) => self.error = Some(err.to_string()),
        }
    }

    /// Отправить изменение и обновить снимок из ответа демона.
    fn set(&mut self, key: &'static str, value: String) {
        let Some(client) = self.client.as_mut() else {
            self.error = Some("нет соединения с демоном — нажми «переподключиться»".into());
            return;
        };
        match client.call(&net::set_json(key, &value)) {
            Ok(reply) => {
                let (settings, error) = from_reply(&reply);
                if key == "paste_keys" {
                    self.paste_keys_edit = settings.paste_keys.clone();
                }
                self.settings = settings;
                self.error = error;
            }
            Err(err) => {
                self.error = Some(err.to_string());
                self.client = None;
            }
        }
    }

    fn disconnected_view(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(48.0);
            ui.heading("демон не отвечает");
            ui.add_space(8.0);
            ui.label(format!("сокет: {}", self.socket.display()));
            ui.label("запусти: systemctl --user start dictad.service");
            ui.add_space(16.0);
            if ui.button("переподключиться").clicked() {
                self.reconnect();
            }
        });
    }

    fn record_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("Запись");
        // Копируем значения: дальше нужен &mut self для отправки команд.
        let s = &self.settings;
        let min_level = s.min_level;
        let auto_stop = s.auto_stop;
        let auto_stop_level = s.auto_stop_level;
        let auto_stop_after = s.auto_stop_after_ms;
        let auto_stop_grace = s.auto_stop_grace_ms;
        let max_record = s.max_record_ms;

        level_slider(ui, self, "min_level", "порог тишины", min_level);
        ui.add_space(8.0);
        checkbox(ui, self, "auto_stop", "авто-стоп по тишине", auto_stop);
        level_slider(ui, self, "auto_stop_level", "уровень тишины", auto_stop_level);
        ms_slider(
            ui,
            self,
            "auto_stop_after_ms",
            "тишина перед остановкой",
            1,
            auto_stop_after,
            200,
            60_000,
        );
        ms_slider(
            ui,
            self,
            "auto_stop_grace_ms",
            "не чувствовать старт записи",
            1,
            auto_stop_grace,
            0,
            20_000,
        );
        ms_slider(
            ui,
            self,
            "max_record_ms",
            "максимальная длительность",
            0,
            max_record,
            1_000,
            600_000,
        );
    }

    fn model_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("Модель");
        if self.settings.reloading {
            ui.label("модель загружается…");
        }
        let model = self.settings.model.clone();
        let models = self.settings.models.clone();
        if models.is_empty() {
            ui.label("модели не найдены");
        } else {
            for path in models {
                let name = model_name(&path);
                let selected = path == model;
                let label = if selected {
                    format!("{name} ✓")
                } else {
                    name
                };
                if ui.add(egui::SelectableLabel::new(selected, label)).clicked() {
                    self.set("model", path);
                }
            }
        }

        ui.separator();
        let s = &self.settings;
        let use_gpu = s.use_gpu;
        let gpu_device = s.gpu_device;
        let n_threads = s.n_threads;
        let beam_size = s.beam_size;

        checkbox(ui, self, "use_gpu", "GPU (иначе CPU)", use_gpu);
        ui.label("устройство GPU:");
        for (device, label) in [
            (0, "0 — быстрое (замер: ~200 мс)"),
            (1, "1 — медленное (~4 с)"),
        ] {
            if ui
                .add(egui::SelectableLabel::new(gpu_device == device, label))
                .clicked()
            {
                self.set("gpu_device", device.to_string());
            }
        }
        int_slider(ui, self, "n_threads", "потоки", n_threads, 1..=32);
        int_slider(ui, self, "beam_size", "beam (1 = greedy, быстрее)", beam_size, 1..=8);
    }

    fn insert_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("Вставка");
        let inject_mode = self.settings.inject_mode.clone();
        ui.label("как вставлять результат:");
        for mode in ["paste", "atspi", "clipboard", "stdout"] {
            if ui
                .add(egui::SelectableLabel::new(inject_mode == mode, mode))
                .clicked()
            {
                self.set("inject_mode", mode.to_string());
            }
        }

        ui.separator();
        let paste_keys = self.settings.paste_keys.clone();
        ui.label("клавиши вставки:");
        for keys in ["ctrl+v", "ctrl+shift+v", "shift+insert"] {
            if ui
                .add(egui::SelectableLabel::new(paste_keys == keys, keys))
                .clicked()
            {
                self.set("paste_keys", keys.to_string());
            }
        }
        // Произвольная комбинация: шлём, только когда поле потеряло фокус.
        let mut edited = self.paste_keys_edit.clone();
        let response = ui.add(egui::TextEdit::singleline(&mut edited).hint_text("ctrl+v"));
        if response.lost_focus() && edited != self.settings.paste_keys {
            self.set("paste_keys", edited.clone());
        }
        self.paste_keys_edit = edited;

        ui.separator();
        let s = &self.settings;
        let atspi_any_role = s.atspi_allow_any_role;
        let clipboard_restore = s.clipboard_restore_ms;
        let paste_delay = s.paste_delay_ms;
        checkbox(
            ui,
            self,
            "atspi_allow_any_role",
            "AT-SPI: писать в любые роли (для диагностики)",
            atspi_any_role,
        );
        ms_slider(
            ui,
            self,
            "clipboard_restore_ms",
            "возврат буфера в фоне",
            1,
            clipboard_restore,
            200,
            60_000,
        );
        delay_slider(ui, self, "потолок ожидания буфера", paste_delay);
    }

    fn system_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("Система");
        let language = self.settings.language.clone();
        ui.label("язык распознавания:");
        for lang in ["ru", "en", "auto"] {
            if ui
                .add(egui::SelectableLabel::new(language == lang, lang))
                .clicked()
            {
                self.set("language", lang.to_string());
            }
        }

        ui.separator();
        ui.label("конфиг (только чтение):");
        let mut config_path = self.settings.config_path.clone();
        ui.add_enabled(false, egui::TextEdit::singleline(&mut config_path));
        ui.label("сокет демона (только чтение):");
        let mut socket = self.socket.display().to_string();
        ui.add_enabled(false, egui::TextEdit::singleline(&mut socket));
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.set_theme(egui::Theme::Dark);

        if self.client.is_some() {
            egui::TopBottomPanel::top("tabs").show(ctx, |ui| {
                ui.horizontal(|ui| {
                    for tab in [Tab::Record, Tab::Model, Tab::Insert, Tab::System] {
                        if ui
                            .add(egui::SelectableLabel::new(self.tab == tab, tab.title()))
                            .clicked()
                        {
                            self.tab = tab;
                        }
                    }
                });
            });
        }

        egui::TopBottomPanel::bottom("status").show(ctx, |ui| {
            if let Some(error) = &self.error {
                ui.colored_label(egui::Color32::from_rgb(255, 120, 120), error);
            } else if self.settings.reloading {
                ui.label("модель загружается…");
            } else {
                ui.weak(format!("конфиг: {}", self.settings.config_path));
            }
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            if self.client.is_none() {
                self.disconnected_view(ui);
                return;
            }
            egui::ScrollArea::vertical().show(ui, |ui| {
                match self.tab {
                    Tab::Record => self.record_tab(ui),
                    Tab::Model => self.model_tab(ui),
                    Tab::Insert => self.insert_tab(ui),
                    Tab::System => self.system_tab(ui),
                }
            });
        });
    }
}

/// Имя модели без пути и без «ggml-», как в панели оверлея.
fn model_name(model: &str) -> String {
    let file = model.rsplit('/').next().unwrap_or(model);
    let stem = file.strip_suffix(".bin").unwrap_or(file);
    stem.strip_prefix("ggml-").unwrap_or(stem).to_string()
}

/// Слайдер уровня 0..1: шлём по отпусканию ползунка, не на каждый кадр.
fn level_slider(ui: &mut egui::Ui, app: &mut App, key: &'static str, label: &str, value: f32) {
    let mut pending = value;
    let response = ui.add(
        egui::Slider::new(&mut pending, 0.05..=1.0)
            .text(label)
            .step_by(0.05)
            .custom_formatter(|v, _| format!("{v:.2}")),
    );
    if response.drag_stopped() || (response.changed() && !ui.ctx().is_using_pointer()) {
        app.set(key, format!("{pending:.2}"));
    }
}

/// Слайдер миллисекунд, который показывает секунды.
fn ms_slider(
    ui: &mut egui::Ui,
    app: &mut App,
    key: &'static str,
    label: &str,
    decimals: usize,
    value: u64,
    min: u64,
    max: u64,
) {
    let mut pending = value;
    let response = ui.add(
        egui::Slider::new(&mut pending, min..=max)
            .text(label)
            .custom_formatter(move |v, _| {
                let secs = v / 1000.0;
                format!("{secs:.decimals$} с")
            }),
    );
    if response.drag_stopped() || (response.changed() && !ui.ctx().is_using_pointer()) {
        app.set(key, pending.to_string());
    }
}

/// Задержка вставки показывается в миллисекундах.
fn delay_slider(ui: &mut egui::Ui, app: &mut App, label: &str, value: u64) {
    let mut pending = value;
    let response = ui.add(
        egui::Slider::new(&mut pending, 0..=3_000u64)
            .text(label)
            .custom_formatter(|v, _| format!("{v:.0} мс")),
    );
    if response.drag_stopped() || (response.changed() && !ui.ctx().is_using_pointer()) {
        app.set("paste_delay_ms", pending.to_string());
    }
}

/// Целочисленный слайдер (потоки, beam).
fn int_slider(
    ui: &mut egui::Ui,
    app: &mut App,
    key: &'static str,
    label: &str,
    value: i32,
    range: std::ops::RangeInclusive<i32>,
) {
    let mut pending = value;
    let response = ui.add(egui::Slider::new(&mut pending, range).text(label));
    if response.drag_stopped() || (response.changed() && !ui.ctx().is_using_pointer()) {
        app.set(key, pending.to_string());
    }
}

/// Чекбокс: демон принимает on/off (и true/false тоже, но шлём его родное).
fn checkbox(ui: &mut egui::Ui, app: &mut App, key: &'static str, label: &str, value: bool) {
    let mut pending = value;
    if ui.checkbox(&mut pending, label).changed() {
        app.set(key, if pending { "on" } else { "off" }.to_string());
    }
}

fn main() -> anyhow::Result<()> {
    let cfg = dicta_core::config::Config::load(
        std::env::var_os("DICTA_CONFIG")
            .as_deref()
            .map(std::path::Path::new),
    )?;
    let socket = cfg.socket.clone();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([460.0, 560.0])
            .with_title("Dicta — настройки")
            // На X11 работает, на KWin/Wayland заголовок окна нужно отметить вручную
            // (или сделать правило Window Rules) — см. README.
            .with_window_level(egui::viewport::WindowLevel::AlwaysOnTop),
        ..Default::default()
    };
    eframe::run_native(
        "dicta-settings",
        options,
        Box::new(move |_cc| Ok(Box::new(App::new(socket.clone())))),
    )
    .map_err(|err| anyhow::anyhow!("eframe: {err}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn model_name_strips_path_prefix_and_extension() {
        assert_eq!(model_name("/x/ggml-medium-q5_0.bin"), "medium-q5_0");
        assert_eq!(model_name("ggml-small.bin"), "small");
        assert_eq!(model_name("/x/model"), "model");
    }

    #[test]
    fn tabs_have_russian_titles() {
        assert_eq!(Tab::Record.title(), "Запись");
        assert_eq!(Tab::Model.title(), "Модель");
        assert_eq!(Tab::Insert.title(), "Вставка");
        assert_eq!(Tab::System.title(), "Система");
    }
}

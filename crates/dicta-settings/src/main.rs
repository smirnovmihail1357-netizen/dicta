//! Отдельное окно настроек dicta (eframe/egui) вместо тесной панели в оверлее.
//!
//! Окно — только клиент сокета: читает снимок `{"cmd":"settings"}`, показывает его
//! вкладками и шлёт `{"cmd":"set","key":...,"value":...}` по каждому изменению.
//! Демон проверяет значение, применяет его к живому конфигу и пишет config.toml;
//! отказ демона показывается красной строкой внизу окна.

mod catalog;
mod hf;
mod net;
mod state;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::Duration;

use eframe::egui;

use hf::{HfFile, HfRepo};
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

/// Идущая загрузка модели: курл в фоне плюс данные для полосы прогресса.
struct Download {
    child: Child,
    url: String,
    file: String,
    dest: PathBuf,
    /// 0 — размер неизвестен, тогда вместо полосы показываем спиннер.
    expected: u64,
}

/// Идущий запрос к Hugging Face API (поиск или дерево репозитория).
struct Fetch {
    child: Child,
    kind: FetchKind,
}

enum FetchKind {
    Search,
    Tree(String),
}

struct App {
    socket: PathBuf,
    client: Option<Client>,
    settings: Settings,
    /// Ответ демона с ошибкой (или ошибка соединения) — красная строка внизу.
    error: Option<String>,
    /// Зелёная строка-подсказка внизу окна (например, «модель скачана»); ошибка важнее её.
    flash_info: Option<String>,
    tab: Tab,
    /// Произвольные клавиши вставки: шлём демону по потере фокуса, не на каждый символ.
    paste_keys_edit: String,
    /// Выбранная строка каталога моделей Hugging Face.
    catalog_pick: usize,
    /// Фоновая загрузка ggml-модели (одновременно только одна).
    download: Option<Download>,
    /// Поиск модели на Hugging Face.
    hf_query: String,
    hf_repos: Vec<HfRepo>,
    /// Выбранный репозиторий: когда он есть, показываем его файлы.
    hf_repo: Option<String>,
    hf_files: Vec<HfFile>,
    /// Ссылка, вставленная пользователем вручную.
    hf_direct_url: String,
    /// Один активный запрос к API (отдельный слот — не мешает загрузке).
    hf_fetch: Option<Fetch>,
}

impl App {
    fn new(socket: PathBuf) -> Self {
        let mut app = Self {
            socket,
            client: None,
            settings: Settings::default(),
            error: None,
            flash_info: None,
            tab: Tab::Record,
            paste_keys_edit: String::new(),
            catalog_pick: 0,
            download: None,
            hf_query: String::new(),
            hf_repos: Vec::new(),
            hf_repo: None,
            hf_files: Vec::new(),
            hf_direct_url: String::new(),
            hf_fetch: None,
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

    /// Перечитать снимок: скачанный файл сразу появится в `settings.models`.
    fn refresh_snapshot(&mut self) {
        let Some(client) = self.client.as_mut() else {
            return;
        };
        match client.call(net::settings_json()) {
            Ok(reply) => {
                let (settings, error) = from_reply(&reply);
                self.settings = settings;
                self.error = error;
            }
            Err(err) => {
                self.error = Some(err.to_string());
                self.client = None;
            }
        }
    }

    /// Скачать модель по готовой ссылке: `.part` не удаляем при ошибке — curl докачает его (`-C -`).
    fn start_download(&mut self, file: String, url: String, expected: u64) {
        if self.download.is_some() {
            return;
        }
        // Кладём рядом с текущей моделью: демон листает .bin из её parent-каталога.
        let dest = catalog::dest_dir(&self.settings.model).join(&file);
        if dest.exists() {
            self.flash_info = Some(format!("{file} уже есть — выбери её в списке выше"));
            return;
        }
        if let Some(dir) = dest.parent().filter(|dir| !dir.as_os_str().is_empty()) {
            if let Err(err) = fs::create_dir_all(dir) {
                self.error = Some(format!("папка моделей {}: {err}", dir.display()));
                return;
            }
        }
        let part = catalog::part_of(&dest);
        let child = Command::new("curl")
            .args(["-L", "--fail", "--retry", "3", "-C", "-", "-o"])
            .arg(&part)
            .arg(&url)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn();
        match child {
            Ok(child) => {
                self.error = None;
                self.flash_info = None;
                self.download = Some(Download {
                    child,
                    url,
                    file,
                    dest,
                    expected,
                });
            }
            Err(err) => self.error = Some(format!("curl не найден: {err}")),
        }
    }

    /// Запрос к Hugging Face API: вывод читаем, поэтому stdout — pipe, а не null.
    fn start_fetch(&mut self, kind: FetchKind) {
        if self.hf_fetch.is_some() {
            return;
        }
        let url = match &kind {
            FetchKind::Search => hf::search_url(&self.hf_query),
            FetchKind::Tree(repo) => hf::tree_url(repo),
        };
        let child = Command::new("curl")
            .args(["-s", "--fail"])
            .arg(&url)
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn();
        match child {
            Ok(child) => {
                self.error = None;
                self.hf_fetch = Some(Fetch { child, kind });
            }
            Err(err) => self.error = Some(format!("curl не найден: {err}")),
        }
    }

    /// Поллинг API-запроса: пока он идёт, просим перерисовать окно через 100 мс.
    fn poll_fetch(&mut self, ctx: &egui::Context) {
        let polled = match self.hf_fetch.as_mut() {
            Some(fetch) => match fetch.child.try_wait() {
                Ok(None) => {
                    ctx.request_repaint_after(Duration::from_millis(100));
                    return;
                }
                Ok(Some(status)) => Some(Ok(status)),
                Err(err) => Some(Err(err.to_string())),
            },
            None => None,
        };
        let Some(polled) = polled else {
            return;
        };
        let mut done = self.hf_fetch.take().expect("запрос только что был");
        match polled {
            Ok(status) if status.success() => {
                let output = match done.child.wait_with_output() {
                    Ok(output) => output,
                    Err(err) => {
                        self.error = Some(format!("не удалось прочитать ответ HF: {err}"));
                        return;
                    }
                };
                let text = String::from_utf8_lossy(&output.stdout);
                match done.kind {
                    FetchKind::Search => {
                        self.hf_repos = hf::parse_search(&text);
                        self.hf_repo = None;
                        self.hf_files.clear();
                        if self.hf_repos.is_empty() {
                            self.flash_info = Some("ничего не нашлось".into());
                        }
                    }
                    FetchKind::Tree(repo) => {
                        self.hf_files = hf::parse_tree(&text);
                        self.hf_repo = Some(repo);
                        if self.hf_files.is_empty() {
                            self.flash_info = Some("в этом репозитории нет .bin файлов".into());
                        }
                    }
                }
            }
            Ok(status) => {
                let _ = done.child.wait();
                self.error = Some(format!(
                    "запрос к Hugging Face не удался (curl код {})",
                    status.code().unwrap_or(-1)
                ));
            }
            Err(err) => {
                let _ = done.child.wait();
                self.error = Some(format!("не удалось проверить curl: {err}"));
            }
        }
    }

    /// Отмена: убить курл и стереть `.part`, чтобы следующий старт начал с нуля.
    fn cancel_download(&mut self) {
        let Some(mut download) = self.download.take() else {
            return;
        };
        let _ = download.child.kill();
        let _ = download.child.wait(); // убрать зомби после kill
        let _ = fs::remove_file(catalog::part_of(&download.dest));
        self.flash_info = Some(format!("загрузка {} прервана", download.file));
    }

    /// Поллинг курла: пока идёт загрузка, просим перерисовать окно через 500 мс.
    fn poll_download(&mut self, ctx: &egui::Context) {
        let polled = match self.download.as_mut() {
            Some(download) => match download.child.try_wait() {
                Ok(None) => {
                    ctx.request_repaint_after(Duration::from_millis(500));
                    return;
                }
                Ok(Some(status)) => Some(Ok(status)),
                Err(err) => Some(Err(err.to_string())),
            },
            None => None,
        };
        let Some(polled) = polled else {
            return;
        };
        // try_wait уже подобрал процесс, можно забирать состояние без wait().
        let done = self.download.take().expect("загрузка только что была");
        match polled {
            Ok(status) if status.success() => {
                match fs::rename(catalog::part_of(&done.dest), &done.dest) {
                    Ok(()) => {
                        self.refresh_snapshot();
                        self.flash_info = Some(format!(
                            "модель {} скачана — выбери её в списке выше",
                            done.file
                        ));
                    }
                    Err(err) => self.error = Some(format!("не удалось переименовать .part: {err}")),
                }
            }
            Ok(status) => {
                self.error = Some(format!(
                    "curl упал (код {}): повтор продолжит загрузку, часть файла сохранена",
                    status.code().unwrap_or(-1)
                ));
            }
            Err(err) => self.error = Some(format!("не удалось проверить curl: {err}")),
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
        level_slider(
            ui,
            self,
            "auto_stop_level",
            "уровень тишины",
            auto_stop_level,
        );
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
                if ui
                    .add(egui::SelectableLabel::new(selected, label))
                    .clicked()
                {
                    self.set("model", path);
                }
            }
        }

        ui.separator();
        if let Some(download) = &self.download {
            // Прогресс виден при любой загрузке: из каталога, поиска или по прямой ссылке.
            let file = download.file.clone();
            let url = download.url.clone();
            let expected = download.expected;
            let len = fs::metadata(catalog::part_of(&download.dest))
                .map(|meta| meta.len())
                .unwrap_or(0);
            ui.label(format!("скачивание {file}"));
            ui.weak(url);
            if expected > 0 {
                let fraction = (len as f32 / expected as f32).min(1.0);
                ui.add(egui::ProgressBar::new(fraction).show_percentage());
                ui.label(format!(
                    "{file}: {:.0} из ≈{:.0} МБ",
                    len as f64 / 1_000_000.0,
                    expected as f64 / 1_000_000.0
                ));
            } else {
                // Размер неизвестен: процента нет, показываем живой счётчик.
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.label(format!(
                        "{file}: скачано {:.0} МБ",
                        len as f64 / 1_000_000.0
                    ));
                });
            }
            if ui.button("отмена").clicked() {
                self.cancel_download();
            }
        }

        ui.heading("Быстрый выбор (проверенные)");
        let current_model = self.settings.model.clone();
        let downloading = self.download.is_some();
        for (i, entry) in catalog::CATALOG.iter().enumerate() {
            let exists = catalog::dest_dir(&current_model).join(entry.file).exists();
            let name = model_name(entry.file);
            let caption = if exists {
                format!("{name} · есть ✓")
            } else {
                name
            };
            ui.horizontal(|ui| {
                if ui
                    .add(egui::SelectableLabel::new(self.catalog_pick == i, caption))
                    .clicked()
                {
                    self.catalog_pick = i;
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.weak(format!(
                        "≈{} МБ · {}",
                        entry.approx_bytes / 1_000_000,
                        entry.note
                    ));
                });
            });
        }
        let pick = &catalog::CATALOG[self.catalog_pick];
        let exists = catalog::dest_dir(&current_model).join(pick.file).exists();
        let clicked = ui
            .add_enabled(
                !exists && !downloading,
                egui::Button::new(if exists {
                    "уже есть"
                } else {
                    "скачать"
                }),
            )
            .clicked();
        if clicked {
            self.start_download(
                pick.file.to_string(),
                catalog::url_for(pick.file),
                pick.approx_bytes,
            );
        }

        ui.separator();
        ui.heading("Найти любую модель");
        ui.horizontal(|ui| {
            ui.add(
                egui::TextEdit::singleline(&mut self.hf_query).hint_text("например: whisper ggml"),
            );
            let busy = self.hf_fetch.is_some();
            if ui
                .add_enabled(
                    !busy && !self.hf_query.trim().is_empty(),
                    egui::Button::new("найти"),
                )
                .clicked()
            {
                self.start_fetch(FetchKind::Search);
            }
        });
        if let Some(repo) = &self.hf_repo {
            let repo = repo.clone();
            ui.label(format!("репозиторий: {repo}"));
            if ui.button("← назад к результатам").clicked() {
                self.hf_repo = None;
            }
            let files = self.hf_files.clone();
            for file in &files {
                let exists = catalog::dest_dir(&current_model).join(&file.path).exists();
                let path = file.path.clone();
                let size = file.size;
                ui.horizontal(|ui| {
                    ui.label(format!("{path} — ≈{} МБ", size / 1_000_000));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if exists {
                            ui.weak("есть ✓");
                        } else if ui
                            .add_enabled(!downloading, egui::Button::new("скачать"))
                            .clicked()
                        {
                            self.start_download(path.clone(), hf::resolve_url(&repo, &path), size);
                        }
                    });
                });
            }
        } else {
            let repos = self.hf_repos.clone();
            for repo in &repos {
                ui.horizontal(|ui| {
                    let clicked = ui
                        .add_enabled(
                            self.hf_fetch.is_none(),
                            egui::SelectableLabel::new(false, repo.id.clone()),
                        )
                        .clicked();
                    ui.weak(format!("— {} скачиваний", repo.downloads));
                    if clicked {
                        self.hf_files.clear();
                        self.start_fetch(FetchKind::Tree(repo.id.clone()));
                    }
                });
            }
        }

        ui.separator();
        ui.heading("Прямая ссылка");
        ui.horizontal(|ui| {
            ui.add(
                egui::TextEdit::singleline(&mut self.hf_direct_url)
                    .hint_text("https://huggingface.co/<repo>/resolve/main/<file>.bin"),
            );
            if ui
                .add_enabled(!downloading, egui::Button::new("скачать"))
                .clicked()
            {
                let url = self.hf_direct_url.trim().to_string();
                let is_http = url.starts_with("http://") || url.starts_with("https://");
                match hf::filename_of(&url) {
                    Some(name) if is_http => self.start_download(name, url, 0),
                    _ => self.error = Some("вставь ссылку на файл (http/https)".into()),
                }
            }
        });

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
        int_slider(
            ui,
            self,
            "beam_size",
            "beam (1 = greedy, быстрее)",
            beam_size,
            1..=8,
        );
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
        // Курлы идут в фоне: проверяем оба слота и поддерживаем перерисовку.
        self.poll_download(ctx);
        self.poll_fetch(ctx);

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
            } else if let Some(info) = &self.flash_info {
                ui.colored_label(egui::Color32::from_rgb(120, 220, 120), info);
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
            egui::ScrollArea::vertical().show(ui, |ui| match self.tab {
                Tab::Record => self.record_tab(ui),
                Tab::Model => self.model_tab(ui),
                Tab::Insert => self.insert_tab(ui),
                Tab::System => self.system_tab(ui),
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
        std::env::var_os("DICTA_CONFIG").as_deref().map(Path::new),
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

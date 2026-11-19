//! dicta-ui — маленький оверлей над всеми окнами: `● REC 0:03 ▁▂▄█  ⏎ готово · esc отмена`.
//!
//! Рендер — wl_shm (софт), слой — zwlr_layer_shell_v1 (KWin, Hyprland, sway и др.).
//! Состояние читается из демона по Unix-сокету каждые 100 мс; клик мышью по оверлею = stop,
//! а если зажать пилюлю и потащить её к другой границе монитора — оверлей переезжает на
//! эту сторону (снизу/сверху горизонтальный, слева/справа вертикальный) и запоминает
//! положение в ui.toml, чтобы в следующий раз появиться там же.
//! Появление и уход — анимация: пилюля выезжает из-за кромки своей стороны (с лёгким
//! пружинным overshoot'ом) и уходит обратно за неё. Геометрия стороны — в `edge.rs`,
//! картинка — в `render.rs`.

mod drag;
mod edge;
mod panel;
mod render;

use std::os::fd::{AsFd, AsRawFd};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use ab_glyph::FontRef;
use anyhow::Context;
use anyhow::Result;
use serde::Deserialize;
use smithay_client_toolkit::{
    compositor::{CompositorHandler, CompositorState},
    delegate_registry,
    output::{OutputHandler, OutputState},
    registry::{ProvidesRegistryState, RegistryState},
    registry_handlers,
    seat::pointer::{PointerEvent, PointerEventKind, PointerHandler},
    seat::{Capability, SeatHandler, SeatState},
    shell::{
        wlr_layer::{
            Anchor, KeyboardInteractivity, Layer, LayerShell, LayerShellHandler, LayerSurface,
            LayerSurfaceConfigure,
        },
        WaylandSurface,
    },
    shm::{slot::SlotPool, Shm, ShmHandler},
};
use wayland_client::{
    globals::registry_queue_init,
    protocol::{
        wl_compositor, wl_output, wl_pointer, wl_region, wl_registry, wl_seat, wl_shm, wl_surface,
    },
    Connection, Dispatch, Proxy, QueueHandle,
};

use panel::Settings;
use render::{ease_in_cubic, ease_out_back, paint, Button, Canvas, Scene, SURF_H, SURF_W};

use edge::{Edge, Placement};

/// Опрос демона.
const POLL_MS: u64 = 100;
/// Кадр во время анимации.
const FRAME_MS: u64 = 16;
/// Как часто просыпаться, когда пилюля скрыта (чтобы заметить начало записи).
const IDLE_MS: u64 = 50;
/// Сколько пикселей надо утащить пилюлю, чтобы движение стало «перестановкой»,
/// а не обычным кликом по ней.

fn debug_on() -> bool {
    std::env::var("DICTA_DEBUG").is_ok()
}

fn env_num(name: &str, default: u64) -> u64 {
    std::env::var(name)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

/// Длительность появления из-за кромки своей стороны (мс). `DICTA_ANIM=0` — без анимации.
fn anim_rise_ms() -> u64 {
    if !anim_on() {
        return 0;
    }
    env_num("DICTA_ANIM_MS", 260)
}

/// Длительность ухода обратно за кромку своей стороны (мс).
fn anim_fall_ms() -> u64 {
    if !anim_on() {
        return 0;
    }
    env_num("DICTA_ANIM_OUT_MS", 170)
}

fn anim_on() -> bool {
    !matches!(
        std::env::var("DICTA_ANIM").unwrap_or_default().as_str(),
        "0" | "off" | "none" | "no"
    )
}

/// Экспериментальные knobs для подбора режима, который понимает конкретный композитор.
fn layer_kind() -> Layer {
    match std::env::var("DICTA_LAYER").unwrap_or_default().as_str() {
        "background" => Layer::Background,
        "bottom" => Layer::Bottom,
        "top" => Layer::Top,
        _ => Layer::Overlay,
    }
}

fn exclusive_zone() -> i32 {
    std::env::var("DICTA_EXCLUSIVE")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(0)
}

fn always_draw() -> bool {
    std::env::var("DICTA_ALWAYS_DRAW").is_ok()
}

/// Для отладки панели: показать её без команды демона.
fn panel_always() -> bool {
    std::env::var("DICTA_PANEL").is_ok()
}

/// Ответ dictad на {"cmd":"status"}.
#[derive(Debug, Clone, Deserialize, Default)]
struct Status {
    #[serde(default)]
    state: String,
    #[serde(default)]
    level: f32,
    #[serde(default)]
    seconds: f32,
    /// Открыта ли панель настроек (её состояние держит демон).
    #[serde(default)]
    panel: bool,
    #[serde(default)]
    #[allow(dead_code)] // зарезервировано: сообщения демона можно показывать в оверлее
    error: Option<String>,
}

/// Всё, что оверлей знает о демоне: одно состояние на два окна (пилюлю и панель).
#[derive(Debug, Default)]
struct Shared {
    status: Status,
    levels: Vec<f32>,
    settings: Option<Settings>,
}

const FONT_PATHS: &[&str] = &[
    "/usr/share/fonts/TTF/DejaVuSans.ttf",
    "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
];

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();

    // dicta-ui --probe: проверить, понимает ли композитор layer shell, и выйти.
    if args.iter().any(|a| a == "--probe") {
        return probe();
    }
    // dicta-ui --preview <dir>: нарисовать кадры анимации в BMP и выйти (без Wayland).
    if let Some(i) = args.iter().position(|a| a == "--preview") {
        let dir = args
            .get(i + 1)
            .map(Path::new)
            .unwrap_or(Path::new("dicta-preview"));
        return preview(dir);
    }
    // dicta-ui --list-edges: показать, где оверлей стоит сейчас и куда умеет вставать.
    if args.iter().any(|a| a == "--list-edges") {
        return list_edges();
    }
    // dicta-ui --edge right|left|top|bottom: записать сторону и выйти. Следующий
    // запуск оверлея (или его перезагрузка демоном) встанет туда.
    if let Some(i) = args.iter().position(|a| a == "--edge") {
        return set_edge(args.get(i + 1).map(String::as_str));
    }
    run()
}

/// Записать сторону в ui.toml, сохранив смещение вдоль края и отступ от него.
fn set_edge(raw: Option<&str>) -> Result<()> {
    let Some(raw) = raw else {
        anyhow::bail!("--edge: укажи bottom, top, left или right (dicta-ui --list-edges)");
    };
    let Some(edge) = Edge::parse(raw) else {
        anyhow::bail!("--edge: не сторона {raw:?} — бывают bottom, top, left, right");
    };
    let path = edge::config_path();
    let mut cfg = edge::load(&path);
    cfg.edge = Some(edge);
    edge::save(&path, &cfg)?;
    println!("сторона: {} -> {}", path.display(), edge.as_str());
    println!(
        "перезапусти оверлей: pkill -x dicta-ui (демон поднимет его заново) или выйди и войди"
    );
    Ok(())
}

/// Текущее положение + все стороны, которые понимает оверлей.
fn list_edges() -> Result<()> {
    let path = edge::config_path();
    let current = edge::load(&path).placement();
    println!("ui.toml: {}", path.display());
    println!(
        "сейчас: {} (вдоль края {}, от края {})",
        current.edge.as_str(),
        current.cross(),
        current.along()
    );
    for edge in Edge::all() {
        println!(
            "  {:<7} {}",
            edge.as_str(),
            if edge.horizontal() {
                "горизонтально"
            } else {
                "вертикально"
            }
        );
    }
    println!("переставить: dicta-ui --edge right|left|top|bottom");
    Ok(())
}

/// Минимальный скан реестра: есть ли у композитора zwlr_layer_shell_v1.
struct Probe {
    found: std::cell::Cell<bool>,
}

impl Dispatch<wl_registry::WlRegistry, ()> for Probe {
    fn event(
        state: &mut Self,
        _proxy: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let wl_registry::Event::Global {
            name, interface, ..
        } = event
        {
            if interface == "zwlr_layer_shell_v1" {
                state.found.set(true);
                println!("layer shell есть (global #{name}) — оверлей будет работать");
            }
        }
    }
}

fn probe() -> Result<()> {
    let conn = Connection::connect_to_env()?;
    let display = conn.display();
    let mut queue = conn.new_event_queue::<Probe>();
    let qh = queue.handle();
    display.get_registry(&qh, ());

    let mut state = Probe {
        found: std::cell::Cell::new(false),
    };
    // Реестр присылает список global'ов одним пакетом; пара раундов хватает.
    for _ in 0..3 {
        queue.blocking_dispatch(&mut state)?;
        if state.found.get() {
            return Ok(());
        }
    }
    eprintln!("layer shell нет: композитор не объявляет zwlr_layer_shell_v1");
    anyhow::bail!("нет layer shell");
}

fn load_font() -> Option<FontRef<'static>> {
    for path in FONT_PATHS {
        if let Ok(bytes) = std::fs::read(path) {
            let boxed: &'static [u8] = Box::leak(bytes.into_boxed_slice());
            if let Ok(font) = FontRef::try_from_slice(boxed) {
                return Some(font);
            }
        }
    }
    None
}

/// Kadры анимации как BMP: чтобы подбирать длительность и кривую, не глядя в композитор.
fn preview(dir: &Path) -> Result<()> {
    std::fs::create_dir_all(dir)?;
    let font = load_font();
    let scene = Scene {
        recording: true,
        seconds: 3.4,
        frame: 0,
        grab: Some((260.0, 20.0)),
    };
    let levels: Vec<f32> = (0..render::WAVE_LEN)
        .map(|i| {
            let x = i as f32 / render::WAVE_LEN as f32;
            (0.35 + 0.55 * (x * 9.0).sin().abs() * (1.0 - x * 0.4)).min(1.0)
        })
        .collect();

    let mut files = Vec::new();
    for i in 0..=10 {
        let t = i as f32 / 10.0;
        let reveal = ease_out_back(t);
        let c = paint(
            SURF_W,
            SURF_H,
            Placement::new(Edge::Bottom).pill_top(SURF_H, reveal),
            0.45 + 0.55 * t,
            false,
            &scene,
            &levels,
            font.as_ref(),
        );
        let path = dir.join(format!("rise_{:02}_t{t:.2}.bmp", i));
        render::write_bmp(&path, &c)?;
        files.push(path);
    }
    for i in 0..=6 {
        let t = i as f32 / 6.0;
        let reveal = 1.0 - ease_in_cubic(t);
        let c = paint(
            SURF_W,
            SURF_H,
            Placement::new(Edge::Bottom).pill_top(SURF_H, reveal),
            0.45 + 0.55 * reveal,
            false,
            &scene,
            &levels,
            font.as_ref(),
        );
        let path = dir.join(format!("fall_{:02}_t{t:.2}.bmp", i));
        render::write_bmp(&path, &c)?;
        files.push(path);
    }
    // Вертикальная ориентация (левая и правая стороны): та же пилюля, повёрнутая на 90°.
    // Вертикальная пилюля (стороны слева/справа): та же толщина, развёрнутая на 90°
    for edge in [Edge::Left, Edge::Right] {
        let placement = Placement::new(edge);
        let c = paint(
            placement.pill_width(),
            placement.pill_height(),
            placement.pill_coord(placement.pill_width(), placement.pill_height(), 1.0),
            1.0,
            true,
            &scene,
            &levels,
            font.as_ref(),
        );
        let path = dir.join(format!("pill_{}.bmp", edge.as_str()));
        render::write_bmp(&path, &c)?;
        files.push(path);
    }
    // Панель настроек: три кадра подъёма и контрольный снимок вместе с пилюлей.
    let settings = Settings {
        language: "ru".into(),
        inject_mode: "paste".into(),
        min_level: 0.35,
        auto_stop: false,
        use_gpu: true,
        paste_keys: "ctrl+v".into(),
        model: "/home/u/.dicta/models/ggml-medium-q5_0.bin".into(),
        models: vec![
            "/home/u/.dicta/models/ggml-medium-q5_0.bin".into(),
            "/home/u/.dicta/models/ggml-large-v3-turbo-q5_0.bin".into(),
        ],
        config_path: "/home/u/.config/dicta/config.toml".into(),
        ..Settings::default()
    };
    for i in [0usize, 4, 10] {
        let reveal = ease_out_back(i as f32 / 10.0);
        let keys = panel::buttons(&settings, true, Edge::Bottom);
        let c = render::paint_panel(
            render::PANEL_W,
            render::PANEL_SURFACE_H,
            Placement::new(Edge::Bottom).panel_coord(render::PANEL_SURFACE_H, reveal),
            0.5 + 0.5 * reveal,
            None,
            "dicta · запись 0:03",
            "порог 0.40 — сохранено в ~/.config/dicta/config.toml",
            "Ctrl+Alt+V — открыть/закрыть · клик по кнопке = применить · удержать и потащить к краю — переставить",
            &keys,
            font.as_ref(),
        );
        let path = dir.join(format!("panel_{:02}_t{:.1}.bmp", i, i as f32 / 10.0));
        render::write_bmp(&path, &c)?;
        files.push(path);
    }

    // Панель на каждой из четырёх сторон: размер окна и координату корпуса берём из тех
    // же Placement-функций, которыми пользуется живой оверлей (так видно, что ни одна
    // кнопка не обрезана краем окна ни на одной стороне).
    for edge in Edge::all() {
        let placement = Placement::new(edge);
        let vertical = !edge.horizontal();
        let origin = placement.panel_origin(1.0);
        let c = render::paint_panel(
            placement.panel_width(),
            placement.panel_height(),
            if vertical { origin.0 } else { origin.1 },
            1.0,
            placement.turn(),
            "dicta · запись 0:03",
            "порог 0.40 — сохранено в ~/.config/dicta/config.toml",
            "Ctrl+Alt+V — открыть/закрыть · клик по кнопке = применить · удержать и потащить к краю — переставить",
            &panel::buttons(&settings, true, edge),
            font.as_ref(),
        );
        let path = dir.join(format!("panel_{}.bmp", edge.as_str()));
        render::write_bmp(&path, &c)?;
        files.push(path);
    }

    // Контрольный снимок: панель и пилюля, обе прибиты к нижней кромке экрана.
    let mut stack = Canvas::new(render::PANEL_W, render::PANEL_SURFACE_H);
    let pill = paint(
        SURF_W,
        SURF_H,
        Placement::new(Edge::Bottom).pill_top(SURF_H, 1.0),
        1.0,
        false,
        &scene,
        &levels,
        font.as_ref(),
    );
    let panel = render::paint_panel(
        render::PANEL_W,
        render::PANEL_SURFACE_H,
        Placement::new(Edge::Bottom).panel_coord(render::PANEL_SURFACE_H, 1.0),
        1.0,
        None,
        "dicta · запись 0:03",
        "порог 0.40 — сохранено в ~/.config/dicta/config.toml",
        "Ctrl+Alt+V — открыть/закрыть · клик по кнопке = применить",
        &panel::buttons(&settings, true, Edge::Bottom),
        font.as_ref(),
    );
    stack.blit(&pill, 0, (render::PANEL_SURFACE_H - SURF_H) as i32);
    stack.blit(&panel, 0, 0);
    let path = dir.join("stack.bmp");
    render::write_bmp(&path, &stack)?;
    files.push(path);

    println!("кадры анимации: {} файлов в {}", files.len(), dir.display());
    for path in &files {
        println!("  {}", path.display());
    }
    Ok(())
}

fn run() -> Result<()> {
    let socket = dicta_core_socket_path();
    let shared: Arc<Mutex<Shared>> = Arc::new(Mutex::new(Shared::default()));
    let settings_due = Arc::new(AtomicBool::new(false));

    {
        let shared = shared.clone();
        let settings_due = settings_due.clone();
        let socket = socket.clone();
        let mut last_state = String::from("startup");
        // Настройки спрашиваем только когда панель открыта: в простое оверлей
        // по-прежнему делает один маленький запрос раз в POLL_MS.
        let mut last_settings = Instant::now() - Duration::from_secs(60);
        thread::spawn(move || loop {
            if let Ok(next) = ask_status(&socket) {
                let state = next.state.clone();
                if debug_on() && state != last_state {
                    eprintln!(
                        "status: {} level={:.2} sec={:.1} panel={}",
                        state, next.level, next.seconds, next.panel
                    );
                }
                let want_settings = if let Ok(mut cur) = shared.lock() {
                    // История уровня за последние ~1.8 с: волна рисуется по ней, а не
                    // «дрожанием». Ведёт её поток опроса, а не поток рисования: частота
                    // кадров разная (100 мс в покое, 16 мс в анимации), а волна должна
                    // быть одинаковой в обеих.
                    if state == "recording" {
                        cur.levels.push(next.level);
                        if cur.levels.len() > render::WAVE_LEN {
                            cur.levels.remove(0);
                        }
                    } else {
                        cur.levels.clear();
                    }
                    let stale =
                        cur.settings.is_none() || last_settings.elapsed() > Duration::from_secs(2);
                    let panel_open = next.panel;
                    cur.status = next;
                    panel_open && (settings_due.load(Ordering::Relaxed) || stale)
                } else {
                    false
                };
                if want_settings {
                    if let Ok(settings) = ask_settings(&socket) {
                        if let Ok(mut cur) = shared.lock() {
                            cur.settings = Some(settings);
                        }
                    }
                    settings_due.store(false, Ordering::Relaxed);
                    last_settings = Instant::now();
                }
                last_state = state;
            }
            thread::sleep(Duration::from_millis(POLL_MS));
        });
    }

    let conn = Connection::connect_to_env().map_err(|err| {
        anyhow::anyhow!("не подключился к Wayland-сессии ({err}) — запустить в сессии, не в SSH")
    })?;
    let (globals, mut event_queue) = registry_queue_init(&conn)?;
    let qh = event_queue.handle();

    let compositor = CompositorState::bind(&globals, &qh)
        .map_err(|err| anyhow::anyhow!("нет wl_compositor: {err}"))?;
    let layer_shell = LayerShell::bind(&globals, &qh).map_err(|err| {
        anyhow::anyhow!("композитор не даёт zwlr_layer_shell_v1 (layer shell): {err}")
    })?;
    let shm = Shm::bind(&globals, &qh).map_err(|err| anyhow::anyhow!("нет wl_shm: {err}"))?;

    let surface = compositor.create_surface(&qh);
    let layer = layer_shell.create_layer_surface(&qh, surface, layer_kind(), Some("dicta"), None);

    // Где стоять — берём из ui.toml (туда пишет перетаскивание мышью и --edge).
    // Сторона задаёт якорь, размер поверхности (горизонтальная/вертикальная) и то,
    // с какой кромки пилюля будет выезжать.
    // Где стоять — берём из ui.toml: туда пишет и перетаскивание мышью, и --edge.
    let ui_path = edge::config_path();
    let placement = edge::load(&ui_path).placement();
    if debug_on() {
        eprintln!(
            "сторона: {} margins {:?}",
            placement.edge.as_str(),
            placement.margins
        );
    }

    // Поверхность на REST_GAP длиннее пилюли: в этом зазоре пилюля и прячется, когда её
    // нет. Поэтому margin вдоль своей стороны — 0, а зазор «зашит» в размер поверхности.
    apply_placement(&layer, placement);
    layer.set_exclusive_zone(exclusive_zone());
    // Ноль: оверлей не перехватывает клавиатуру — клавиши остаются у фокусного окна.
    layer.set_keyboard_interactivity(KeyboardInteractivity::None);
    layer.commit();

    // Панель настроек — вторая поверхность того же процесса: та же сторона, тот же слой,
    // то же поведение (не перехватывает клавиатуру, не перекрывает клики мимо неё).
    let panel_surface = compositor.create_surface(&qh);
    let panel_layer = layer_shell.create_layer_surface(
        &qh,
        panel_surface.clone(),
        layer_kind(),
        Some("dicta-panel"),
        None,
    );
    apply_panel_placement(&panel_layer, placement);
    // Ноль и здесь: панель — не окно ввода, она только кнопок.
    panel_layer.set_keyboard_interactivity(KeyboardInteractivity::None);
    panel_layer.commit();

    // пул с запасом: одновременно живут пилюля, панель, ореол и орб
    let pool = SlotPool::new(
        (SURF_W * SURF_H)
            .max(render::PANEL_W * render::PANEL_SURFACE_H)
            .max(render::ORB_W * render::ORB_W) as usize
            * 12,
        &shm,
    )?;

    let mut ui = Ui {
        registry_state: RegistryState::new(&globals),
        seat_state: SeatState::new(&globals, &qh),
        output_state: OutputState::new(&globals, &qh),
        shm,
        exit: false,
        first_configure: true,
        pool,
        width: placement.pill_width(),
        height: placement.pill_height(),
        layer,
        pointer: None,
        shared: shared.clone(),
        settings_due: settings_due.clone(),
        font: load_font(),
        frame: 0,
        compositor: compositor.wl_compositor().clone(),
        mapped: None,
        reveal: Reveal::new(),
        dirty: false,
        shown: Status::default(),
        panel_layer,
        panel_surface,
        panel_width: placement.panel_width(),
        panel_height: placement.panel_height(),
        panel_reveal: Reveal::new(),
        panel_mapped: None,
        panel_keys: Vec::new(),
        notice: String::new(),
        panel_rest_sig: None,
        placement,
        ui_path,
        drag: drag::Drag::new(),
        screen: None,
        hint_drawn: None,
        hint_configured: false,
        pill_output: None,
        layer_shell,
        compositor_state: compositor,
        drag_output: None,
        hint_layer: None,
        hint_surface: None,
        orb_layer: None,
        orb_surface: None,
        orb_configured: false,
        orb_alpha: 0.0,
        orb_age: 0,
        orb_depart: None,
    };

    let mut next_draw = Instant::now();
    loop {
        event_queue.dispatch_pending(&mut ui)?;
        if ui.exit {
            break;
        }
        let animating = ui.tick();
        let showing = ui.reveal.want == Some(true) || ui.panel_reveal.want == Some(true);
        if !ui.first_configure && (showing || animating || ui.dirty) {
            let now = Instant::now();
            // Кадр рисуем не чаще, чем нужно: во время анимации — каждые FRAME_MS,
            // в покое — каждые POLL_MS (волна уровня обновляется раз на опрос демона).
            if ui.dirty || now >= next_draw {
                ui.draw_pill(&qh);
                ui.draw_panel(&qh);
                ui.draw_hint(&qh);
                ui.draw_orb(&qh);
                ui.dirty = false;
                next_draw = now + Duration::from_millis(if animating { FRAME_MS } else { POLL_MS });
            }
        }
        let wake = if animating {
            FRAME_MS
        } else if showing {
            POLL_MS
        } else {
            IDLE_MS
        };
        event_queue.flush()?;
        if let Some(guard) = event_queue.prepare_read() {
            let mut fd = libc::pollfd {
                fd: conn.as_fd().as_raw_fd(),
                events: libc::POLLIN,
                revents: 0,
            };
            let result = unsafe { libc::poll(&mut fd, 1, wake as i32) };
            if result < 0 {
                let err = std::io::Error::last_os_error();
                if err.kind() != std::io::ErrorKind::Interrupted {
                    return Err(err.into());
                }
            } else if result > 0 {
                guard.read()?;
            }
        }
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Anim {
    None,
    /// окно едет вверх из-под кромки экрана
    Rise,
    /// окно уходит обратно за кромку
    Fall,
}

/// Анимация одного окна: 0.0 — целиком под нижней кромкой, 1.0 — на своём месте.
/// Пилюля и панель анимируются независимо (панель может открыться, когда пилюли нет).
#[derive(Debug)]
struct Reveal {
    value: f32,
    anim: Anim,
    from: f32,
    start: Instant,
    dur: Duration,
    /// видим ли окно по состоянию демона
    want: Option<bool>,
}

impl Reveal {
    fn new() -> Self {
        Self {
            value: 0.0,
            anim: Anim::None,
            from: 0.0,
            start: Instant::now(),
            dur: Duration::from_millis(0),
            want: None,
        }
    }

    fn t(&self) -> f32 {
        let dur = self.dur.as_secs_f32();
        if dur <= 0.0 {
            return 1.0;
        }
        self.start.elapsed().as_secs_f32() / dur
    }

    /// Направить окно к цели (true — показать). Возвращает true, если нужен кадр.
    fn step(&mut self, target: bool, dirty: &mut bool) -> bool {
        if self.want != Some(target) {
            self.want = Some(target);
            *dirty = true;
            self.from = self.value;
            self.start = Instant::now();
            self.dur = Duration::from_millis(if target {
                anim_rise_ms()
            } else {
                anim_fall_ms()
            });
            if !anim_on() || self.dur.is_zero() {
                self.value = if target { 1.0 } else { 0.0 };
                self.anim = Anim::None;
            } else if target {
                self.anim = Anim::Rise;
            } else if self.value > 0.0 {
                self.anim = Anim::Fall;
            } else {
                self.anim = Anim::None;
            }
        }
        match self.anim {
            Anim::None => false,
            Anim::Rise => {
                let t = self.t();
                // ease-out-back: быстрый вылет, лёгкое превышение, мягкая усадка на место
                self.value = self.from + (1.0 - self.from) * ease_out_back(t);
                if t >= 1.0 {
                    self.value = 1.0;
                    self.anim = Anim::None;
                }
                true
            }
            Anim::Fall => {
                let t = self.t();
                let eased = ease_in_cubic(t);
                self.value = self.from * (1.0 - eased);
                if t >= 1.0 {
                    self.value = 0.0;
                    self.anim = Anim::None;
                }
                true
            }
        }
    }
}

struct Ui {
    registry_state: RegistryState,
    seat_state: SeatState,
    output_state: OutputState,
    shm: Shm,

    exit: bool,
    first_configure: bool,
    pool: SlotPool,
    width: u32,
    height: u32,
    layer: LayerSurface,
    pointer: Option<wl_pointer::WlPointer>,
    shared: Arc<Mutex<Shared>>,
    /// «настройки изменились, переспроси демон» — ставится после клика по кнопке
    settings_due: Arc<AtomicBool>,
    font: Option<FontRef<'static>>,
    frame: u32,
    compositor: wl_compositor::WlCompositor,
    /// был ли последний commit пустым (пишу только тогда, когда состояние меняется)
    mapped: Option<bool>,

    /// анимация пилюли
    reveal: Reveal,
    /// нужен ли кадр немедленно (смена цели, первый configure)
    dirty: bool,
    /// последний «живой» статус: пока им рисуем, пока пилюля уходит
    shown: Status,

    /// окно панели настроек (второй layer surface того же процесса)
    panel_layer: LayerSurface,
    panel_surface: wl_surface::WlSurface,
    panel_width: u32,
    panel_height: u32,
    panel_reveal: Reveal,
    panel_mapped: Option<bool>,
    /// кнопки последнего нарисованного кадра: по ним узнаём, куда кликнули
    panel_keys: Vec<Button>,
    /// строка результата: что ответил демон на последнюю команду
    notice: String,
    /// подпись последнего нарисованного покоящегося кадра панели. Пока она не
    /// поменялась, второй буфер не гоняем: кадр панели — 114 400 пикселей, и
    /// перерисовывать его каждые 100 мс ради неподвижных кнопок смысла нет.
    panel_rest_sig: Option<String>,

    /// где оверлей стоит: сторона монитора и смещения (меняется перетаскиванием)
    placement: Placement,
    /// файл, в который положение сохраняется, чтобы следующий запуск встал туда же
    ui_path: std::path::PathBuf,
    /// машина перетаскивания: клик/сдвиг/грань-кандидат без примеси Wayland
    /// (сценарии гоняются юнит-тестами в drag.rs)
    drag: drag::Drag,
    /// размер экрана в логических пикселях (первый wl_output, который его сообщил):
    /// по нему курсор во время перетаскивания переводится в координаты экрана
    screen: Option<(i32, i32)>,
    /// какой ореол уже отрисован: перерисовываем только при смене места
    hint_drawn: Option<Placement>,
    /// композитор прислал первый configure ореолу: только после него можно
    /// прицеплять буфер (иначе protocol error и разрыв соединения)
    hint_configured: bool,
    /// выход, на котором стоит пилюля (surface_enter): его геометрия —
    /// авторитетная для зон прилипания; второй монитор её не перезапишет
    pill_output: Option<wl_output::WlOutput>,
    /// способ создавать новые слой-поверхности: нужен для ленивых ореола и орба
    layer_shell: LayerShell,
    /// он же для wl_surface: sctk сам ведёт их диспетчеризацию
    compositor_state: CompositorState,
    /// выход, на котором созданы поверхности перетаскивания (выход пилюли):
    /// если пилюля переедет на другой экран, они будут пересозданы
    drag_output: Option<wl_output::WlOutput>,
    /// поверхность ореола: то же окно, что у пилюли, на грани-кандидате.
    /// None — ещё не создана: она ленивая, потому что привязывается к выходу
    /// пилюли, который становится известен только после surface_enter
    hint_layer: Option<LayerSurface>,
    hint_surface: Option<wl_surface::WlSurface>,
    /// орб — интерфейс, собранный в кружок у курсора во время перетаскивания:
    /// маленькое окно (ORB_W), якорится к верхнему левому углу и ставится
    /// маргинами под курсор — двигается без перерисовки
    orb_layer: Option<LayerSurface>,
    orb_surface: Option<wl_surface::WlSurface>,
    /// орб прошёл первый configure (протокольный танец layer-shell)
    orb_configured: bool,
    /// фейд-ин орба: от нуля к единице за несколько кадров
    orb_alpha: f32,
    /// кадров с появления орба: pop-анимация сборки (scale по ease_out_back)
    orb_age: u32,
    /// орб улетает к месту закрепления: откуда, куда и доля пути
    orb_depart: Option<Depart>,
}

/// Улёт орба при отпускании: кружок скользит к центру будущей пилюли,
/// сжимаясь и растворяясь на лету — интерфейс «выходит из кружка» на месте.
#[derive(Debug, Clone, Copy)]
struct Depart {
    from: (i32, i32),
    to: (i32, i32),
    /// 0 = только что отпустили, 1 = растворился
    t: f32,
}

/// Якорь и размер поверхности пилюли по стороне. Поверхность длиннее пилюли на
/// REST_GAP вдоль своей стороны — в этом зазоре пилюля и прячется.
fn apply_placement(layer: &LayerSurface, p: Placement) {
    layer.set_anchor(anchor_for(p.edge));
    // margin вдоль края (смещение) и от края (отступ) — из самой стороны
    let (t, r, b, l) = p.margins;
    layer.set_margin(t, r, b, l);
    layer.set_size(p.pill_width(), p.pill_height());
}

/// То же для окна панели: оно во столько же раз длиннее и стоит рядом с пилюлей.
fn apply_panel_placement(layer: &LayerSurface, p: Placement) {
    layer.set_anchor(anchor_for(p.edge));
    let (t, r, b, l) = p.panel_margins();
    layer.set_margin(t, r, b, l);
    layer.set_size(p.panel_width(), p.panel_height());
}

fn anchor_for(edge: Edge) -> Anchor {
    match edge {
        Edge::Bottom => Anchor::BOTTOM,
        Edge::Top => Anchor::TOP,
        Edge::Left => Anchor::LEFT,
        Edge::Right => Anchor::RIGHT,
    }
}

impl CompositorHandler for Ui {
    fn scale_factor_changed(
        &mut self,
        _c: &Connection,
        _q: &QueueHandle<Self>,
        _s: &wl_surface::WlSurface,
        _n: i32,
    ) {
    }
    fn transform_changed(
        &mut self,
        _c: &Connection,
        _q: &QueueHandle<Self>,
        _s: &wl_surface::WlSurface,
        _t: wl_output::Transform,
    ) {
    }
    fn frame(
        &mut self,
        _c: &Connection,
        _q: &QueueHandle<Self>,
        _s: &wl_surface::WlSurface,
        _t: u32,
    ) {
    }
    fn surface_enter(
        &mut self,
        _c: &Connection,
        _q: &QueueHandle<Self>,
        s: &wl_surface::WlSurface,
        o: &wl_output::WlOutput,
    ) {
        // Пилюля вошла в свой выход: прилипание к граням должно мериться по
        // экрану, на котором она стоит, а не по последнему приславшему.
        if s.id() == self.layer.wl_surface().id() {
            let name = self
                .output_state
                .info(o)
                .and_then(|i| i.name.clone())
                .unwrap_or_default();
            eprintln!("dicta-ui: пилюля на выходе {name}");
            self.pill_output = Some(o.clone());
            self.remember_screen(o);
        }
    }
    fn surface_leave(
        &mut self,
        _c: &Connection,
        _q: &QueueHandle<Self>,
        _s: &wl_surface::WlSurface,
        _o: &wl_output::WlOutput,
    ) {
    }
}

impl OutputHandler for Ui {
    fn output_state(&mut self) -> &mut OutputState {
        &mut self.output_state
    }
    fn new_output(&mut self, _c: &Connection, _q: &QueueHandle<Self>, o: wl_output::WlOutput) {
        // выход пилюли ещё не известен — берём любой; после surface_enter
        // обновляет только он сам
        if self.pill_output.as_ref().map(|p| *p == o).unwrap_or(true) {
            self.remember_screen(&o);
        }
    }
    fn update_output(&mut self, _c: &Connection, _q: &QueueHandle<Self>, o: wl_output::WlOutput) {
        if self.pill_output.as_ref().map(|p| *p == o).unwrap_or(true) {
            self.remember_screen(&o);
        }
    }
    fn output_destroyed(
        &mut self,
        _c: &Connection,
        _q: &QueueHandle<Self>,
        o: wl_output::WlOutput,
    ) {
        if self.pill_output.as_ref().map(|p| *p == o).unwrap_or(false) {
            self.pill_output = None;
        }
    }
}

impl SeatHandler for Ui {
    fn seat_state(&mut self) -> &mut SeatState {
        &mut self.seat_state
    }
    fn new_seat(&mut self, _c: &Connection, _q: &QueueHandle<Self>, _s: wl_seat::WlSeat) {}
    fn new_capability(
        &mut self,
        _c: &Connection,
        q: &QueueHandle<Self>,
        seat: wl_seat::WlSeat,
        capability: Capability,
    ) {
        if capability == Capability::Pointer && self.pointer.is_none() {
            if let Ok(pointer) = self.seat_state.get_pointer(q, &seat) {
                self.pointer = Some(pointer);
            }
        }
    }
    fn remove_capability(
        &mut self,
        _c: &Connection,
        _q: &QueueHandle<Self>,
        _s: wl_seat::WlSeat,
        capability: Capability,
    ) {
        if capability == Capability::Pointer {
            self.pointer = None;
        }
    }
    fn remove_seat(&mut self, _c: &Connection, _q: &QueueHandle<Self>, _s: wl_seat::WlSeat) {}
}

impl PointerHandler for Ui {
    fn pointer_frame(
        &mut self,
        _c: &Connection,
        _q: &QueueHandle<Self>,
        _p: &wl_pointer::WlPointer,
        events: &[PointerEvent],
    ) {
        for event in events {
            let (x, y) = event.position;
            match event.kind {
                PointerEventKind::Press { .. } => self.on_press(event, x, y),
                PointerEventKind::Release { .. } => self.on_release(event, x, y),
                PointerEventKind::Motion { .. } => self.on_motion(x, y),
                _ => {}
            }
        }
    }
}

impl Ui {
    /// Нажали на пилюлю: запоминаем точку — дальше по ней решаем, был это клик или
    /// перетаскивание к другой границе монитора. Кнопка панели применяется сразу.
    fn on_press(&mut self, event: &PointerEvent, x: f64, y: f64) {
        if event.surface.id() == self.panel_surface.id() {
            let origin = self.placement.panel_origin(self.panel_reveal.value);
            let index = edge::button_at_turned(&self.panel_keys, x, y, &self.placement, origin);
            if let Some(index) = index {
                let id = self.panel_keys[index].id;
                self.press(id);
            }
            return;
        }
        if event.surface.id() != self.layer.wl_surface().id() {
            return;
        }
        // Клик по пилюле = «закончить и вставить», но только если его не тащили.
        self.drag.press(x, y);
        self.dirty = true; // кружок захвата вокруг курсора
    }

    /// Снимок мира для машины перетаскивания: где пилюля, какого размера её
    /// окно и каков экран.
    fn drag_world(&self) -> drag::World {
        drag::World {
            placement: self.placement,
            window: (self.width, self.height),
            screen: self.screen,
        }
    }

    /// Запомнить размер экрана, о котором рассказал wl_output: по нему курсор
    /// во время перетаскивания переводится в координаты экрана и ищется
    /// грань-кандидат для фиксации.
    fn remember_screen(&mut self, o: &wl_output::WlOutput) {
        if let Some((w, h)) = self
            .output_state
            .info(o)
            .and_then(|i| i.logical_size)
            .filter(|&(w, h)| w > 0 && h > 0)
        {
            if self.screen != Some((w, h)) {
                // в журнал: без размера экрана прилипание к граням не работает
                let name = self
                    .output_state
                    .info(o)
                    .and_then(|i| i.name.clone())
                    .unwrap_or_default();
                eprintln!("dicta-ui: экран {w}x{h} ({name}) — прилипание к граням активно");
                self.screen = Some((w, h));
            }
        }
    }

    /// Отпустили пилюлю. Всё, что решало перетаскивание, уже применено в
    /// on_motion — здесь остаётся только отличить клик («остановить и вставить»)
    /// от перетаскивания.
    fn on_release(&mut self, event: &PointerEvent, x: f64, y: f64) {
        if event.surface.id() != self.layer.wl_surface().id() {
            return;
        }
        // Машина сама решает, чем кончилось: клик — «остановить и вставить»,
        // грань — закрепиться, иначе — накопленный сдвиг вдоль своей стороны.
        // Орб при этом не просто исчезает: запоминаем, откуда он улетает
        // (курсор) — к месту, где встанет пилюля.
        let from = self.screen.zip(self.drag.cursor()).map(|(screen, (x, y))| {
            edge::cursor_abs(&self.placement, self.width, self.height, screen, x, y)
        });
        let outcome = self.drag.release(x, y);
        // улёт орба — только после настоящего перетаскивания: клик орб не
        // показывал, и кружок не должен выплывать из ниоткуда
        let was_drag = !matches!(outcome, drag::Release::Click);
        match outcome {
            drag::Release::Click => {
                let _ = ask_command(&dicta_core_socket_path_static(), "stop");
            }
            drag::Release::Dock(target) => self.apply_placement(target),
            drag::Release::Slide(along) => {
                let dim = self.screen.map(|(w, h)| {
                    if self.placement.edge.horizontal() {
                        w
                    } else {
                        h
                    }
                });
                let mut slid = self.placement.shifted_along(along);
                if let Some(dim) = dim {
                    slid = slid.clamped_cross(dim);
                }
                self.apply_placement(slid);
            }
            drag::Release::Nothing => {}
        }
        if was_drag {
            if let (Some(from), Some(screen)) = (from, self.screen) {
                let to = edge::body_center(&self.placement, screen);
                self.orb_depart = Some(Depart { from, to, t: 0.0 });
            }
        }
        // следующий drag начнётся со свежего pop'а и фейд-ина
        self.orb_age = 0;
        self.orb_alpha = 0.0;
        // ореол гаснет, пилюля возвращается из кармана, орб улетает к ней
        self.dirty = true;
    }

    /// Движение с зажатой кнопкой: машина копит сдвиг, ищет грань-кандидата
    /// и ведёт кружок захвата; всё видимое рисует главный цикл по dirty.
    /// Позицию орба при этом коммитим сразу, не дожидаясь цикла рисования —
    /// иначе орб отстаёт от курсора на кадр и читается как «ниже курсора».
    fn on_motion(&mut self, x: f64, y: f64) {
        let world = self.drag_world();
        if !self.drag.motion(x, y, &world) {
            return;
        }
        self.dirty = true;
        if self.drag.is_active() && self.orb_configured {
            if let Some(screen) = self.screen {
                if let Some(orb_layer) = &self.orb_layer {
                    let (cx, cy) =
                        edge::cursor_abs(&self.placement, self.width, self.height, screen, x, y);
                    orb_layer.set_margin(cy - render::ORB_HALF, 0, 0, cx - render::ORB_HALF);
                    orb_layer.commit();
                }
            }
        }
    }

    /// Переставить оверлей на заданную сторону (кнопка «край» в панели настроек).
    fn move_to_edge(&mut self, edge: Edge) {
        self.apply_placement(self.placement.with_edge(edge));
    }

    /// Применить положение: размеры обоих окон, их маргины и запись в ui.toml.
    fn apply_placement(&mut self, placement: Placement) {
        if placement == self.placement {
            return;
        }
        self.placement = placement;
        self.width = placement.pill_width();
        self.height = placement.pill_height();
        self.panel_width = placement.panel_width();
        self.panel_height = placement.panel_height();
        apply_placement(&self.layer, placement);
        apply_panel_placement(&self.panel_layer, placement);
        self.layer.commit();
        self.panel_layer.commit();
        // Композитор ответит configure на новый размер — перерисовку берём на себя сразу,
        // пока его ответ не пришёл: пилюля должна оказаться на новой стороне без паузы.
        self.dirty = true;
        // новое положение — новый кадр: подпись покоя панели больше не годится
        self.panel_rest_sig = None;
        let cfg = edge::UiConfig::from_placement(&placement);
        if let Err(err) = edge::save(&self.ui_path, &cfg) {
            eprintln!("не сохранил положение ({}): {err}", self.ui_path.display());
        }
        if debug_on() {
            eprintln!(
                "сторона -> {} margins {:?}",
                placement.edge.as_str(),
                placement.margins
            );
        }
    }
}

impl ShmHandler for Ui {
    fn shm_state(&mut self) -> &mut Shm {
        &mut self.shm
    }
}

impl LayerShellHandler for Ui {
    fn closed(&mut self, _c: &Connection, _q: &QueueHandle<Self>, l: &LayerSurface) {
        // Композитор может закрыть ореол или орб сам — это не повод гасить
        // приложение: сбрасываем поверхность, при следующем перетаскивании
        // она лениво создастся заново. Умираем только если закрыли пилюлю:
        // без неё делать нечего.
        let id = l.wl_surface().id();
        if self
            .orb_layer
            .as_ref()
            .map(|s| s.wl_surface().id() == id)
            .unwrap_or(false)
        {
            eprintln!("dicta-ui: орб закрыт композитором — пересоздадим при надобности");
            self.orb_layer = None;
            self.orb_surface = None;
            self.orb_configured = false;
            self.orb_depart = None;
            return;
        }
        if self
            .hint_layer
            .as_ref()
            .map(|s| s.wl_surface().id() == id)
            .unwrap_or(false)
        {
            eprintln!("dicta-ui: ореол закрыт композитором — пересоздадим при надобности");
            self.hint_layer = None;
            self.hint_surface = None;
            self.hint_configured = false;
            self.hint_drawn = None;
            return;
        }
        eprintln!("dicta-ui: пилюля закрыта композитором — выходим");
        self.exit = true;
    }
    fn configure(
        &mut self,
        _c: &Connection,
        _q: &QueueHandle<Self>,
        layer: &LayerSurface,
        configure: LayerSurfaceConfigure,
        _serial: u32,
    ) {
        let given = configure.new_size;
        if self
            .orb_layer
            .as_ref()
            .map(|l| l.wl_surface().id() == layer.wl_surface().id())
            .unwrap_or(false)
        {
            // Орб: первый configure открывает дорогу буферу; позицию он получает
            // маргинами по ходу движения, размер фиксирован.
            if !self.orb_configured {
                self.orb_configured = true;
                self.dirty = true;
            }
            return;
        }
        if self
            .hint_layer
            .as_ref()
            .map(|l| l.wl_surface().id() == layer.wl_surface().id())
            .unwrap_or(false)
        {
            // Ореол: первый configure — сигнал, что буфер теперь можно цеплять.
            // Дальше размер задан явно вместе с положением; перерисовка не нужна —
            // главный цикл сам рисует его по смене dock_hint.
            if !self.hint_configured {
                self.hint_configured = true;
                self.dirty = true;
            }
            return;
        }
        if layer.wl_surface().id() == self.panel_layer.wl_surface().id() {
            // Панель: размер задан явно, перерисовываем только если композитор его поменял.
            if given != (0, 0) && given != (self.panel_width, self.panel_height) {
                self.panel_width = given.0;
                self.panel_height = given.1;
                self.dirty = true;
            }
            if debug_on() {
                eprintln!(
                    "configure панель: ({}x{}) -> {:?}",
                    self.placement.panel_width(),
                    self.placement.panel_height(),
                    given
                );
            }
            return;
        }

        let size_changed = given != (self.width, self.height) && given.0 != 0 && given.1 != 0;
        if given.0 != 0 && given.1 != 0 {
            self.width = given.0;
            self.height = given.1;
        }
        if debug_on() {
            eprintln!(
                "configure пилюля: requested ({}x{}) -> suggested {:?}",
                self.placement.pill_width(),
                self.placement.pill_height(),
                given
            );
        }
        let was_first = self.first_configure;
        self.first_configure = false;
        // Кадр берёт на себя главный цикл. Перерисовывать по случаю и поводу
        // нельзя: композиторы (особенно wlroots) присылают configure в ответ на
        // каждый commit — получится бесконечный обмен commit'ами. Нужен только
        // самый первый configure и случай, когда композитор изменил размер.
        if was_first || size_changed {
            self.dirty = true;
        }
    }
}

impl Ui {
    /// Один шаг анимации обоих окон; true, если хоть одно ещё движется.
    fn tick(&mut self) -> bool {
        let (pill_target, panel_target, live) = {
            let Ok(shared) = self.shared.lock() else {
                return false;
            };
            (
                // во время перетаскивания пилюля собирается в орб у курсора:
                // сама прячется в карман и возвращается по отпусканию
                !self.drag.is_active()
                    && (always_draw()
                        || shared.status.state == "recording"
                        || shared.status.state == "transcribing"),
                shared.status.panel || panel_always(),
                shared.status.clone(),
            )
        };
        if pill_target {
            // пока пилюля на экране, рисуем живой статус (таймер, волна, подсказка)
            self.shown = live;
        }
        let pill = self.reveal.step(pill_target, &mut self.dirty);
        let panel = self.panel_reveal.step(panel_target, &mut self.dirty);
        pill || panel
    }

    /// Клик по кнопке панели: отправить команду демону и показать его ответ.
    fn press(&mut self, id: &'static str) {
        if id == "edge" {
            // «край» — локальное действие демону не передаётся: это положение окна.
            self.move_to_edge(self.placement.edge.next());
            return;
        }
        let (settings, recording) = match self.shared.lock() {
            Ok(shared) => (
                shared.settings.clone().unwrap_or_default(),
                shared.status.state == "recording",
            ),
            Err(_) => return,
        };
        let Some(request) = panel::request_for(id, &settings, recording) else {
            return;
        };
        let payload = request.json();
        if debug_on() {
            eprintln!("panel: клик {id} -> {}", payload.trim());
        }
        self.notice = match ask_payload(&dicta_core_socket_path_static(), &payload) {
            Ok(reply) => notice_from_reply(&reply),
            Err(err) => format!("демон не ответил: {err}"),
        };
        // значения после изменения спрашиваем заново
        self.settings_due.store(true, Ordering::Relaxed);
        self.dirty = true;
    }

    /// Панель настроек: вторая поверхность того же процесса, та же анимация снизу.
    fn draw_panel(&mut self, q: &QueueHandle<Self>) {
        let (width, height) = (self.panel_width, self.panel_height);
        let reveal = self.panel_reveal.value;
        let vertical = !self.placement.edge.horizontal();
        // Корпус панели в координатах её поверхности: сдвиг по стороне + подъём.
        let origin = self.placement.panel_origin(reveal);
        let stride: i32 = (width * 4) as i32;

        if reveal <= 0.0 {
            if self.panel_mapped == Some(false) {
                return;
            }
            let Ok((buffer, mem)) = self.pool.create_buffer(
                width as i32,
                height as i32,
                stride,
                wl_shm::Format::Argb8888,
            ) else {
                return;
            };
            mem.fill(0);
            self.panel_layer
                .wl_surface()
                .damage_buffer(0, 0, width as i32, height as i32);
            let _ = buffer.attach_to(self.panel_layer.wl_surface());
            let region = self.compositor.create_region(q, ());
            self.panel_layer
                .wl_surface()
                .set_input_region(Some(&region));
            region.destroy();
            self.panel_layer.commit();
            self.panel_mapped = Some(false);
            self.panel_rest_sig = None;
            return;
        }

        let (settings, recording, state) = match self.shared.lock() {
            Ok(shared) => (
                shared.settings.clone().unwrap_or_default(),
                shared.status.state == "recording",
                shared.status.state.clone(),
            ),
            Err(_) => return,
        };
        let keys = panel::buttons(&settings, recording, self.placement.edge);
        let notice = self.notice.clone();
        // Состав кадра определяется подписями кнопок, заголовком и строкой ответа:
        // если они те же, что в последнем покоящемся кадре, рисовать нечего.
        let sig = format!(
            "{state}|{}|{}",
            notice,
            keys.iter()
                .map(|k| k.label.as_str())
                .collect::<Vec<_>>()
                .join("|")
        );
        if reveal >= 1.0
            && self.panel_mapped == Some(true)
            && self.panel_rest_sig.as_deref() == Some(&sig)
        {
            return;
        }
        let header = format!("dicta · {}", state);
        // Подсказка одна на все стороны: вертикальная грань показывает повёрнутый
        // кадр горизонтальной панели, так что длинная строка влезает и там.
        let hint = if settings.reloading {
            "модель загружается: порог и авто-стоп менять можно, смену модели повтори после загрузки"
        } else {
            "Ctrl+Alt+V — открыть/закрыть · клик по кнопке = применить · удержать пилюлю и потащить к краю — переставить"
        };
        // Параллельно с подъёмом — фейд: панель проявляется, а не выпрыгивает.
        let alpha = 0.5 + 0.5 * reveal.clamp(0.0, 1.0);
        let canvas = render::paint_panel(
            width,
            height,
            if vertical { origin.0 } else { origin.1 },
            alpha,
            self.placement.turn(),
            &header,
            &notice,
            hint,
            &keys,
            self.font.as_ref(),
        );
        self.panel_keys = keys;

        let Ok((buffer, mem)) = self.pool.create_buffer(
            width as i32,
            height as i32,
            stride,
            wl_shm::Format::Argb8888,
        ) else {
            return;
        };
        canvas.write_into(mem);
        self.panel_layer
            .wl_surface()
            .damage_buffer(0, 0, width as i32, height as i32);
        let _ = buffer.attach_to(self.panel_layer.wl_surface());

        // Клики ловим только по корпусу панели: полоса рядом с ней (там пилюля и рабочий
        // стол) должна оставаться кликабельной для остальных окон.
        let region = self.compositor.create_region(q, ());
        if vertical {
            region.add(origin.0.max(0), 0, render::PANEL_H as i32, height as i32);
        } else {
            region.add(0, origin.1.max(0), width as i32, render::PANEL_H as i32);
        }
        self.panel_layer
            .wl_surface()
            .set_input_region(Some(&region));
        region.destroy();

        self.panel_layer.commit();
        self.panel_mapped = Some(true);
        // Пока кадр не на месте (анимация), подпись покоя не запоминаем: последний
        // кадр подъёма обязан быть отправлен.
        self.panel_rest_sig = if reveal >= 1.0 { Some(sig) } else { None };
        if debug_on() {
            eprintln!(
                "panel: reveal={:.3} origin={}x{} кнопок {}",
                reveal,
                origin.0,
                origin.1,
                self.panel_keys.len()
            );
        }
    }

    /// Ореол места фиксации на грани-кандидате: то же окно и то же положение,
    /// что будет у пилюли, — контур корпуса и свечение вокруг. Рисуется только
    /// при смене места; когда кандидат пропадает (курсор ушёл от грани),
    /// поверхность очищается отсоединением буфера.
    /// Ореол места фиксации на грани-кандидате: то же окно и то же положение,
    /// что будет у пилюли, — контур корпуса и свечение вокруг. Поверхность
    /// ленивая: создаётся на выходе пилюли при первом перетаскивании (выход
    /// известен только после surface_enter, а ореол без него уедет на чужой
    /// экран). Рисуется только при смене места; когда кандидат пропадает,
    /// поверхность гасится.
    fn draw_hint(&mut self, q: &QueueHandle<Self>) {
        if self.first_configure {
            return;
        }
        if self.hint_layer.is_none() && !self.ensure_drag_layers(q, true) {
            return;
        }
        let hint_layer = self.hint_layer.clone().unwrap();
        let hint_surface = self.hint_surface.clone().unwrap();
        let Some(p) = self.drag.hint() else {
            // Скрытие. Нулевой буфер не просто прячет: по спецификации он
            // АНМАПИТ поверхность и возвращает её в состояние «только что
            // создана» — следующее появление обязано повторить начальный
            // танец (пустой commit, configure, буфер), поэтому сбрасываем и
            // флаг. Во время перетаскивания вместо анмапа рисуем прозрачный
            // буфер: соседние кандидаты у зоны не должны гонять танец заново
            // на каждой смене стороны — от этого ореол мерцал бы на границах.
            if self.drag.held() {
                if let Some(prev) = self.hint_drawn.take() {
                    let (w, h) = (prev.pill_width(), prev.pill_height());
                    let stride = (w * 4) as i32;
                    let Ok((buffer, mem)) = self.pool.create_buffer(
                        w as i32,
                        h as i32,
                        stride,
                        wl_shm::Format::Argb8888,
                    ) else {
                        return;
                    };
                    mem.fill(0);
                    hint_surface.damage_buffer(0, 0, w as i32, h as i32);
                    let _ = buffer.attach_to(&hint_surface);
                    hint_layer.commit();
                }
            } else if let Some(prev) = self.hint_drawn.take() {
                // Анмап (attach None) возвращает поверхность в состояние «до
                // configure», и KWin потом не присылает configure заново —
                // второй раз ореол уже не появлялся. Прячем как пилюлю:
                // прозрачный буфер, поверхность остаётся замапленной и
                // настроенной, следующий drag просто рисует поверх.
                let (w, h) = (prev.pill_width(), prev.pill_height());
                let stride = (w * 4) as i32;
                let Ok((buffer, mem)) =
                    self.pool
                        .create_buffer(w as i32, h as i32, stride, wl_shm::Format::Argb8888)
                else {
                    return;
                };
                mem.fill(0);
                hint_surface.damage_buffer(0, 0, w as i32, h as i32);
                let _ = buffer.attach_to(&hint_surface);
                hint_layer.commit();
            }
            return;
        };
        if self.hint_configured && self.hint_drawn.as_ref().map(|d| d.edge) == Some(p.edge) {
            // та же грань, только позиция вдоль неё: двигаем окно маргинами,
            // буфер не трогаем — картинка ореола от смещения не меняется
            apply_placement(&hint_layer, p);
            hint_layer.commit();
            self.hint_drawn = Some(p);
            return;
        }
        if !self.hint_configured {
            // layer-shell требует configure до первого буфера: показываем
            // геометрию пустым коммитом и ждём ответа композитора — буфер
            // прицепит следующий кадр, когда configure придёт
            apply_placement(&hint_layer, p);
            hint_layer.commit();
            return;
        }
        // то же окно, что у пилюли на этой грани: якорь, маргины, размер
        apply_placement(&hint_layer, p);
        let (w, h) = (p.pill_width(), p.pill_height());
        let stride = (w * 4) as i32;
        let Ok((buffer, mem)) =
            self.pool
                .create_buffer(w as i32, h as i32, stride, wl_shm::Format::Argb8888)
        else {
            eprintln!("dicta-ui: ореол: буфер не выделился");
            return;
        };
        let rect = p.pill_rect(w, h, 1.0);
        let canvas = render::paint_dock_hint(w, h, rect, 0.9);
        canvas.write_into(mem);
        hint_surface.damage_buffer(0, 0, w as i32, h as i32);
        let _ = buffer.attach_to(&hint_surface);
        hint_layer.commit();
        self.hint_drawn = Some(p);
    }

    /// Орб — интерфейс, собранный в кружок у курсора: показывается, пока идёт
    /// перетаскивание (dragged), ставится маргинами ровно под курсор (двигается
    /// без перерисовки буфера) и дышит кольцом; в зоне прилипания кольцо
    /// загорается и обрастает свечением. По отпусканию анмапится.
    fn draw_orb(&mut self, q: &QueueHandle<Self>) {
        if self.first_configure {
            return;
        }
        let active = self.drag.is_active();
        if !active {
            // Перетаскивание кончилось. Орб не просто исчезает: он скользит к
            // месту, где встанет пилюля, сжимаясь и растворяясь на лету, —
            // интерфейс «выходит из кружка» на месте закрепления.
            if let Some(depart) = &mut self.orb_depart {
                depart.t += 1.0 / 14.0;
                if depart.t >= 1.0 || self.orb_layer.is_none() {
                    self.orb_depart = None;
                } else {
                    let k = render::ease_in_cubic(depart.t);
                    let lerp = |from: i32, to: i32| -> i32 {
                        from + ((to - from) as f32 * k).round() as i32
                    };
                    let (mx, my) = (
                        lerp(depart.from.0, depart.to.0) - render::ORB_HALF,
                        lerp(depart.from.1, depart.to.1) - render::ORB_HALF,
                    );
                    if let Some(orb_layer) = &self.orb_layer {
                        orb_layer.set_margin(my, 0, 0, mx);
                    }
                    let canvas = render::paint_orb(
                        1.0 - depart.t,
                        1.0 - 0.4 * depart.t,
                        false,
                        self.frame,
                        false,
                    );
                    self.attach_orb(canvas);
                    self.dirty = true; // дорисовать следующий кадр улёта
                    return;
                }
            }
            // улёт кончился (или его не было): гасим прозрачным буфером,
            // не анмапом — KWin закрывал анмапнутый орб и не переconfigure'ил
            // бы его заново; замапленный невидимый орб ничему не мешает
            // (input region пустая)
            if self.orb_depart.is_none() && self.orb_configured {
                let canvas = Canvas::new(render::ORB_W, render::ORB_W);
                self.attach_orb(canvas);
            }
            return;
        }
        if self.orb_layer.is_none() && !self.ensure_drag_layers(q, false) {
            return;
        }
        let orb_layer = self.orb_layer.clone().unwrap();
        let orb_surface = self.orb_surface.clone().unwrap();
        let Some(screen) = self.screen else { return };
        let Some((x, y)) = self.drag.cursor() else {
            return;
        };
        let (cx, cy) = edge::cursor_abs(&self.placement, self.width, self.height, screen, x, y);
        // окно центрируется на курсоре: маргины от верхнего левого угла экрана
        let (mx, my) = (cx - render::ORB_HALF, cy - render::ORB_HALF);
        orb_layer.set_margin(my, 0, 0, mx);
        if !self.orb_configured {
            // начальный танец layer-shell: геометрия пустым коммитом,
            // буфер — после configure
            orb_layer.commit();
            return;
        }
        // появление: pop по ease_out_back — интерфейс «собирается в кружок»
        // у курсора, вырастая из точки; дальше просто дышит
        self.orb_age += 1;
        let spawn = (self.orb_age as f32 / 10.0).min(1.0);
        let scale = if spawn < 1.0 {
            0.3 + 0.7 * render::ease_out_back(spawn)
        } else {
            1.0
        };
        self.orb_alpha = (self.orb_alpha + 0.34).min(1.0);
        let recording = self
            .shared
            .lock()
            .map(|s| s.status.state == "recording")
            .unwrap_or(false);
        let canvas = render::paint_orb(
            self.orb_alpha,
            scale,
            recording,
            self.frame,
            self.drag.hint().is_some(),
        );
        let _ = orb_surface;
        let w = render::ORB_W;
        let stride = (w * 4) as i32;
        let Ok((buffer, mem)) =
            self.pool
                .create_buffer(w as i32, w as i32, stride, wl_shm::Format::Argb8888)
        else {
            eprintln!("dicta-ui: орб: буфер не выделился");
            return;
        };
        canvas.write_into(mem);
        orb_surface.damage_buffer(0, 0, w as i32, w as i32);
        let _ = buffer.attach_to(&orb_surface);
        orb_layer.commit();
    }

    /// Прицепить кадр орба к его поверхности (общий хвост рисования).
    fn attach_orb(&mut self, canvas: Canvas) {
        let Some(orb_layer) = self.orb_layer.clone() else {
            return;
        };
        let Some(orb_surface) = self.orb_surface.clone() else {
            return;
        };
        let w = render::ORB_W;
        let stride = (w * 4) as i32;
        let Ok((buffer, mem)) =
            self.pool
                .create_buffer(w as i32, w as i32, stride, wl_shm::Format::Argb8888)
        else {
            return;
        };
        canvas.write_into(mem);
        orb_surface.damage_buffer(0, 0, w as i32, w as i32);
        let _ = buffer.attach_to(&orb_surface);
        orb_layer.commit();
    }

    /// Создать недостающие поверхности перетаскивания (ореол и/или орб) на
    /// выходе пилюли: всё прилипание живёт в координатах её экрана, и без
    /// привязки композитор разложил бы их по своему «текущему» выходу — на
    /// двух мониторах это уводит орб и ореол на чужой экран.
    fn ensure_drag_layers(&mut self, q: &QueueHandle<Self>, hint: bool) -> bool {
        let Some(output) = self.pill_output.clone() else {
            return false;
        };
        if hint && self.hint_layer.is_some() || !hint && self.orb_layer.is_some() {
            return true;
        }
        let surface = self.compositor_state.create_surface(q);
        let (namespace, anchor_or) = if hint {
            ("dicta-hint", None)
        } else {
            ("dicta-orb", Some(Anchor::TOP | Anchor::LEFT))
        };
        let layer = self.layer_shell.create_layer_surface(
            q,
            surface.clone(),
            layer_kind(),
            Some(namespace),
            Some(&output),
        );
        layer.set_keyboard_interactivity(KeyboardInteractivity::None);
        if let Some(anchor) = anchor_or {
            layer.set_anchor(anchor);
            layer.set_size(render::ORB_W, render::ORB_W);
        }
        // кликов эти поверхности никогда не ловят
        let region = self.compositor.create_region(q, ());
        surface.set_input_region(Some(&region));
        region.destroy();
        if hint {
            self.hint_layer = Some(layer);
            self.hint_surface = Some(surface);
        } else {
            self.orb_layer = Some(layer);
            self.orb_surface = Some(surface);
        }
        self.drag_output = Some(output);
        true
    }

    fn draw_pill(&mut self, q: &QueueHandle<Self>) {
        if self.first_configure {
            return;
        }
        let (width, height) = (self.width, self.height);
        let stride: i32 = (width * 4) as i32;
        let reveal = self.reveal.value;
        // Ориентация берётся из стороны: горизонтальная пилюля выезжает по вертикали,
        // вертикальная — по горизонтали.
        let vertical = !self.placement.edge.horizontal();
        // для низа/верха это координата по вертикали, для лева/права — по горизонтали:
        // paint принимает именно её
        let coord = self.placement.pill_coord(width, height, reveal);

        if debug_on() && (self.reveal.anim != Anim::None || self.frame % 30 == 0) {
            eprintln!(
                "draw #{} t={:.3} reveal={:.3} coord={} state={}",
                self.frame,
                self.reveal.t(),
                reveal,
                coord,
                self.shown.state
            );
        }

        if reveal <= 0.0 {
            // Пилюли нет: пустой буфер (поверхность остаётсяMapped, чтобы пилюля
            // появилась мгновенно) и пустая input region — клики идут сквозь неё.
            if self.mapped == Some(false) {
                if let Ok(mut shared) = self.shared.lock() {
                    shared.levels.clear();
                }
                return;
            }
            let Ok((buffer, mem)) = self.pool.create_buffer(
                width as i32,
                height as i32,
                stride,
                wl_shm::Format::Argb8888,
            ) else {
                return;
            };
            mem.fill(0);
            self.layer
                .wl_surface()
                .damage_buffer(0, 0, width as i32, height as i32);
            let _ = buffer.attach_to(self.layer.wl_surface());
            let region = self.compositor.create_region(q, ());
            self.layer.wl_surface().set_input_region(Some(&region));
            region.destroy();
            self.layer.commit();
            self.mapped = Some(false);
            if let Ok(mut shared) = self.shared.lock() {
                shared.levels.clear();
            }
            return;
        }

        let scene = Scene {
            recording: self.shown.state == "recording",
            seconds: self.shown.seconds,
            frame: self.frame,
            grab: self.drag.cursor(),
        };
        let hist: Vec<f32> = self
            .shared
            .lock()
            .map(|s| s.levels.clone())
            .unwrap_or_default();
        // Параллельно с подъёмом — фейд: появляется не «выпрыгиванием», а проявлением.
        let alpha = 0.45 + 0.55 * reveal.clamp(0.0, 1.0);
        let canvas: Canvas = paint(
            width,
            height,
            coord,
            alpha,
            vertical,
            &scene,
            &hist,
            self.font.as_ref(),
        );

        let Ok((buffer, mem)) = self.pool.create_buffer(
            width as i32,
            height as i32,
            stride,
            wl_shm::Format::Argb8888,
        ) else {
            if debug_on() {
                eprintln!("create_buffer не удался");
            }
            return;
        };
        canvas.write_into(mem);
        self.layer
            .wl_surface()
            .damage_buffer(0, 0, width as i32, height as i32);
        let _ = buffer.attach_to(self.layer.wl_surface());

        // Клик ловим только по пилюле: прозрачная полоса вокруг неё не должна
        // мешать окну под оверлеем.
        let (rx, ry, rw, rh) = self.placement.pill_rect(width, height, reveal);
        let region = self.compositor.create_region(q, ());
        region.add(rx, ry, rw as i32, rh as i32);
        self.layer.wl_surface().set_input_region(Some(&region));
        region.destroy();

        self.layer.commit();
        self.mapped = Some(true);
        self.frame = self.frame.wrapping_add(1);
    }
}

fn dicta_core_socket_path() -> std::path::PathBuf {
    if let Ok(explicit) = std::env::var("DICTA_CONFIG") {
        if let Ok(raw) = std::fs::read_to_string(&explicit) {
            if let Some(line) = raw.lines().find(|l| l.trim().starts_with("socket")) {
                if let Some(path) = line.split('"').nth(1) {
                    return std::path::PathBuf::from(path);
                }
            }
        }
    }
    std::env::var("XDG_RUNTIME_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| std::env::temp_dir())
        .join("dicta.sock")
}

/// Путь к сокету один раз на процесс: нужен в обработчике клика.
fn dicta_core_socket_path_static() -> std::path::PathBuf {
    use std::sync::OnceLock;
    static SOCKET: OnceLock<std::path::PathBuf> = OnceLock::new();
    SOCKET.get_or_init(dicta_core_socket_path).clone()
}

fn ask_status(socket: &std::path::Path) -> Result<Status> {
    let raw = ask_command(socket, "status")?;
    let parsed: Status = serde_json::from_str(&raw)?;
    Ok(parsed)
}

fn ask_command(socket: &std::path::Path, cmd: &str) -> Result<String> {
    ask_payload(socket, &format!("{{\"cmd\":\"{cmd}\"}}\n"))
}

/// То же, но с готовым JSON: команды панели несут key/value.
fn ask_payload(socket: &std::path::Path, payload: &str) -> Result<String> {
    use std::io::{BufRead, BufReader, Write};
    let stream = std::os::unix::net::UnixStream::connect(socket)?;
    stream.set_read_timeout(Some(Duration::from_secs(5)))?;
    let mut writer = stream.try_clone()?;
    let mut reader = BufReader::new(stream);
    writer.write_all(payload.as_bytes())?;
    writer.flush()?;
    let mut line = String::new();
    reader.read_line(&mut line)?;
    Ok(line.trim().to_string())
}

/// Что показать строкой результата панели: сначала ошибку, потом сообщение демона.
#[derive(Deserialize)]
struct DaemonReply {
    #[serde(default)]
    text: Option<String>,
    #[serde(default)]
    error: Option<String>,
}

fn notice_from_reply(raw: &str) -> String {
    match serde_json::from_str::<DaemonReply>(raw) {
        Ok(reply) => reply.error.or(reply.text).unwrap_or_default(),
        Err(_) => String::new(),
    }
}

/// Снимок настроек из демона (для подписей кнопок).
fn ask_settings(socket: &std::path::Path) -> Result<Settings> {
    #[derive(Deserialize)]
    struct Wrapper {
        #[serde(default)]
        settings: Option<Settings>,
    }
    let raw = ask_command(socket, "settings")?;
    let parsed: Wrapper =
        serde_json::from_str(&raw).context("демон ответил непонятно на settings")?;
    Ok(parsed.settings.unwrap_or_default())
}

delegate_registry!(Ui);
smithay_client_toolkit::delegate_dispatch2!(Ui);

impl ProvidesRegistryState for Ui {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry_state
    }
    registry_handlers![OutputState, SeatState];
}

impl Dispatch<wl_region::WlRegion, ()> for Ui {
    fn event(
        _: &mut Self,
        _: &wl_region::WlRegion,
        _: wl_region::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pill_rises_from_behind_the_bottom_edge() {
        let bottom = Placement::new(Edge::Bottom);
        // в покое — на render::REST_GAP выше кромки экрана
        assert_eq!(
            bottom.pill_top(SURF_H, 1.0),
            (SURF_H - render::TRAVEL) as i32
        );
        // старт — целиком под кромкой: верх пилюли равен высоте поверхности
        assert_eq!(bottom.pill_top(SURF_H, 0.0), SURF_H as i32);
        // середина пути между ними
        assert!(bottom.pill_top(SURF_H, 0.5) < bottom.pill_top(SURF_H, 0.0));
        assert!(bottom.pill_top(SURF_H, 0.5) > bottom.pill_top(SURF_H, 1.0));
        // overshoot пружинной кривой не должен обрезаться верхним краем поверхности
        let peak = (0..=100)
            .map(|i| ease_out_back(i as f32 / 100.0))
            .fold(f32::MIN, f32::max);
        let overshoot_px = (peak - 1.0) * render::TRAVEL as f32;
        assert!(overshoot_px > 2.0, "пружина должна быть заметна");
        assert!(
            overshoot_px < render::HEADROOM as f32,
            "overshoot {overshoot_px}px не должен обрезаться краем поверхности"
        );
    }

    #[test]
    fn hidden_frame_draws_nothing() {
        let scene = Scene {
            recording: true,
            seconds: 1.0,
            frame: 0,
            grab: None,
        };
        let bottom = Placement::new(Edge::Bottom);
        let canvas = paint(
            render::SURF_W,
            SURF_H,
            bottom.pill_top(SURF_H, 0.0),
            1.0,
            false,
            &scene,
            &[],
            None,
        );
        assert!(canvas.pixels().iter().all(|b| *b == 0));
    }

    #[test]
    fn resting_frame_draws_pill() {
        let scene = Scene {
            recording: true,
            seconds: 1.0,
            frame: 0,
            grab: None,
        };
        let bottom = Placement::new(Edge::Bottom);
        let canvas = paint(
            render::SURF_W,
            SURF_H,
            bottom.pill_top(SURF_H, 1.0),
            1.0,
            false,
            &scene,
            &[],
            None,
        );
        let px = canvas.pixels();
        // центр пилюли непрозрачен, нижний зазор (12 px) — пуст
        let rest = bottom.pill_top(SURF_H, 1.0) as usize;
        let mid_row = rest + 2 + 15;
        assert_eq!(px[(mid_row * render::SURF_W as usize + 200) * 4 + 3], 0xE6);
        let gap_row = (SURF_H - 1) as usize;
        assert!(px[(gap_row * render::SURF_W as usize + 200) * 4 + 3] == 0);
    }

    #[test]
    fn vertical_frame_draws_a_turned_pill() {
        // сторона слева/справа: та же толщина, только вдоль высоты кадра
        let scene = Scene {
            recording: true,
            seconds: 1.0,
            frame: 0,
            grab: None,
        };
        let left = Placement::new(Edge::Left);
        let (w, h) = (left.pill_width(), left.pill_height());
        assert_eq!((w, h), (SURF_H, render::SURF_W));
        let canvas = paint(
            w,
            h,
            left.pill_coord(w, h, 1.0),
            1.0,
            true,
            &scene,
            &[0.8, 0.4],
            None,
        );
        let px = canvas.pixels();
        // корпус непрозрачен вдоль всей высоты, а левый край кадра (карман) — пуст
        let rest = left.pill_left(left.pill_width(), 1.0) as usize;
        let mid = h as usize / 2 * w as usize;
        assert!(px[(mid + rest - 2) * 4 + 3] == 0, "карман у кромки пуст");
        assert!(
            px[(mid + rest + 2) * 4 + 3] > 0x80,
            "корпус начинается от покоя"
        );
        assert!(px[(mid + rest + 13) * 4 + 3] > 0x80);
        assert!(px[mid * 4 + 3] == 0);
    }

    #[test]
    fn text_metrics_are_pixels_not_font_units() {
        // Метрики ab_glyph — в дизайн-единицах шрифта; если перемножать их с
        // PxScale напрямую, advance символа выходит ~20 000 пикселей и от всей
        // строки остаётся первая буква. Проверка на этом и ломается регрессия.
        let Some(font) = load_font() else {
            return; // шрифта нет — проверять не на чем
        };
        let width = render::text_width(&font, "MMMM", ab_glyph::PxScale::from(14.0));
        assert!(
            width > 20.0 && width < 90.0,
            "ширина «MMMM» при 14 px получилась {width} — метрики снова в единицах шрифта"
        );
    }

    #[test]
    fn panel_frame_draws_its_labels() {
        let Some(font) = load_font() else {
            return;
        };
        let settings = Settings {
            min_level: 0.35,
            model: "/m/ggml-medium-q5_0.bin".into(),
            models: vec!["/m/ggml-medium-q5_0.bin".into()],
            ..Settings::default()
        };
        let keys = panel::buttons(&settings, true, Edge::Bottom);
        let canvas = render::paint_panel(
            render::PANEL_W,
            render::PANEL_SURFACE_H,
            0,
            1.0,
            None,
            "dicta · запись 0:03",
            "порог 0.40 — сохранено",
            "Ctrl+Alt+V — открыть/закрыть",
            &keys,
            Some(&font),
        );
        let px = canvas.pixels();
        let ink = |x0: usize, y0: usize, x1: usize, y1: usize| -> usize {
            let mut n = 0;
            for y in y0..y1 {
                for x in x0..x1 {
                    let i = (y * render::PANEL_W as usize + x) * 4;
                    let (r, g, b) = (px[i], px[i + 1], px[i + 2]);
                    if r.max(g).max(b) > 90 {
                        n += 1;
                    }
                }
            }
            n
        };
        // заголовок, все три ряда кнопок и обе строки снизу должны иметь чернила
        for (name, x0, y0, x1, y1) in [
            ("заголовок", 10usize, 10usize, 300, 28),
            ("ряд записи", 12, 36, 136, 58),
            ("ряд режимов", 12, 70, 260, 92),
            ("ряд значений", 20, 104, 240, 126),
            ("результат", 10, 129, 510, 143),
            ("подсказка", 10, 141, 510, 155),
        ] {
            assert!(ink(x0, y0, x1, y1) > 60, "{name}: подписи не отрисованы");
        }
    }

    #[test]
    fn vertical_panel_draws_the_same_buttons_turned() {
        let Some(font) = load_font() else {
            return;
        };
        let settings = Settings::default();
        let keys = panel::buttons(&settings, true, Edge::Left);
        let left = Placement::new(Edge::Left);
        let (w, h) = (left.panel_width(), left.panel_height());
        let oy = left.panel_origin(1.0).0;
        let canvas = render::paint_panel(
            w,
            h,
            oy,
            1.0,
            left.turn(),
            "dicta · запись 0:03",
            "порог 0.40 — сохранено",
            "левая грань",
            &keys,
            Some(&font),
        );
        let px = canvas.pixels();
        // корпус панели лежит вдоль кадра: его середина непрозрачна от верха до низа
        let column = (oy + (render::PANEL_H / 2) as i32) as usize;
        let mut dark_rows = 0;
        for y in 0..h as usize {
            let i = (y * w as usize + column) * 4;
            if px[i + 3] > 0x80 {
                dark_rows += 1;
            }
        }
        assert!(
            dark_rows > h as usize / 2,
            "корпус вертикальной панели отрисован только на {dark_rows} строк из {h}"
        );
        // внутри каждой кнопки обязан быть текст: панель — повёрнутый кадр
        // горизонтальной, подписи остаются строками и не режутся в столбик
        let bright = |x0: usize, y0: usize, x1: usize, y1: usize| -> usize {
            let mut n = 0;
            for y in y0..y1 {
                for x in x0..x1 {
                    let i = (y * w as usize + x) * 4;
                    if px[i].max(px[i + 1]).max(px[i + 2]) > 90 && px[i + 3] > 0x80 {
                        n += 1;
                    }
                }
            }
            n
        };
        // прямоугольник кнопки в системе повёрнутого окна: разметка живёт в
        // горизонтальной панели (src_oy + key.y по вертикали), поворот переносит
        // её в полосу поперёк кадра, координата вдоль — по y
        let src_oy = render::Turn::Cw.src_oy(oy);
        let turned = |key: &render::Button| -> (usize, usize, usize, usize) {
            let sy0 = (src_oy + key.y as i32) as usize;
            let sy1 = sy0 + render::PANEL_BTN_H as usize;
            (
                (render::PANEL_SURFACE_H as usize - 1 - (sy1 - 1)),
                key.x as usize,
                (render::PANEL_SURFACE_H as usize - 1 - sy0 + 1),
                key.x as usize + key.w as usize,
            )
        };
        for (name, key) in [
            ("запись", &keys[0]),
            ("авто-стоп", &keys[5]),
            ("закрыть", &keys[12]),
        ] {
            let (x0, y0, x1, y1) = turned(key);
            let n = bright(x0 + 4, y0 + 2, x1 - 2, y1 - 2);
            assert!(n > 25, "кнопка {name} без подписи: светлых пикселей {n}");
        }
        // заголовок (вверху разметки) и ответ демона (внизу) после поворота —
        // свободные полосы у дальнего и ближнего края корпуса
        let head_x0 =
            (render::PANEL_SURFACE_H as usize - 1 - (src_oy as usize + 28)).max((oy + 2) as usize);
        assert!(
            bright(head_x0, 12, head_x0 + 20, 300) > 40,
            "заголовок не отрисован"
        );
        let note_y = 129usize + src_oy as usize;
        let note_x0 = (render::PANEL_SURFACE_H as usize - 1 - (note_y + 13)).max((oy + 2) as usize);
        assert!(
            bright(note_x0, 12, note_x0 + 20, 400) > 40,
            "строка ответа демона не отрисована"
        );
    }

    #[test]
    fn hidden_and_overshooting_frames_survive_on_every_side() {
        // На уходе за кромку координата корпуса отрицательная, при overshoot'е —
        // за пределами кадра. Canvas живёт на u32: раньше это кончалось паникой
        // «attempt to add/subtract with overflow» ровно в момент, когда пилюля
        // пряталась на левой или верхней стороне.
        let font = load_font();
        let levels = [0.2f32, 0.6, 0.9, 0.4];
        for edge in Edge::all() {
            let p = Placement::new(edge);
            let vertical = !edge.horizontal();
            for reveal in [0.0f32, 0.3, 1.0, render::EASE_MAX] {
                let scene = Scene {
                    recording: reveal > 0.5,
                    seconds: 3.0,
                    frame: 7,
                    grab: None,
                };
                let pill = paint(
                    p.pill_width(),
                    p.pill_height(),
                    p.pill_coord(p.pill_width(), p.pill_height(), reveal),
                    0.5 + 0.5 * reveal,
                    vertical,
                    &scene,
                    &levels,
                    font.as_ref(),
                );
                assert_eq!(
                    (pill.w, pill.h),
                    (p.pill_width(), p.pill_height()),
                    "{edge:?} reveal={reveal}: размер кадра пилюли поменялся"
                );

                let origin = p.panel_origin(reveal);
                let keys = panel::buttons(&Settings::default(), true, edge);
                let panel_frame = render::paint_panel(
                    p.panel_width(),
                    p.panel_height(),
                    if vertical { origin.0 } else { origin.1 },
                    1.0,
                    p.turn(),
                    "dicta · запись 0:03",
                    "порог 0.40 — сохранено",
                    "клик = применить",
                    &keys,
                    font.as_ref(),
                );
                assert_eq!(
                    (panel_frame.w, panel_frame.h),
                    (p.panel_width(), p.panel_height()),
                    "{edge:?} reveal={reveal}: размер кадра панели поменялся"
                );
            }
        }
    }
}

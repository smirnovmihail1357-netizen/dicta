//! dicta-ui — маленький оверлей над всеми окнами: `● REC 0:03 ▁▂▄█  ⏎ готово · esc отмена`.
//!
//! Рендер — wl_shm (софт), слой — zwlr_layer_shell_v1 (KWin, Hyprland, sway и др.).
//! Состояние читается из демона по Unix-сокету каждые 100 мс; клик мышью по оверлею = stop.

use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use ab_glyph::{point, Font, FontRef, GlyphId, PxScale};
use anyhow::Result;
use serde::Deserialize;
use smithay_client_toolkit::{
    compositor::{CompositorHandler, CompositorState, FrameCallbackData},
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
    protocol::{wl_output, wl_pointer, wl_registry, wl_seat, wl_shm, wl_surface},
    Connection, Dispatch, QueueHandle,
};

const W: u32 = 520;
const H: u32 = 34;
const POLL_MS: u64 = 100;

fn debug_on() -> bool {
    std::env::var("DICTA_DEBUG").is_ok()
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

/// Ответ dictad на {"cmd":"status"}.
#[derive(Debug, Clone, Deserialize, Default)]
struct Status {
    #[serde(default)]
    state: String,
    #[serde(default)]
    level: f32,
    #[serde(default)]
    seconds: f32,
    #[serde(default)]
    #[allow(dead_code)] // зарезервировано: сообщения демона можно показывать в оверлее
    error: Option<String>,
}

const FONT_PATHS: &[&str] = &[
    "/usr/share/fonts/TTF/DejaVuSans.ttf",
    "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
];

fn main() -> Result<()> {
    // dicta-ui --probe: проверить, понимает ли композитор layer shell, и выйти.
    if std::env::args().any(|a| a == "--probe") {
        return probe();
    }
    run()
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

fn run() -> Result<()> {
    let socket = dicta_core_socket_path();
    let status: Arc<Mutex<Status>> = Arc::new(Mutex::new(Status::default()));

    // Опрос демона.
    {
        let status = status.clone();
        let socket = socket.clone();
        let mut last_state = String::from("startup");
        thread::spawn(move || loop {
            if let Ok(next) = ask_status(&socket) {
                if debug_on() && next.state != last_state {
                    eprintln!(
                        "status: {} level={:.2} sec={:.1}",
                        next.state, next.level, next.seconds
                    );
                    last_state = next.state.clone();
                }
                if let Ok(mut cur) = status.lock() {
                    *cur = next;
                }
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
    layer.set_anchor(Anchor::BOTTOM);
    layer.set_margin(0, 0, 12, 0);
    layer.set_size(W, H);
    layer.set_exclusive_zone(exclusive_zone());
    // Ноль: оверлей не перехватывает клавиатуру — клавиши остаются у фокусного окна.
    layer.set_keyboard_interactivity(KeyboardInteractivity::None);
    layer.commit();

    let pool = SlotPool::new((W * H * 4) as usize, &shm)?;

    let mut ui = Ui {
        registry_state: RegistryState::new(&globals),
        seat_state: SeatState::new(&globals, &qh),
        output_state: OutputState::new(&globals, &qh),
        shm,
        exit: false,
        first_configure: true,
        pool,
        width: W,
        height: H,
        layer,
        pointer: None,
        status: status.clone(),
        socket,
        levels: Vec::new(),
        font: load_font(),
        frame: 0,
    };

    loop {
        event_queue.blocking_dispatch(&mut ui)?;
        if ui.exit {
            break;
        }
    }
    Ok(())
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
    status: Arc<Mutex<Status>>,
    socket: std::path::PathBuf,
    /// история уровня за последние ~1.8 с: волна рисуется по ней, а не «дрожанием»
    levels: Vec<f32>,
    font: Option<FontRef<'static>>,
    frame: u32,
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
        q: &QueueHandle<Self>,
        _s: &wl_surface::WlSurface,
        _t: u32,
    ) {
        if debug_on() && self.frame % 30 == 0 {
            eprintln!("frame-событие #{}", self.frame);
        }
        self.draw(q);
    }
    fn surface_enter(
        &mut self,
        _c: &Connection,
        _q: &QueueHandle<Self>,
        _s: &wl_surface::WlSurface,
        _o: &wl_output::WlOutput,
    ) {
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
    fn new_output(&mut self, _c: &Connection, _q: &QueueHandle<Self>, _o: wl_output::WlOutput) {}
    fn update_output(&mut self, _c: &Connection, _q: &QueueHandle<Self>, _o: wl_output::WlOutput) {}
    fn output_destroyed(
        &mut self,
        _c: &Connection,
        _q: &QueueHandle<Self>,
        _o: wl_output::WlOutput,
    ) {
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
            if matches!(event.kind, PointerEventKind::Press { .. }) {
                // Клик по оверлею = «закончить и вставить».
                let _ = ask_command(&self.socket, "stop");
            }
        }
    }
}

impl ShmHandler for Ui {
    fn shm_state(&mut self) -> &mut Shm {
        &mut self.shm
    }
}

impl LayerShellHandler for Ui {
    fn closed(&mut self, _c: &Connection, _q: &QueueHandle<Self>, _l: &LayerSurface) {
        self.exit = true;
    }
    fn configure(
        &mut self,
        _c: &Connection,
        q: &QueueHandle<Self>,
        _l: &LayerSurface,
        configure: LayerSurfaceConfigure,
        _serial: u32,
    ) {
        if configure.new_size.0 != 0 && configure.new_size.1 != 0 {
            self.width = configure.new_size.0;
            self.height = configure.new_size.1;
        }
        if debug_on() {
            eprintln!(
                "configure: requested ({}x{}) -> suggested {:?}",
                W, H, configure.new_size
            );
        }
        if self.first_configure {
            self.first_configure = false;
        }
        self.draw(q);
    }
}

/// ARGB как u32 (0xAARRGGBB), пишется little-endian — как ожидает wl_shm Argb8888.
struct Canvas {
    w: u32,
    h: u32,
    px: Vec<u8>,
}

impl Canvas {
    fn new(w: u32, h: u32) -> Self {
        Self {
            w,
            h,
            px: vec![0; (w * h * 4) as usize],
        }
    }
    fn blend(&mut self, x: u32, y: u32, color: u32) {
        if x >= self.w || y >= self.h {
            return;
        }
        let a_src = ((color >> 24) & 0xFF) as u32;
        if a_src == 0 {
            return;
        }
        let idx = ((y * self.w + x) * 4) as usize;
        let dst = u32::from_le_bytes([
            self.px[idx],
            self.px[idx + 1],
            self.px[idx + 2],
            self.px[idx + 3],
        ]);
        let mix = |s: u32, d: u32| -> u32 { (s * a_src + d * (255 - a_src)) / 255 };
        let out = (255u32) << 24
            | mix((color >> 16) & 0xFF, (dst >> 16) & 0xFF) << 16
            | mix((color >> 8) & 0xFF, (dst >> 8) & 0xFF) << 8
            | mix(color & 0xFF, dst & 0xFF);
        self.px[idx..idx + 4].copy_from_slice(&out.to_le_bytes());
    }
    fn rect(&mut self, x0: u32, y0: u32, w: u32, h: u32, color: u32) {
        for y in y0..(y0 + h) {
            for x in x0..(x0 + w) {
                self.blend(x, y, color);
            }
        }
    }
    fn circle(&mut self, cx: u32, cy: u32, r: u32, color: u32) {
        for dy in 0..=(2 * r) {
            for dx in 0..=(2 * r) {
                let ddx = dx as i32 - r as i32;
                let ddy = dy as i32 - r as i32;
                if ddx * ddx + ddy * ddy <= (r * r) as i32 {
                    self.blend(cx + dx, cy + dy, color);
                }
            }
        }
    }
    fn write_into(&self, canvas: &mut [u8]) {
        canvas.copy_from_slice(&self.px);
    }
}

fn argb(a: u8, r: u8, g: u8, b: u8) -> u32 {
    ((a as u32) << 24) | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
}

impl Ui {
    fn draw(&mut self, q: &QueueHandle<Self>) {
        if self.first_configure {
            return;
        }
        let (width, height) = (self.width, self.height);
        let stride: i32 = (width * 4) as i32;

        let status = self.status.lock().map(|s| s.clone()).unwrap_or_default();

        // idle -> полностью прозрачный кадр: оверлея как будто нет.
        //
        // Пустой буфер attach'ить ОБЯЗАТЕЛЬНО: surface становится mapped только на
        // commit, в котором присоединён буфер. Если коммитить «ничего» (как делал
        // раньше), композитор никогда не пришлёт frame callback -> draw() больше не
        // вызывается -> при старте записи оверлей не появляется вообще.
        if status.state == "idle" && !always_draw() {
            match self.pool.create_buffer(
                width as i32,
                height as i32,
                stride,
                wl_shm::Format::Argb8888,
            ) {
                Ok((buffer, mut mem)) => {
                    let empty: &mut [u8] = &mut mem;
                    empty.fill(0);
                    self.layer
                        .wl_surface()
                        .damage_buffer(0, 0, width as i32, height as i32);
                    self.layer
                        .wl_surface()
                        .frame(q, FrameCallbackData(self.layer.wl_surface().clone()));
                    let _ = buffer.attach_to(self.layer.wl_surface());
                }
                Err(err) => {
                    if debug_on() {
                        eprintln!("idle create_buffer не удался: {err}");
                    }
                }
            }
            self.layer.commit();
            self.frame = self.frame.wrapping_add(1);
            return;
        }

        let recording = status.state == "recording";
        if recording {
            // статус приходит ~10 раз в секунду, кадры — 60: сдвигаем раз в 6 кадров
            if self.frame % 6 == 0 {
                self.levels.push(status.level);
                if self.levels.len() > 18 {
                    self.levels.remove(0);
                }
            }
        } else if !self.levels.is_empty() {
            self.levels.clear();
        }

        let mut c = Canvas::new(width, height);
        let transcribing = status.state == "transcribing";

        if recording || transcribing || always_draw() {
            // фон-пилюля
            let pill = if std::env::var("DICTA_FLASH").is_ok() {
                argb(0xFF, 0x00, 0xFF, 0x00)
            } else {
                argb(0xE6, 0x18, 0x1A, 0x1E)
            };
            c.rect(0, 2, width, height - 4, pill);
            c.rect(0, 0, width, 2, argb(0x00, 0, 0, 0));
            // индикатор
            if recording {
                // без мигания: оно выглядело как «рябь»; пульс даёт сама волна истории
                c.circle(18, height / 2, 5, argb(0xFF, 0xE5, 0x4B, 0x4B));
            } else {
                // «сканер» при распознавании: яркая голова со шлейфом
                let period = 14;
                let step = (self.frame / 2) % period;
                let head = if step < period / 2 {
                    step
                } else {
                    period - step
                };
                for i in 0..period {
                    let d = (i as i32 - head as i32).abs();
                    let color = if d < 3 {
                        argb(0xFF, 0x89, 0xB4, 0xFA)
                    } else {
                        argb(0xFF, 0x45, 0x4A, 0x52)
                    };
                    c.rect(12 + (i as u32) * 8, height / 2 - 4, 6, 8, color);
                }
            }

            let scale = PxScale::from(17.0);
            let mut x = 32.0f32;
            x = self.text(
                &mut c,
                if recording { "REC" } else { "… " },
                x,
                11.0,
                scale,
                argb(0xFF, 0xE5, 0x4B, 0x4B),
            );
            let _ = self.text(
                &mut c,
                &format!(
                    "{:01}:{:02}",
                    (status.seconds as u32) / 60,
                    (status.seconds as u32) % 60
                ),
                x + 4.0,
                11.0,
                scale,
                argb(0xFF, 0xF2, 0xF3, 0xF4),
            );

            // волна уровня: 18 колонок
            let bars = 18u32;
            let bw = 7u32;
            let gap = 2u32;
            let x0 = 12.0 + (width as f32 - (bars * (bw + gap)) as f32) / 2.0 + 40.0;
            for i in 0..bars as usize {
                // newest справа; если истории ещё нет — колонка пустая
                let jitter = if self.levels.len() > i {
                    self.levels[self.levels.len() - 1 - i]
                } else {
                    0.0
                };
                let h = ((jitter * 22.0).round() as u32).max(2);
                let shade = if jitter > 0.6 {
                    argb(0xFF, 0x89, 0xB4, 0xFA)
                } else if jitter > 0.3 {
                    argb(0xFF, 0x6E, 0x9E, 0xE8)
                } else {
                    argb(0xFF, 0x3C, 0x55, 0x77)
                };
                c.rect(
                    (x0 + (i as u32 * (bw + gap)) as f32) as u32,
                    height - 6 - h,
                    bw,
                    h,
                    shade,
                );
            }

            let hint = if recording {
                "клик = готово"
            } else {
                "распознаю…"
            };
            self.text_right(
                &mut c,
                hint,
                (width as f32) - 12.0,
                11.0,
                PxScale::from(15.0),
                argb(0xFF, 0x9A, 0xA0, 0xA8),
            );
        }

        let (buffer, canvas) = match self.pool.create_buffer(
            width as i32,
            height as i32,
            stride,
            wl_shm::Format::Argb8888,
        ) {
            Ok(pair) => pair,
            Err(err) => {
                if debug_on() {
                    eprintln!("create_buffer не удался: {err}");
                }
                return;
            }
        };
        if debug_on() && self.frame % 30 == 0 {
            eprintln!(
                "draw #{} state={} level={:.2}",
                self.frame, status.state, status.level
            );
        }
        c.write_into(canvas);

        self.layer
            .wl_surface()
            .damage_buffer(0, 0, width as i32, height as i32);
        self.layer
            .wl_surface()
            .frame(q, FrameCallbackData(self.layer.wl_surface().clone()));
        let _ = buffer.attach_to(self.layer.wl_surface());
        self.layer.commit();
        self.frame = self.frame.wrapping_add(1);
    }

    fn text(&self, c: &mut Canvas, s: &str, mut x: f32, y: f32, scale: PxScale, color: u32) -> f32 {
        let Some(font) = &self.font else { return x };
        for ch in s.chars() {
            let id: GlyphId = font.glyph_id(ch);
            let glyph = id.with_scale_and_position(scale, point(x, y));
            if let Some(outlined) = font.outline_glyph(glyph.clone()) {
                let bounds = outlined.px_bounds();
                let ox = bounds.min.x.max(0.0) as i32;
                let oy = bounds.min.y.max(0.0) as i32;
                outlined.draw(|gx, gy, coverage| {
                    let px = ox + gx as i32;
                    let py = oy + gy as i32;
                    if px < 0 || py < 0 {
                        return;
                    }
                    let a = (coverage * 255.0).clamp(0.0, 1.0) as u32;
                    let with_alpha = (color & 0x00FF_FFFF) | (a << 24);
                    c.blend(px as u32, py as u32, with_alpha);
                });
            }
            x += font.h_advance_unscaled(id) * scale.x;
        }
        x
    }

    fn text_right(&self, c: &mut Canvas, s: &str, right: f32, y: f32, scale: PxScale, color: u32) {
        let Some(font) = &self.font else { return };
        let width: f32 = s
            .chars()
            .map(|ch| font.h_advance_unscaled(font.glyph_id(ch)) * scale.x)
            .sum();
        self.text(c, s, (right - width).max(0.0), y, scale, color);
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

fn ask_status(socket: &std::path::Path) -> Result<Status> {
    let raw = ask_command(socket, "status")?;
    let parsed: Status = serde_json::from_str(&raw)?;
    Ok(parsed)
}

fn ask_command(socket: &std::path::Path, cmd: &str) -> Result<String> {
    use std::io::{BufRead, BufReader, Write};
    let stream = std::os::unix::net::UnixStream::connect(socket)?;
    stream.set_read_timeout(Some(Duration::from_secs(5)))?;
    let mut writer = stream.try_clone()?;
    let mut reader = BufReader::new(stream);
    writer.write_all(format!("{{\"cmd\":\"{cmd}\"}}\n").as_bytes())?;
    writer.flush()?;
    let mut line = String::new();
    reader.read_line(&mut line)?;
    Ok(line.trim().to_string())
}

delegate_registry!(Ui);
smithay_client_toolkit::delegate_dispatch2!(Ui);

impl ProvidesRegistryState for Ui {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry_state
    }
    registry_handlers![OutputState, SeatState];
}

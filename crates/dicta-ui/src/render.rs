//! Рендер кадра оверлея без Wayland: пилюля, индикатор, волна, текст.
//!
//! Геометрия вынесена из `main.rs` в чистые функции: `pill_top()` переводит долю
//! `reveal` (0 — пилюля целиком под нижней кромкой экрана, 1 — покой) в координату верха
//! пилюли. Благодаря анимацию видно и без композитора: `dicta-ui --preview out/` пишет
//! кадры в BMP.

use std::path::Path;

use ab_glyph::{point, Font, FontRef, GlyphId, PxScale};

/// Высота «виртуального блока» пилюли — ровно та, что была у поверхности до анимации.
pub const PILL_H: u32 = 34;
/// Нижний край поверхности ниже пилюли: сколько пустых строк остаётся у кромки экрана.
/// Это тот же margin, что был у прежнего оверлея (12); плюс 2 px внутренней отступы
/// пилюли — получается привычный зазор 14 px от низа пилюли до кромки экрана.
/// Держать этот зазор внутри поверхности (а не через margin) нужно для анимации:
/// именно в этой полосе пилюля «прячется», когда её не видно.
pub const REST_GAP: u32 = 12;
/// Запас сверху: пружинная кривая появления на пару пикселей overshoot'ит место покоя.
pub const HEADROOM: u32 = 8;
/// Высота поверхности: снизу REST_GAP (в этом зазоре пилюля и «прячется»), сверху HEADROOM.
pub const SURF_H: u32 = TRAVEL + HEADROOM;
/// Путь пилюли: на сколько поверхность уходит за кромку экрана (там пилюля и
/// «прячется»). Для горизонтальных сторон — высота пилюли + REST_GAP.
pub const TRAVEL: u32 = PILL_H + REST_GAP;
/// Ширина поверхности пилюли в горизонтальной ориентации.
pub const SURF_W: u32 = 520;
/// Ширина ряда кнопок панели в покое (тот же ряд, что у горизонтальной панели).
/// Длина волны уровня: столько последних замеров держим.
pub const WAVE_LEN: usize = 18;

// --- панель настроек ---

/// Ширина панели (та же, что у пилюли: они стоят друг над другом).
pub const PANEL_W: u32 = 520;
/// Высота панели в покое: заголовок, три ряда кнопок, строка результата и подсказка.
pub const PANEL_H: u32 = 158;
pub const PANEL_BTN_H: u32 = 26;
pub const PANEL_PAD: u32 = 10;
pub const PANEL_GAP_X: u32 = 8;
/// Под панелью её собственный «карман»: пилюля (PILL_H) + её зазор (REST_GAP) + воздух.
/// Именно из этой полосы панель и выезжает, как пилюля — из-под нижней кромки экрана.
pub const PANEL_BELOW: u32 = PILL_H + REST_GAP + 8;
/// Путь панели от «целиком за кромкой» до покоя (тот же принцип, что у TRAVEL).
pub const PANEL_TRAVEL: u32 = PANEL_H + PANEL_BELOW;
/// Высота поверхности панели.
pub const PANEL_SURFACE_H: u32 = PANEL_TRAVEL + HEADROOM;
pub const EASE_MAX: f32 = 1.2;

/// Тон кнопки — только визуальный: главное действие, остановка, текущее значение, нейтральная.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Tone {
    Action,
    Stop,
    Value,
    Flat,
}

impl Tone {
    fn fill(self) -> u32 {
        match self {
            Tone::Action => argb(0xFF, 0x2C, 0x4F, 0x6B),
            Tone::Stop => argb(0xFF, 0x7A, 0x2E, 0x2E),
            Tone::Value => argb(0xFF, 0x2A, 0x31, 0x3A),
            Tone::Flat => argb(0xFF, 0x23, 0x27, 0x2E),
        }
    }

    fn text(self) -> u32 {
        match self {
            Tone::Action => argb(0xFF, 0xEF, 0xF6, 0xFF),
            Tone::Stop => argb(0xFF, 0xFF, 0xE1, 0xE1),
            Tone::Value => argb(0xFF, 0x89, 0xB4, 0xFA),
            Tone::Flat => argb(0xFF, 0x9A, 0xA0, 0xA8),
        }
    }
}

/// Кнопка панели: прямоугольник + подпись. `id` рендер не понимает — он нужен
/// вызывающему коду, чтобы по клику опознать команду.
#[derive(Debug, Clone)]
pub struct Button {
    pub id: &'static str,
    pub label: String,
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub tone: Tone,
}

/// Панель настроек: корпус, заголовок, кнопки, строка результата и подсказка.
/// `vertical = false` — сторона снизу/сверху: ряды кнопок идут по горизонтали.
/// `vertical = true` — сторона слева/справа: те же кнопки развёрнуты на 90° (ряд
/// становится колонкой, подпись читается сверху вниз). `oy` — верх корпуса для
/// горизонтальной панели и её левый край для вертикальной.
pub fn paint_panel(
    w: u32,
    h: u32,
    oy: i32,
    alpha: f32,
    vertical: bool,
    header: &str,
    notice: &str,
    hint: &str,
    keys: &[Button],
    font: Option<&FontRef<'static>>,
) -> Canvas {
    let k = alpha.clamp(0.0, 1.0);
    let mut c = Canvas::new(w, h);
    if vertical {
        // корпус той же толщины PANEL_H, только вдоль высоты кадра
        c.round_rect(
            (oy + 2).max(0) as u32,
            0,
            PANEL_H - 4,
            h,
            14,
            fade(argb(0xF2, 0x14, 0x16, 0x1A), k),
        );
        // светлая кромка вдоль края — панель не должна выглядеть ещё одной пилюлей
        c.rect(
            (oy + 3).max(0) as u32,
            14,
            1,
            h - 28,
            fade(argb(0x40, 0x8B, 0x93, 0x9E), k),
        );
    } else {
        c.round_rect(
            0,
            oy + 2,
            w,
            PANEL_H - 4,
            14,
            fade(argb(0xF2, 0x14, 0x16, 0x1A), k),
        );
        // тонкая светлая кромка сверху — панель не должна выглядеть ещё одной пилюлей
        c.rect(14, oy + 3, w - 28, 1, fade(argb(0x40, 0x8B, 0x93, 0x9E), k));
    }

    let Some(font) = font else { return c };
    // Поперёк вертикального корпуса кнопки занимают полосу [oy+34, oy+128]: слева от них
    // остаются 32 px под заголовок и подсказку, справа 26 px под строку ответа демона.
    // Вниз по корпусу — вся его длина, и каждый столбец обязан в неё влезть.
    let head_room = (h as f32 - 24.0) * 0.55;
    let note_x = oy as f32 + 130.0;
    let mut head_len = 0usize;
    if vertical {
        // заголовок — вдоль корпуса
        head_len = text_column(
            &mut c,
            font,
            header,
            oy as f32 + 10.0,
            12.0,
            15.0,
            head_room,
            fade(argb(0xFF, 0xE8, 0xEC, 0xF2), k),
        );
    } else {
        text(
            &mut c,
            font,
            header,
            PANEL_PAD as f32,
            oy as f32 + 10.0,
            PxScale::from(15.0),
            fade(argb(0xFF, 0xE8, 0xEC, 0xF2), k),
        );
    }

    let scale = PxScale::from(14.0);
    for key in keys {
        if vertical {
            // ряд становится колонкой: координата вдоль панели — это y, а ширина кнопки — высота
            let x = oy + key.y as i32;
            let y = key.x as i32;
            c.round_rect(
                x.max(0) as u32,
                y,
                PANEL_BTN_H,
                key.w,
                9,
                fade(key.tone.fill(), k),
            );
            // бюджет подписи — по длине кнопки: символы столбцом идут вниз, а не вширь
            let label = fit_vertical(&key.label, key.w as f32 - 12.0, 14.0);
            let lines = label.chars().count() as f32;
            let y0 = y as f32 + ((key.w as f32 - lines * 14.0) / 2.0).max(6.0);
            for (i, ch) in label.chars().enumerate() {
                text(
                    &mut c,
                    font,
                    &format!("{ch}"),
                    x as f32 + 8.0,
                    y0 + i as f32 * 14.0,
                    scale,
                    fade(key.tone.text(), k),
                );
            }
        } else {
            let y = oy + key.y as i32;
            c.round_rect(key.x, y, key.w, PANEL_BTN_H, 9, fade(key.tone.fill(), k));
            let label = fit_text(font, &key.label, key.w as f32 - 12.0, scale);
            let width = text_width(font, &label, scale);
            let x = key.x as f32 + (key.w as f32 - width) / 2.0;
            text(
                &mut c,
                font,
                &label,
                x,
                y as f32 + 6.0,
                scale,
                fade(key.tone.text(), k),
            );
        }
    }

    if vertical {
        // ответ демона — правее кнопок, своим столбцом вдоль корпуса
        text_column(
            &mut c,
            font,
            notice,
            note_x,
            12.0,
            13.0,
            h as f32 - 24.0,
            fade(argb(0xFF, 0xC8, 0xCB, 0xD2), k),
        );
        // подсказка — тем же столбцом сразу под заголовком
        let hint_y = 12.0 + 15.0 * head_len as f32 + 16.0;
        text_column(
            &mut c,
            font,
            hint,
            oy as f32 + 10.0,
            hint_y,
            12.0,
            (h as f32 - 8.0 - hint_y).max(0.0),
            fade(argb(0xFF, 0x77, 0x7E, 0x88), k),
        );
    } else {
        if !notice.is_empty() {
            let scale = PxScale::from(13.0);
            let line = fit_text(font, notice, (w - 2 * PANEL_PAD) as f32, scale);
            text(
                &mut c,
                font,
                &line,
                PANEL_PAD as f32,
                oy as f32 + 129.0,
                scale,
                fade(argb(0xFF, 0xC8, 0xCB, 0xD2), k),
            );
        }
        let scale = PxScale::from(12.0);
        let line = fit_text(font, hint, (w - 2 * PANEL_PAD) as f32, scale);
        text(
            &mut c,
            font,
            &line,
            PANEL_PAD as f32,
            oy as f32 + 141.0,
            scale,
            fade(argb(0xFF, 0x77, 0x7E, 0x88), k),
        );
    }
    c
}

pub fn text_width(font: &FontRef<'static>, s: &str, scale: PxScale) -> f32 {
    let fx = units_to_px(font, scale);
    s.chars()
        .map(|ch| font.h_advance_unscaled(font.glyph_id(ch)) * fx)
        .sum()
}

/// Подпись внутри вертикальной кнопки: символы складываются столбцом вниз по ДЛИНЕ
/// кнопки, поэтому бюджет считаем по длине (`key.w`), а не по толщине (`PANEL_BTN_H`).
fn fit_vertical(s: &str, room: f32, step: f32) -> String {
    let all: Vec<char> = s.chars().collect();
    let max = (room / step).floor().max(0.0) as usize;
    if all.len() <= max {
        return s.to_string();
    }
    if max <= 1 {
        return String::new();
    }
    all.iter().take(max - 1).collect::<String>() + "…"
}

/// Столбец текста вдоль корпуса: символы идут вниз с шагом `step` (он же масштаб шрифта)
/// и занимают не больше длины `room`. Возвращает число поставленных символов.
fn text_column(
    c: &mut Canvas,
    font: &FontRef<'static>,
    s: &str,
    x: f32,
    y: f32,
    step: f32,
    room: f32,
    color: u32,
) -> usize {
    let all: Vec<char> = s.chars().collect();
    let max = (room / step).floor().max(0.0) as usize;
    let taken = all.len().min(max.max(1));
    let scale = PxScale::from(step);
    for (i, ch) in all.iter().take(taken).enumerate() {
        text(c, font, &format!("{ch}"), x, y + step * i as f32, scale, color);
    }
    // обрезку отмечаем «…» на последнем занятом месте, а не сверх длины столбца
    if taken < all.len() {
        text(c, font, "…", x, y + step * (taken - 1) as f32, scale, color);
    }
    taken
}

/// Обрезать подпись по ширину кнопки, оставив многоточие.
fn fit_text(font: &FontRef<'static>, s: &str, max_w: f32, scale: PxScale) -> String {
    if text_width(font, s, scale) <= max_w {
        return s.to_string();
    }
    let mut out = String::new();
    for ch in s.chars() {
        let trial = format!("{out}{ch}…");
        if text_width(font, &trial, scale) > max_w {
            break;
        }
        out.push(ch);
    }
    out.push('…');
    out
}

/// Что именно рисуем (снимок статуса демона; во время ухода пилюли он заморожен).
#[derive(Debug, Clone, Default)]
pub struct Scene {
    pub recording: bool,
    pub seconds: f32,
    pub frame: u32,
}

/// Пружинное появление: быстро вверх, лёгкое превышение, мягкая усадка на место.
pub fn ease_out_back(t: f32) -> f32 {
    const C1: f32 = 1.70158;
    const C3: f32 = C1 + 1.0;
    let t = t.clamp(0.0, 1.0);
    1.0 + C3 * (t - 1.0).powi(3) + C1 * (t - 1.0).powi(2)
}

/// уход: сначала медленно, затем пилюля «проваливается» за кромку.
pub fn ease_in_cubic(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * t
}

/// ARGB как u32 (0xAARRGGBB), пишется little-endian — как ожидает wl_shm Argb8888.
pub struct Canvas {
    pub w: u32,
    pub h: u32,
    px: Vec<u8>,
}

impl Canvas {
    pub fn new(w: u32, h: u32) -> Self {
        Self {
            w,
            h,
            px: vec![0; (w * h * 4) as usize],
        }
    }

    /// Пиксели кадра (для тестов и отладочных выгрузок).
    #[allow(dead_code)]
    pub fn pixels(&self) -> &[u8] {
        &self.px
    }

    /// Смешивание в premultiplied-alpha (как требует wl_shm Argb8888).
    /// Каналы в буфере хранятся уже умноженными на альфу; color — обычный
    /// непромультуплированный 0xAARRGGBB.
    pub fn blend(&mut self, x: u32, y: u32, color: u32) {
        if x >= self.w || y >= self.h {
            return;
        }
        let a_src = ((color >> 24) & 0xFF) as f32 / 255.0;
        if a_src == 0.0 {
            return;
        }
        let idx = ((y * self.w + x) * 4) as usize;
        let dst = u32::from_le_bytes([
            self.px[idx],
            self.px[idx + 1],
            self.px[idx + 2],
            self.px[idx + 3],
        ]);
        let a_dst = ((dst >> 24) & 0xFF) as f32 / 255.0;
        let inv = 1.0 - a_src;
        // over: out = src_pm + dst_pm * (1 - src_a); out_a = src_a + dst_a * (1 - src_a)
        let out_a = (a_src + a_dst * inv).clamp(0.0, 1.0) * 255.0;
        let ch = |src_ch: u32, dst_ch: u32| -> u8 {
            (src_ch as f32 * a_src + dst_ch as f32 * inv).clamp(0.0, 255.0) as u8
        };
        self.px[idx] = ch(color & 0xFF, dst & 0xFF);
        self.px[idx + 1] = ch((color >> 8) & 0xFF, (dst >> 8) & 0xFF);
        self.px[idx + 2] = ch((color >> 16) & 0xFF, (dst >> 16) & 0xFF);
        self.px[idx + 3] = out_a as u8;
    }

    pub fn rect(&mut self, x0: u32, y0: i32, w: u32, h: u32, color: u32) {
        if y0 < 0 {
            // часть прямоугольника выше поверхности: обрезаем, а не сдвигаем
            let lost = (-y0) as u32;
            if lost >= h {
                return;
            }
            self.rect(x0, 0, w, h - lost, color);
            return;
        }
        // границы цикла — по кадру: x0/y0 сюда попадают из i32-координат анимации,
        // а отрицательная координата после `as u32` превращается в 4 млрд, и обычный
        // `x0 + w` лопался («attempt to add with overflow») прямо на уходе пилюли
        // за кромку левой/верхней стороны.
        let y_end = (y0 as u32).saturating_add(h).min(self.h);
        let x_end = x0.saturating_add(w).min(self.w);
        for y in (y0 as u32)..y_end {
            for x in x0..x_end {
                self.blend(x, y, color);
            }
        }
    }

    pub fn round_rect(&mut self, x0: u32, y0: i32, w: u32, h: u32, r: u32, color: u32) {
        if y0 < 0 {
            let lost = (-y0) as u32;
            if lost >= h {
                return;
            }
            self.round_rect(x0, 0, w, h - lost, r, color);
            return;
        }
        let a_src = ((color >> 24) & 0xFF) as f32;
        let y_end = (y0 as u32).saturating_add(h).min(self.h);
        let x_end = x0.saturating_add(w).min(self.w);
        for y in (y0 as u32)..y_end {
            for x in x0..x_end {
                // 2x2 суперсэмплинг по пикселю — сглаживаем кромку радиуса
                let mut cov: f32 = 0.0;
                for sy in 0..2 {
                    for sx in 0..2 {
                        if rounded_inside(
                            x as f32 + 0.25 + sx as f32 * 0.5,
                            y as f32 + 0.25 + sy as f32 * 0.5,
                            x0 as f32,
                            y0 as f32,
                            w as f32,
                            h as f32,
                            r as f32,
                        ) {
                            cov += 1.0;
                        }
                    }
                }
                if cov > 0.0 {
                    let a = (a_src * cov / 4.0).min(255.0) as u32;
                    self.blend(x, y, (color & 0x00FF_FFFF) | (a << 24));
                }
            }
        }
    }

    pub fn circle(&mut self, cx: u32, cy: i32, r: u32, color: u32) {
        for dy in 0..=(2 * r) {
            let py = cy + dy as i32 - r as i32;
            if py < 0 {
                continue;
            }
            let ddy = dy as i32 - r as i32;
            for dx in 0..=(2 * r) {
                let ddx = dx as i32 - r as i32;
                if ddx * ddx + ddy * ddy <= (r * r) as i32 {
                    self.blend(cx.saturating_add(dx), py as u32, color);
                }
            }
        }
    }

    pub fn write_into(&self, canvas: &mut [u8]) {
        canvas.copy_from_slice(&self.px);
    }
    /// Прилепить другой кадр поверх этого (для контрольного снимка: пилюля + панель).
    pub fn blit(&mut self, src: &Canvas, x: i32, y: i32) {
        for sy in 0..src.h as i32 {
            for sx in 0..src.w as i32 {
                let (dx, dy) = (x + sx, y + sy);
                if dx < 0 || dy < 0 || dx >= self.w as i32 || dy >= self.h as i32 {
                    continue;
                }
                let si = ((sy as usize) * src.w as usize + sx as usize) * 4;
                let di = ((dy as usize) * self.w as usize + dx as usize) * 4;
                let src_px = u32::from_le_bytes([
                    src.px[si],
                    src.px[si + 1],
                    src.px[si + 2],
                    src.px[si + 3],
                ]);
                let dst_px = u32::from_le_bytes([
                    self.px[di],
                    self.px[di + 1],
                    self.px[di + 2],
                    self.px[di + 3],
                ]);
                let a = (src_px >> 24) & 0xFF;
                if a == 0 {
                    continue;
                }
                let inv = 255 - a;
                let mix = |shift: u32| {
                    let s_ch = (src_px >> shift) & 0xFF;
                    let d_ch = (dst_px >> shift) & 0xFF;
                    (s_ch + (d_ch * inv) / 255).min(0xFF)
                };
                let out = (mix(24) << 24) | (mix(16) << 16) | (mix(8) << 8) | mix(0);
                self.px[di..di + 4].copy_from_slice(&out.to_le_bytes());
            }
        }
    }
}

/// Точка внутри скруглённого прямоугольника (ближайшая точка ядра на расстоянии r).
pub fn rounded_inside(px: f32, py: f32, x0: f32, y0: f32, w: f32, h: f32, r: f32) -> bool {
    if px < x0 || py < y0 || px >= x0 + w || py >= y0 + h {
        return false;
    }
    let r = r.min(w / 2.0).min(h / 2.0);
    let cx = px.max(x0 + r).min(x0 + w - r);
    let cy = py.max(y0 + r).min(y0 + h - r);
    let dx = px - cx;
    let dy = py - cy;
    dx * dx + dy * dy <= r * r
}

pub fn argb(a: u8, r: u8, g: u8, b: u8) -> u32 {
    ((a as u32) << 24) | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
}

/// Умножить альфу цвета на k (0..=1) — для фейда вместе с подъёмом.
fn fade(color: u32, k: f32) -> u32 {
    if k >= 1.0 {
        return color;
    }
    let a = (((color >> 24) & 0xFF) as f32 * k).clamp(0.0, 255.0) as u32;
    (color & 0x00FF_FFFF) | (a << 24)
}

/// Один кадр оверлея. Горизонтальная пилюля: корпус на всю ширину кадра, верх в `oy`.
/// Вертикальная (`vertical = true`, сторона слева/справа): тот же набор — индикатор,
/// таймер, волна уровня и подпись, только разложенный сверху вниз по высоте кадра.
pub fn paint(
    w: u32,
    h: u32,
    oy: i32,
    alpha: f32,
    vertical: bool,
    scene: &Scene,
    levels: &[f32],
    font: Option<&FontRef<'static>>,
) -> Canvas {
    if vertical {
        return paint_vertical(w, h, oy, alpha, scene, levels, font);
    }
    let mut c = Canvas::new(w, h);
    let recording = scene.recording;
    let k = alpha.clamp(0.0, 1.0);

    // фон-пилюля
    let pill = if std::env::var("DICTA_FLASH").is_ok() {
        argb(0xFF, 0x00, 0xFF, 0x00)
    } else {
        argb(0xE6, 0x18, 0x1A, 0x1E)
    };
    // пилюля со скруглёнными углами (радиус = половина высоты пилюли)
    c.round_rect(0, oy + 2, w, PILL_H - 4, (PILL_H - 4) / 2, fade(pill, k));
    // индикатор
    if recording {
        // без мигания: оно выглядело как «рябь»; пульс даёт сама волна истории
        c.circle(
            18,
            oy + (PILL_H / 2) as i32,
            5,
            fade(argb(0xFF, 0xE5, 0x4B, 0x4B), k),
        );
    } else {
        // «сканер» при распознавании: яркая голова со шлейфом
        let period = 14;
        let step = (scene.frame / 2) % period;
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
            c.rect(
                12 + (i as u32) * 8,
                oy + (PILL_H / 2) as i32 - 4,
                6,
                8,
                fade(color, k),
            );
        }
    }

    if let Some(font) = font {
        let scale = PxScale::from(17.0);
        let mut x = 32.0f32;
        x = text(
            &mut c,
            font,
            if recording { "REC" } else { "… " },
            x,
            oy as f32 + 9.0,
            scale,
            fade(argb(0xFF, 0xE5, 0x4B, 0x4B), k),
        );
        let _ = text(
            &mut c,
            font,
            &format!(
                "{:01}:{:02}",
                (scene.seconds as u32) / 60,
                (scene.seconds as u32) % 60
            ),
            x + 4.0,
            oy as f32 + 9.0,
            scale,
            fade(argb(0xFF, 0xF2, 0xF3, 0xF4), k),
        );

        // волна уровня: 18 колонок
        let bars = 18u32;
        let bw = 7u32;
        let gap = 2u32;
        let x0 = 12.0 + (w as f32 - (bars * (bw + gap)) as f32) / 2.0 + 40.0;
        for i in 0..bars as usize {
            // newest справа; если истории ещё нет — колонка пустая
            let level = if levels.len() > i {
                levels[levels.len() - 1 - i]
            } else {
                0.0
            };
            let bh = ((level * 22.0).round() as u32).max(2);
            let shade = if level > 0.6 {
                argb(0xFF, 0x89, 0xB4, 0xFA)
            } else if level > 0.3 {
                argb(0xFF, 0x6E, 0x9E, 0xE8)
            } else {
                argb(0xFF, 0x3C, 0x55, 0x77)
            };
            c.rect(
                (x0 + (i as u32 * (bw + gap)) as f32) as u32,
                oy + (PILL_H as i32 - 6) - bh as i32,
                bw,
                bh,
                fade(shade, k),
            );
        }

        let hint = if recording {
            "клик = готово"
        } else {
            "распознаю…"
        };
        text_right(
            &mut c,
            font,
            hint,
            (w as f32) - 12.0,
            oy as f32 + 10.0,
            PxScale::from(15.0),
            fade(argb(0xFF, 0x9A, 0xA0, 0xA8), k),
        );
    }

    c
}

/// Вертикальная пилюля (сторона слева/справа): тот же набор, разложенный сверху вниз.
/// Индикатор вверху, ниже таймер, потом волна уровня вдоль всей высоты и подпись.
pub fn paint_vertical(
    w: u32,
    h: u32,
    ox: i32,
    alpha: f32,
    scene: &Scene,
    levels: &[f32],
    font: Option<&FontRef<'static>>,
) -> Canvas {
    let mut c = Canvas::new(w, h);
    let k = alpha.clamp(0.0, 1.0);
    let recording = scene.recording;
    let pill = if std::env::var("DICTA_FLASH").is_ok() {
        argb(0xFF, 0x00, 0xFF, 0xFF)
    } else {
        argb(0xE6, 0x18, 0x1A, 0x1E)
    };
    // корпус: та же толщина PILL_H, только вдоль высоты кадра
    c.round_rect(
        (ox + 2).max(0) as u32,
        0,
        PILL_H - 4,
        h,
        (PILL_H - 4) / 2,
        fade(pill, k),
    );

    let cx = ox as f32 + (PILL_H as f32) / 2.0;
    // центр корпуса в целых координатах: на уходе за кромку ox отрицательный, и
    // `cx as u32 - 4` превращалось в вычитание из нуля по u32 (паника). Считаем в i32.
    let cx_i = ox + (PILL_H / 2) as i32;
    if recording {
        c.circle(cx_i.max(0) as u32, 18, 5, fade(argb(0xFF, 0xE5, 0x4B, 0x4B), k));
    } else {
        // «сканер» при распознавании: голова со шлейфом идёт сверху вниз
        let period = 14;
        let step = (scene.frame / 2) % period;
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
            c.rect(
                (cx_i - 4).max(0) as u32,
                (12 + i as u32 * 8) as i32,
                8,
                6,
                fade(color, k),
            );
        }
    }

    let Some(font) = font else { return c };
    // таймер — поперёк корпуса (он широкий: PILL_H px на текст не хватит, поэтому
    // цифры рисуем друг под другом)
    let scale = PxScale::from(15.0);
    let seconds = format!(
        "{:01}:{:02}",
        (scene.seconds as u32) / 60,
        (scene.seconds as u32) % 60
    );
    for (i, ch) in seconds.chars().enumerate() {
        let glyph = format!("{ch}");
        let width = text_width(font, &glyph, scale);
        text(
            &mut c,
            font,
            &glyph,
            cx - width / 2.0,
            34.0 + i as f32 * 17.0,
            scale,
            fade(argb(0xFF, 0xF2, 0xF3, 0xF4), k),
        );
    }

    // волна уровня: те же 18 замеров, только полоски идут поперёк корпуса
    let bars = 18u32;
    let bh = 6u32;
    let gap = 2u32;
    let y0 = (h as u32).saturating_sub(bars * (bh + gap));
    for i in 0..bars as usize {
        let level = if levels.len() > i {
            levels[levels.len() - 1 - i]
        } else {
            0.0
        };
        let bw = ((level * (PILL_H as f32 - 14.0)).round() as u32).max(3);
        let shade = if level > 0.6 {
            argb(0xFF, 0x89, 0xB4, 0xFA)
        } else if level > 0.3 {
            argb(0xFF, 0x6E, 0x9E, 0xE8)
        } else {
            argb(0xFF, 0x3C, 0x3C, 0x77)
        };
        c.rect(
            (ox + 2).max(0) as u32,
            (y0 + i as u32 * (bh + gap)) as i32,
            bw,
            bh,
            fade(shade, k),
        );
    }

    // подпись — вниз кадра (там же, где у горизонтальной пилюли «клик = готово»)
    let hint = if recording { "готово" } else { "…" };
    let scale = PxScale::from(13.0);
    let width = text_width(font, hint, scale);
    text(
        &mut c,
        font,
        hint,
        cx - width / 2.0,
        (h - 20) as f32,
        scale,
        fade(argb(0xFF, 0x9A, 0xA0, 0xA8), k),
    );
    c
}

/// Пересчёт дизайн-единиц шрифта в пиксели.
///
/// PxScale в ab_glyph задаёт высоту строки (`height_unscaled`), а метрики
/// (`h_advance_unscaled`, `ascent_unscaled`) — в единицах дизайна шрифта
/// (у DejaVu их 2048 на em). Если перемножать их напрямую, advance символа
/// получается ~20 000 пикселей: от всей строки остаётся первая буква, а
/// подписи кнопок «обрезаются» до многоточия. Отсюда этот коэффициент.
fn units_to_px(font: &FontRef<'static>, scale: PxScale) -> f32 {
    let h = font.height_unscaled();
    if h <= 0.0 {
        return 0.0;
    }
    scale.x / h
}

/// Метка рисуется «сверху»: `y` — верх строки, а не базовая линия (так проще
/// раскладывать строки внутри пилюли и панели).
pub fn text(
    c: &mut Canvas,
    font: &FontRef<'static>,
    s: &str,
    mut x: f32,
    y: f32,
    scale: PxScale,
    color: u32,
) -> f32 {
    let h = font.height_unscaled().max(1.0);
    let fx = scale.x / h;
    let fy = scale.y / h;
    let baseline = y + font.ascent_unscaled() * fy;
    for ch in s.chars() {
        let id: GlyphId = font.glyph_id(ch);
        let glyph = id.with_scale_and_position(scale, point(x, baseline));
        if let Some(outlined) = font.outline_glyph(glyph.clone()) {
            let bounds = outlined.px_bounds();
            let ox = bounds.min.x as i32;
            let oy = bounds.min.y as i32;
            outlined.draw(|gx, gy, coverage| {
                let px = ox + gx as i32;
                let py = oy + gy as i32;
                if px < 0 || py < 0 {
                    return;
                }
                // coverage уже 0.0..=1.0: альфу надо брать от неё, а не «клампать» её саму
                let a = (coverage.clamp(0.0, 1.0) * 255.0) as u32;
                let with_alpha = (color & 0x00FF_FFFF) | (a << 24);
                c.blend(px as u32, py as u32, with_alpha);
            });
        }
        x += font.h_advance_unscaled(id) * fx;
    }
    x
}

pub fn text_right(
    c: &mut Canvas,
    font: &FontRef<'static>,
    s: &str,
    right: f32,
    y: f32,
    scale: PxScale,
    color: u32,
) {
    let width = text_width(font, s, scale);
    text(c, font, s, (right - width).max(0.0), y, scale, color);
}

/// Записать кадр как BMP (для `--preview`): premultiplied пиксели смешиваются с
/// подложкой, как это сделал бы композитор поверх рабочего стола.
pub fn write_bmp(path: &Path, canvas: &Canvas) -> std::io::Result<()> {
    use std::io::Write;
    let (w, h) = (canvas.w, canvas.h);
    let row = (w * 3) as usize;
    let data = row * h as usize;
    let file = 54 + data;
    let mut out = Vec::with_capacity(file);
    out.extend_from_slice(b"BM");
    out.extend_from_slice(&(file as u32).to_le_bytes());
    out.extend_from_slice(&[0u8; 4]);
    out.extend_from_slice(&54u32.to_le_bytes());
    out.extend_from_slice(&40u32.to_le_bytes());
    out.extend_from_slice(&(w as i32).to_le_bytes());
    out.extend_from_slice(&(h as i32).to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes());
    out.extend_from_slice(&24u16.to_le_bytes());
    out.extend_from_slice(&0u32.to_le_bytes());
    out.extend_from_slice(&(data as u32).to_le_bytes());
    out.extend_from_slice(&[0u8; 16]);

    let bg = argb(0xFF, 0x3A, 0x44, 0x52); // грубая «рабочий стол» подложка
                                           // BMP хранит строки снизу вверх
    for y in (0..h).rev() {
        for x in 0..w {
            let i = ((y * w + x) * 4) as usize;
            let a = canvas.px[i + 3] as f32 / 255.0;
            let ch = |src: u8, dst: u32| -> u8 {
                (src as f32 + (dst & 0xFF) as f32 * (1.0 - a)).clamp(0.0, 255.0) as u8
            };
            out.push(ch(canvas.px[i], bg));
            out.push(ch(canvas.px[i + 1], bg >> 8));
            out.push(ch(canvas.px[i + 2], bg >> 16));
        }
    }
    let mut f = std::fs::File::create(path)?;
    f.write_all(&out)
}

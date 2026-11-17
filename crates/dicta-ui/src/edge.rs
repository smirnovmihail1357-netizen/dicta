//! Геометрия оверлея относительно границы монитора: на какой он стороне, горизонтальный
//! или вертикальный, и как он выезжает из-за этой границы.
//!
//! Сторона задаёт три вещи: якорь слоя (`Anchor`), ориентацию (снизу/сверху — горизонтальная
//! пилюля 520×34, слева/справа — вертикальная 34×520) и то, с какой кромки пилюля выезжает
//! при появлении. Положение переживает перезапуск: оно лежит в `ui.toml` рядом с конфигом
//! демона, а пишет его либо перетаскивание мышью, либо `--edge`.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::render;

/// Сторона монитора, к которой прижат оверлей.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Edge {
    #[serde(rename = "bottom")]
    Bottom,
    #[serde(rename = "right")]
    Right,
    #[serde(rename = "top")]
    Top,
    #[serde(rename = "left")]
    Left,
}

impl Edge {
    pub fn parse(s: &str) -> Option<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "bottom" | "низ" | "снизу" => Some(Edge::Bottom),
            "right" | "право" | "справа" => Some(Edge::Right),
            "top" | "верх" | "сверху" => Some(Edge::Top),
            "left" | "лево" | "слева" => Some(Edge::Left),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Edge::Bottom => "bottom",
            Edge::Right => "right",
            Edge::Top => "top",
            Edge::Left => "left",
        }
    }

    /// Короткая русская подпись для кнопки «край».
    pub fn label(self) -> &'static str {
        match self {
            Edge::Bottom => "низ",
            Edge::Right => "право",
            Edge::Top => "верх",
            Edge::Left => "лево",
        }
    }

    /// Все стороны — для `--list-edges`.
    pub fn all() -> [Edge; 4] {
        [Edge::Bottom, Edge::Right, Edge::Top, Edge::Left]
    }

    /// Следующая сторона по часовой: кнопка «край» проходит все четыре за четыре клика.
    pub fn next(self) -> Edge {
        match self {
            Edge::Bottom => Edge::Left,
            Edge::Left => Edge::Top,
            Edge::Top => Edge::Right,
            Edge::Right => Edge::Bottom,
        }
    }

    /// Низ/верх — горизонтальная пилюля; лево/право — вертикальная.
    pub fn horizontal(self) -> bool {
        matches!(self, Edge::Bottom | Edge::Top)
    }

    /// Какой из маргинов (top, right, bottom, left) отвечает за отступ от этой стороны.
    pub fn along_margin(self, margins: (i32, i32, i32, i32)) -> i32 {
        let (t, r, b, l) = margins;
        match self {
            Edge::Bottom => b,
            Edge::Top => t,
            Edge::Left => l,
            Edge::Right => r,
        }
    }

    /// Какой маргин отвечает за смещение вдоль этой стороны. Слой якорится к своей
    /// стороне и к одному из соседних краёв экрана, поэтому «вдоль» — это маргин того
    /// угла, от которого считаем: низ/верх меряют влево (left), лево/право — вниз (top).
    pub fn cross_margin(self, margins: (i32, i32, i32, i32)) -> i32 {
        let (t, _r, _b, l) = margins;
        match self {
            Edge::Bottom | Edge::Top => l,
            Edge::Left | Edge::Right => t,
        }
    }
}

/// Положение оверлея: сторона плюс четыре маргина слоя. Маргины — не «придуманные» числа,
/// а ровно то, что уйдёт в `set_margin(top, right, bottom, left)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Placement {
    pub edge: Edge,
    pub margins: (i32, i32, i32, i32),
}

/// Композитор всё равно обрезал бы такой сдвиг; ограничиваем, чтобы перетаскивание
/// не увело положение в бесконечность.
const CROSS_LIMIT: i32 = 4096;
/// Отступ от края имеет смысл только до размеров панели задач/дока.
const INSET_LIMIT: i32 = 512;

impl Default for Placement {
    fn default() -> Self {
        Self {
            edge: Edge::Bottom,
            margins: (0, 0, 0, 0),
        }
    }
}

impl Placement {
    pub fn new(edge: Edge) -> Self {
        Self {
            edge,
            margins: (0, 0, 0, 0),
        }
    }

    /// Смещение вдоль своей стороны в пикселях экрана: 0 = по центру стороны, больше —
    /// правее (низ/верх) или ниже (лево/право).
    pub fn cross(self) -> i32 {
        self.edge.cross_margin(self.margins)
    }

    /// Отступ от своей стороны внутрь экрана (панель задач, док): какой из маргинов
    /// за него отвечает.
    pub fn along(self) -> i32 {
        self.edge.along_margin(self.margins)
    }

    /// Отодвинуться от своей стороны на `px` (там панель задач, док, вырез экрана).
    pub fn with_along(mut self, px: i32) -> Self {
        let (t, r, b, l) = self.margins;
        let px = px.clamp(0, INSET_LIMIT);
        self.margins = match self.edge {
            Edge::Bottom => (t, r, b + px, l),
            Edge::Top => (t + px, r, b, l),
            Edge::Left => (t, r, b, l + px),
            Edge::Right => (t, r + px, b, l),
        };
        self
    }

    /// Встать вдоль своей стороны на `px` от её центра: вправо для низа/верха,
    /// вниз для лева/права.
    pub fn with_cross(mut self, px: i32) -> Self {
        let (t, r, b, l) = self.margins;
        let px = px.clamp(-CROSS_LIMIT, CROSS_LIMIT);
        self.margins = match self.edge {
            Edge::Bottom | Edge::Top => (t, r, b, px),
            Edge::Left | Edge::Right => (px, r, b, l),
        };
        self
    }

    /// Переехать на другую сторону: отступ от края сохраняется, смещение заново 0.
    pub fn with_edge(self, edge: Edge) -> Self {
        Placement::new(edge).with_along(self.along())
    }

    /// Толщина полосы оверлея: горизонтальной стороны — высота, вертикальной — ширина.
    pub fn thickness(self) -> u32 {
        render::PILL_H
    }

    /// Размер поверхности пилюли: вдоль своей стороны SURF_W, поперёк — толщина плюс
    /// зазор REST_GAP, в котором пилюля прячется.
    pub fn pill_width(self) -> u32 {
        match self.edge {
            Edge::Bottom | Edge::Top => render::SURF_W,
            Edge::Left | Edge::Right => self.pocket(),
        }
    }

    pub fn pill_height(self) -> u32 {
        match self.edge {
            Edge::Bottom | Edge::Top => self.pocket(),
            Edge::Left | Edge::Right => render::SURF_W,
        }
    }

    /// Толщина стороны + карман, в котором пилюля прячется (анимация не требует margin).
    fn pocket(self) -> u32 {
        self.thickness() + render::REST_GAP + render::HEADROOM
    }

    /// Ширина окна панели: карман (PANEL_TRAVEL) плюс ряд кнопок, кончающийся на HEADROOM
    /// от дальнего края, — ровно PANEL_SURFACE_H. Если пилюлю сдвинули вдоль стороны, окно
    /// растёт на этот сдвиг, иначе панель уедет за его край.
    pub fn panel_width(self) -> u32 {
        match self.edge {
            Edge::Bottom | Edge::Top => render::PANEL_SURFACE_H,
            // вдоль края окно панели то же, что у горизонтальной стороны: карман плюс корпус;
            // сдвиг пилюли вдоль стороны добавляет столько же запаса
            Edge::Left | Edge::Right => {
                let base = render::PANEL_TRAVEL + render::HEADROOM;
                if self.cross() > 0 {
                    base + self.cross() as u32
                } else {
                    base
                }
            }
        }
    }

    pub fn panel_height(self) -> u32 {
        match self.edge {
            Edge::Bottom | Edge::Top => render::PANEL_SURFACE_H,
            Edge::Left | Edge::Right => render::PANEL_W,
        }
    }

    /// Маргины окна панели: та же сторона и то же смещение, что у пилюли — ряд кнопок
    /// панели начинается ровно там же, где пилюля (её окно короче панели).
    pub fn panel_margins(self) -> (i32, i32, i32, i32) {
        self.margins
    }

    /// Путь пилюли от «целиком за кромкой» до покоя: толщина корпуса плюс зазор покоя.
    pub fn travel(self) -> u32 {
        match self.edge {
            Edge::Bottom | Edge::Top => render::TRAVEL,
            Edge::Left | Edge::Right => self.thickness() + render::REST_GAP,
        }
    }

    /// Верх пилюли в координатах её окна. Для низа/верха это координата выезда,
    /// для лева/права пилюля заполняет окно вдоль своей стороны — здесь 0.
    pub fn pill_top(self, canvas_h: u32, reveal: f32) -> i32 {
        match self.edge {
            Edge::Bottom => slide(canvas_h, self.travel(), reveal),
            // окно верхней стороны начинается у кромки экрана и растёт внутрь неё
            Edge::Top => flip(
                canvas_h,
                slide(canvas_h, self.travel(), reveal),
                self.thickness(),
            ),
            Edge::Left | Edge::Right => 0,
        }
    }

    /// Левый край пилюли. Для лева/права это координата выезда, для низа/верха пилюля
    /// заполняет окно по ширине — здесь 0.
    pub fn pill_left(self, canvas_w: u32, reveal: f32) -> i32 {
        match self.edge {
            Edge::Right => slide(canvas_w, self.travel(), reveal),
            Edge::Left => flip(
                canvas_w,
                slide(canvas_w, self.travel(), reveal),
                self.thickness(),
            ),
            Edge::Bottom | Edge::Top => 0,
        }
    }

    /// Координата выезда корпуса в ориентации своей стороны: для низа/верха это верх
    /// кадра, для лева/права — левый край. Её принимает `paint`/`paint_panel`.
    pub fn pill_coord(self, w: u32, h: u32, reveal: f32) -> i32 {
        match self.edge {
            Edge::Bottom | Edge::Top => self.pill_top(h, reveal),
            Edge::Left | Edge::Right => self.pill_left(w, reveal),
        }
    }

    /// Прямоугольник кликабельной области пилюли в координатах её поверхности.
    pub fn pill_rect(self, canvas_w: u32, canvas_h: u32, reveal: f32) -> (i32, i32, u32, u32) {
        if self.edge.horizontal() {
            let oy = self.pill_top(canvas_h, reveal);
            (0, oy.max(0), canvas_w, render::PILL_H)
        } else {
            let ox = self.pill_left(canvas_w, reveal);
            (ox.max(0), 0, render::PILL_H, canvas_h)
        }
    }

    /// Верхний левый угол корпуса панели в координатах её поверхности.
    pub fn panel_origin(self, reveal: f32) -> (i32, i32) {
        match self.edge {
            Edge::Bottom | Edge::Top => (0, self.panel_coord(self.panel_width(), reveal)),
            Edge::Left | Edge::Right => (self.panel_coord(self.panel_width(), reveal), 0),
        }
    }

    /// Координата начала ряда кнопок панели при доле `reveal`: 0 = целиком за кромкой
    /// своей стороны, 1 = на своём месте. Для низа/верха это верх кадра, для лева/права —
    /// левый край (выезд идёт по горизонтали). Сдвиг пилюли вдоль стороны уносит ряд за
    /// место покоя ровно на столько же.
    pub fn panel_coord(self, canvas: u32, reveal: f32) -> i32 {
        // Ряд кнопок панели покоится там же, где пилюля (её окно короче панели), а карман
        // панели занимает у кромки экрана полосу PANEL_TRAVEL.
        let coord = slide(canvas, render::PANEL_TRAVEL, reveal);
        match self.edge {
            Edge::Bottom | Edge::Right => coord,
            Edge::Top | Edge::Left => flip(canvas, coord, render::PANEL_H),
        }
    }

    /// Куда потащили пилюлю: новое положение по смещению мыши (в координатах экрана).
    ///
    /// Правило одно для всех сторон: тянешь к границе монитора — оверлей переезжает на эту
    /// сторону; тянешь вдоль своей границы — просто едет вдоль неё.
    pub fn dragged(self, dx: f64, dy: f64) -> Self {
        self.dragged_px(dx.round() as i32, dy.round() as i32)
    }

    /// То же, но в целых пикселях (так удобнее проверять).
    pub fn dragged_px(self, dx: i32, dy: i32) -> Self {
        if dx == 0 && dy == 0 {
            return self;
        }
        match self.target_edge(dx, dy) {
            // переезд: тянули к другой границе экрана
            Some(edge) if edge != self.edge => self.with_edge(edge),
            None => self,
            // движение вдоль своей стороны — сдвиг; наружу за свою кромку — стоя на месте
            Some(_) => self.with_cross(self.cross() + along_axis(self.edge, dx, dy)),
        }
    }

    /// Сторона монитора, к которой смотрит это перемещение мыши: вверх по экрану — верхняя,
    /// вправо — правая. Движение вдоль своей стороны (низ/верх меряют по x, лево/право — по
    /// y) возвращает текущую сторону: оно меняет смещение, а не сторону.
    pub fn target_edge(&self, dx: i32, dy: i32) -> Option<Edge> {
        if dx == 0 && dy == 0 {
            return None;
        }
        // Порог маленький: достаточно лёгкого рывка к границе, но дрожание мыши в пару
        // пикселей сторону не меняет.
        const MIN: i32 = 4;
        if dx.abs() >= dy.abs() {
            if dx > MIN {
                return Some(Edge::Right);
            }
            if dx < -MIN {
                return Some(Edge::Left);
            }
        } else if dy > MIN {
            return Some(Edge::Bottom);
        } else if dy < -MIN {
            return Some(Edge::Top);
        }
        Some(self.edge)
    }
}

/// Координата корпуса при доле `reveal`: 0 = целиком за кромкой экрана, 1 = на своём месте.
/// Считаем для стороны, чьё окно приято к кромке и растёт внутрь экрана (низ, право):
/// там «целиком за кромкой» — за дальним краем окна (`canvas`), а покой — на `travel` от
/// ближнего края: окно ровно по карману, `canvas = travel + HEADROOM`, где travel — путь
/// корпуса, а REST_GAP остаётся зазором от корпуса до кромки.
fn slide(canvas: u32, travel: u32, reveal: f32) -> i32 {
    debug_assert!(canvas >= travel + render::HEADROOM);
    let hidden = canvas as i32;
    let rest = (canvas - travel) as i32;
    let r = reveal.clamp(0.0, render::EASE_MAX);
    (hidden as f32 + (rest - hidden) as f32 * r).round() as i32
}

/// Отражение координаты для сторон, чьё окно начинается у кромки экрана и растёт внутрь
/// (верх, лево): там корпус при уходе уходит за нулевой край окна, а покой прижат к нему
/// так, что HEADROOM достаётся дальнему краю.
fn flip(canvas: u32, coord: i32, thickness: u32) -> i32 {
    canvas as i32 - coord - thickness as i32
}

/// Компонента перемещения вдоль данной стороны: низ/верх меряют по x, лево/право — по y.
/// Движение наружу (влево для левой стороны, вправо для правой) даёт 0: за кромкой экрана
/// композитор ничего не показывает, так что там пилюля стоит, а не едет.
fn along_axis(edge: Edge, dx: i32, dy: i32) -> i32 {
    match edge {
        Edge::Bottom | Edge::Top => dx,
        Edge::Left if dx < 0 => 0,
        Edge::Right if dx > 0 => 0,
        Edge::Left | Edge::Right => dy,
    }
}

/// Прямоугольник кнопки в координатах поверхности панели. `origin` — верхний левый угол
/// корпуса. На вертикальных сторонах ряд кнопок идёт вдоль края (по y), а сами кнопки —
/// поперёк него (по x), поэтому координаты кнопки меняются местами.
pub fn button_rect(b: &render::Button, origin: (i32, i32), vertical: bool) -> (i32, i32, u32, u32) {
    if vertical {
        (
            origin.0 + b.y as i32,
            origin.1 + b.x as i32,
            render::PANEL_BTN_H,
            b.w,
        )
    } else {
        (
            origin.0 + b.x as i32,
            origin.1 + b.y as i32,
            b.w,
            render::PANEL_BTN_H,
        )
    }
}

/// Кнопка под точкой (x, y) в координатах поверхности панели.
pub fn button_at(
    keys: &[render::Button],
    x: f64,
    y: f64,
    origin: (i32, i32),
    vertical: bool,
) -> Option<usize> {
    let (x, y) = (x as i32, y as i32);
    keys.iter().position(|b| {
        let (bx, by, bw, bh) = button_rect(b, origin, vertical);
        x >= bx && x < bx + bw as i32 && y >= by && y < by + bh as i32
    })
}

/// Положение оверлея на диске. Отдельно от `config.toml`: тот файл читает и пишет демон,
/// а это — состояние интерфейса, о котором демону знать не нужно.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct UiConfig {
    /// Сторона монитора: bottom | right | top | left.
    #[serde(default)]
    pub edge: Option<Edge>,
    /// Смещение вдоль края в пикселях (0 = по центру стороны).
    #[serde(default)]
    pub offset: i32,
    /// Отступ от края внутрь экрана в пикселях (панель задач, док).
    #[serde(default)]
    pub inset: i32,
}

impl UiConfig {
    pub fn placement(&self) -> Placement {
        Placement::new(self.edge.unwrap_or(Edge::Bottom))
            .with_along(self.inset.max(0))
            .with_cross(self.offset)
    }

    /// Положение обратно в плоские значения: нужно, чтобы сохранить перетаскивание.
    pub fn from_placement(p: &Placement) -> Self {
        UiConfig {
            edge: Some(p.edge),
            offset: p.cross(),
            inset: p.along().max(0),
        }
    }
}

/// `$XDG_CONFIG_HOME/dicta/ui.toml`, а если задан `DICTA_CONFIG` — рядом с ним.
pub fn config_path() -> PathBuf {
    if let Some(explicit) = std::env::var_os("DICTA_CONFIG") {
        let path = PathBuf::from(explicit);
        let dir = path.parent().unwrap_or(Path::new("."));
        return dir.join("ui.toml");
    }
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("dicta")
        .join("ui.toml")
}

/// Прочитать положение: нет файла или битый TOML — нижняя кромка экрана, как раньше.
pub fn load(path: &Path) -> UiConfig {
    match std::fs::read_to_string(path) {
        Ok(raw) => toml::from_str::<UiConfig>(&raw).unwrap_or_else(|err| {
            eprintln!("ui.toml: не понял ({err}) — беру нижнюю кромку");
            UiConfig::default()
        }),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => UiConfig::default(),
        Err(err) => {
            eprintln!("ui.toml: не прочитать ({err}) — беру нижнюю кромку");
            UiConfig::default()
        }
    }
}

/// Записать положение: временный файл + rename, как `config::set_key` в демоне.
pub fn save(path: &Path, cfg: &UiConfig) -> anyhow::Result<()> {
    let text = toml::to_string_pretty(cfg)?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).ok();
    }
    let tmp = path.with_extension("toml.tmp");
    std::fs::write(&tmp, &text)?;
    std::fs::rename(&tmp, path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn edge_parses_four_sides_and_labels() {
        assert_eq!(Edge::parse("bottom"), Some(Edge::Bottom));
        assert_eq!(Edge::parse(" RIGHT "), Some(Edge::Right));
        assert_eq!(Edge::parse("верх"), Some(Edge::Top));
        assert_eq!(Edge::parse("corner"), None);
        assert_eq!(Edge::Left.label(), "лево");
    }

    #[test]
    fn horizontal_only_for_bottom_and_top() {
        assert!(Edge::Bottom.horizontal());
        assert!(Edge::Top.horizontal());
        assert!(!Edge::Left.horizontal());
        assert!(!Edge::Right.horizontal());
    }

    #[test]
    fn next_edge_walks_the_full_circle() {
        let mut e = Edge::Bottom;
        let mut seen = vec![e];
        for _ in 0..4 {
            e = e.next();
            seen.push(e);
        }
        assert_eq!(
            seen,
            vec![
                Edge::Bottom,
                Edge::Left,
                Edge::Top,
                Edge::Right,
                Edge::Bottom
            ]
        );
    }

    #[test]
    fn default_placement_is_the_old_bottom_pill() {
        let p = Placement::default();
        assert_eq!(p.edge, Edge::Bottom);
        assert_eq!(p.margins, (0, 0, 0, 0));
        assert_eq!(p.cross(), 0);
        assert_eq!(p.along(), 0);
        assert_eq!(p.pill_width(), render::SURF_W);
        assert_eq!(p.pill_height(), render::SURF_H);
    }

    #[test]
    fn vertical_side_swaps_the_pill_size() {
        let p = Placement::new(Edge::Right);
        assert_eq!(p.pill_width(), render::SURF_H);
        assert_eq!(p.pill_height(), render::SURF_W);
        // панель разворачивается вместе с пилюлей: окно вдоль края то же, что у горизонтальной
        assert_eq!(
            Placement::new(Edge::Left).panel_width(),
            render::PANEL_SURFACE_H
        );
        assert_eq!(Placement::new(Edge::Left).panel_height(), render::PANEL_W);
    }

    #[test]
    fn margins_land_on_the_side_being_used() {
        let p = Placement::new(Edge::Bottom).with_cross(-120).with_along(40);
        assert_eq!(p.margins, (0, 0, 40, -120));
        assert_eq!(p.along(), 40);
        assert_eq!(p.cross(), -120);

        let p = Placement::new(Edge::Right).with_cross(-30).with_along(10);
        assert_eq!(p.margins, (-30, 10, 0, 0));
        assert_eq!(p.along(), 10);
        assert_eq!(p.cross(), -30);

        assert_eq!(
            Placement::new(Edge::Top).with_along(12).margins,
            (12, 0, 0, 0)
        );
        assert_eq!(
            Placement::new(Edge::Left).with_along(12).margins,
            (0, 0, 0, 12)
        );
    }

    #[test]
    fn panel_is_pinned_to_the_pill_on_every_side() {
        // окно панели делит с пилюлей и сторону, и смещение вдоль неё: ряд кнопок
        // начинается ровно там же, где пилюля
        let p = Placement::new(Edge::Left).with_along(20);
        assert_eq!(p.panel_margins(), (0, 0, 0, 20));
        assert_eq!(p.panel_margins(), p.margins);
        let p = Placement::new(Edge::Bottom).with_along(48).with_cross(-70);
        assert_eq!(p.panel_margins(), (0, 0, 48, -70));
        // ряд кнопок кончается низом корпуса: окно ровно по карману плюс корпус
        assert_eq!(
            Placement::new(Edge::Top).panel_width(),
            render::PANEL_SURFACE_H
        );
    }

    #[test]
    fn moving_to_another_side_keeps_the_inset() {
        let p = Placement::new(Edge::Bottom).with_along(48).with_cross(-70);
        let p = p.with_edge(Edge::Left);
        assert_eq!(p.edge, Edge::Left);
        assert_eq!(p.along(), 48);
        assert_eq!(p.cross(), 0);
    }

    #[test]
    fn dragging_across_the_screen_changes_the_side() {
        let bottom = Placement::default();
        // тянули вверх через весь экран — верхняя сторона
        assert_eq!(bottom.dragged_px(0, -600).edge, Edge::Top);
        // вправо и чуть вниз — правая сторона
        assert_eq!(bottom.dragged_px(320, 40).edge, Edge::Right);
        let right = Placement::new(Edge::Right);
        // от правой границы влево через весь экран — левая сторона
        assert_eq!(right.dragged_px(-260, 10).edge, Edge::Left);
        // вниз по экрану — нижняя сторона
        assert_eq!(right.dragged_px(40, 400).edge, Edge::Bottom);
        // вверх по экрану — верхняя, с любой стороны
        assert_eq!(
            Placement::new(Edge::Left).dragged_px(-10, -300).edge,
            Edge::Top
        );
    }

    #[test]
    fn dragging_outward_or_along_the_side_only_shifts_it() {
        let p = Placement::new(Edge::Bottom).with_cross(10);
        // вдоль нижней кромки: рывок вправо — это правая граница экрана, сторона меняется
        let moved = p.dragged_px(70, 2);
        assert_eq!(moved.edge, Edge::Right);
        assert_eq!(moved.cross(), 0);
        // за свою кромку наружу (вниз) — остаёмся снизу и не сдвигаемся вдоль неё
        let moved = p.dragged_px(0, 90);
        assert_eq!(moved.edge, Edge::Bottom);
        assert_eq!(moved.cross(), 10);
        // левая сторона: рывок влево — это её кромка экрана, остаёмся слева и не едем
        let left = Placement::new(Edge::Left).with_cross(10);
        let moved = left.dragged_px(-90, 0);
        assert_eq!(moved.edge, Edge::Left);
        assert_eq!(moved.cross(), 10);
        // вниз по экрану от левой стороны — переезд на нижнюю
        let moved = left.dragged_px(0, 300);
        assert_eq!(moved.edge, Edge::Bottom);
        // дрожание меньше порога — сторона не меняется
        assert_eq!(left.dragged_px(2, 1).edge, Edge::Left);
        // ноль — ничего не происходит
        assert_eq!(p.dragged_px(0, 0), p);
    }

    #[test]
    fn drag_is_rounded_to_pixels() {
        // движение вниз от нижней кромки остаётся снизу, сдвиг вдоль неё округлён до пикселя
        let p = Placement::default().dragged(3.4, 12.6);
        assert_eq!(p.edge, Edge::Bottom);
        assert_eq!(p.cross(), 3);
        // движение вверх — уже верхняя сторона
        let p = Placement::default().dragged(3.4, -12.6);
        assert_eq!(p.edge, Edge::Top);
    }

    #[test]
    fn shift_and_inset_are_bounded() {
        let p = Placement::default().with_cross(99_000);
        assert_eq!(p.cross(), CROSS_LIMIT);
        let p = Placement::default().with_along(-500).with_along(90_000);
        assert_eq!(p.along(), INSET_LIMIT);
    }

    #[test]
    fn pill_rises_from_behind_its_own_edge() {
        let bottom = Placement::new(Edge::Bottom);
        // старт — целиком за кромкой экрана: корпус за дальним краем своего окна
        assert_eq!(bottom.pill_top(render::SURF_H, 0.0), render::SURF_H as i32);
        // покой — на REST_GAP над кромкой экрана: низ корпуса на SURF_H - REST_GAP
        assert_eq!(
            bottom.pill_top(render::SURF_H, 1.0) + render::PILL_H as i32,
            (render::SURF_H - render::REST_GAP) as i32
        );
        // середина пути — между ними
        let mid = bottom.pill_top(render::SURF_H, 0.5);
        assert!(
            mid < bottom.pill_top(render::SURF_H, 0.0)
                && mid > bottom.pill_top(render::SURF_H, 1.0)
        );
        let peak = (0..=100)
            .map(|i| render::ease_out_back(i as f32 / 100.0))
            .fold(f32::MIN, f32::max);
        assert!(
            render::SURF_H as f32 - render::TRAVEL as f32 * peak >= 0.0,
            "overshoot пружины обрезается ближним краем окна"
        );
        // левая сторона: окно начинается у кромки, поэтому старт — за нулевым краем,
        // покой — на REST_GAP от кромки; вдоль стороны пилюля заполняет окно
        let left = Placement::new(Edge::Left);
        assert_eq!(
            left.pill_left(render::SURF_H, 0.0),
            -(render::PILL_H as i32)
        );
        assert_eq!(left.pill_left(render::SURF_H, 1.0), render::REST_GAP as i32);
        assert_eq!(left.pill_top(render::SURF_H, 0.5), 0);
        // правая сторона зеркальна левой: тот же путь выезда вдоль своей стороны
        let right = Placement::new(Edge::Right);
        assert_eq!(right.pill_left(render::SURF_H, 0.0), render::SURF_H as i32);
        // правый край корпуса в покое — на REST_GAP от кромки экрана
        assert_eq!(
            right.pill_left(render::SURF_H, 1.0) + render::PILL_H as i32,
            (render::SURF_H - render::REST_GAP) as i32
        );
        // верхняя сторона зеркальна нижней
        let top = Placement::new(Edge::Top);
        assert_eq!(top.pill_top(render::SURF_H, 0.0), -(render::PILL_H as i32));
        assert_eq!(top.pill_top(render::SURF_H, 1.0), render::REST_GAP as i32);
    }

    #[test]
    fn pill_click_area_follows_the_orientation() {
        let bottom = Placement::new(Edge::Bottom);
        assert_eq!(
            bottom.pill_rect(render::SURF_W, render::SURF_H, 1.0).2,
            render::SURF_W
        );
        // прижат к нижней кромке своей поверхности: сдвиг вверх на HEADROOM
        assert_eq!(
            bottom.pill_rect(render::SURF_W, render::SURF_H, 1.0).3,
            render::PILL_H
        );
        let right = Placement::new(Edge::Right);
        let (ox, oy, w, h) = right.pill_rect(right.pill_width(), right.pill_height(), 1.0);
        assert_eq!((w, h), (render::PILL_H, render::SURF_W));
        // в покое корпус пилюли прижат к своей стороне: REST_GAP до кромки экрана
        assert_eq!((ox, oy), ((render::SURF_H - render::TRAVEL) as i32, 0));
        let left = Placement::new(Edge::Left);
        assert_eq!(
            left.pill_rect(left.pill_width(), left.pill_height(), 1.0).0,
            render::REST_GAP as i32
        );
    }

    #[test]
    fn panel_buttons_are_hit_tested_in_both_orientations() {
        let settings = crate::panel::Settings::default();
        let keys = crate::panel::buttons(&settings, false, Edge::Bottom);

        // горизонтально: центр кнопки «закрыть» лежит в её прямоугольнике
        let close = keys.iter().find(|k| k.id == "close").unwrap();
        let origin = (0, 8);
        let index = button_at(
            &keys,
            close.x as f64 + 10.0,
            origin.1 as f64 + close.y as f64 + 10.0,
            origin,
            false,
        );
        assert_eq!(index.map(|i| keys[i].id), Some("close"));

        // вертикально: те же кнопки, но ряд стал колонкой — координаты меняются местами
        let origin = (render::HEADROOM as i32, 0);
        let index = button_at(
            &keys,
            origin.0 as f64 + close.y as f64 + 10.0,
            close.x as f64 + 10.0,
            origin,
            true,
        );
        assert_eq!(index.map(|i| keys[i].id), Some("close"));
        // мимо корпуса — ничего
        assert_eq!(button_at(&keys, 2.0, 2.0, origin, true), None);
    }

    #[test]
    fn config_round_trips_through_toml() {
        let dir = std::env::temp_dir().join("dicta-ui-edge-test");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("ui.toml");
        let cfg = UiConfig {
            edge: Some(Edge::Right),
            offset: -120,
            inset: 8,
        };
        save(&path, &cfg).unwrap();
        let back = load(&path);
        assert_eq!(back, cfg);

        // положение -> файл -> положение: перетаскивание сохраняется без потерь
        let p = back.placement();
        assert_eq!(p.edge, Edge::Right);
        assert_eq!(p.along(), 8);
        assert_eq!(p.cross(), -120);
        assert_eq!(UiConfig::from_placement(&p).placement(), p);

        // и после переезда на другую сторону: тянули влево — левая сторона
        let moved = UiConfig::from_placement(&p.dragged_px(-400, 30));
        save(&path, &moved).unwrap();
        assert_eq!(load(&path).edge, Some(Edge::Left));
    }

    #[test]
    fn missing_or_broken_config_falls_back_to_the_bottom() {
        let dir = std::env::temp_dir().join("dicta-ui-edge-fallback");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("none.toml");
        assert_eq!(load(&path).placement(), Placement::default());

        std::fs::write(&path, "edge = \"corner\"\noffset = не число\n").unwrap();
        assert_eq!(load(&path).placement(), Placement::default());
    }
}

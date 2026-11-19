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

    /// Смежная грань на конце своей стороны: у низа/верха концы — лево и право,
    /// у лева/права — верх и низ. Нужно, когда пилюлю дотащили вдоль её грани
    /// до упора в угол экрана: курсор остаётся внутри пилюли и сам до «зоны у
    /// грани» не доходит — тогда кандидатом объявляет упор пилюли.
    pub fn adjacent(self, toward_start: bool) -> Edge {
        match self {
            Edge::Bottom | Edge::Top => {
                if toward_start {
                    Edge::Left
                } else {
                    Edge::Right
                }
            }
            Edge::Left | Edge::Right => {
                if toward_start {
                    Edge::Top
                } else {
                    Edge::Bottom
                }
            }
        }
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

    /// Как повёрнут кадр панели на этой стороне: левая грань — по часовой,
    /// правая — против, горизонтальные — не повёрнуты.
    pub fn turn(self) -> Option<render::Turn> {
        match self.edge {
            Edge::Left => Some(render::Turn::Cw),
            Edge::Right => Some(render::Turn::Ccw),
            Edge::Bottom | Edge::Top => None,
        }
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

    /// Ширина окна панели: вдоль стороны — столько, сколько занимает ряд кнопок (PANEL_W),
    /// поперёк — карман (PANEL_TRAVEL) плюс HEADROOM, то есть PANEL_SURFACE_H. Окно обязано
    /// вмещать весь ряд: иначе последние кнопки каждого ряда обрезаются по краю окна.
    pub fn panel_width(self) -> u32 {
        match self.edge {
            // низ/верх: ряд кнопок идёт по x, поэтому по x окно — PANEL_W
            Edge::Bottom | Edge::Top => render::PANEL_W,
            // лево/право: по x лежит карман плюс толщина ряда
            Edge::Left | Edge::Right => self.panel_across(),
        }
    }

    pub fn panel_height(self) -> u32 {
        match self.edge {
            Edge::Bottom | Edge::Top => self.panel_across(),
            Edge::Left | Edge::Right => render::PANEL_W,
        }
    }

    /// Размер окна поперёк стороны: карман панели (путь подъёма) плюс HEADROOM.
    /// Именно эта величина — «каdr», из которого `panel_coord` считает координату выезда:
    /// `slide` и `flip` дают покой корпуса на расстоянии PANEL_BELOW от кромки при любом
    /// её значении, поэтому сдвиг пилюли вдоль стороны окно не меняет (маргины носят окно
    /// целиком вместе с рядом: сдвиг уходит в маргин, а не в размер).
    fn panel_across(self) -> u32 {
        render::PANEL_TRAVEL + render::HEADROOM
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
    /// Кадр для `panel_coord` — размер окна поперёк стороны: для низа/верха это высота,
    /// для лева/права ширина (там подъём идёт по горизонтали).
    pub fn panel_origin(self, reveal: f32) -> (i32, i32) {
        let across = self.panel_across();
        match self.edge {
            Edge::Bottom | Edge::Top => (0, self.panel_coord(across, reveal)),
            Edge::Left | Edge::Right => (self.panel_coord(across, reveal), 0),
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

    /// Сдвинуться вдоль своей стороны на столько пикселей, на сколько двинулась
    /// мышь (вправо для низа/верха, вниз для лева/права). Приращение, а не
    /// накопленный сдвиг: после переезда на другую сторону система координат
    /// поверхности меняется, и накопленный сдвиг от точки нажатия теряет смысл.
    pub fn shifted_along(self, px: i32) -> Self {
        self.with_cross(self.cross() + px)
    }

    /// Уложить смещение так, чтобы окно пилюли (SURF_W вдоль стороны) целиком
    /// осталось на экране: центр окна не дальше половины экрана от центра
    /// стороны. Слишком короткий экран — пилюля встаёт по центру.
    pub fn clamped_cross(self, screen_along: i32) -> Self {
        let slack = (screen_along - render::SURF_W as i32) / 2;
        // экран длиннее окна — центр окна гуляет в пределах ±slack;
        // короче — пилюля встаёт по центру стороны
        self.with_cross(self.cross().clamp(-slack.max(0), slack.max(0)))
    }
}

/// Приращение мыши вдоль данной стороны: низ/верх меряют по x, лево/право — по y.
/// Наружу за свою кромку пилюля не едет — та компонента учитывается отдельно
/// (`across_delta`), а здесь только сдвиг вдоль.
pub fn along_delta(edge: Edge, dx: i32, dy: i32) -> i32 {
    match edge {
        Edge::Bottom | Edge::Top => dx,
        Edge::Left | Edge::Right => dy,
    }
}

/// Расстояние до грани монитора, внутри которого перетаскиваемая пилюля считает,
/// что её несут к этой грани: в этой зоне показывается ореол места будущей
/// фиксации, а по отпусканию мыши пилюля закрепляется на ней. «Почти к грани».
pub const DOCK_SNAP_PX: i32 = 32;

/// Грань, к которой курсор ближе всего, — если он в DOCK_SNAP_PX от неё. В
/// углу экрана побеждает ближе лежащая грань.
pub fn nearest_edge(cx: i32, cy: i32, screen: (i32, i32)) -> Option<Edge> {
    let (w, h) = screen;
    let dist = |e: Edge| match e {
        Edge::Left => cx,
        Edge::Right => w - cx,
        Edge::Top => cy,
        Edge::Bottom => h - cy,
    };
    let best = Edge::all()
        .into_iter()
        .min_by_key(|e| dist(*e))
        .expect("стороны не пусты");
    (dist(best) <= DOCK_SNAP_PX).then_some(best)
}

/// Позиция окна пилюли на экране: якорь на своей грани, вдоль неё окно
/// центрировано и сдвинуто маргинами (cross — вдоль, along — от грани внутрь).
pub fn window_origin(p: &Placement, w: u32, h: u32, screen: (i32, i32)) -> (i32, i32) {
    let (sw, sh) = screen;
    let (t, r, b, l) = p.margins;
    match p.edge {
        Edge::Bottom => ((sw - w as i32) / 2 + l, sh - h as i32 - b),
        Edge::Top => ((sw - w as i32) / 2 + l, t),
        Edge::Left => (l, (sh - h as i32) / 2 + t),
        Edge::Right => (sw - w as i32 - r, (sh - h as i32) / 2 + t),
    }
}

/// Центр корпуса пилюли в покое на экране: куда прилетает орб при отпускании —
/// интерфейс «выходит из кружка» ровно там, где встанет пилюля.
pub fn body_center(p: &Placement, screen: (i32, i32)) -> (i32, i32) {
    let (w, h) = (p.pill_width(), p.pill_height());
    let (ox, oy) = window_origin(p, w, h, screen);
    let (rx, ry, rw, rh) = p.pill_rect(w, h, 1.0);
    (ox + rx + rw as i32 / 2, oy + ry + rh as i32 / 2)
}

/// Курсор в координатах экрана: локальная точка окна пилюли плюс позиция окна.
/// Работает и когда курсор ушёл с окна (неявный захват): локальные координаты
/// просто выходят за его пределы.
pub fn cursor_abs(p: &Placement, w: u32, h: u32, screen: (i32, i32), x: f64, y: f64) -> (i32, i32) {
    let (ox, oy) = window_origin(p, w, h, screen);
    (ox + x.round() as i32, oy + y.round() as i32)
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

/// Прямоугольник кнопки в координатах горизонтальной панели. `origin` — верхний
/// левый угол корпуса (сдвиг выезда). Вертикальные грани используют ту же
/// разметку: их окно — повёрнутый кадр, а клики возвращает `unturn`.
pub fn button_rect(b: &render::Button, origin: (i32, i32)) -> (i32, i32, u32, u32) {
    (
        origin.0 + b.x as i32,
        origin.1 + b.y as i32,
        b.w,
        render::PANEL_BTN_H,
    )
}

/// Кнопка под точкой (x, y) в координатах горизонтальной панели.
pub fn button_at(keys: &[render::Button], x: f64, y: f64, origin: (i32, i32)) -> Option<usize> {
    let (x, y) = (x as i32, y as i32);
    keys.iter().position(|b| {
        let (bx, by, bw, bh) = button_rect(b, origin);
        x >= bx && x < bx + bw as i32 && y >= by && y < by + bh as i32
    })
}

/// Точка клика на панели в координатах горизонтальной разметки. Вертикальная
/// грань показывает повёрнутый на 90° кадр: клик приходит в системе окна
/// (поперёк × вдоль), а кнопки размечены в системе горизонтальной панели
/// (`PANEL_W × PANEL_SURFACE_H`) — точка возвращается туда.
pub fn unturn(edge: Edge, x: f64, y: f64) -> (f64, f64) {
    match edge {
        Edge::Left => (y, render::PANEL_SURFACE_H as f64 - 1.0 - x),
        Edge::Right => (render::PANEL_W as f64 - 1.0 - y, x),
        Edge::Bottom | Edge::Top => (x, y),
    }
}

/// Кнопка под точкой клика при любом положении панели. Горизонтальная сторона
/// берёт разметку как есть; вертикальная сначала возвращает точку из повёрнутого
/// окна. `origin` — скользящее начало корпуса (`panel_origin`): для вертикальной
/// стороны это координата поперёк окна, для горизонтальной — верх корпуса.
pub fn button_at_turned(
    keys: &[render::Button],
    x: f64,
    y: f64,
    p: &Placement,
    origin: (i32, i32),
) -> Option<usize> {
    match p.turn() {
        None => button_at(keys, x, y, origin),
        Some(turn) => {
            let (sx, sy) = unturn(p.edge, x, y);
            button_at(keys, sx, sy, (0, turn.src_oy(origin.0)))
        }
    }
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
        // окно панели меряется так же, как окно пилюли: вдоль стороны — длина ряда (PANEL_W),
        // поперёк — карман плюс корпус (PANEL_SURFACE_H)
        assert_eq!(Placement::new(Edge::Top).panel_width(), render::PANEL_W);
        assert_eq!(
            Placement::new(Edge::Top).panel_height(),
            render::PANEL_SURFACE_H
        );
    }

    /// Регрессия на обрезанные ряды: на любой стороне окно панели обязано вместить корпус
    /// целиком и каждую кнопку. Вертикальная сторона показывает повёрнутый кадр
    /// горизонтальной панели: если кнопка влезает в горизонтальную разметку, она влезает
    /// и в повёрнутое окно, поэтому проверяем разметку и совпадение размеров окна.
    #[test]
    fn panel_window_holds_every_button_on_every_side() {
        for edge in Edge::all() {
            let p = Placement::new(edge).with_along(24).with_cross(-120);
            let keys = crate::panel::buttons(&crate::panel::Settings::default(), false, edge);
            assert!(!keys.is_empty(), "{edge:?}: пустой ряд кнопок");
            // скользящее начало корпуса в горизонтальной разметке
            let origin = (
                0,
                match p.turn() {
                    Some(turn) => turn.src_oy(p.panel_origin(1.0).0),
                    None => p.panel_origin(1.0).1,
                },
            );
            let (w, h) = (render::PANEL_W as i32, render::PANEL_SURFACE_H as i32);
            for b in &keys {
                let (x, y, bw, bh) = button_rect(b, origin);
                assert!(
                    x >= 0 && y >= 0 && (x + bw as i32) <= w && (y + bh as i32) <= h,
                    "{}: кнопка {} ({x},{y} {bw}x{bh}) не влезает в разметку {w}x{h}",
                    edge.as_str(),
                    b.id
                );
            }
            // окно стороны совпадает с кадром: горизонтальная сторона — прямая
            // разметка, вертикальная — её поворот на 90°
            let window = match p.turn() {
                None => (render::PANEL_W, render::PANEL_SURFACE_H),
                Some(_) => (render::PANEL_SURFACE_H, render::PANEL_W),
            };
            assert_eq!(
                (p.panel_width(), p.panel_height()),
                window,
                "{}: окно панели не совпадает с кадром разметки",
                edge.as_str()
            );
        }
    }

    #[test]
    fn moving_to_another_side_keeps_the_inset() {
        let p = Placement::new(Edge::Bottom).with_along(48).with_cross(-70);
        let p = p.with_edge(Edge::Left);
        assert_eq!(p.edge, Edge::Left);
        assert_eq!(p.along(), 48);
        assert_eq!(p.cross(), 0);
    }

    /// Как on_motion делит приращение мыши: вдоль своей стороны едет пилюля.
    #[test]
    fn deltas_split_the_pointer_movement() {
        // нижняя/верхняя сторона меряет вдоль по x
        assert_eq!(along_delta(Edge::Bottom, 30, 0), 30);
        assert_eq!(along_delta(Edge::Top, -20, 5), -20);
        // левая/правая — по y
        assert_eq!(along_delta(Edge::Left, 0, 40), 40);
        assert_eq!(along_delta(Edge::Right, 0, -40), -40);
    }

    #[test]
    fn shifting_along_keeps_the_side_and_moves_the_cross() {
        let p = Placement::new(Edge::Bottom).with_cross(10);
        let moved = p.shifted_along(70);
        assert_eq!(moved.edge, Edge::Bottom);
        assert_eq!(moved.cross(), 80);
        // наружу за свою кромку поперёк пилюля не едет — только вдоль
        let left = Placement::new(Edge::Left).with_cross(10);
        let moved = left.shifted_along(30);
        assert_eq!(moved.edge, Edge::Left);
        assert_eq!(moved.cross(), 40);
        // сдвиг ограничен тем же пределом, что и явное смещение
        let far = Placement::new(Edge::Bottom).shifted_along(99_000);
        assert_eq!(far.cross(), CROSS_LIMIT);
        // вниз по экрану от левой стороны — это вдоль, а не переезд
        let moved = Placement::new(Edge::Right).shifted_along(-300);
        assert_eq!(moved.edge, Edge::Right);
        assert_eq!(moved.cross(), -300);
    }

    /// Смежные грани: конец стороны низа/верха — лево и право, конец лева/права —
    /// верх и низ. По упору в угол курсор сам к грани не доходит — нужен упор.
    #[test]
    fn adjacent_edge_sits_where_the_side_ends() {
        assert_eq!(Edge::Bottom.adjacent(true), Edge::Left);
        assert_eq!(Edge::Bottom.adjacent(false), Edge::Right);
        assert_eq!(Edge::Top.adjacent(true), Edge::Left);
        assert_eq!(Edge::Top.adjacent(false), Edge::Right);
        assert_eq!(Edge::Left.adjacent(true), Edge::Top);
        assert_eq!(Edge::Left.adjacent(false), Edge::Bottom);
        assert_eq!(Edge::Right.adjacent(true), Edge::Top);
        assert_eq!(Edge::Right.adjacent(false), Edge::Bottom);
    }

    /// Прилипание: грань-кандидат — та, к которой курсор поднесли ближе
    /// DOCK_SNAP_PX; в углу побеждает ближе лежащая грань.
    #[test]
    fn nearest_edge_wins_where_the_cursor_is_closest() {
        let screen = (1920, 1080);
        // курсор в центре — никакого кандидата
        assert_eq!(nearest_edge(960, 540, screen), None);
        // поднесли к левой грани — левая
        assert_eq!(nearest_edge(20, 540, screen), Some(Edge::Left));
        // к верхней — верхняя (это и был недоступный «вверх»)
        assert_eq!(nearest_edge(960, 25, screen), Some(Edge::Top));
        assert_eq!(nearest_edge(960, 1080 - 30, screen), Some(Edge::Bottom));
        assert_eq!(nearest_edge(1900, 540, screen), Some(Edge::Right));
        // в углу побеждает ближе лежащая грань
        assert_eq!(nearest_edge(10, 4, screen), Some(Edge::Top));
        assert_eq!(nearest_edge(4, 10, screen), Some(Edge::Left));
        // ровно на границе зоны (32 px) — кандидат ещё есть
        assert_eq!(nearest_edge(32, 540, screen), Some(Edge::Left));
        assert_eq!(nearest_edge(33, 540, screen), None);
    }

    /// Курсор в координатах экрана: локальная точка окна пилюли плюс позиция
    /// самого окна (якорь на грани, центр вдоль неё, маргины cross/along).
    #[test]
    fn cursor_maps_from_window_to_screen() {
        let screen = (1920, 1080);
        // пилюля на нижней грани, смещена влево на 100, отступ от грани 48:
        // окно 520x54, левый край (1920-520)/2 - 100 = 600, верх 1080-54-48 = 978
        let p = Placement::new(Edge::Bottom).with_cross(-100).with_along(48);
        let (ox, oy) = window_origin(&p, render::SURF_W, render::SURF_H, screen);
        assert_eq!((ox, oy), (600, 978));
        assert_eq!(
            cursor_abs(&p, render::SURF_W, render::SURF_H, screen, 10.0, 5.0),
            (610, 983)
        );
        // левая грань: вдоль — вниз, окно центрировано по вертикали
        let p = Placement::new(Edge::Left).with_cross(-200);
        let (ox, oy) = window_origin(&p, render::SURF_H, render::SURF_W, screen);
        assert_eq!(ox, 0); // along = 0: окно прижато к грани
        assert_eq!(oy, (1080 - render::SURF_W as i32) / 2 - 200);
    }

    /// Центр корпуса: у нижней пилюли по центру экрана — на высоте покоя.
    #[test]
    fn body_center_is_the_resting_pill_middle() {
        let screen = (2560, 1440);
        let p = Placement::new(Edge::Bottom);
        let (cx, cy) = body_center(&p, screen);
        assert_eq!(cx, 1280);
        // покой: REST_GAP (12) от кромки, корпус 34: центр на 12+17 от низа
        assert_eq!(cy, 1440 - 12 - 17);
        let p = Placement::new(Edge::Left).with_cross(100);
        let (cx, cy) = body_center(&p, screen);
        assert_eq!(cx, 12 + 17);
        assert_eq!(
            cy,
            (1440 - render::SURF_W as i32) / 2 + 100 + render::SURF_W as i32 / 2
        );
    }

    /// Смещение вдоль грани не должно уводить окно пилюли за экран.
    #[test]
    fn cross_is_clamped_to_the_screen() {
        let p = Placement::new(Edge::Bottom)
            .shifted_along(90_000)
            .clamped_cross(1920);
        // окно 520: центр может отъехать от центра экрана на (1920-520)/2 = 700
        assert_eq!(p.cross(), 700);
        let p = Placement::new(Edge::Right)
            .shifted_along(-90_000)
            .clamped_cross(1080);
        assert_eq!(p.cross(), -(1080 - render::SURF_W as i32) / 2);
        // экран короче окна — пилюля встаёт по центру
        let p = Placement::new(Edge::Bottom)
            .with_cross(120)
            .clamped_cross(400);
        assert_eq!(p.cross(), 0);
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
        );
        assert_eq!(index.map(|i| keys[i].id), Some("close"));

        // вертикально (левая грань): клик приходит в системе повёрнутого окна,
        // button_at_turned возвращает его в разметку — и попадает в ту же кнопку
        let p = Placement::new(Edge::Left);
        let oy = p.panel_origin(1.0).0;
        let src_oy = render::Turn::Cw.src_oy(oy);
        let cx = close.x as f64 + close.w as f64 / 2.0;
        let cy = src_oy as f64 + close.y as f64 + 10.0;
        // обратный поворот к unturn: dst(x, y) = src(y, PANEL_SURFACE_H - 1 - x)
        let (dx, dy) = (render::PANEL_SURFACE_H as f64 - 1.0 - cy, cx);
        let index = button_at_turned(&keys, dx, dy, &p, (oy, 0));
        assert_eq!(index.map(|i| keys[i].id), Some("close"));
        // мимо корпуса (карман у кромки) — ничего
        assert_eq!(button_at_turned(&keys, 2.0, 2.0, &p, (oy, 0)), None);
        // и правая грань: там поворот против часовой, точка возвращается зеркально
        let p = Placement::new(Edge::Right);
        let oy = p.panel_origin(1.0).0;
        let (dx, dy) = (cy, render::PANEL_W as f64 - 1.0 - cx);
        let index = button_at_turned(&keys, dx, dy, &p, (oy, 0));
        assert_eq!(index.map(|i| keys[i].id), Some("close"));
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
        let moved = UiConfig::from_placement(&p.with_edge(Edge::Left));
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

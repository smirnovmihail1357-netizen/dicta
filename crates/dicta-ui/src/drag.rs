//! Машина перетаскивания пилюли: чистое состояние без Wayland.
//!
//! Живёт отдельно от Ui, чтобы сценарии перетаскивания можно было проигрывать
//! синтетическими событиями мыши в юнит-тестах — «зажал у нижней грани,
//! протащил к верхней, отпустил» — без композитора и живого сеанса. Ui
//! остаётся только доставка событий и рисование результата.
//!
//! Решения машины:
//! - короткое движение без порога — это клик («остановить и вставить»);
//! - пока идёт перетаскивание, пилюля собрана в орб и не двигается: сдвиг
//!   вдоль своей стороны копится (`along`) и применяется по отпусканию;
//! - грань-кандидат (`hint`) — где покажет ореол и где закрепится отпускание:
//!   курсор поднесён к грани монитора, либо пилюля упёрлась в конец своей
//!   стороны (курсор остаётся внутри неё и до зоны у грани сам не доходит).

use crate::edge::{self, Placement};
use crate::render;

/// Порог втягивания: движение меньше этого — ещё клик, не перетаскивание.
pub const DRAG_THRESHOLD: i32 = 60;

/// Снимок мира, который Ui передаёт машине перед каждым событием.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct World {
    /// где стоит пилюля: сторона и маргины
    pub placement: Placement,
    /// размер окна пилюли (wl_surface)
    pub window: (u32, u32),
    /// размер экрана в логических пикселях (None = композитор не сказал)
    pub screen: Option<(i32, i32)>,
}

/// Чем кончилось отпускание кнопки.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Release {
    /// короткое движение без порога — «остановить и вставить»
    Click,
    /// закрепиться на грани-кандидате (показывала ореол)
    Dock(Placement),
    /// сдвинуться вдоль своей стороны на столько пикселей
    Slide(i32),
    /// никуда не подвезли — остаться на месте
    Nothing,
}

/// Грань-кандидат: курсор у грани монитора (DOCK_SNAP_PX), либо пилюля упёрлась
/// в конец своей стороны — курсор в ловушке внутри неё. `slid` — положение
/// пилюли со скопленным сдвигом (без применения), по нему виден упор в угол.
pub fn candidate(world: &World, x: f64, y: f64, slid: &Placement) -> Option<Placement> {
    let screen = world.screen?;
    let edge = world.placement.edge;
    let (cx, cy) = edge::cursor_abs(
        &world.placement,
        world.window.0,
        world.window.1,
        screen,
        x,
        y,
    );
    // своя грань кандидатом не бывает: ореол не должен рисоваться поверх
    // самой пилюли, когда её оттягивают наружу
    let by_cursor = edge::nearest_edge(cx, cy, screen).filter(|target| *target != edge);
    let by_corner = {
        let dim = if edge.horizontal() {
            screen.0
        } else {
            screen.1
        };
        let slack = (dim - render::SURF_W as i32) / 2;
        let cross = slid.cross();
        if slack > 0 && cross <= -slack {
            Some(edge.adjacent(true))
        } else if slack > 0 && cross >= slack {
            Some(edge.adjacent(false))
        } else {
            None
        }
    };
    let target = by_cursor.or(by_corner)?;
    // пилюля встанет на новой грани по положению курсора вдоль неё
    let (cursor_along, dim) = if target.horizontal() {
        (cx, screen.0)
    } else {
        (cy, screen.1)
    };
    Some(
        Placement::new(target)
            .with_along(world.placement.along())
            .with_cross(cursor_along - dim / 2)
            .clamped_cross(dim),
    )
}

/// Состояние одного перетаскивания: от зажатия до отпускания.
#[derive(Debug, Default)]
pub struct Drag {
    /// где нажали (координаты окна пилюли); None = кнопка не зажата
    origin: Option<(f64, f64)>,
    /// курсор сейчас (координаты окна пилюли): кружок захвата и центр орба
    cursor: Option<(f64, f64)>,
    /// последняя точка: приращения считаются от неё, а не от точки нажатия —
    /// после переякоривания координаты поверхности меняются
    last: (f64, f64),
    /// перетаскивание началось: пилюля собрана в орб, кнопка не «клик»
    active: bool,
    /// сколько протащили вдоль своей стороны — применится по отпусканию
    along: i32,
    /// грань-кандидат: где показывает ореол и где закрепится отпускание
    hint: Option<Placement>,
}

impl Drag {
    pub fn new() -> Self {
        Self::default()
    }

    /// Кнопка зажата на пилюле.
    pub fn press(&mut self, x: f64, y: f64) {
        self.origin = Some((x, y));
        self.cursor = Some((x, y));
        self.last = (x, y);
        self.active = false;
        self.along = 0;
        self.hint = None;
    }

    /// Курсор в координатах окна пилюли, пока кнопка зажата.
    pub fn cursor(&self) -> Option<(f64, f64)> {
        self.cursor
    }

    /// Перетаскивание идёт: пилюля собрана в орб, по отпусканию вернётся.
    pub fn is_active(&self) -> bool {
        self.active
    }

    /// Кнопка ещё держится (порог мог не быть пройден).
    pub fn held(&self) -> bool {
        self.origin.is_some()
    }

    /// Грань-кандидат: где рисуется ореол.
    pub fn hint(&self) -> Option<Placement> {
        self.hint
    }

    /// Движение с зажатой кнопкой. Возвращает true, если видимое поменялось
    /// (кружок захвата, ореол или позиция орба) и кадр стоит перерисовать.
    ///
    /// Сдвиг вдоль стороны копится без применения: пилюля сейчас собрана в
    /// орб и невидима, а двигать её окно на каждом событии — лишние коммиты
    /// и увод орба от центра курсора (позиция орба считается от окна пилюли).
    pub fn motion(&mut self, x: f64, y: f64, world: &World) -> bool {
        let Some((ox, oy)) = self.origin else {
            return false;
        };
        if !self.active
            && (x - ox).abs() < DRAG_THRESHOLD as f64
            && (y - oy).abs() < DRAG_THRESHOLD as f64
        {
            // ещё клик: только кружок захвата ведёт за курсором
            self.cursor = Some((x, y));
            return true;
        }
        // приращение от последнего события
        let (dx, dy) = (x - self.last.0, y - self.last.1);
        self.last = (x, y);
        self.cursor = Some((x, y));
        if !self.active {
            self.active = true;
        } else if dx == 0.0 && dy == 0.0 {
            return true; // позиция не менялась, но курсор тот же
        }
        let edge = world.placement.edge;
        self.along += edge::along_delta(edge, dx.round() as i32, dy.round() as i32);
        let slid = match world
            .screen
            .map(|(w, h)| if edge.horizontal() { w } else { h })
        {
            Some(dim) => world.placement.shifted_along(self.along).clamped_cross(dim),
            None => world.placement.shifted_along(self.along),
        };
        let hint = candidate(world, x, y, &slid);
        if hint != self.hint {
            match (&hint, &self.hint) {
                (Some(p), _) => eprintln!(
                    "dicta-ui: кандидат {} (смещение {})",
                    p.edge.as_str(),
                    p.cross()
                ),
                (None, Some(_)) => eprintln!("dicta-ui: кандидат снят"),
                (None, None) => {}
            }
            self.hint = hint;
        }
        true
    }

    /// Отпустили кнопку: клик, закрепление на грани-кандидате или сдвиг
    /// вдоль своей стороны. Сбрасывает всё состояние перетаскивания.
    pub fn release(&mut self, x: f64, y: f64) -> Release {
        let Some((ox, oy)) = self.origin.take() else {
            return Release::Nothing;
        };
        self.cursor = None;
        let was_active = self.active;
        let along = self.along;
        let hint = self.hint.take();
        self.active = false;
        self.along = 0;
        if !was_active
            && (x - ox).abs() < DRAG_THRESHOLD as f64
            && (y - oy).abs() < DRAG_THRESHOLD as f64
        {
            return Release::Click;
        }
        if let Some(target) = hint {
            return Release::Dock(target);
        }
        if along != 0 {
            return Release::Slide(along);
        }
        Release::Nothing
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::edge::Edge;

    /// Домашний экран из журнала: пилюля на нижней грани DP-2 (2560x1440).
    fn bottom_world() -> World {
        World {
            placement: Placement::new(Edge::Bottom),
            window: (520, 54),
            screen: Some((2560, 1440)),
        }
    }

    /// Курсор в координатах окна пилюли для точки экрана (abs).
    fn local(world: &World, abs: (i32, i32)) -> (f64, f64) {
        let (ox, oy) = edge::window_origin(
            &world.placement,
            world.window.0,
            world.window.1,
            world.screen.unwrap(),
        );
        (abs.0 as f64 - ox as f64, abs.1 as f64 - oy as f64)
    }

    /// Короткое движение — это клик, перетаскивание не началось.
    #[test]
    fn short_press_is_a_click_not_a_drag() {
        let mut d = Drag::new();
        d.press(10.0, 10.0);
        assert!(d.held());
        d.motion(12.0, 14.0, &bottom_world());
        assert!(!d.is_active(), "порог не пройден — это ещё клик");
        assert_eq!(d.release(13.0, 11.0), Release::Click);
        assert!(!d.held());
        assert!(!d.is_active(), "после отпускания машина чиста");
        assert_eq!(d.cursor(), None);
    }

    /// Сценарий «зажал у нижней грани, утащил к верхней, отпустил»: пилюля
    /// закрепляется на верхней грани по положению курсора.
    #[test]
    fn dragging_from_bottom_to_top_docks_on_top() {
        let world = bottom_world();
        let mut d = Drag::new();
        let start = local(&world, (1280, 1400));
        d.press(start.0, start.1);
        // тащим вверх ступеньками — как реальные события мыши
        let mut abs_y = 1400;
        while abs_y > 32 {
            abs_y -= 90;
            let p = local(&world, (1280, abs_y));
            d.motion(p.0, p.1, &world);
            assert!(d.is_active());
        }
        // курсор в зоне верхней грани: ореол показывает закрепление там
        let hint = d
            .hint()
            .expect("курсор у верхней грани — кандидат обязан быть");
        assert_eq!(hint.edge, Edge::Top);
        // пилюля встанет по курсору: он в центре по x — смещение 0
        assert_eq!(hint.cross(), 0);
        // честный release в координатах окна:
        let top = local(&world, (1280, 30));
        match d.release(top.0, top.1) {
            Release::Dock(p) => {
                assert_eq!(p.edge, Edge::Top);
                assert_eq!(p.cross(), 0);
            }
            other => panic!("ожидали закрепление сверху, получили {other:?}"),
        }
        assert!(!d.is_active(), "отпускание всегда гасит перетаскивание");
        assert_eq!(d.hint(), None);
    }

    /// Тащили вдоль своей стороны, не поднося к грани: отпускание — сдвиг.
    #[test]
    fn dragging_along_the_edge_without_a_side_ends_in_slide() {
        let world = bottom_world();
        let mut d = Drag::new();
        let start = local(&world, (1280, 1400));
        d.press(start.0, start.1);
        for i in 1..=20 {
            let p = local(&world, (1280 + i * 20, 1400));
            d.motion(p.0, p.1, &world);
        }
        assert!(d.is_active());
        assert_eq!(d.hint(), None, "вдали от граней кандидата нет");
        match d.release(local(&world, (1680, 1400)).0, start.1) {
            Release::Slide(px) => assert_eq!(px, 400),
            other => panic!("ожидали сдвиг вдоль грани, получили {other:?}"),
        }
    }

    /// Курсор в ловушке: тащили вдоль к углу, пилюля упёрлась в конец экрана —
    /// кандидатом становится смежная грань у этого угла.
    #[test]
    fn cornered_drag_offers_the_adjacent_edge() {
        let world = bottom_world();
        let mut d = Drag::new();
        let start = local(&world, (1280, 1400));
        d.press(start.0, start.1);
        // длинный рывок вправо: курсор остаётся внутри пилюли, но её
        // накопленный сдвиг упирается в правый конец экрана
        let p = local(&world, (2600, 1400));
        d.motion(p.0, p.1, &world);
        let hint = d.hint().expect("упор в угол — кандидат обязан быть");
        assert_eq!(hint.edge, Edge::Right);
        // курсор при этом в зоне правой грани и сам: крест по его положению
        match d.release(p.0, p.1) {
            Release::Dock(target) => assert_eq!(target.edge, Edge::Right),
            other => panic!("ожидали закрепление справа, получили {other:?}"),
        }
    }

    /// Отпускание всегда возвращает машину в исходное состояние — регрессия
    /// на «залип в состоянии кружка» (dragged раньше сбрасывался только
    /// при следующем нажатии).
    #[test]
    fn release_always_resets_the_machine() {
        let world = bottom_world();
        let mut d = Drag::new();
        let start = local(&world, (1280, 1400));
        d.press(start.0, start.1);
        let p = local(&world, (1400, 1000));
        d.motion(p.0, p.1, &world);
        assert!(d.is_active());
        let _ = d.release(p.0, p.1);
        assert!(!d.is_active());
        assert!(!d.held());
        assert_eq!(d.cursor(), None);
        assert_eq!(d.hint(), None);
        // и следующее нажатие начинается с чистого листа
        d.press(5.0, 5.0);
        assert!(!d.is_active());
        assert_eq!(d.release(6.0, 6.0), Release::Click);
    }

    /// Экран неизвестен (композитор промолчал): перетаскивание живёт,
    /// но прилипание к граням не работает — только сдвиг вдоль.
    #[test]
    fn without_screen_there_is_no_docking_only_slide() {
        let world = World {
            screen: None,
            ..bottom_world()
        };
        let mut d = Drag::new();
        d.press(260.0, 14.0);
        d.motion(500.0, 14.0, &world);
        assert!(d.is_active());
        assert_eq!(d.hint(), None);
        match d.release(500.0, 14.0) {
            Release::Slide(px) => assert_eq!(px, 240),
            other => panic!("ожидали сдвиг, получили {other:?}"),
        }
    }

    /// Сценарий с левой грани вверх — то, что раньше не работало вовсе:
    /// курсор дошёл до верхней зоны, кандидат — верхняя грань.
    #[test]
    fn from_left_edge_to_top_works() {
        let world = World {
            placement: Placement::new(Edge::Left),
            window: (54, 520),
            screen: Some((2560, 1440)),
        };
        let mut d = Drag::new();
        // окно пилюли: x прижат к грани, центр по вертикали
        let start = local(&world, (26, 720));
        d.press(start.0, start.1);
        let mut abs_y = 720;
        while abs_y > 20 {
            abs_y -= 110;
            let p = local(&world, (26, abs_y));
            d.motion(p.0, p.1, &world);
        }
        let hint = d
            .hint()
            .expect("курсор у верхней грани — кандидат обязан быть");
        assert_eq!(hint.edge, Edge::Top);
        match d.release(local(&world, (26, 20)).0, local(&world, (26, 20)).1) {
            Release::Dock(p) => assert_eq!(p.edge, Edge::Top),
            other => panic!("ожидали закрепление сверху, получили {other:?}"),
        }
    }
}

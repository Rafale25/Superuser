fn point_aabb(pos: (i32, i32), rect_pos: (i32, i32), rect_size: (i32, i32)) -> bool {
    return rect_pos.0 < pos.0
        && pos.0 < rect_pos.0 + rect_size.0
        && rect_pos.1 < pos.1
        && pos.1 < rect_pos.1 + rect_size.1;
}

// #[derive(Default)]
// #[derive(Copy, Clone)]
#[derive(Default, Clone)]
pub struct Manual {
    pub pos: (i32, i32),
    pub size: (i32, i32),
    pub text: String,
    pub bg_color: (i32, i32, i32),
}

impl Manual {
    pub fn new(
        center: (i32, i32),
        size: (i32, i32),
        text: String,
        bg_color: (i32, i32, i32),
    ) -> Manual {
        let mut manual = Manual {
            pos: (0, 0),
            size,
            text,
            bg_color,
        };

        manual.set_center(center);

        return manual;
    }

    pub fn set_center(&mut self, pos: (i32, i32)) {
        self.pos = (pos.0 - self.size.0 / 2, pos.1 - self.size.1 / 2);
    }
}

#[derive(Default)]
pub struct ManualBoard {
    pub pos: (i32, i32),
    pub size: (i32, i32),
    pub manuals: Vec<Manual>,
    pub manual_printing: Vec<PrintAnimation>,
}

// pub struct PrintAnimation<'a> {
pub struct PrintAnimation {
    // pub manual: &'a Manual,
    pub manual: Manual,
    pub moving_timer: f32, // 400ms
    pub pause_timer: f32,  // 200ms
    pub is_moving: bool,   // bool
}

impl ManualBoard {
    pub fn new(window_width: i32, window_height: i32) -> ManualBoard {
        let pos: (i32, i32) = (window_width / 2, ((window_height as f32) * 0.25) as i32);
        let size: (i32, i32) = (window_width / 2, ((window_height as f32) * 0.75) as i32);

        // println!("{} {}", window_width, window_height);
        // println!("{} {}, {} {}", pos.0, pos.1, size.0, size.1);

        ManualBoard {
            pos,
            size,
            manuals: Vec::new(),
            manual_printing: Vec::new(),
        }
    }

    fn center(&self) -> (i32, i32) {
        (
            self.pos.0 + (self.size.0 / 2),
            self.pos.1 + (self.size.1 / 2),
        )
    }

    pub fn update_printer(&mut self, dt: f32) {
        if self.manual_printing.len() <= 0 {
            return;
        };

        let pm = &mut self.manual_printing[0];

        let board_y_bottom = self.pos.1 + self.size.1;

        if pm.manual.pos.1 + pm.manual.size.1 < board_y_bottom {
            self.manuals.insert(0, pm.manual.clone());
            self.manual_printing.remove(0);
            return;
        }

        if pm.is_moving {
            pm.moving_timer -= dt;
            pm.manual.pos.1 -= 2;
            if pm.moving_timer < 0.0 {
                pm.is_moving = false;
                pm.moving_timer = 0.3;
            }
        } else {
            pm.pause_timer -= dt;
            if pm.pause_timer < 0.0 {
                pm.pause_timer = 0.15;
                pm.is_moving = true;
            }
        }
    }

    pub fn print_manual(&mut self, mut manual: Manual) {
        let center = self.center();
        manual.pos = (center.0 - manual.size.0 / 2, self.pos.1 + self.size.1);

        let pa = PrintAnimation {
            manual,
            moving_timer: 0.3,
            pause_timer: 0.15,
            is_moving: true,
        };
        self.manual_printing.push(pa);
    }

    pub fn mouse_drag(&mut self, mouse_pos: (i32, i32), previous_mouse_pos: (i32, i32)) {
        if !point_aabb(mouse_pos, self.pos, self.size) {
            return;
        };

        // println!("{} {}", mouse_pos.0, mouse_pos.1);

        let mut is_dragged = false;

        // move element in front of the list (will be first to rendered)
        for (i, manual) in self.manuals.iter().enumerate() {
            if point_aabb(previous_mouse_pos, manual.pos, manual.size) {
                // self.manuals.swap(0, i);
                let m = self.manuals.remove(i);
                self.manuals.insert(0, m);
                is_dragged = true;
                break;
            }
        }

        if is_dragged {
            let dx = mouse_pos.0 - previous_mouse_pos.0;
            let dy = mouse_pos.1 - previous_mouse_pos.1;

            self.manuals[0].pos.0 += dx;
            self.manuals[0].pos.1 += dy;
            // self.manuals.last_mut().unwrap().pos.0 += dx;
            // self.manuals.last_mut().unwrap().pos.1 += dy;

            // TODO: add check so manual don't go too much outside of it reserved space
        }
    }
}

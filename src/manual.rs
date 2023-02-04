fn point_aabb(pos: (i32, i32), rect_pos: (i32, i32), rect_size: (i32, i32)) -> bool {
    return rect_pos.0 < pos.0
        && pos.0 < rect_pos.0 + rect_size.0
        && rect_pos.1 < pos.1
        && pos.1 < rect_pos.1 + rect_size.1;
}

#[derive(Default)]
pub struct Manual {
    pub pos: (i32, i32),
    pub size: (i32, i32),
    // images
}

impl Manual {
    pub fn new(center: (i32, i32), size: (i32, i32)) -> Manual {
        let mut manual = Manual { pos: (0, 0), size };

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
    pub manuals: [Manual; 1],
}

impl ManualBoard {
    pub fn new(window_width: i32, window_height: i32) -> ManualBoard {
        let pos: (i32, i32) = (window_width / 2, ((window_height as f32) * 0.25) as i32);
        let size: (i32, i32) = (window_width / 2, ((window_height as f32) * 0.75) as i32);

        ManualBoard {
            pos,
            size,
            manuals: [Manual::new(
                (pos.0 + size.0 / 2, pos.1 + size.1 / 2),
                (200, 400),
            )],
        }
    }
    pub fn center(&self) -> (i32, i32) {
        (
            self.pos.0 + (self.size.0 / 2),
            self.pos.1 + (self.size.1 / 2),
        )
    }

    pub fn mouse_drag(&mut self, mouse_pos: (i32, i32), previous_mouse_pos: (i32, i32)) {
        // move element in front of the list (will be first to rendered)
        let mut is_dragged = false;

        for (i, manual) in self.manuals.iter().enumerate() {
            if point_aabb(previous_mouse_pos, manual.pos, manual.size) {
                self.manuals.swap(0, i);
                is_dragged = true;
                break;
            }
        }

        if is_dragged {
            let dx = mouse_pos.0 - previous_mouse_pos.0;
            let dy = mouse_pos.1 - previous_mouse_pos.1;

            self.manuals[0].pos.0 += dx;
            self.manuals[0].pos.1 += dy;

            // TODO: add check so manual don't go too much outside of it reserved space
        }
    }
}

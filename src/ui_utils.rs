use sdl2::rect::Rect;

pub struct SimpleLayoutBuilder {
    start_x: i32,
    start_y: i32,
    mouse: Option<(u16, u16)>,
    max_height: i32,
    x: i32,
    y: i32,
}

impl SimpleLayoutBuilder {
    pub fn new(start: (i32, i32), mouse: Option<(u16, u16)>) -> Self {
        Self {
            start_x: start.0,
            start_y: start.1,
            mouse,
            max_height: 0,
            x: 0,
            y: 0,
        }
    }

    pub fn add_rect(&mut self, rect: Rect) -> (bool, Rect) {
        let width = rect.width() as i32;
        let height = rect.height() as i32;

        let x = self.start_x + self.x;
        let y = self.start_y + self.y;

        let hovered = if let Some((mx, my)) = self.mouse {
            let (mx, my) = (mx as i32, my as i32);
            x <= mx && mx < x + width && y <= my && my < y + height
        } else {
            false
        };

        let result = (hovered, Rect::new(x, y, rect.width(), rect.height()));

        self.max_height = i32::max(self.max_height, height);
        self.x += width;

        result
    }

    pub fn next_row(&mut self) {
        self.y += self.max_height;
        self.max_height = 0;
        self.x = 0;
    }
}

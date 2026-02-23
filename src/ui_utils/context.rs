use crate::*;
use sdl2::{
    ttf::Font,
    pixels::Color,
    render::SurfaceCanvas,
    mouse::MouseButton,
    rect::Rect,
};

pub struct UiContext<'a> {
    pub font: Font<'a, 'a>,
}

impl UiContext<'_> {
    pub fn add_button(
            &self,
            canvas: &mut SurfaceCanvas,
            layout: &mut super::SimpleLayoutBuilder,
            interact: Option<ModuleInteractInfo>,
            text: &str,
            width: Option<u32>) -> bool {

        let mut text = self.font.render(text).solid(Color::WHITE).unwrap();
        let (rect, shift) = if let Some(width) = width {
            let char_size = self.font.size_of_char('m').unwrap();
            let rect = Rect::new(
                0,
                0,
                char_size.0 * width,
                char_size.1
            );
            (rect, ((rect.width() - text.rect().width()) / 2) as i32)
        } else {
            (text.rect(), 0)
        };

        let (hovered, rect) = layout.add_rect(rect);

        let (pressed, clicked) = if let Some(info) = interact {
            let pressed = info.event_pump.mouse_state().left();
            let clicked = if let Some(button) = info.click {
                button == MouseButton::Left
            } else {
                false
            };
            (pressed, clicked)
        } else {
            (false, false)
        };

        let color = if hovered {
            if pressed {
                Color::WHITE
            } else {
                Color::GRAY
            }
        } else {
            Color::BLACK
        };

        text.set_color_mod(color);
        text.blit(rect.left_shifted(shift), canvas.surface_mut(), rect).unwrap();
        canvas.set_draw_color(color);
        canvas.draw_rect(rect).unwrap();
        hovered && clicked
    }
}

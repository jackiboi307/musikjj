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
    pub fn add_label(
            &self,
            canvas: &mut SurfaceCanvas,
            layout: &mut super::SimpleLayoutBuilder,
            text: &str,
            width: Option<u32>) {

        let width = if let Some(width) = width {
            width
        } else {
            text.chars().count() as u32
        };

        let text = self.font.render(text).solid(Color::BLACK).unwrap();
        let char_size = self.font.size_of_char('m').unwrap();
        let rect = Rect::new(
            0,
            0,
            char_size.0 * width,
            char_size.1
        );
        let (_, rect) = layout.add_rect(rect);

        text.blit(text.rect(), canvas.surface_mut(), rect).unwrap();
    }

    pub fn add_button(
            &self,
            canvas: &mut SurfaceCanvas,
            layout: &mut super::SimpleLayoutBuilder,
            interact: &Option<ModuleInteractInfo>,
            text: &str,
            width: Option<u32>) -> bool {

        const MARGIN: u8 = 5;

        let width = if let Some(width) = width {
            width
        } else {
            text.chars().count() as u32
        };

        let mut text = self.font.render(text).solid(Color::WHITE).unwrap();
        let char_size = self.font.size_of_char('m').unwrap();
        let rect = Rect::new(
            0,
            0,
            char_size.0 * width + MARGIN as u32 * 2,
            char_size.1
        );
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
                Color::RGB(140, 140, 140)
            } else {
                Color::RGB(90, 90, 90)
            }
        } else {
            Color::BLACK
        };

        text.set_color_mod(color);
        text.blit(text.rect(), canvas.surface_mut(), rect.right_shifted(MARGIN.into())).unwrap();
        canvas.set_draw_color(color);
        canvas.draw_rect(rect).unwrap();
        hovered && clicked
    }
}

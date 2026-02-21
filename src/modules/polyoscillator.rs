use crate::*;

pub struct PolyOscillator {
    oscillators: Vec<(Oscillator, bool)>,
}

impl PolyOscillator {
    pub fn new() -> Self {
        Self {
            oscillators: Vec::new(),
        }
    }

    pub fn set_oscillators(&mut self, amount: usize) {
        self.oscillators.clear();
        for _ in 0..amount {
            self.oscillators.push((Oscillator::new(), false));
        }
    }

    pub fn set_freqs(&mut self, freqs: Vec<f32>) {
        for (i, oscillator) in self.oscillators.iter_mut().enumerate() {
            if i < freqs.len() {
                oscillator.0.set_waveform(freqs[i]);
                oscillator.1 = true;
            } else {
                oscillator.1 = false;
            }
        }
    }
}

impl Module for PolyOscillator {
    fn tick(&mut self) -> Option<Data> {
        let mut value = 0.0;
        // let mut active = 0;
        for osc in self.oscillators.iter_mut() {
            if !osc.0.waveform.is_empty() && osc.1 {
                value += osc.0.tick().unwrap().audio();
                // active += 1;
            }
        }
        Some(Data::Audio(value))
    }

    define_module! {
        title: "PolyOscillator",
        output: Audio,
        inputs: [(Notes, "notes")],
    }

    fn send(&mut self, _input: usize, data: Data) {
        self.set_freqs(data.notes().iter().map(|note| note.freq()).collect())
    }

    // fn draw(&mut self, font: &sdl2::ttf::Font, interact: Option<ModuleInteractInfo>)
    //     -> Option<sdl2::surface::Surface<'_>> {

    //     use sdl2::{
    //         surface::Surface,
    //         pixels::{Color, PixelFormatEnum},
    //     };

    //     let (width, height) = (200, 200);

    //     let mut canvas =
    //         Surface::new(width, height, PixelFormatEnum::RGBA32)
    //         .unwrap().into_canvas().unwrap();

    //     let mouse_pos = interact.as_ref().and_then(|info| Some((info.x, info.y)));
    //     let mut layout = crate::ui_utils::SimpleLayoutBuilder::new((0, 0), mouse_pos);

    //     let mut text = font.render("test1").solid(Color::WHITE).unwrap();
    //     let (hovered, rect) = layout.add_rect(text.rect());
    //     if hovered {
    //         text.set_color_mod(Color::RED);
    //     } else {
    //         text.set_color_mod(Color::BLACK);
    //     }
    //     text.blit(text.rect(), canvas.surface_mut(), rect).unwrap();
    //     layout.next_row();

    //     let text = font.render(",test2").solid(Color::BLACK).unwrap();
    //     let (hovered, rect) = layout.add_rect(text.rect());
    //     text.blit(text.rect(), canvas.surface_mut(), rect).unwrap();

    //     if let Some(info) = interact {
    //         if info.click.is_some() && hovered {
    //             println!("clicked");
    //         }
    //     }

    //     Some(canvas.into_surface())
    // }
}

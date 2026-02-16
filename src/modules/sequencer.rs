use crate::*;
use std::time::{SystemTime, Duration};

macro_rules! sequence {
    ($root:expr, [$($note:expr $(,)?)*]) => {
        vec![$(
            vec![Note::Midi($root + $note)].into_boxed_slice(),
        )*]
    }
}

// fn create_power_chord(note: Note) -> [Note; 3] {
//     let freq = note.freq();
//     [
//         Note::Freq(freq),
//         Note::Freq(freq * 4.0 / 3.0),
//         Note::Freq(freq * 2.0),
//     ]
// }

pub struct Sequencer {
    pub sequence: Vec<Box<[Note]>>,
    step: usize,
    next_step: SystemTime,
    step_duration: Duration,
}

impl Sequencer {
    pub fn new() -> Self {
        let sequence = sequence!(
            ROOT, [0, 4, 7, 9, 6, 1, 8, 2]
        );

        Self {
            sequence,
            step: 0,
            next_step: SystemTime::UNIX_EPOCH,
            step_duration: Duration::from_secs_f32(60.0 / BPM / 4.0),
        }
    }
}

impl Module for Sequencer {
    fn tick(&mut self) -> Data {
        let now = SystemTime::now();

        if self.next_step <= now {
            // println!("{:?}", now.duration_since(self.next_step).unwrap());

            let length = self.sequence.len();
            if length <= self.step {
                self.step = 0;
            }

            let notes = &self.sequence[self.step];

            self.step = (self.step + 1) % length;
            self.next_step = now + self.step_duration;

            Data::Notes(notes.clone())

        } else {
            Data::Notes(Box::new([]))
        }
    }
    
    fn draw(&self, width: u32, height: u32, font: &sdl2::ttf::Font) -> Option<sdl2::surface::Surface<'_>> {
        use termabc::prelude::*;

        let mut canvas = sdl2::surface::Surface::new(width, height,
            sdl2::pixels::PixelFormatEnum::RGBA32).unwrap()
            .into_canvas().unwrap();
        let rect = canvas.surface().rect();

        let (font_width, font_height) = font.size_of_char('a').unwrap();

        let mut tui = InstructionBuffer::new(12, 12, None);
        tui.addstr(0, 0, "hej världen", Some(&Style::new().fg(Color::True(255, 0, 0))));
        tui.addstr(0, 1, "hej världen", Some(&Style::new().fg(Color::True(0, 255, 0))));
        tui.addstr(0, 2, "hej världen", Some(&Style::new().fg(Color::True(0, 0, 255))));

        for ((x, y), (ch, fg, bg)) in tui.render_to_chars().iter() {
            let rendered = font.render_char(*ch)
                .solid(sdl2::pixels::Color::RGB(fg.0, fg.1, fg.2))
                .unwrap();

            rendered.blit(rendered.rect(), canvas.surface_mut(), sdl2::rect::Rect::new(
                (font_width * *x as u32) as i32,
                (font_height * *y as u32) as i32,
                rect.width(),
                rendered.height()
            )).unwrap();

            if let Some(bg) = bg {
                canvas.set_draw_color(sdl2::pixels::Color::RGB(bg.0, bg.1, bg.2));
                canvas.fill_rect(rendered.rect()).unwrap();
            }
        }

        Some(canvas.into_surface())
    }

    define_module! {
        title: "Sequencer",
        output: Notes,
        inputs: [],
    }
}

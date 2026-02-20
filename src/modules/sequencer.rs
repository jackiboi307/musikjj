use crate::*;
use std::time::{SystemTime, Duration};

macro_rules! sequence {
    ($($note:expr $(,)?)*) => {
        vec![$(
            vec![$note],
        )*]
    }
}

pub struct Sequencer {
    pub sequence: Vec<Vec<u8>>,
    step: usize,
    next_step: SystemTime,
    step_duration: Duration,
}

impl Sequencer {
    pub fn new() -> Self {
        let sequence = sequence![0, 4, 7, 9, 6, 1, 8, 2];

        Self {
            sequence,
            step: 0,
            next_step: SystemTime::UNIX_EPOCH,
            step_duration: Duration::from_secs_f32(60.0 / BPM / 4.0),
        }
    }
}

impl Module for Sequencer {
    fn tick(&mut self) -> Option<Data> {
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

            // TODO remove transposition of 57 semitones
            Some(Data::Notes(notes.into_iter().map(|note| Note::Midi(57 + note)).collect()))

        } else {
            None
        }
    }

    define_module! {
        title: "Sequencer",
        output: Notes,
        inputs: [],
    }

    fn draw(&mut self, _font: &sdl2::ttf::Font, interact: Option<ModuleInteractInfo>)
        -> Option<sdl2::surface::Surface<'_>> {

        const SCALE_LEN: u16 = 12;

        use sdl2::{
            surface::Surface,
            pixels::{Color, PixelFormatEnum},
            mouse::MouseButton,
            rect::Rect,
        };

        let (width, height) = (400, 200);

        let mut canvas =
            Surface::new(width, height, PixelFormatEnum::RGBA32)
            .unwrap().into_canvas().unwrap();

        let note_width = width / self.sequence.len() as u32;
        let note_height = height / SCALE_LEN as u32;
        canvas.set_draw_color(Color::RGB(0, 0, 200));
        for (i, notes) in self.sequence.iter().enumerate() {
            for note in notes {
                let note = SCALE_LEN as i32 - 1 - *note as i32;
                canvas.fill_rect(Rect::new(
                    note_width as i32 * i as i32,
                    note_height as i32 * note,
                    note_width,
                    note_height
                )).unwrap();
            }
        }

        canvas.set_draw_color(Color::RGB(0, 0, 0));
        for x in 0..self.sequence.len() {
            for y in 0..SCALE_LEN {
                canvas.draw_rect(Rect::new(
                    note_width as i32 * x as i32,
                    note_height as i32 * y as i32,
                    note_width,
                    note_height
                )).unwrap();
            }
        }

        if let Some(info) = interact {
            if let Some(btn) = info.click {
                if btn == MouseButton::Left {
                    let i = (info.x as u32 / note_width) as usize;
                    let note = (SCALE_LEN as u8 - 1).checked_sub((info.y as u32 / note_height) as u8);
                    if let Some(note) = note {
                        if (note as u16) < SCALE_LEN {
                            if let Some(index) = self.sequence[i].iter()
                                    .position(|a| *a == note) {
                                self.sequence[i].remove(index);
                            } else {
                                self.sequence[i].push(note);
                            }
                        }
                    }
                }
            }
        }

        Some(canvas.into_surface())
    }
}

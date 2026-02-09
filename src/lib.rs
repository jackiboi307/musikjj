mod utils;
pub use utils::*;

mod components;
pub use components::*;

pub const ROOT: u8 = 12 * 3;
pub const BPM: f32 = 180.0;

pub trait AudioGenerator {
    fn tick(&mut self, sample_rate: f32) -> f32;
}

pub trait AudioProcessor {
    fn tick(&mut self, sample_rate: f32, value: f32) -> f32;
    fn step(&mut self) {}
}

pub trait NoteGenerator {
    fn note_tick(&mut self) -> Option<Box<[Note]>>;
}

// pub trait NoteProcessor {
//     fn tick(&mut self, notes: &[Note]) -> Option<Box<[Note]>>;
// }

#[derive(Clone, Copy)]
pub enum Note {
    Midi(u8),
    Freq(f32),
}

impl Note {
    pub fn freq(self) -> f32 {
        match self {
            Self::Midi(note) => midi_to_freq(note),
            Self::Freq(freq) => freq,
        }
    }
}

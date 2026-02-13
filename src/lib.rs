mod utils;
pub use utils::*;

mod components;
pub use components::*;

use std::sync::atomic::*;

pub static SAMPLE_RATE: AtomicU32 = AtomicU32::new(0);

pub fn set_sample_rate(sample_rate: u32) {
    SAMPLE_RATE.store(sample_rate, Ordering::Relaxed);
}

pub fn get_sample_rate() -> u32 {
    SAMPLE_RATE.load(Ordering::Relaxed)
}

pub const ROOT: u8 = 12 * 4;
pub const BPM: f32 = 180.0;

#[derive(Debug, Clone)]
pub enum DataType {
    Audio,
    Notes,
}

#[derive(Debug, Clone)]
pub enum Data {
    Audio(f32),
    Notes(Box<[Note]>),
}

#[derive(Debug, Clone, Copy)]
pub enum Note {
    Midi(u8),
    Freq(f32),
}

pub trait Module {
    fn get_output_type(&self) -> DataType;
    fn get_inputs(&self) -> Vec<(DataType, &'static str)>;
    fn tick(&mut self) -> Data;
    fn send(&mut self, _input: usize, _data: Data);
    fn as_any(&mut self) -> &mut dyn std::any::Any;
}

impl Data {
    fn audio(self) -> f32 {
        match self {
            Self::Audio(value) => value,
            _ => panic!()
        }
    }

    fn notes(self) -> Box<[Note]> {
        match self {
            Self::Notes(notes) => notes,
            _ => panic!()
        }
    }
}

impl Note {
    pub fn freq(self) -> f32 {
        match self {
            Self::Midi(note) => midi_to_freq(note),
            Self::Freq(freq) => freq,
        }
    }
}

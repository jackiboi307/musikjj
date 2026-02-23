use crate::*;
use std::f32::consts::TAU;

#[allow(dead_code)]
#[derive(Clone, Copy)]
pub enum Waveshape {
    Sine,
    Square,
    Saw,
}

pub struct Oscillator {
    pub waveform: Box<[f32]>,
    pub waveshape: Waveshape,
    index: usize,
}

impl Oscillator {
    pub fn new(waveshape: Waveshape) -> Self {
        Self {
            waveshape,
            waveform: vec![].into(),
            index: 0,
        }
    }

    pub fn set_waveform(&mut self, freq: f32) {
        self.waveform = self.get_waveform(freq);
        self.index = 0;
    }

    fn get_waveform(&self, freq: f32) -> Box<[f32]> {
        let mut waveform = Vec::new();
        let max = get_sample_rate() as usize / freq as usize;
        for i in 0..max {
            let i = i as f32 / max as f32;
            let value = match self.waveshape {
                Waveshape::Sine => self.calculate_sine(i),
                Waveshape::Square => self.calculate_square(i),
                Waveshape::Saw => self.calculate_saw(i),
            };

            waveform.push(value);
        }
        waveform.into()
    }

    fn calculate_sine(&self, index: f32) -> f32 {
        (index * TAU).sin()
    }

    fn calculate_square(&self, index: f32) -> f32 {
        if index < 0.5 { 1.0 } else { 0.0 }
    }

    fn calculate_saw(&self, index: f32) -> f32 {
        index
    }
}

impl Module for Oscillator {
    fn tick(&mut self) -> Option<Data> {
        Some(if !self.waveform.is_empty() {
            self.index = (self.index + 1) % self.waveform.len();
            Data::Audio(self.waveform[self.index])
        } else {
            Data::Audio(0.0)
        })
    }

    define_module! {
        title: "Oscillator",
        output: Audio,
        inputs: [(Notes, "note")],
    }

    fn send(&mut self, _input: usize, data: Data) {
        self.set_waveform(data.notes()[0].freq());
    }
}


impl Waveshape {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Sine => "sine",
            Self::Square => "square",
            Self::Saw => "saw",
        }
    }
}

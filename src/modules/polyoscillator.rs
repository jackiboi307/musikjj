use crate::*;

pub struct PolyOscillator {
    oscillators: Vec<Oscillator>,
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
            self.oscillators.push(Oscillator::new());
        }
    }

    pub fn set_freqs(&mut self, freqs: Vec<f32>) {
        for (i, oscillator) in self.oscillators.iter_mut().enumerate() {
            if i < freqs.len() {
                oscillator.set_waveform(freqs[i]);
            } else {
                break
            }
        }
    }
}

impl Module for PolyOscillator {
    fn tick(&mut self) -> Data {
        let mut value = 0.0;
        let mut active = 0;
        for osc in self.oscillators.iter_mut() {
            if !osc.waveform.is_empty() {
                value += osc.tick().audio();
                active += 1;
            }
        }
        Data::Audio(value / active as f32)
    }

    define_module! {
        title: "PolyOscillator",
        output: Audio,
        inputs: [(Notes, "notes")],
    }

    fn send(&mut self, _input: usize, data: Data) {
        self.set_freqs(data.notes().iter().map(|note| note.freq()).collect())
    }
}

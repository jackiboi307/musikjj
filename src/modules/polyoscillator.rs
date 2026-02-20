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
}

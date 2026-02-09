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

    pub fn set_freqs(&mut self, freqs: &[f32], sample_rate: f32) {
        for (i, oscillator) in self.oscillators.iter_mut().enumerate() {
            if i < freqs.len() {
                oscillator.set_waveform(freqs[i], sample_rate);
            } else {
                break
            }
        }
    }
}

impl AudioGenerator for PolyOscillator {
    fn tick(&mut self, sample_rate: f32) -> f32 {
        let mut value = 0.0;
        let mut active = 0;
        for osc in self.oscillators.iter_mut() {
            if !osc.waveform.is_empty() {
                value += osc.tick(sample_rate);
                active += 1;
            }
        }
        value / active as f32
    }
}


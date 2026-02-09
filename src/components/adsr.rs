use crate::*;

pub struct Adsr {
    index: f32,
    decay: f32,
}

impl Adsr {
    pub fn new() -> Self {
        Self {
            index: 0.0,
            decay: 0.1,
        }
    }
}

impl AudioProcessor for Adsr {
    fn tick(&mut self, sample_rate: f32, value: f32) -> f32 {
        self.index += 1.0 / self.decay / sample_rate;
        value * f32::max(0.0, 1.0 - self.index)
    }

    fn step(&mut self) {
        self.index = 0.0;
    }
}

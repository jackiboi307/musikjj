use crate::*;

pub struct Adsr {
    input: f32,
    index: f32,
    decay: f32,
}

impl Adsr {
    pub fn new() -> Self {
        Self {
            input: 0.0,
            index: 0.0,
            decay: 0.10,
        }
    }
}

impl Module for Adsr {
    fn tick(&mut self) -> Option<Data> {
        self.index += 1.0 / self.decay / get_sample_rate() as f32;
        Some(Data::Audio(self.input * f32::max(0.0, 1.0 - self.index)))
    }

    define_module! {
        title: "Adsr",
        output: Audio,
        inputs: [(Audio, "audio"), (Notes, "gate")],
    }

    fn send(&mut self, input: usize, data: Data) {
        match input {
            0 => { self.input = data.audio() }
            1 => {
                let notes = data.notes();
                if !notes.is_empty() {
                    self.index = 0.0;
                }
            }
            _ => panic!()
        }
    }
}

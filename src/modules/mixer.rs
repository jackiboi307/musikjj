use crate::*;

pub struct Mixer {
    values: Vec<f32>,
    input_count: usize,
}

impl Mixer {
    pub fn new(input_count: usize) -> Self {
        Self {
            values: Vec::new(),
            input_count,
        }
    }
}

impl Module for Mixer {
    fn tick(&mut self) -> Option<Data> {
        let data = Data::Audio(self.values.iter().sum::<f32>() / self.values.len() as f32);
        self.values.clear();
        Some(data)
    }

    define_module! {
        title: "Mixer",
        output: Audio,
    }

    fn get_inputs(&self) -> Vec<(DataType, &'static str)>
        { vec![(DataType::Audio, "input"); self.input_count] }

    fn send(&mut self, _input: usize, data: Data) {
        self.values.push(data.audio())
    }
}

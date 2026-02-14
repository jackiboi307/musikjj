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
    fn tick(&mut self) -> Data {
        let data = Data::Audio(self.values.iter().sum::<f32>() / self.values.len() as f32);
        self.values.clear();
        data
    }

    fn title(&self) -> &'static str { "Mixer" }
    fn get_output_type(&self) -> DataType { DataType::Audio }
    fn get_inputs(&self) -> Vec<(DataType, &'static str)>
        { vec![(DataType::Audio, "input"); self.input_count] }

    fn send(&mut self, _input: usize, data: Data) {
        self.values.push(data.audio())
    }

    fn as_any(&mut self) -> &mut dyn std::any::Any { self }
}

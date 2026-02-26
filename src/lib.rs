mod utils;
pub use utils::*;

mod modules;
pub use modules::*;

pub mod ui_utils;
pub use ui_utils::UiContext;

use std::sync::atomic::*;
use std::collections::HashMap;

pub static SAMPLE_RATE: AtomicU32 = AtomicU32::new(0);

pub fn set_sample_rate(sample_rate: u32) {
    SAMPLE_RATE.store(sample_rate, Ordering::Relaxed);
}

pub fn get_sample_rate() -> u32 {
    SAMPLE_RATE.load(Ordering::Relaxed)
}

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

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Note {
    Midi(u8),
    Freq(f32),
}

pub struct ModuleInteractInfo<'a> {
    pub x: u16,
    pub y: u16,
    pub click: Option<sdl2::mouse::MouseButton>,
    pub event_pump: &'a sdl2::EventPump,
}

type SerializeableData = HashMap<String, serde_json::Value>;

pub trait Module {
    fn title(&self) -> &'static str;
    fn get_output_type(&self) -> DataType;
    fn get_inputs(&self) -> Vec<(DataType, &'static str)>;
    fn tick(&mut self) -> Option<Data>;
    fn send(&mut self, _input: usize, _data: Data) {}
    fn as_any(&mut self) -> &mut dyn std::any::Any;
    fn draw(&mut self, _ui: &UiContext<'_>, _interact: Option<ModuleInteractInfo>)
        -> Option<sdl2::surface::Surface<'_>> { None }
    fn execute(&self, _cmd: String) {
        println!("Module::execute is not implemented for: {}", self.title());
    }
    fn get_data(&self) -> SerializeableData { todo!() }
    fn load_data(&self, _data: SerializeableData) { todo!() }
}

#[macro_export]
macro_rules! define_module {
    (
        $(title: $title:expr,)?
        $(output: $output_type:ident,)?
        $(inputs: [$(($input_type:ident, $input_label:expr)$(,)?)*],)?
    ) => {
        $(fn title(&self) -> &'static str { $title })?
        $(fn get_output_type(&self) -> DataType { DataType::$output_type })?
        $(fn get_inputs(&self) -> Vec<(DataType, &'static str)>
            { vec![$((DataType::$input_type, $input_label),)*] })?
        fn as_any(&mut self) -> &mut dyn std::any::Any { self }
    }
}

impl Data {
    fn audio(self) -> f32 {
        match self {
            Self::Audio(value) => value,
            _ => 0.0
        }
    }

    fn notes(self) -> Box<[Note]> {
        match self {
            Self::Notes(notes) => notes,
            _ => Box::new([])
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

    pub fn transpose(self, amount: i16) -> Self {
        match self {
            // TODO fix this math
            Self::Midi(note) => Self::Midi((note as i16 + amount) as u8),
            Self::Freq(freq) => Self::Freq(freq + amount.signum() as f32 * midi_to_freq(amount.abs() as u8)),
        }
    }
}

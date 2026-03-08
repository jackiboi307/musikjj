mod oscillator;
pub use oscillator::{Oscillator, Waveshape};

mod polyoscillator;
pub use polyoscillator::PolyOscillator;

mod adsr;
pub use adsr::Adsr;

mod sequencer;
pub use sequencer::Sequencer;

mod mixer;
pub use mixer::Mixer;

mod transpose;
pub use transpose::Transpose;

use crate::*;

macro_rules! define_module_from_id {
    ($($module:ident,)*) => {
        pub fn module_from_id(id: &str) -> Option<Box<dyn Module + Send>> {
            $(
                let module = $module::new();
                if id == module.id() {
                    return Some(Box::new(module))
                }
            )*
            todo!()
        }
    }
}

define_module_from_id! {
    PolyOscillator,
    Adsr,
    Sequencer,
    Mixer,
    Transpose,
}

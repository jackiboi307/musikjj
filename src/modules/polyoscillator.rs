use crate::*;
use super::oscillator::Waveshape;

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
            self.oscillators.push((Oscillator::new(Waveshape::Sine), false));
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

    fn current_waveshape(&self) -> Waveshape {
        // assume the same waveshape is used for all oscillators
        self.oscillators[0].0.waveshape
    }

    fn cycle_waveshape(&mut self) {
        use Waveshape::*;
        let new = match self.current_waveshape() {
            Sine => Square,
            Square => Saw,
            Saw => Sine,
        };
        for (oscillator, _) in self.oscillators.iter_mut() {
            oscillator.waveshape = new;
        }
    }
}

impl Module for PolyOscillator {
    fn tick(&mut self) -> Option<Data> {
        let mut value = 0.0;
        let mut active = 0;
        for osc in self.oscillators.iter_mut() {
            if !osc.0.waveform.is_empty() && osc.1 {
                value += osc.0.tick().unwrap().audio();
                active += 1;
            }
        }
        Some(Data::Audio(if active != 0 {
            value / active as f32
        } else {
            0.0
        }))
    }

    define_module! {
        title: "PolyOscillator",
        output: Audio,
        inputs: [(Notes, "notes")],
    }

    fn send(&mut self, _input: usize, data: Data) {
        self.set_freqs(data.notes().iter().map(|note| note.freq()).collect())
    }

    fn draw(&mut self, ui: &UiContext, interact: Option<ModuleInteractInfo>)
        -> Option<sdl2::surface::Surface<'_>> {

        use sdl2::{
            surface::Surface,
            pixels::PixelFormatEnum,
        };

        let (width, height) = (200, 200);

        let mut canvas =
            Surface::new(width, height, PixelFormatEnum::RGBA32)
            .unwrap().into_canvas().unwrap();

        let mouse_pos = interact.as_ref().and_then(|info| Some((info.x, info.y)));
        let mut layout = crate::ui_utils::SimpleLayoutBuilder::new((0, 0), mouse_pos);

        if ui.add_button(&mut canvas, &mut layout, interact, self.current_waveshape().as_str(), Some(6)) {
            self.cycle_waveshape();
        }

        Some(canvas.into_surface())
    }
}

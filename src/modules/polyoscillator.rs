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

    fn add_oscillator(&mut self, waveshape: Waveshape) {
        self.oscillators.push((Oscillator::new(waveshape), false));
    }

    pub fn set_oscillators(&mut self, waveshape: Waveshape, amount: usize) {
        self.oscillators.clear();
        for _ in 0..amount {
            self.add_oscillator(waveshape);
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

        let (width, height) = (250, 50);

        let mut canvas =
            Surface::new(width, height, PixelFormatEnum::RGBA32)
            .unwrap().into_canvas().unwrap();

        let mouse_pos = interact.as_ref().and_then(|info| Some((info.x, info.y)));
        let mut layout = crate::ui_utils::SimpleLayoutBuilder::new((0, 0), mouse_pos);

        ui.add_label(&mut canvas, &mut layout, self.current_waveshape().as_str(), Some(6));
        if ui.add_button(&mut canvas, &mut layout, &interact, "cycle waveshape", None) {
            self.cycle_waveshape();
        }

        layout.next_row();

        ui.add_label(&mut canvas, &mut layout, &*format!("oscs: {}", self.oscillators.len()), Some(8));
        if ui.add_button(&mut canvas, &mut layout, &interact, "+", None) {
            self.add_oscillator(self.current_waveshape());
        }
        if ui.add_button(&mut canvas, &mut layout, &interact, "-", None) {
            if 1 < self.oscillators.len() {
                self.oscillators.pop();
            }
        }

        Some(canvas.into_surface())
    }
}

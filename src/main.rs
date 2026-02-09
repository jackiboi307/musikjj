use cpal::{
    FromSample, Sample,
    traits::{DeviceTrait, HostTrait, StreamTrait},
    SizedSample, I24, U24,
};

use std::{
    io::{self, Write},
    sync::{Arc, Mutex},
    time::{SystemTime, Duration},
    f32::consts::TAU,
};

const ROOT: u8 = 45;

fn main() -> anyhow::Result<()> {
    let app = Arc::new(Mutex::new(App::new()));
    let stream = stream_setup_for(Arc::clone(&app))?;
    println!("Playing...");
    stream.play()?;

    println!("Enter notes (example: '0 4 7 4' for a major arp)");
    loop {
        let notes = read_line(">>> ")?;
        let notes: Vec<f32> = notes
            .split_ascii_whitespace()
            .map(|note| midi_to_freq(ROOT + note.trim().parse::<u8>().unwrap()))
            .collect();
        let mut app = app.lock().unwrap();
        app.sequence = notes;
    }
}

fn read_line(prompt: &str) -> io::Result<String> {
    let mut buffer = String::new();
    let stdin = io::stdin();
    print!("{prompt}");
    let _ = io::stdout().flush();
    stdin.read_line(&mut buffer)?;
    Ok(buffer)
}

macro_rules! sequence {
    ($root:expr, [$($note:expr $(,)?)*]) => {
        vec![$(
            midi_to_freq($root + $note),
        )*]
    }
}

trait Generator {
    fn tick(&mut self, sample_rate: f32) -> f32;
}

trait Processor {
    fn tick(&mut self, sample_rate: f32, value: f32) -> f32;
    fn step(&mut self) {}
}

struct App {
    sequence: Vec<f32>,
    step: usize,
    generator: PolyOscillator,
    processors: Vec<Box<dyn Processor + Send>>,
}

impl App {
    fn new() -> Self {
        let sequence = sequence!(
            ROOT, [0, 4, 7, 9, 6, 1, 8, 2]
        );

        let mut oscillator = PolyOscillator::new();
        oscillator.set_oscillators(3);

        Self {
            sequence,
            step: 0,
            generator: oscillator,
            processors: vec![Box::new(Adsr::new())],
        }
    }

    fn step(&mut self, sample_rate: f32) {
        let length = self.sequence.len();
        if length <= self.step {
            self.step = length - 1;
        }

        let notes = create_power_chord(self.sequence[self.step]);
        self.generator.set_freqs(&notes, sample_rate);
        self.step = (self.step + 1) % length;

        for processor in self.processors.iter_mut() {
            processor.step();
        }
    }
}

impl Generator for App {
    fn tick(&mut self, sample_rate: f32) -> f32 {
        let mut value = self.generator.tick(sample_rate);
        for processor in self.processors.iter_mut() {
            value = processor.tick(sample_rate, value);
        }
        value
    }
}

#[allow(dead_code)]
enum Waveshape {
    Sine,
    Square,
    Saw,
}

struct Oscillator {
    waveshape: Waveshape,
    waveform: Box<[f32]>,
    index: usize,
}

impl Oscillator {
    fn new() -> Self {
        Self {
            waveshape: Waveshape::Square,
            waveform: vec![].into(),
            index: 0,
        }
    }

    fn get_waveform(&self, freq: f32, sample_rate: f32) -> Box<[f32]> {
        let mut waveform = Vec::new();
        let max = sample_rate as usize / freq as usize;
        for i in 0..max {
            let i = i as f32 / max as f32;
            let value = match self.waveshape {
                Waveshape::Sine => self.calculate_sine(i),
                Waveshape::Square => self.calculate_square(i),
                Waveshape::Saw => self.calculate_saw(i),
            };

            waveform.push(value);
        }
        waveform.into()
    }

    fn set_waveform(&mut self, freq: f32, sample_rate: f32) {
        self.waveform = self.get_waveform(freq, sample_rate);
        self.index = 0;
    }

    fn calculate_sine(&self, index: f32) -> f32 {
        (index * TAU).sin()
    }

    fn calculate_square(&self, index: f32) -> f32 {
        if index < 0.5 { 1.0 } else { 0.0 }
    }

    fn calculate_saw(&self, index: f32) -> f32 {
        index
    }

    // fn calculate_tan(&self, index: f32) -> f32 {
    //     f32::max(-1.0, f32::min(1.0, (index * TAU).tan()))
    // }
}

impl Generator for Oscillator {
    fn tick(&mut self, _sample_rate: f32) -> f32 {
        if !self.waveform.is_empty() {
            self.index = (self.index + 1) % self.waveform.len();
            self.waveform[self.index]
        } else {
            0.0
        }
    }
}

struct PolyOscillator {
    oscillators: Vec<Oscillator>,
}

impl PolyOscillator {
    fn new() -> Self {
        Self {
            oscillators: Vec::new(),
        }
    }

    fn set_oscillators(&mut self, amount: usize) {
        self.oscillators.clear();
        for _ in 0..amount {
            self.oscillators.push(Oscillator::new());
        }
    }

    fn set_freqs(&mut self, freqs: &[f32], sample_rate: f32) {
        for (i, oscillator) in self.oscillators.iter_mut().enumerate() {
            if i < freqs.len() {
                oscillator.set_waveform(freqs[i], sample_rate);
            } else {
                break
            }
        }
    }
}

impl Generator for PolyOscillator {
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

struct Adsr {
    index: f32,
    decay: f32,
}

impl Adsr {
    fn new() -> Self {
        Self {
            index: 0.0,
            decay: 0.1,
        }
    }
}

impl Processor for Adsr {
    fn tick(&mut self, sample_rate: f32, value: f32) -> f32 {
        self.index += 1.0 / self.decay / sample_rate;
        value * f32::max(0.0, 1.0 - self.index)
    }

    fn step(&mut self) {
        self.index = 0.0;
    }
}

fn stream_setup_for(app: Arc<Mutex<App>>) -> Result<cpal::Stream, anyhow::Error> {
    let (_host, device, config) = host_device_setup()?;

    match config.sample_format() {
        cpal::SampleFormat::I8  => make_stream::<i8> (app, &device, &config.into()),
        cpal::SampleFormat::I16 => make_stream::<i16>(app, &device, &config.into()),
        cpal::SampleFormat::I24 => make_stream::<I24>(app, &device, &config.into()),
        cpal::SampleFormat::I32 => make_stream::<i32>(app, &device, &config.into()),
        cpal::SampleFormat::I64 => make_stream::<i64>(app, &device, &config.into()),
        cpal::SampleFormat::U8  => make_stream::<u8> (app, &device, &config.into()),
        cpal::SampleFormat::U16 => make_stream::<u16>(app, &device, &config.into()),
        cpal::SampleFormat::U24 => make_stream::<U24>(app, &device, &config.into()),
        cpal::SampleFormat::U32 => make_stream::<u32>(app, &device, &config.into()),
        cpal::SampleFormat::U64 => make_stream::<u64>(app, &device, &config.into()),
        cpal::SampleFormat::F32 => make_stream::<f32>(app, &device, &config.into()),
        cpal::SampleFormat::F64 => make_stream::<f64>(app, &device, &config.into()),
        sample_format => Err(anyhow::Error::msg(format!(
            "Unsupported sample format '{sample_format}'"
        ))),
    }
}

fn host_device_setup()
    -> Result<(cpal::Host, cpal::Device, cpal::SupportedStreamConfig),
    anyhow::Error> {

    let host = cpal::default_host();
    let devices: Vec<_> = host.output_devices()?.collect();
    println!("Devices:");
    for (i, device) in devices.iter().enumerate() {
        println!("\t{i}: {}: {}", device.id()?, device.description()?);
    }

    let input = read_line("Select device > ")?;
    let device: cpal::Device = devices[input.trim_end().parse::<usize>().unwrap()].clone();

    println!("Output device: {}", device.id()?);

    let config = device.default_output_config()?;
    println!("Default output config: {config:#?}");

    Ok((host, device, config))
}

fn create_power_chord(freq: f32) -> [f32; 3] {
    [freq, freq * 4.0 / 3.0, freq * 2.0]
}

fn midi_to_freq(note: u8) -> f32 {
    440.0 * 2.0_f32.powf((note as f32 - 69.0) / 12.0)
}

fn make_stream<T>(
        app: Arc<Mutex<App>>,
        device: &cpal::Device,
        config: &cpal::StreamConfig,
    ) -> Result<cpal::Stream, anyhow::Error>
    where T: SizedSample + FromSample<f32> {

    let num_channels = config.channels as usize;
    let sample_rate = config.sample_rate as f32;

    let err_fn = |err| eprintln!(
        "Error building output sound stream: {err}");

    let mut last_step_time = SystemTime::UNIX_EPOCH;
    let step_duration = Duration::from_millis(80);

    let stream = device.build_output_stream(
        config,
        move |output: &mut [T], _: &cpal::OutputCallbackInfo| {
            {
                let mut app = app.lock().unwrap();

                let now = SystemTime::now();
                let elapsed = now
                    .duration_since(last_step_time)
                    .unwrap();

                if elapsed >= step_duration {
                    app.step(sample_rate);
                    last_step_time = now;
                }
            }

            process_frame(output, &app, num_channels, sample_rate)
        },
        err_fn,
        None,
    )?;

    Ok(stream)
}

fn process_frame<SampleType>(
        output: &mut [SampleType],
        app: &Arc<Mutex<App>>,
        num_channels: usize,
        sample_rate: f32,
    ) where SampleType: Sample + FromSample<f32> {

    for frame in output.chunks_mut(num_channels) {
        let value: SampleType = {
            let mut app = app.lock().unwrap();
            SampleType::from_sample(app.tick(sample_rate))
        };

        // copy the same value to all channels
        for sample in frame.iter_mut() {
            *sample = value;
        }
    }
}

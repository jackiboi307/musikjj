use cpal::{
    FromSample, Sample,
    traits::{DeviceTrait, HostTrait, StreamTrait},
    SizedSample, I24, U24,
};

use std::io::{self, Write};
use std::time::{SystemTime, Duration};
use std::f32::consts::TAU;

fn main() -> anyhow::Result<()> {
    let stream = stream_setup_for()?;
    println!("Playing...");
    stream.play()?;
    loop {}
}

trait Oscillator {
    fn tick(&mut self) -> f32;
}

struct BasicOscillator {
    sample_rate: f32,
    waveform: Box<[f32]>,
    index: usize,
}

impl BasicOscillator {
    fn new(sample_rate: f32) -> Self {
        Self {
            sample_rate,
            waveform: vec![].into(),
            index: 0,
        }
    }

    fn get_waveform(&self, freq: f32) -> Box<[f32]> {
        let mut waveform = Vec::new();
        let max = self.sample_rate as usize / freq as usize;
        for i in 0..max {
            waveform.push(
                self.calculate_sine(i as f32 / max as f32));
        }
        waveform.into()
    }

    fn set_waveform(&mut self, freq: f32) {
        self.waveform = self.get_waveform(freq);
        self.index = 0;
    }

    fn calculate_sine(&self, index: f32) -> f32 {
        (index * TAU).sin()
    }

    fn calculate_tan(&self, index: f32) -> f32 {
        f32::max(-1.0, f32::min(1.0, (index * TAU).tan()))
    }

    fn calculate_square(&self, index: f32) -> f32 {
        if index < 0.5 { 1.0 } else { 0.0 }
    }

    fn calculate_saw(&self, index: f32) -> f32 {
        index
    }
}

impl Oscillator for BasicOscillator {
    fn tick(&mut self) -> f32 {
        self.index = (self.index + 1) % self.waveform.len();
        self.waveform[self.index]
    }
}

struct PolyOscillator {
    oscillators: Vec<BasicOscillator>,
}

impl PolyOscillator {
    fn new(rate: f32, amount: usize) -> Self {
        let mut oscillators = Vec::new();
        for _ in 0..amount {
            oscillators.push(BasicOscillator::new(rate));
        }

        Self {
            oscillators,
        }
    }

    fn set_freqs(&mut self, freqs: &[f32]) {
        for (i, oscillator) in self.oscillators.iter_mut().enumerate() {
            oscillator.set_waveform(freqs[i])
        }
    }
}

impl Oscillator for PolyOscillator {
    fn tick(&mut self) -> f32 {
        let mut value = 0.0;
        for osc in self.oscillators.iter_mut() {
            value += osc.tick();
        }
        value / self.oscillators.len() as f32
    }
}

fn stream_setup_for() -> Result<cpal::Stream, anyhow::Error> {
    let (_host, device, config) = host_device_setup()?;

    match config.sample_format() {
        cpal::SampleFormat::I8  => make_stream::<i8> (&device, &config.into()),
        cpal::SampleFormat::I16 => make_stream::<i16>(&device, &config.into()),
        cpal::SampleFormat::I24 => make_stream::<I24>(&device, &config.into()),
        cpal::SampleFormat::I32 => make_stream::<i32>(&device, &config.into()),
        cpal::SampleFormat::I64 => make_stream::<i64>(&device, &config.into()),
        cpal::SampleFormat::U8  => make_stream::<u8> (&device, &config.into()),
        cpal::SampleFormat::U16 => make_stream::<u16>(&device, &config.into()),
        cpal::SampleFormat::U24 => make_stream::<U24>(&device, &config.into()),
        cpal::SampleFormat::U32 => make_stream::<u32>(&device, &config.into()),
        cpal::SampleFormat::U64 => make_stream::<u64>(&device, &config.into()),
        cpal::SampleFormat::F32 => make_stream::<f32>(&device, &config.into()),
        cpal::SampleFormat::F64 => make_stream::<f64>(&device, &config.into()),
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

    let mut buffer = String::new();
    let stdin = io::stdin();
    print!("Select device > ");
    let _ = io::stdout().flush();
    stdin.read_line(&mut buffer)?;
    let device: cpal::Device = devices[buffer.trim_end().parse::<usize>().unwrap()].clone();

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
        device: &cpal::Device,
        config: &cpal::StreamConfig,
    ) -> Result<cpal::Stream, anyhow::Error>
    where T: SizedSample + FromSample<f32> {

    let num_channels = config.channels as usize;
    let err_fn = |err| eprintln!(
        "Error building output sound stream: {err}");

    let root = 57;
    let sequence = [
        midi_to_freq(root),
        midi_to_freq(root + 4),
        midi_to_freq(root + 7),
        midi_to_freq(root + 9),
        midi_to_freq(root + 6),
        midi_to_freq(root + 1),
        midi_to_freq(root + 8),
        midi_to_freq(root + 2),
    ];

    let rate = config.sample_rate as f32;
    let mut oscillator = PolyOscillator::new(rate, 3);

    let length = sequence.len();
    let mut step = 0;
    let mut last_step_time = SystemTime::UNIX_EPOCH;
    let step_duration = Duration::from_millis(100);

    let stream = device.build_output_stream(
        config,
        move |output: &mut [T], _: &cpal::OutputCallbackInfo| {
            let now = SystemTime::now();
            let elapsed = now
                .duration_since(last_step_time)
                .unwrap();

            if elapsed >= step_duration {
                oscillator.set_freqs(&create_power_chord(sequence[step]));
                step = (step + 1) % length;

                let lag = elapsed - step_duration;
                let lag = if lag < step_duration {
                    lag } else { Duration::ZERO };

                last_step_time = now - lag;
            }

            process_frame(output, &mut oscillator, num_channels)
        },
        err_fn,
        None,
    )?;

    Ok(stream)
}

fn process_frame<SampleType>(
        output: &mut [SampleType],
        oscillator: &mut impl Oscillator,
        num_channels: usize,
    ) where SampleType: Sample + FromSample<f32> {

    for frame in output.chunks_mut(num_channels) {
        let value: SampleType = SampleType::from_sample(oscillator.tick());

        // copy the same value to all channels
        for sample in frame.iter_mut() {
            *sample = value;
        }
    }
}

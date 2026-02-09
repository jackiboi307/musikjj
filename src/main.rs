use cpal::{
    FromSample, Sample,
    traits::{DeviceTrait, HostTrait, StreamTrait},
    SizedSample, I24, U24,
};

use std::{
    io::{self, Write},
    sync::{Arc, Mutex},
};

use musikjj::*;

fn main() -> anyhow::Result<()> {
    let app = Arc::new(Mutex::new(App::new()));
    let stream = stream_setup_for(Arc::clone(&app))?;
    println!("Playing...");
    stream.play()?;

    println!("Enter notes (example: '0 4 7 4' for a major arp)");
    loop {
        let notes = read_line(">>> ")?;
        let notes: Vec<Note> = notes
            .split_ascii_whitespace()
            .map(|note| Note::Midi(ROOT + note.trim().parse::<u8>().unwrap()))
            .collect();
        let mut app = app.lock().unwrap();
        app.sequencer.sequence = notes;
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

struct App {
    sample_rate: f32,
    sequencer: Sequencer,
    generator: PolyOscillator,
    processors: Vec<Box<dyn AudioProcessor + Send>>,
}

impl App {
    fn new() -> Self {
        let mut oscillator = PolyOscillator::new();
        oscillator.set_oscillators(3);

        Self {
            sample_rate: 0.0,
            sequencer: Sequencer::new(),
            generator: oscillator,
            processors: vec![Box::new(Adsr::new())],
        }
    }

    fn step(&mut self) {
        for processor in self.processors.iter_mut() {
            processor.step();
        }
    }

    fn get_sample(&mut self) -> f32 {
        let mut value = self.generator.tick(self.sample_rate);
        for processor in self.processors.iter_mut() {
            value = processor.tick(self.sample_rate, value);
        }
        value
    }

    fn tick(&mut self) {
        if let Some(notes) = self.sequencer.note_tick() {
            let notes: Vec<_> = notes.iter().map(|note| note.clone().freq()).collect();
            self.generator.set_freqs(&*notes, self.sample_rate);
            self.step();
        }
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

fn make_stream<T>(
        app: Arc<Mutex<App>>,
        device: &cpal::Device,
        config: &cpal::StreamConfig,
    ) -> Result<cpal::Stream, anyhow::Error>
    where T: SizedSample + FromSample<f32> {

    let num_channels = config.channels as usize;

    {
        let mut app = app.lock().unwrap();
        app.sample_rate = config.sample_rate as f32;
    }

    let err_fn = |err| eprintln!(
        "Error building output sound stream: {err}");

    let stream = device.build_output_stream(
        config,
        move |output: &mut [T], _: &cpal::OutputCallbackInfo| {
            {
                let mut app = app.lock().unwrap();
                app.tick();

                // let now = SystemTime::now();
                // let elapsed = now
                //     .duration_since(last_step_time)
                //     .unwrap();

                // if elapsed >= step_duration {
                //     app.step();
                //     last_step_time = now;
                // }
            }

            process_frame(output, &app, num_channels)
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
    ) where SampleType: Sample + FromSample<f32> {

    for frame in output.chunks_mut(num_channels) {
        let value: SampleType = {
            let mut app = app.lock().unwrap();
            SampleType::from_sample(app.get_sample())
        };

        // copy the same value to all channels
        for sample in frame.iter_mut() {
            *sample = value;
        }
    }
}

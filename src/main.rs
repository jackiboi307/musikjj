mod gui;
use musikjj::*;

use cpal::{
    FromSample, Sample,
    traits::{DeviceTrait, HostTrait, StreamTrait},
    SizedSample, I24, U24,
};

use std::{
    io::{self, Write},
    sync::{Arc, Mutex},
    collections::HashMap,
};

fn main() -> anyhow::Result<()> {
    let app = Arc::new(Mutex::new(App::new()));

    // {
    //     let mut app = app.lock().unwrap();
    //     app.init();
    // }

    let stream = stream_setup_for(Arc::clone(&app))?;
    println!("Playing...");
    stream.play()?;

    let mut gui = gui::Gui::new();
    gui.run(Arc::clone(&app));
    Ok(())
}

#[allow(dead_code)]
fn read_line(prompt: &str) -> io::Result<String> {
    let mut buffer = String::new();
    let stdin = io::stdin();
    print!("{prompt}");
    let _ = io::stdout().flush();
    stdin.read_line(&mut buffer)?;
    Ok(buffer)
}

type ModuleId = u16;

struct App {
    modules: HashMap<ModuleId, Box<dyn Module + Send>>,
    conns: HashMap<(ModuleId, usize), ModuleId>,
    cached: HashMap<ModuleId, Data>,
    next_id: ModuleId,
}

impl App {
    fn new() -> Self {
        Self {
            modules: HashMap::new(),
            conns: HashMap::new(),
            cached: HashMap::new(),
            next_id: 1,
        }
    }

    fn init(&mut self) {
        let mut osc = PolyOscillator::new();
        osc.set_oscillators(3);

        self.insert_module(Box::new(osc));
        self.insert_module(Box::new(Sequencer::new()));
        self.insert_module(Box::new(Adsr::new()));

        // self.connect(seq, (osc, 0));
        // self.connect(seq, (adsr, 1));
        // self.connect(osc, (adsr, 0));
        // self.connect(adsr, (0, 0));
    }

    fn module(&mut self, module: ModuleId) -> &mut Box<dyn Module + Send> {
        self.modules.get_mut(&module).unwrap()
    }

    fn insert_module(&mut self, module: Box<dyn Module + Send>) -> ModuleId {
        self.modules.insert(self.next_id, module);
        self.next_id += 1;
        self.next_id - 1
    }

    fn connect(&mut self, output: ModuleId, input: (ModuleId, usize)) {
        if let Some(existing_out) = self.conns.get(&input) {
            if output == *existing_out {
                self.conns.remove(&input);
                return
            }
        }

        self.conns.insert(input, output);
    }

    fn get_output(&mut self, id: ModuleId) -> Data {
        // TODO do not clone?

        if let Some(data) = self.cached.get(&id) {
            return data.clone()
        }

        let data = if id != 0 {
            let inputs = self.modules[&id].get_inputs();
            for input_index in 0..inputs.len() {
                if let Some(input_id) = self.conns.get(&(id, input_index)) {
                    let data = self.get_output(*input_id);
                    self.module(id).send(input_index, data);
                }
            }
            self.module(id).tick()

        } else {
            if let Some(input_id) = self.conns.get(&(id, 0)) {
                self.get_output(*input_id)
            } else {
                if id == 0 {
                    Data::Audio(0.0)
                } else {
                    panic!()
                }
            }
        };

        self.cached.insert(id, data.clone());
        data
    }

    fn tick(&mut self) -> f32 {
        self.cached.clear();
        match self.get_output(0) {
            Data::Audio(audio) => audio,
            _ => 0.0
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

    // default to id 6 (pulseaudio)
    let device = devices[6].clone();

    // println!("Devices:");
    // for (i, device) in devices.iter().enumerate() {
    //     println!("\t{i}: {}: {}", device.id()?, device.description()?);
    // }

    // let input = read_line("Select device > ")?;
    // let device: cpal::Device = devices[input.trim_end().parse::<usize>().unwrap()].clone();

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
    set_sample_rate(config.sample_rate);

    {
        let mut app = app.lock().unwrap();
        app.init();
    }

    let err_fn = |err| eprintln!(
        "Error building output sound stream: {err}");

    let stream = device.build_output_stream(
        config,
        move |output: &mut [T], _: &cpal::OutputCallbackInfo| {
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
            SampleType::from_sample(app.tick() * 1.0)
        };

        for sample in frame.iter_mut() {
            *sample = value;
        }
    }
}

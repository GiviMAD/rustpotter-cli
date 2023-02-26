use std::fs::File;
use std::io::BufWriter;
use std::sync::{mpsc, Arc, Mutex};

use clap::Args;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, Sample};
#[derive(Args, Debug)]
/// Record wav audio with spec 16000hz 16bit 1 channel int. Press "Ctrl + c" to stop and save.
#[clap()]
pub struct RecordCommand {
    #[clap()]
    /// Generated record path
    output_path: String,
    #[clap(short = 'i', long)]
    /// Input device index used for record
    device_index: Option<usize>,
    #[clap(short, long)]
    /// Input device index used for record
    config_index: Option<usize>,
}
pub fn record(command: RecordCommand) -> Result<(), String> {
    //get the host
    let host = cpal::default_host();

    //get the default input device
    // Set up the input device and stream with the default input config.
    let device = get_device(command.device_index, host);

    //get default config - channels, sample_rate,buffer_size, sample_format
    println!(
        "Input device: {}",
        device.name().map_err(|err| err.to_string())?
    );
    let config = get_config(command.config_index, &device);
    println!("Input device config: {:?}", config);
    let spec = wav_spec_from_config(&config);
    let writer = hound::WavWriter::create(command.output_path.to_string(), spec).unwrap();
    let writer = Arc::new(Mutex::new(Some(writer)));
    println!("Begin recording...");
    // Run the input stream on a separate thread.
    let writer_2 = writer.clone();
    let err_fn = move |err| {
        eprintln!("an error occurred on stream: {}", err);
    };
    let err_cb = move |err: cpal::BuildStreamError| err.to_string();
    let stream = match config.sample_format() {
        cpal::SampleFormat::I16 => device
            .build_input_stream(
                &config.into(),
                move |data, _: &_| write_input_data::<i16, i16>(data, &writer_2),
                err_fn,
                None,
            )
            .map_err(err_cb)?,
        cpal::SampleFormat::I32 => device
            .build_input_stream(
                &config.into(),
                move |data, _: &_| write_input_data::<i32, i32>(data, &writer_2),
                err_fn,
                None,
            )
            .map_err(err_cb)?,
        cpal::SampleFormat::F32 => device
            .build_input_stream(
                &config.into(),
                move |data, _: &_| write_input_data::<f32, f32>(data, &writer_2),
                err_fn,
                None,
            )
            .map_err(err_cb)?,
        _ => return Err("Only support sample formats: i16, i32, f32".to_string())?,
    };
    stream.play().expect("Unable to record");
    let (tx, rx) = mpsc::channel();
    ctrlc::set_handler(move || tx.send(()).expect("Could not send signal on channel."))
        .expect("Unable to listen keyboard");
    println!("Press 'Ctrl + c' to stop.");
    rx.recv().expect("Program failed");
    drop(stream);
    writer
        .lock()
        .unwrap()
        .take()
        .unwrap()
        .finalize()
        .expect("Unable to save file");
    println!("Recording {} complete!", &command.output_path);
    Ok(())
}


fn write_input_data<T, U>(
    input: &[T],
    writer: &Arc<Mutex<Option<hound::WavWriter<BufWriter<File>>>>>,
) where
    T: Sample,
    U: Sample + hound::Sample + FromSample<T>,
{
    if let Ok(mut guard) = writer.try_lock() {
        if let Some(writer) = guard.as_mut() {
            for &sample in input.iter() {
                let sample: U = U::from_sample(sample);
                writer.write_sample(sample).ok();
            }
        }
    }
}

fn wav_spec_from_config(config: &cpal::SupportedStreamConfig) -> hound::WavSpec {
    hound::WavSpec {
        channels: config.channels() as _,
        sample_rate: config.sample_rate().0 as _,
        bits_per_sample: (config.sample_format().sample_size() * 8) as _,
        sample_format: if config.sample_format().is_float() {
            hound::SampleFormat::Float
        } else {
            hound::SampleFormat::Int
        },
    }
}

// cpal utils for selecting input device and stream config

pub(crate) fn is_compatible_format(format: &cpal::SampleFormat) -> bool {
    match format {
        cpal::SampleFormat::I16 => true,
        cpal::SampleFormat::I32 => true,
        cpal::SampleFormat::F32 => true,
        _ => false,
    }
}
pub(crate) fn is_compatible_buffer_size(
    supported_buffer_size: &cpal::SupportedBufferSize,
    buffer_size: u32,
) -> bool {
    match *supported_buffer_size {
        cpal::SupportedBufferSize::Range { max, min } => min <= buffer_size && buffer_size <= max,
        // assume compatible
        cpal::SupportedBufferSize::Unknown => true,
    }
}

pub(crate) fn get_config(
    config_index: Option<usize>,
    device: &cpal::Device,
) -> cpal::SupportedStreamConfig {
    config_index.map_or_else(
        || {
            let default_config = device
                .default_input_config()
                .expect("Failed to get default input config");
            if is_compatible_format(&default_config.sample_format()) {
                default_config
            } else {
                // look for any compatible configuration
                device
                    .supported_input_configs()
                    .expect("Failed to list input configs")
                    .find(|sc| is_compatible_format(&sc.sample_format()))
                    .map(|sc| sc.with_max_sample_rate())
                    .expect("Failed to get default input config")
            }
        },
        |config_index| {
            device
                .supported_input_configs()
                .expect("Failed to list input configs")
                .enumerate()
                .find_map(|(i, d)| {
                    if i == config_index && is_compatible_format(&d.sample_format()) {
                        Some(d.with_max_sample_rate())
                    } else {
                        None
                    }
                })
                .expect("Unavailable or incompatible configuration selected")
        },
    )
}

pub(crate) fn get_device(device_index: Option<usize>, host: cpal::Host) -> cpal::Device {
    let device = device_index
        .map_or_else(
            || host.default_input_device(),
            |device_index| {
                host.input_devices()
                    .expect("Failed to list input device")
                    .enumerate()
                    .find_map(|(i, d)| if i == device_index { Some(d) } else { None })
            },
        )
        .expect("Failed to find input device");
    device
}
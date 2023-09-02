use std::{sync::mpsc, time::SystemTime};

use crate::cli::record::{self, is_compatible_buffer_size};
use clap::Args;
use cpal::{
    traits::{DeviceTrait, StreamTrait},
    SizedSample,
};
use gag::Gag;
use rustpotter::{
    Rustpotter, RustpotterConfig, RustpotterDetection, Sample, SampleFormat, ScoreMode, VADMode,
};
use time::OffsetDateTime;

#[derive(Args, Debug)]
/// Spot wakewords.
#[clap()]
pub struct SpotCommand {
    #[clap(num_args = 1.., required = true)]
    /// Model path list.
    model_path: Vec<String>,
    #[clap(short = 'i', long)]
    /// Input device index used for record.
    device_index: Option<usize>,
    #[clap(short, long)]
    /// Input device config index used for record.
    config_index: Option<usize>,
    #[clap(short = 'w', long)]
    /// Display host warnings
    host_warnings: bool,
    #[clap(long, default_value_t = 16000)]
    /// Preferred sample rate, if not available for the selected config min sample rate is used.
    sample_rate: u32,
    #[clap(long)]
    /// Set the stream buffer size to a near value to the rustpotter buffer size.
    custom_buffer_size: bool,
    #[clap(long)]
    /// Set the stream buffer size.
    manual_buffer_size: Option<u32>,
    #[clap(short, long, default_value_t = 0.5)]
    /// Default detection threshold, only applies to models without threshold.
    threshold: f32,
    #[clap(short, long, default_value_t = 0.)]
    /// Default detection averaged threshold, only applies to models without averaged threshold.
    averaged_threshold: f32,
    #[clap(short, long, default_value_t = 10)]
    /// Minimum number of partial detections
    min_scores: usize,
    #[clap(short = 's', long, default_value_t = ScoreMode::Max)]
    /// How to calculate a unified score
    score_mode: ScoreMode,
    #[clap(short = 'v', long)]
    /// Enabled vad detection.
    vad_mode: Option<VADMode>,
    #[clap(short = 'g', long)]
    /// Enables a gain-normalizer audio filter.
    gain_normalizer: bool,
    #[clap(long, default_value_t = 0.1)]
    /// Min gain applied by the gain-normalizer filter.
    min_gain: f32,
    #[clap(long, default_value_t = 1.)]
    /// Max gain applied by the gain-normalizer filter.
    max_gain: f32,
    #[clap(long)]
    /// Set the rms level reference used by the gain normalizer filter.
    /// If unset the max wakeword rms level is used.
    gain_ref: Option<f32>,
    #[clap(short, long)]
    /// Enables a band-pass audio filter.
    band_pass: bool,
    #[clap(long, default_value_t = 80.)]
    /// Band-pass audio filter low cutoff.
    low_cutoff: f32,
    #[clap(long, default_value_t = 400.)]
    /// Band-pass audio filter high cutoff.
    high_cutoff: f32,
    #[clap(long, default_value_t = 0.22)]
    /// Used to express the result as a probability. (Advanced)
    score_ref: f32,
    #[clap(short, long)]
    /// Log partial detections.
    debug: bool,
    #[clap(long)]
    /// Log rms level ref, gain applied per frame and frame rms level.
    debug_gain: bool,
    #[clap(short, long)]
    /// Path to create records, one on the first partial detection and another each one that scores better.
    record_path: Option<String>,
}

pub fn spot(command: SpotCommand) -> Result<(), String> {
    let mut stderr_gag = None;
    if !command.host_warnings {
        stderr_gag = Some(Gag::stderr().unwrap());
    }
    println!("Spotting using models: {:?}!", command.model_path);
    // select input device and config
    let host = cpal::default_host();
    let host_name = host.id().name();
    if command.debug {
        println!("Audio backend: {}", host_name);
    }
    let device = record::get_device(command.device_index, host);
    println!(
        "Input device: {}",
        device.name().map_err(|err| err.to_string())?
    );
    let device_config = record::get_config(command.config_index, &device, command.sample_rate);
    println!(
        "Input device config: Sample Rate: {}, Channels: {}, Format: {}",
        device_config.sample_rate().0,
        device_config.channels(),
        device_config.sample_format()
    );
    // disable gag after device config
    if let Some(stderr_gag) = stderr_gag {
        drop(stderr_gag);
    }
    let bits_per_sample = (device_config.sample_format().sample_size() * 8) as u16;
    // configure rustpotter
    let mut config = RustpotterConfig::default();
    config.fmt.sample_rate = device_config.sample_rate().0 as _;
    config.fmt.channels = device_config.channels();
    config.fmt.sample_format = if device_config.sample_format().is_float() {
        SampleFormat::float_of_size(bits_per_sample)
    } else {
        SampleFormat::int_of_size(bits_per_sample)
    }
    .expect("Unsupported wav format");
    config.detector.avg_threshold = command.averaged_threshold;
    config.detector.threshold = command.threshold;
    config.detector.min_scores = command.min_scores;
    config.detector.score_mode = command.score_mode;
    config.detector.score_ref = command.score_ref;
    config.detector.vad_mode = command.vad_mode;
    config.detector.record_path = command.record_path;
    config.filters.gain_normalizer.enabled = command.gain_normalizer;
    config.filters.gain_normalizer.gain_ref = command.gain_ref;
    config.filters.gain_normalizer.min_gain = command.min_gain;
    config.filters.gain_normalizer.max_gain = command.max_gain;
    config.filters.band_pass.enabled = command.band_pass;
    config.filters.band_pass.low_cutoff = command.low_cutoff;
    config.filters.band_pass.high_cutoff = command.high_cutoff;
    if command.debug {
        println!("Rustpotter config:\n{:?}", config);
    }
    let mut rustpotter = Rustpotter::new(&config)?;

    let required_buffer_size: Option<u32> = if command.custom_buffer_size
        || command.manual_buffer_size.is_some()
    {
        let mut required_buffer_size = command
            .manual_buffer_size
            .unwrap_or(rustpotter.get_samples_per_frame() as u32);
        if host_name == "ALSA" && required_buffer_size % 2 != 0 {
            // force even buffer size to workaround issue mentioned here https://github.com/RustAudio/cpal/pull/582#pullrequestreview-1095655011
            required_buffer_size += 1;
        }
        if !is_compatible_buffer_size(device_config.buffer_size(), required_buffer_size) {
            clap::Error::raw(
                clap::error::ErrorKind::Io,
                "Required buffer size does not matches device configuration, try selecting other.\n",
            )
            .exit();
        }
        Some(required_buffer_size)
    } else {
        None
    };
    for path in command.model_path {
        println!("Loading wakeword file: {}", path);
        rustpotter.add_wakeword_from_file("w", &path)?;
    }
    if command.debug_gain {
        println!(
            "Gain Normalizer RMS level reference: {}",
            rustpotter.get_rms_level_ref()
        );
    }
    println!("Begin recording...");
    let stream_config = cpal::StreamConfig {
        channels: device_config.channels(),
        sample_rate: device_config.sample_rate(),
        buffer_size: required_buffer_size
            .map_or(cpal::BufferSize::Default, cpal::BufferSize::Fixed),
    };
    if command.debug {
        println!("Audio stream config: {:?}", stream_config);
    }
    let buffer_i8: Vec<i16> = Vec::new();
    let buffer_i16: Vec<i16> = Vec::new();
    let buffer_i32: Vec<i32> = Vec::new();
    let buffer_f32: Vec<f32> = Vec::new();
    let stream = match device_config.sample_format() {
        cpal::SampleFormat::I8 => init_spot_stream(
            &device,
            &stream_config,
            rustpotter,
            buffer_i8,
            command.debug,
            command.debug_gain,
        )?,
        cpal::SampleFormat::I16 => init_spot_stream(
            &device,
            &stream_config,
            rustpotter,
            buffer_i16,
            command.debug,
            command.debug_gain,
        )?,
        cpal::SampleFormat::I32 => init_spot_stream(
            &device,
            &stream_config,
            rustpotter,
            buffer_i32,
            command.debug,
            command.debug_gain,
        )?,
        cpal::SampleFormat::F32 => init_spot_stream(
            &device,
            &stream_config,
            rustpotter,
            buffer_f32,
            command.debug,
            command.debug_gain,
        )?,
        _ => return Err("Only support sample formats: i16, i32, f32".to_string())?,
    };
    stream.play().expect("Unable to record");
    let (tx, rx) = mpsc::channel();
    ctrlc::set_handler(move || tx.send(()).expect("Could not send signal on channel."))
        .expect("Error setting Ctrl-C handler");
    println!("Press 'Ctrl + c' to stop.");
    rx.recv().expect("Program failed");
    drop(stream);
    println!("Stopped by user request");
    Ok(())
}

fn init_spot_stream<S: Sample + SizedSample>(
    device: &cpal::Device,
    stream_config: &cpal::StreamConfig,
    mut rustpotter: Rustpotter,
    mut buffer: Vec<S>,
    debug: bool,
    debug_gain: bool,
) -> Result<cpal::Stream, String> {
    let error_callback = move |err| {
        eprintln!("an error occurred on stream: {}", err);
    };
    let mut partial_detection_counter = 0;
    let rustpotter_samples_per_frame = rustpotter.get_samples_per_frame();
    let data_callback = move |data: &[S], _: &_| {
        run_detection(
            &mut rustpotter,
            data,
            &mut buffer,
            rustpotter_samples_per_frame,
            &mut partial_detection_counter,
            debug,
            debug_gain,
        )
    };
    device
        .build_input_stream(stream_config, data_callback, error_callback, None)
        .map_err(|err: cpal::BuildStreamError| err.to_string())
}

fn run_detection<T: Sample>(
    rustpotter: &mut Rustpotter,
    data: &[T],
    buffer: &mut Vec<T>,
    rustpotter_samples_per_frame: usize,
    partial_detection_counter: &mut usize,
    debug: bool,
    debug_gain: bool,
) {
    buffer.extend_from_slice(data);
    while buffer.len() >= rustpotter_samples_per_frame {
        let detection = rustpotter.process_samples(
            buffer
                .drain(0..rustpotter_samples_per_frame)
                .as_slice()
                .into(),
        );
        print_detection(
            &*rustpotter,
            detection,
            partial_detection_counter,
            debug,
            debug_gain,
            get_time_string,
        );
    }
}

pub(crate) fn print_detection(
    rustpotter: &Rustpotter,
    detection: Option<RustpotterDetection>,
    partial_detection_counter: &mut usize,
    debug: bool,
    debug_gain: bool,
    time_getter: impl Fn() -> String,
) {
    if debug_gain {
        println!(
            "Frame volume info: RMS={}, Gain={}",
            rustpotter.get_rms_level(),
            rustpotter.get_gain()
        )
    }
    let partial_detection = rustpotter.get_partial_detection();
    *partial_detection_counter = match detection {
        Some(detection) => {
            println!("Wakeword detection: [{}] {:?}", time_getter(), detection);
            0
        }
        None => partial_detection.map_or_else(
            || {
                if debug && *partial_detection_counter > 0 {
                    println!("Partial detection discarded");
                }
                0
            },
            |detection| {
                if debug && *partial_detection_counter < detection.counter {
                    println!("Partial detected: [{}] {:?}", time_getter(), detection);
                }
                detection.counter
            },
        ),
    };
}
fn get_time_string() -> String {
    let dt: OffsetDateTime = SystemTime::now().into();
    format!("{:02}:{:02}:{:02}", dt.hour(), dt.minute(), dt.second())
}

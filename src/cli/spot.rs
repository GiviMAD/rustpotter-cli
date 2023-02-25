use std::{sync::mpsc, time::SystemTime};

use crate::cli::record::{self, is_compatible_buffer_size};
use clap::Args;
use cpal::traits::{DeviceTrait, StreamTrait};
use rustpotter::{Rustpotter, RustpotterConfig, RustpotterDetection, ScoreMode};
use time::OffsetDateTime;

#[derive(Args, Debug)]
/// Spot wakewords.
#[clap()]
pub struct SpotCommand {
    #[clap(num_args = 1.., required = true)]
    /// Model path list.
    model_path: Vec<String>,
    #[clap(short, long)]
    /// Input device index used for record
    device_index: Option<usize>,
    #[clap(short, long)]
    /// Input device index used for record
    config_index: Option<usize>,
    #[clap(short, long, default_value_t = 0.5)]
    /// Default detection threshold, only applies to models without threshold.
    threshold: f32,
    #[clap(short, long, default_value_t = 0.)]
    /// Default detection averaged threshold, only applies to models without averaged threshold, defaults to threshold/2.
    averaged_threshold: f32,
    #[clap(short, long, default_value_t = 10)]
    /// Minimum number of partial detections
    min_scores: usize,
    #[clap(short = 's', long, default_value_t = ClapScoreMode::Max)]
    score_mode: ClapScoreMode,
    #[clap(short = 'b', long)]
    /// Enables a band-pass audio filter.
    band_pass: bool,
    #[clap(long, default_value_t = 80.)]
    /// Band-pass audio filter low cutoff.
    low_cutoff: f32,
    #[clap(long, default_value_t = 400.)]
    /// Band-pass audio filter high cutoff.
    high_cutoff: f32,
    #[clap(short = 'g', long)]
    /// Enables a gain-normalizer audio filter.
    gain_normalizer: bool,
    #[clap(short, long)]
    /// Enables rustpotter debug log
    verbose: bool,
}

pub fn spot(command: SpotCommand) -> Result<(), String> {
    println!("Spotting using models: {:?}!", command.model_path);
    // select input device and config
    let host = cpal::default_host();
    let device = record::get_device(command.device_index, host);
    println!(
        "Input device: {}",
        device.name().map_err(|err| err.to_string())?
    );
    let device_config = record::get_config(command.config_index, &device);
    println!("Input device config: {:?}", device_config);
    // configure rustpotter
    let mut config = RustpotterConfig::default();
    config.fmt.sample_rate = device_config.sample_rate().0 as _;
    config.fmt.bits_per_sample = (device_config.sample_format().sample_size() * 8) as _;
    config.fmt.channels = device_config.channels();
    config.fmt.sample_format = if device_config.sample_format().is_float() {
        hound::SampleFormat::Float
    } else {
        hound::SampleFormat::Int
    };
    config.detector.avg_threshold = command.averaged_threshold;
    config.detector.threshold = command.threshold;
    config.detector.min_scores = command.min_scores;
    config.detector.score_mode = command.score_mode.into();
    config.filters.gain_normalizer = command.gain_normalizer;
    config.filters.band_pass = command.band_pass;
    config.filters.low_cutoff = command.low_cutoff;
    config.filters.high_cutoff = command.high_cutoff;
    if command.verbose {
        println!("Rustpotter config:\n{:?}", config);
    }
    let mut rustpotter = Rustpotter::new(&config)?;
    if !is_compatible_buffer_size(
        &device_config.buffer_size(),
        rustpotter.get_samples_per_frame() as u32,
    ) {
        clap::Error::raw(
            clap::error::ErrorKind::Io,
            "Rustpotter required buffer size does not matches device configuration, try selecting other.\n",
        )
        .exit();
    }
    for path in command.model_path {
        let result = rustpotter.add_wakeword_from_file(&path);
        if let Err(error) = result {
            clap::Error::raw(clap::error::ErrorKind::InvalidValue, error + "\n").exit();
        }
    }
    println!("Begin recording...");
    let err_fn = move |err| {
        eprintln!("an error occurred on stream: {}", err);
    };
    let err_cb = move |err: cpal::BuildStreamError| err.to_string();
    let stream_config = cpal::StreamConfig {
        channels: device_config.channels(),
        sample_rate: device_config.sample_rate(),
        buffer_size: cpal::BufferSize::Fixed(rustpotter.get_samples_per_frame() as u32),
    };
    let mut partial_detection_counter = 0;
    let stream = match device_config.sample_format() {
        cpal::SampleFormat::I16 => device
            .build_input_stream(
                &stream_config,
                move |data: &[i16], _: &_| {
                    print_detection(
                        rustpotter.process_short_buffer(data),
                        rustpotter.get_partial_detection(),
                        &mut partial_detection_counter,
                        command.verbose,
                    );
                },
                err_fn,
                None,
            )
            .map_err(err_cb)?,
        cpal::SampleFormat::I32 => device
            .build_input_stream(
                &stream_config,
                move |data: &[i32], _: &_| {
                    print_detection(
                        rustpotter.process_int_buffer(data),
                        rustpotter.get_partial_detection(),
                        &mut partial_detection_counter,
                        command.verbose,
                    );
                },
                err_fn,
                None,
            )
            .map_err(err_cb)?,
        cpal::SampleFormat::F32 => device
            .build_input_stream(
                &stream_config,
                move |data: &[f32], _: &_| {
                    print_detection(
                        rustpotter.process_float_buffer(data),
                        rustpotter.get_partial_detection(),
                        &mut partial_detection_counter,
                        command.verbose,
                    );
                },
                err_fn,
                None,
            )
            .map_err(err_cb)?,
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

pub(crate) fn print_detection(
    detection: Option<RustpotterDetection>,
    partial_detection: Option<&RustpotterDetection>,
    partial_detection_counter: &mut usize,
    verbose: bool,
) {
    let dt: OffsetDateTime = SystemTime::now().into();
    match detection {
        Some(detection) => {
            *partial_detection_counter = 0;
            println!(
                "Wakeword detection: {:02}:{:02}:{:02}\n{:?}",
                dt.hour(),
                dt.minute(),
                dt.second(),
                detection
            );
        }
        None => {
            if verbose {
                *partial_detection_counter = partial_detection.map_or_else(
                    || {
                        if *partial_detection_counter > 0 {
                            println!("Partial detection discarded");
                        }
                        0
                    },
                    |detection| {
                        if *partial_detection_counter < detection.counter {
                            println!(
                                "Partial detection: {:02}:{:02}:{:02}\n{:?}",
                                dt.hour(),
                                dt.minute(),
                                dt.second(),
                                detection
                            );
                        }
                        detection.counter
                    },
                );
            }
        }
    };
}

#[derive(clap::ValueEnum, Clone, Debug)]
pub(crate) enum ClapScoreMode {
    Max,
    Avg,
    Median,
}
impl std::fmt::Display for ClapScoreMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ClapScoreMode::Avg => write!(f, "avg"),
            ClapScoreMode::Max => write!(f, "max"),
            ClapScoreMode::Median => write!(f, "median"),
        }
    }
}
impl From<ClapScoreMode> for ScoreMode {
    fn from(value: ClapScoreMode) -> Self {
        match value {
            ClapScoreMode::Avg => ScoreMode::Average,
            ClapScoreMode::Max => ScoreMode::Max,
            ClapScoreMode::Median => ScoreMode::Median,
        }
    }
}

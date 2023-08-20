use clap::Args;
use hound::{SampleFormat, WavReader};
use rustpotter::{Rustpotter, RustpotterConfig, Sample};
use std::{fs::File, io::BufReader};

use super::spot::{print_detection, ClapScoreMode};

#[derive(Args, Debug)]
/// Test wakeword file against a wav sample, detector is automatically configured according to the sample spec
#[clap()]
pub struct TestCommand {
    #[clap()]
    /// Model to test.
    model_path: String,
    #[clap()]
    /// Wav record to test.
    sample_path: String,
    #[clap(short, long, default_value_t = 0.5)]
    /// Default detection threshold, only applies to models without threshold.
    threshold: f32,
    #[clap(short, long, default_value_t = 0.2)]
    /// Default detection averaged threshold, only applies to models without averaged threshold.
    averaged_threshold: f32,
    #[clap(short, long, default_value_t = 10)]
    /// Minimum number of partial detections
    min_scores: usize,
    #[clap(short = 's', long, default_value_t = ClapScoreMode::Max)]
    /// How to calculate a unified score, no applies to wakeword models.
    score_mode: ClapScoreMode,
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
    /// Used to express the score as value in range 0 - 1.
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
pub fn test(command: TestCommand) -> Result<(), String> {
    println!(
        "Testing file {} against model {}!",
        command.sample_path, command.model_path,
    );
    // Read wav file
    let file_reader =
        BufReader::new(File::open(command.sample_path).map_err(|err| err.to_string())?);
    let mut wav_reader = WavReader::new(file_reader).map_err(|err| err.to_string())?;
    let wav_specs = wav_reader.spec();
    let mut config = RustpotterConfig::default();
    let sample_rate = wav_specs.sample_rate as usize;
    config.fmt = wav_specs.try_into()?;
    config.detector.avg_threshold = command.averaged_threshold;
    config.detector.threshold = command.threshold;
    config.detector.min_scores = command.min_scores;
    config.detector.score_mode = command.score_mode.into();
    config.detector.score_ref = command.score_ref;
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
    println!("Loading wakeword file: {}", command.model_path);
    rustpotter.add_wakeword_from_file(&command.model_path)?;
    let mut partial_detection_counter = 0;
    let mut chunk_counter = 0;
    let chunk_size = rustpotter.get_samples_per_frame();
    match wav_specs.sample_format {
        SampleFormat::Int => match wav_specs.bits_per_sample {
            8 => run_detection::<i8>(
                &mut wav_reader,
                &mut rustpotter,
                chunk_size,
                &mut chunk_counter,
                &mut partial_detection_counter,
                sample_rate,
                command.debug,
                command.debug_gain,
            ),
            16 => run_detection::<i16>(
                &mut wav_reader,
                &mut rustpotter,
                chunk_size,
                &mut chunk_counter,
                &mut partial_detection_counter,
                sample_rate,
                command.debug,
                command.debug_gain,
            ),
            32 => run_detection::<i32>(
                &mut wav_reader,
                &mut rustpotter,
                chunk_size,
                &mut chunk_counter,
                &mut partial_detection_counter,
                sample_rate,
                command.debug,
                command.debug_gain,
            ),
            _ => panic!("Unsupported wav format"),
        },
        SampleFormat::Float => match wav_specs.bits_per_sample {
            32 => run_detection::<f32>(
                &mut wav_reader,
                &mut rustpotter,
                chunk_size,
                &mut chunk_counter,
                &mut partial_detection_counter,
                sample_rate,
                command.debug,
                command.debug_gain,
            ),
            _ => panic!("Unsupported wav format"),
        },
    };
    Ok(())
}

fn run_detection<T: Sample + hound::Sample>(
    wav_reader: &mut WavReader<BufReader<File>>,
    rustpotter: &mut Rustpotter,
    chunk_size: usize,
    chunk_counter: &mut usize,
    partial_detection_counter: &mut usize,
    sample_rate: usize,
    debug: bool,
    debug_gain: bool,
) {
    let mut buffer = wav_reader
        .samples::<T>()
        .map(Result::unwrap)
        .collect::<Vec<_>>();
    buffer.append(&mut vec![T::get_zero(); chunk_size * 100]);
    buffer.chunks_exact(chunk_size).for_each(|chunk| {
        *chunk_counter += 1;
        let detection = rustpotter.process_samples(chunk.into());
        print_detection(
            rustpotter,
            detection,
            partial_detection_counter,
            debug,
            debug_gain,
            || get_time_string(*chunk_counter, chunk_size, sample_rate),
        );
    });
}
fn get_time_string(chunk_number: usize, chunk_size: usize, sample_rate: usize) -> String {
    let total_seconds = (chunk_number * chunk_size) as f32 / sample_rate as f32;
    let minutes = (total_seconds / 60.).floor() as i32;
    let seconds = (total_seconds % 60.).floor() as i32;
    format!("00:{:02}:{:02}", minutes, seconds)
}

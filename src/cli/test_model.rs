use clap::Args;
use hound::{SampleFormat, WavReader};
use rustpotter::{Rustpotter, RustpotterConfig};
use std::{fs::File, io::BufReader};

use super::spot::{print_detection, ClapScoreMode};

#[derive(Args, Debug)]
/// Test model file against a wav sample, detector is automatically configured according to the sample spec
#[clap()]
pub struct TestModelCommand {
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
    /// How to calculate a unified score
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
    #[clap(long, default_value_t = 5)]
    /// Band size of the comparison. (Advanced)
    comparator_band_size: u16,
    #[clap(long, default_value_t = 0.22)]
    /// Used to express the result as a probability. (Advanced)
    comparator_ref: f32,
    #[clap(short, long)]
    /// Log partial detections.
    debug: bool,
    #[clap(long)]
    /// Log rms level ref, gain applied per frame and frame rms level.
    debug_gain: bool,
}
pub fn test(command: TestModelCommand) -> Result<(), String> {
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
    config.fmt.sample_rate = sample_rate;
    config.fmt.bits_per_sample = wav_specs.bits_per_sample;
    config.fmt.channels = wav_specs.channels;
    config.detector.avg_threshold = command.averaged_threshold;
    config.detector.threshold = command.threshold;
    config.detector.min_scores = command.min_scores;
    config.detector.score_mode = command.score_mode.into();
    config.detector.comparator_band_size = command.comparator_band_size;
    config.detector.comparator_ref = command.comparator_ref;
    config.filters.gain_normalizer.enabled = command.gain_normalizer;
    config.filters.gain_normalizer.gain_ref = command.gain_ref;
    config.filters.gain_normalizer.min_gain = command.min_gain;
    config.filters.gain_normalizer.max_gain = command.max_gain;
    config.filters.band_pass.enabled = command.band_pass;
    config.filters.band_pass.low_cutoff = command.low_cutoff;
    config.filters.band_pass.high_cutoff = command.high_cutoff;
    let mut rustpotter = Rustpotter::new(&config)?;
    if let Err(error) = rustpotter.add_wakeword_from_file(&command.model_path) {
        clap::Error::raw(
            clap::error::ErrorKind::InvalidValue,
            error.to_string() + "\n",
        )
        .exit();
    }
    let mut partial_detection_counter = 0;
    let mut chunk_counter = 0;
    let chunk_size = rustpotter.get_samples_per_frame();
    match wav_specs.sample_format {
        SampleFormat::Int => {
            let mut buffer = wav_reader
                .samples::<i32>()
                .map(Result::unwrap)
                .collect::<Vec<_>>();
            buffer.append(&mut vec![0; chunk_size * 100]);
            buffer
                .chunks_exact(chunk_size)
                .for_each(|chunk| {
                    chunk_counter+=1;
                    let detection = rustpotter.process_i32(chunk);
                    print_detection(
                        &rustpotter,
                        detection,
                        &mut partial_detection_counter,
                        command.debug,
                        command.debug_gain,
                        || { get_time_string(chunk_counter, chunk_size, sample_rate) },
                    );
                });
        }
        SampleFormat::Float => {
            let mut buffer = wav_reader
                .samples::<f32>()
                .map(Result::unwrap)
                .collect::<Vec<_>>();
            buffer.append(&mut vec![0.; chunk_size * 100]);
            buffer
                .chunks_exact(chunk_size)
                .for_each(|chunk| {
                    chunk_counter+=1;
                    let detection = rustpotter.process_f32(chunk);
                    print_detection(
                        &rustpotter,
                        detection,
                        &mut partial_detection_counter,
                        command.debug,
                        command.debug_gain,
                        || { get_time_string(chunk_counter, chunk_size, sample_rate) },
                    );
                });
        }
    };
    Ok(())
}
fn get_time_string(chunk_number: usize, chunk_size: usize, sample_rate: usize) -> String {
    let total_seconds = (chunk_number * chunk_size) as f32 / sample_rate as f32;
    let minutes = (total_seconds / 60.).floor() as i32;
    let seconds = (total_seconds % 60.).floor() as i32;
    format!("00:{:02}:{:02}", minutes, seconds)
}
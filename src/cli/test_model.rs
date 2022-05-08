use clap::Args;
use hound::{SampleFormat, WavReader};
use rustpotter::WakewordDetectorBuilder;
use std::{fs::File, io::BufReader};

use crate::utils::enable_rustpotter_log;

#[derive(Args, Debug)]
/// Test model file against a wav sample, detector is automatically configured according to the sample spec
#[clap()]
pub struct TestModelCommand {
    #[clap()]
    /// Output model path
    model_path: String,
    #[clap()]
    /// Sample record path
    sample_path: String,
    #[clap(short, long, default_value_t = 0.5)]
    /// Default detection threshold, only applies to models without threshold
    threshold: f32,
    #[clap(short, long, default_value_t = 0.2)]
    /// Default detection averaged threshold, only applies to models without averaged threshold
    averaged_threshold: f32,
    #[clap(long)]
    /// Enables rustpotter debug log
    debug: bool,
}
pub fn test(command: TestModelCommand) -> Result<(), String> {
    println!(
        "Testing file {} against model {}!",
        command.sample_path, command.model_path,
    );
    if command.debug {
        enable_rustpotter_log();
    }
    let mut detector_builder = WakewordDetectorBuilder::new();
    let reader =
        BufReader::new(File::open(command.sample_path).or_else(|err| Err(err.to_string()))?);
    let mut wav_reader = WavReader::new(reader).or_else(|err| Err(err.to_string()))?;
    let wav_specs = wav_reader.spec();
    detector_builder.set_averaged_threshold(command.averaged_threshold);
    detector_builder.set_threshold(command.threshold);
    detector_builder.set_sample_rate(wav_specs.sample_rate as usize);
    detector_builder.set_bits_per_sample(wav_specs.bits_per_sample);
    detector_builder.set_sample_format(wav_specs.sample_format);
    // multi-channel still not supported
    assert!(wav_specs.channels == 1);
    let mut word_detector = detector_builder.build();
    let add_wakeword_result = word_detector.add_keyword_from_model_file(command.model_path, true);
    if add_wakeword_result.is_err() {
        clap::Error::raw(
            clap::ErrorKind::InvalidValue,
            add_wakeword_result.unwrap_err() + "\n",
        )
        .exit();
    }
    match wav_specs.sample_format {
        SampleFormat::Int => {
            let mut buffer = wav_reader
                .samples::<i32>()
                .map(Result::unwrap)
                .collect::<Vec<_>>();
            buffer.append(&mut vec![0; word_detector.get_samples_per_frame()]);
            buffer
                .chunks_exact(word_detector.get_samples_per_frame())
                .filter_map(|chunk| word_detector.process(chunk))
                .for_each(|detection| {
                    println!(
                        "Detected '{}' with score {}!",
                        detection.wakeword, detection.score
                    )
                });
            println!("Done!");
            Ok(())
        }
        SampleFormat::Float => {
            let mut buffer = wav_reader
                .samples::<f32>()
                .map(Result::unwrap)
                .collect::<Vec<_>>();
            buffer.append(&mut vec![0.; word_detector.get_samples_per_frame()]);
            buffer
                .chunks_exact(word_detector.get_samples_per_frame())
                .filter_map(|chunk| word_detector.process_f32(chunk))
                .for_each(|detection| {
                    println!(
                        "Detected '{}' with score {}!",
                        detection.wakeword, detection.score
                    )
                });
            println!("Done!");
            Ok(())
        }
    }
}

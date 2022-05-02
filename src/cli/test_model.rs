use clap::Args;
use rustpotter::detector;
use std::{fs, path::Path};

#[derive(Args, Debug)]
/// Test model file with samples
#[clap()]
pub struct TestModelCommand {
    #[clap()]
    /// Output model path
    model_path: String,
    #[clap()]
    /// Sample record path
    sample_path: String,
    #[clap(short = 'a', long)]
    /// Enables template averaging
    average_templates: bool,
    #[clap(short = 't', long, default_value_t = 0.)]
    /// Customize detection threshold
    threshold: f32,
    #[clap(short = 'r', long, default_value_t = 16000)]
    /// Sample record sample rate
    sample_rate: usize,
}
pub fn test(command: TestModelCommand) -> Result<(), String> {
    println!(
        "Testing file {} against model {}!",
        command.sample_path, command.model_path,
    );
    let mut detector_builder = detector::FeatureDetectorBuilder::new();
    detector_builder.set_threshold(command.threshold);
    detector_builder.set_sample_rate(command.sample_rate);
    match get_audio_buffer(command.sample_path) {
        Ok(buffer) => {
            let mut word_detector = detector_builder.build();
            let mut test_buffer: Vec<u8> = Vec::new();
            test_buffer.append(&mut buffer.to_vec());
            test_buffer.append(&mut vec![0; word_detector.get_samples_per_frame()]);
            let add_wakeword_result = word_detector.add_keyword_from_model(
                command.model_path,
                command.average_templates,
                true,
            );
            if add_wakeword_result.is_err() {
                clap::Error::raw(
                    clap::ErrorKind::InvalidValue,
                    add_wakeword_result.unwrap_err() + "\n",
                )
                .exit();
            }
            test_buffer
                .chunks_exact(2)
                .into_iter()
                .map(|bytes| i16::from_le_bytes([bytes[0], bytes[1]]))
                .collect::<Vec<_>>()
                .chunks_exact(word_detector.get_samples_per_frame())
                .filter_map(|chunk| word_detector.process_pcm_signed(chunk))
                .for_each(|detection| {
                    println!(
                        "Detected '{}' with score {}!",
                        detection.wakeword, detection.score
                    )
                });
            println!("Done!")
        }
        Err(message) => {
            clap::Error::raw(clap::ErrorKind::InvalidValue, message + "\n").exit();
        }
    };
    Ok(())
}

fn get_audio_buffer(audio_path: String) -> Result<Vec<u8>, String> {
    let path = Path::new(&audio_path);
    if !path.exists() || !path.is_file() {
        return Err(String::from("Can not read file"));
    }
    match fs::read(path) {
        Ok(input) => {
            let mut input_copy = input.to_vec();
            input_copy.drain(0..44);
            Ok(input_copy)
        }
        Err(..) => Err(String::from("Can not read file")),
    }
}

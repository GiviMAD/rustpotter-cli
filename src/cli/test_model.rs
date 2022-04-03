use std::{fs, path::Path};

use clap::Args;

use rustpotter::detector;

use super::AudioArgs;


#[derive(Args, Debug)]
/// Test model file with samples
#[clap()]
pub struct TestModelCommand {
    #[clap()]
    /// Output model path
    model_path:String,
    #[clap()]
    /// Sample record path
    sample_path:String,
    #[clap(short='a', long)]
    /// Enables template averaging
    average_templates: bool,
    #[clap(flatten)]
    audio_args: AudioArgs,
}
pub fn test(command: TestModelCommand) {
    println!("Test!");
    println!("Has Command {:?}!", command);
    let mut detector_builder = detector::FeatureDetectorBuilder::new();
    detector_builder.set_threshold(0.);
    detector_builder.set_frame_length_ms(command.audio_args.frame_length_ms);
    detector_builder.set_frame_shift_ms(command.audio_args.frame_shift_ms);
    match get_audio_buffer(command.sample_path) {
        Ok(buffer) => {
            let mut buffer_copy = buffer.to_vec();
            buffer_copy.append(& mut vec![0; detector_builder.get_samples_per_frame()]);
            let mut word_detector = detector_builder.build(|kw| println!("Detected '{}' with score {}!", kw.wakeword, kw.score));
            let add_wakeword_result = word_detector.add_keyword_from_model(command.model_path, command.average_templates, true);
            if add_wakeword_result.is_err() {
                clap::Error::raw(clap::ErrorKind::InvalidValue, add_wakeword_result.unwrap_err()+"\n").exit();
            }
            word_detector.process_bytes(buffer_copy);
            println!("Done!")
        },
        Err(message) => {
            clap::Error::raw(clap::ErrorKind::InvalidValue, message+"\n").exit();
        },
    }
    
}

fn get_audio_buffer (audio_path: String) -> Result<Vec<u8>, String> {
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
        Err(..) => {
            Err(String::from("Can not read file"))
        }
    }
}
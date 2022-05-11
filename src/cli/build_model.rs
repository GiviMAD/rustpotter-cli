use std::{fs::File, io::BufReader};

use clap::Args;
use hound::WavReader;
use rustpotter::WakewordDetectorBuilder;

#[derive(Args, Debug)]
/// Build model file from samples
#[clap()]
pub struct BuildModelCommand {
    #[clap(short = 'n', long)]
    /// The term emitted on the spot event
    model_name: String,
    #[clap(short = 'p', long)]
    /// Generated model path
    model_path: String,
    #[clap(min_values = 1, required = true)]
    /// List of sample record paths
    sample_path: Vec<String>,
    #[clap(short = 't', long)]
    /// Threshold to configure in the generated model, overwrites the detector threshold
    threshold: Option<f32>,
    #[clap(short = 'a', long)]
    /// Averaged threshold to configure in the generated model, overwrites the detector averaged threshold
    averaged_threshold: Option<f32>,
}
pub fn build(command: BuildModelCommand) -> Result<(), String> {
    println!("Start building {}!", command.model_path);
    println!("From samples:");
    for path in &command.sample_path {
        let reader = BufReader::new(File::open(path).or_else(|err| Err(err.to_string()))?);
        let wav_spec = WavReader::new(reader)
            .or_else(|err| Err(err.to_string()))?
            .spec();
        println!("{}: {:?}", path, wav_spec);
    }
    let mut word_detector = WakewordDetectorBuilder::new().build();
    word_detector.add_wakeword(
        command.model_name.clone(),
        false,
        command.averaged_threshold,
        command.threshold,
        command.sample_path,
    );
    match word_detector.generate_wakeword_model_file(command.model_name.clone(), command.model_path)
    {
        Ok(_) => {
            println!("{} created!", command.model_name);
        }
        Err(message) => {
            clap::Error::raw(clap::ErrorKind::InvalidValue, message + "\n").exit();
        }
    };
    Ok(())
}

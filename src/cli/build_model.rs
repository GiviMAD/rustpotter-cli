use std::{fs::File, io::BufReader};

use clap::Args;
use hound::WavReader;
use rustpotter::WakewordDetectorBuilder;

use crate::utils::enable_rustpotter_log;

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
    #[clap(long)]
    /// Enables rustpotter debug log
    debug: bool,
}
pub fn build(command: BuildModelCommand) -> Result<(), String> {
    println!("Start building {}!", command.model_path);
    println!("From samples:");
    for path in &command.sample_path {
        let reader = BufReader::new(File::open(path).map_err(|err| err.to_string())?);
        let wav_spec = WavReader::new(reader).map_err(|err| err.to_string())?
            .spec();
        println!("{}: {:?}", path, wav_spec);
    }
    if command.debug {
        enable_rustpotter_log();
    }
    let mut word_detector = WakewordDetectorBuilder::new().build();
    word_detector.add_wakeword_with_wav_files(
        &command.model_name,
        false,
        command.averaged_threshold,
        command.threshold,
        command.sample_path,
    ).map_err(|e| e.to_string())?;
    match word_detector.generate_wakeword_model_file(command.model_name.clone(), command.model_path)
    {
        Ok(_) => {
            println!("{} created!", command.model_name);
        }
        Err(error) => {
            clap::Error::raw(clap::ErrorKind::InvalidValue, error.to_string() + "\n").exit();
        }
    };
    Ok(())
}

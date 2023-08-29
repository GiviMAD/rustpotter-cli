use std::{fs::File, io::BufReader};

use clap::Args;
use hound::WavReader;
use rustpotter::{WakewordRef, WakewordRefBuildFromFiles, WakewordSave};

#[derive(Args, Debug)]
/// Creates a wakeword reference using wav audio files.
#[clap()]
pub struct BuildCommand {
    #[clap(short = 'n', long)]
    /// The term emitted on the spot event
    model_name: String,
    #[clap(short = 'p', long)]
    /// Generated model path
    model_path: String,
    #[clap(num_args = 1.., required = true)]
    /// List of sample record paths
    sample_path: Vec<String>,
    #[clap(short = 't', long)]
    /// Threshold to configure in the generated model, overwrites the detector threshold
    threshold: Option<f32>,
    #[clap(short = 'a', long)]
    /// Averaged threshold to configure in the generated model, overwrites the detector averaged threshold
    averaged_threshold: Option<f32>,
    #[clap(short = 'c', long, default_value_t = 16)]
    /// Number of extracted mel-frequency cepstral coefficients
    mfcc_size: u16,
}
pub fn build_ref(command: BuildCommand) -> Result<(), String> {
    println!("Start building {}!", command.model_path);
    println!("From samples:");
    for path in &command.sample_path {
        let reader = BufReader::new(File::open(path).map_err(|err| err.to_string())?);
        let wav_spec = WavReader::new(reader)
            .map_err(|err| err.to_string())?
            .spec();
        println!("{}: {:?}", path, wav_spec);
    }
    let wakeword = WakewordRef::new_from_sample_files(
        command.model_name.clone(),
        command.threshold,
        command.averaged_threshold,
        command.sample_path,
        command.mfcc_size,
    )?;
    wakeword.save_to_file(&command.model_path)?;
    println!("{} created!", command.model_name);
    Ok(())
}

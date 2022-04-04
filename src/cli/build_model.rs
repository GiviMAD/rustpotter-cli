use clap::{Args};

use rustpotter::detector;

use super::AudioArgs;

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
    #[clap(flatten)]
    audio_args: AudioArgs,
}
pub fn build(command: BuildModelCommand) {
    println!("Start building {}!", command.model_path);
    let mut detector_builder = detector::FeatureDetectorBuilder::new();
    detector_builder.set_frame_length_ms(command.audio_args.frame_length_ms);
    detector_builder.set_frame_shift_ms(command.audio_args.frame_shift_ms);
    let mut word_detector = detector_builder.build();
    word_detector.add_keyword(
        command.model_name.clone(),
        false,
        true,
        None,
        command.sample_path,
    );
    match word_detector.create_wakeword_model(command.model_name.clone(), command.model_path) {
        Ok(_) => {
            println!("{} created!", command.model_name);
        }
        Err(message) => {
            clap::Error::raw(clap::ErrorKind::InvalidValue, message+"\n").exit();
        }
    };
}

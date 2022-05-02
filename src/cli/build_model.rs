use clap::Args;
use rustpotter::detector;

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
    #[clap(short = 'r', long, default_value_t = 16000)]
    /// Sample record sample rate
    sample_rate: usize,
}
pub fn build(command: BuildModelCommand) -> Result<(), String> {
    println!("Start building {}!", command.model_path);
    let mut detector_builder = detector::FeatureDetectorBuilder::new();
    detector_builder.set_sample_rate(command.sample_rate);
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
            clap::Error::raw(clap::ErrorKind::InvalidValue, message + "\n").exit();
        }
    };
    Ok(())
}

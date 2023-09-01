use clap::Args;
use rustpotter::{ModelType, WakewordLoad, WakewordModel, WakewordModelTrain, WakewordSave};

#[derive(Args, Debug)]
/// Train wakeword model, using wav audio files
#[clap()]
pub struct TrainCommand {
    #[clap()]
    /// Generated model path
    model_path: String,
    #[clap(short = 't', long, default_value_t = ClapModelType::MEDIUM)]
    /// Generated model type
    model_type: ClapModelType,
    #[clap(long, required = true)]
    /// Train data directory path
    train_dir: String,
    #[clap(long, required = true)]
    /// Test data directory path
    test_dir: String,
    #[clap(short = 'l', long, default_value_t = 0.03)]
    /// Training learning rate
    learning_rate: f64,
    #[clap(short = 'e', long, default_value_t = 1000)]
    /// Number of backward and forward cycles to run
    epochs: usize,
    #[clap(long, default_value_t = 1)]
    /// Number of epochs for testing the model and print the progress.  
    test_epochs: usize,
    #[clap(short = 'c', long, default_value_t = 16)]
    /// Number of extracted mel-frequency cepstral coefficients
    mfcc_size: u16,
    #[clap(short = 'm')]
    /// Model to continue training from
    wakeword_model: Option<String>,
}
pub fn train(command: TrainCommand) -> Result<(), String> {
    println!("Start training {}!", command.model_path);
    let model: Option<WakewordModel> = if let Some(wakeword_model_path) = command.wakeword_model {
        Some(WakewordModel::load_from_file(&wakeword_model_path)?)
    } else {
        None
    };
    let wakeword = WakewordModel::train_from_sample_dirs(
        command.model_type.into(),
        command.train_dir,
        command.test_dir,
        command.learning_rate,
        command.epochs,
        command.test_epochs,
        command.mfcc_size,
        model,
    )
    .map_err(|err| err.to_string())?;
    wakeword.save_to_file(&command.model_path)?;
    println!("{} created!", command.model_path);
    Ok(())
}

#[derive(clap::ValueEnum, Clone, Debug)]
pub(crate) enum ClapModelType {
    SMALL,
    MEDIUM,
    LARGE,
}
impl std::fmt::Display for ClapModelType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ClapModelType::SMALL => write!(f, "small"),
            ClapModelType::MEDIUM => write!(f, "medium"),
            ClapModelType::LARGE => write!(f, "large"),
        }
    }
}
impl From<ClapModelType> for ModelType {
    fn from(value: ClapModelType) -> Self {
        match value {
            ClapModelType::SMALL => ModelType::SMALL,
            ClapModelType::MEDIUM => ModelType::MEDIUM,
            ClapModelType::LARGE => ModelType::LARGE,
        }
    }
}

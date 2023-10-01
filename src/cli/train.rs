use clap::Args;
use rustpotter::{
    ModelType, WakewordLoad, WakewordModel, WakewordModelTrain, WakewordModelTrainOptions,
    WakewordSave,
};

#[derive(Args, Debug)]
/// Train wakeword model, using wav audio files
#[clap()]
pub struct TrainCommand {
    #[clap()]
    /// Generated model path
    model_path: String,
    #[clap(short = 't', long, default_value_t = ModelType::Medium)]
    /// Generated model type
    model_type: ModelType,
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
    #[clap(short = 'm', long)]
    /// Model to continue training from
    from_model: Option<String>,
}
pub fn train(command: TrainCommand) -> Result<(), String> {
    println!("Start training {}!", command.model_path);
    let model: Option<WakewordModel> = if let Some(wakeword_model_path) = command.from_model {
        Some(WakewordModel::load_from_file(&wakeword_model_path)?)
    } else {
        None
    };
    let options = WakewordModelTrainOptions::new(
        command.model_type,
        command.learning_rate,
        command.epochs,
        command.test_epochs,
        command.mfcc_size,
    );
    let wakeword =
        WakewordModel::train_from_dirs(options, command.train_dir, command.test_dir, model)
            .map_err(|err| err.to_string())?;
    wakeword.save_to_file(&command.model_path)?;
    println!("{} created!", command.model_path);
    Ok(())
}

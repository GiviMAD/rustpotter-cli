use clap::{Parser, Subcommand};
mod build;
mod devices;
mod filter;
mod record;
mod spot;
mod test;
mod train;
use self::{
    build::{build_ref, BuildCommand},
    devices::{devices, DevicesCommand},
    filter::{filter, FilterCommand},
    record::{record, RecordCommand},
    spot::{spot, SpotCommand},
    test::{test, TestCommand},
    train::{train, TrainCommand},
};

#[derive(Parser, Debug)]
/// CLI for RustPotter: an open source wakeword spotter forged in rust
#[clap(author, version, about, long_about = None, arg_required_else_help = true)]
struct Cli {
    #[clap(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Build wakeword reference from wav audio files.
    ///
    /// These wakewords offers worst quality detection than the wakeword models but requires low number of records  (recommended 3 to 8).
    ///
    /// The file size and the cpu consumption depends on the number of sample files for built it.
    ///
    Build(BuildCommand),
    /// Train wakeword model from wav audio files.
    ///
    /// This wakeword type requires a more sample files to be created, but produces high fidelity results.
    ///
    /// The file size and the cpu consumption depends on the model type and the duration on the longer audio sample on the training folder.
    ///
    /// It's required to setup a train and test folders containing wav files labeled as something (for example "[ok_casa]20:44:32.wav") and
    /// others without any tag ("20:46:32.wav.wav" and "[none]20:46:32.wav" is equivalent).
    ///
    /// The will train a basic classification neural network for the available labels, that the tool can use emit detections when a label other
    /// than "none" is predicted.
    ///
    /// The weight initialization is not fixed and can produce different results per execution but the
    ///
    /// Tested with a training set of 155 affirmative samples and 1355 noise/ambient samples over a test set of 108 samples.
    /// I obtain a round 96% of accuracy using the different model types, and all work nice in real live,
    /// the small and medium models can require setting a higher threshold or min partial detections to avoid false detections,
    /// but other than that all seems to be reliable.
    ///
    Train(TrainCommand),
    /// List available audio devices and configurations
    ///
    /// Useful in order to know how to configure the input and format
    /// for the "record" and "spot" commands.
    Devices(DevicesCommand),
    /// Apply available filters to a wav audio file
    Filter(FilterCommand),
    /// Record wav audio file
    Record(RecordCommand),
    /// Spot wakewords in real time
    Spot(SpotCommand),
    /// Spot wakewords against a wav file  
    Test(TestCommand),
}

pub(crate) fn run_cli() {
    let cli = Cli::parse();
    match cli.command.unwrap() {
        Command::Build(command) => build_ref(command),
        Command::Devices(command) => devices(command),
        Command::Filter(command) => filter(command),
        Command::Record(command) => record(command),
        Command::Spot(command) => spot(command),
        Command::Test(command) => test(command),
        Command::Train(command) => train(command),
    }
    .expect("Command failed");
}

use clap::{Parser, Subcommand};
mod build_model;
mod devices;
mod record;
mod spot;
mod test_model;
mod filter;
use self::{
    build_model::{build, BuildModelCommand},
    devices::{devices, DevicesCommand},
    record::{record, RecordCommand},
    spot::{spot, SpotCommand},
    test_model::{test, TestModelCommand}, 
    filter::{filter, FilterCommand},
};

#[derive(Parser, Debug)]
/// CLI for RustPotter: an open source wakeword spotter forged in rust
#[clap(author, version, about, long_about = None, arg_required_else_help = true)]
struct CLI {
    #[clap(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Build wakeword model from wav audio files
    BuildModel(BuildModelCommand),
    /// List audio devices and configurations
    Devices(DevicesCommand),
    /// Apply available filters to a wav audio file.
    Filter(FilterCommand),
    /// Record wav audio file
    Record(RecordCommand),
    /// Spot wakewords in real time
    Spot(SpotCommand),
    /// Spot wakewords against a wav file  
    TestModel(TestModelCommand),
}

pub(crate) fn run_cli() {
    let cli = CLI::parse();
    match cli.command.unwrap() {
        Commands::Record(command) => record(command),
        Commands::Filter(command) => filter(command),
        Commands::BuildModel(command) => build(command),
        Commands::TestModel(command) => test(command),
        Commands::Spot(command) => spot(command),
        Commands::Devices(command) => devices(command),
    }
    .expect("Command failed");
}

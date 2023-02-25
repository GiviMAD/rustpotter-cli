use clap::{Parser, Subcommand};
mod build_model;
mod devices;
mod record;
mod spot;
mod test_model;
use self::{
    build_model::{build, BuildModelCommand},
    devices::{devices, DevicesCommand},
    record::{record, RecordCommand},
    spot::{spot, SpotCommand},
    test_model::{test, TestModelCommand},
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
    /// Record audio sample
    Record(RecordCommand),
    /// Build wakeword model
    BuildModel(BuildModelCommand),
    /// Test model accuracy against a sample  
    TestModel(TestModelCommand),
    /// Spot wakeword using model
    Spot(SpotCommand),
    /// List audio devices
    Devices(DevicesCommand),
}

pub(crate) fn run_cli() {
    let cli = CLI::parse();
    match cli.command.unwrap() {
        Commands::Record(command) => record(command),
        Commands::BuildModel(command) => build(command),
        Commands::TestModel(command) => test(command),
        Commands::Spot(command) => spot(command),
        Commands::Devices(command) => devices(command),
    }
    .expect("Command failed");
}

use clap::{Parser, Subcommand};
mod record;
mod build_model;
mod test_model;
mod spot;
mod devices;
use self::{record::{record,RecordCommand}, build_model::{BuildModelCommand, build}, test_model::{test, TestModelCommand}, spot::{spot, SpotCommand}, devices::{devices, DevicesCommand}};
#[derive(Parser, Debug)]
/// RustPotter: the free personal wakeword spotter written on rust
#[clap(author, version, about, long_about = None, arg_required_else_help = true)]
struct CLI {
    #[clap(subcommand)]
    command: Option<Commands>,
}


#[derive(Subcommand, Debug)]
/// Record audio sample
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


pub fn run_cli() {
    let cli = CLI::parse();
    match cli.command.unwrap() {
        Commands::Record(command) => record(command),
        Commands::BuildModel(command) => build(command),
        Commands::TestModel(command) => test(command),
        Commands::Spot(command) => spot(command),
        Commands::Devices(command) => devices(command),
    }
}

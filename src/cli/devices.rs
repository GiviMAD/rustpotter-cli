extern crate cpal;
use clap::Args;
use cpal::traits::{DeviceTrait, HostTrait};
use gag::Gag;

use crate::cli::record::is_compatible_format;
/// List audio device configs
#[derive(Args, Debug)]
#[clap()]
pub struct DevicesCommand {
    #[clap(long, short)]
    /// Display available record formats by device
    configs: bool,
    #[clap(long, short)]
    /// Filter device configs by max channel number
    max_channels: Option<u16>,
    #[clap(short = 'w', long)]
    /// Display host warnings
    host_warnings: bool,
}
pub fn devices(command: DevicesCommand) -> Result<(), String> {
    let default_host = cpal::default_host();
    let stderr_gag = Gag::stderr().unwrap();
    if command.host_warnings {
        drop(stderr_gag);
    }
    println!("Audio hosts:\n  - {:?}", default_host.id());
    let default_in = default_host
        .default_input_device()
        .map(|e| e.name().unwrap());
    if let Some(def_in) = default_in {
        println!("Default input device:\n  - {}", def_in);
    } else {
        println!("No default input device");
    }
    let devices = default_host
        .input_devices()
        .map_err(|err| err.to_string())?;
    println!("Available Devices: ");
    for (device_index, device) in devices.enumerate() {
        println!(
            "{} - {}",
            device_index,
            device.name().map_err(|err| err.to_string())?
        );

        // Input configs
        if command.configs {
            if let Ok(default_config) = device.default_input_config() {
                println!("  Default input stream config:\n      - Sample Rate: {}, Channels: {}, Format: {}, Supported: {}", default_config.sample_rate().0, default_config.channels(), default_config.sample_format(), is_compatible_format(&default_config.sample_format()));
            }
        }
        if command.configs {
            let input_configs = match device.supported_input_configs() {
                Ok(f) => f.collect(),
                Err(e) => {
                    println!("  Error getting supported input configs: {:?}", e);
                    Vec::new()
                }
            };
            println!("  All supported input stream configs:");
            for (config_index, config) in input_configs.into_iter().enumerate() {
                if command.max_channels.is_none()
                    || config.channels() <= command.max_channels.unwrap()
                {
                    println!(
                        "    {} - Sample Rate: {} - {}, Channels: {}, Format: {}, Supported: {}",
                        config_index,
                        config.min_sample_rate().0,
                        config.max_sample_rate().0,
                        config.channels(),
                        config.sample_format(),
                        is_compatible_format(&config.sample_format())
                    );
                }
            }
        }
    }
    Ok(())
}

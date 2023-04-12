extern crate cpal;
use clap::Args;
use cpal::traits::{DeviceTrait, HostTrait};
/// List audio device configs
#[derive(Args, Debug)]
#[clap()]
pub struct DevicesCommand {
    #[clap(long)]
    /// Filter device configs by max channel number
    max_channels: Option<u16>,
}
pub fn devices(command: DevicesCommand) -> Result<(), String> {
    println!("Supported hosts:\n  {:?}", cpal::ALL_HOSTS);
    let default_host = cpal::default_host();
    let host_id = default_host.id();
    println!("Using hosts:\n  {:?}", default_host.id());
    println!("{}", host_id.name());
    let host = cpal::host_from_id(host_id).map_err(|err| err.to_string())?;
    let default_in = host.default_input_device().map(|e| e.name().unwrap());
    if let Some(def_in) = default_in {
        println!("  Default input device:\n    {}", def_in);
    } else {
        println!("  No default input device");
    }
    let devices = host.input_devices().map_err(|err| err.to_string())?;
    println!("  Devices: ");
    for (device_index, device) in devices.enumerate() {
        println!(
            "  {} - \"{}\"",
            device_index,
            device.name().map_err(|err| err.to_string())?
        );

        // Input configs
        if let Ok(conf) = device.default_input_config() {
            println!("    Default input stream config:\n      {:?}", conf);
        }
        let input_configs = match device.supported_input_configs() {
            Ok(f) => f.collect(),
            Err(e) => {
                println!("    Error getting supported input configs: {:?}", e);
                Vec::new()
            }
        };
        if !input_configs.is_empty() {
            println!("    All supported input stream configs:");
            for (config_index, config) in input_configs.into_iter().enumerate() {
                if command.max_channels.is_none() || config.channels() <= command.max_channels.unwrap() {
                    println!("      {} - {:?}", config_index, config);
                }
            }
        }
    }
    Ok(())
}

use clap::Args;
use pv_recorder::RecorderBuilder;

#[derive(Args, Debug)]
/// Record audio sample
#[clap()]
pub struct DevicesCommand {}
pub fn devices(_: DevicesCommand) {
    println!("Available record audio devices:");
    let recorder = RecorderBuilder::new()
        .init()
        .expect("Failed to initialize recorder");
    let audio_devices = recorder.get_audio_devices();
    match audio_devices {
        Ok(audio_devices) => {
            for (idx, device) in audio_devices.iter().enumerate() {
                println!("{}: {:?}", idx, device);
            }
        }
        Err(err) => panic!("Failed to get audio devices: {}", err),
    };
}

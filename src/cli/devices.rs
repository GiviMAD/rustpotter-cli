use clap::Args;
use pv_recorder::RecorderBuilder;
#[cfg(feature = "dist")]
use crate::pv_recorder_utils::_get_pv_recorder_lib;
#[derive(Args, Debug)]
/// Record audio sample
#[clap()]
pub struct DevicesCommand {}
pub fn devices(_: DevicesCommand) -> Result<(), String> {
    #[cfg(feature = "dist")]
    let mut recorder_builder = RecorderBuilder::new();
    #[cfg(not(feature = "dist"))]
    let recorder_builder = RecorderBuilder::new();
    #[cfg(feature = "dist")]
    let lib_temp_path = _get_pv_recorder_lib();
    #[cfg(feature = "dist")]
    recorder_builder.library_path(lib_temp_path.to_path_buf().as_path());
    let recorder =  recorder_builder.init()
    .expect("Failed to initialize recorder");
    println!("Available record audio devices:");
    let audio_devices = recorder.get_audio_devices();
    match audio_devices {
        Ok(audio_devices) => {
            for (idx, device) in audio_devices.iter().enumerate() {
                println!("{}: {:?}", idx, device);
            }
        }
        Err(err) => panic!("Failed to get audio devices: {}", err),
    };
    #[cfg(all(feature = "dist", not(target_os = "windows")))]
    lib_temp_path.close().expect("Unable to remove temp file");
    Ok(())
}

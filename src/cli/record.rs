use std::sync::atomic::{AtomicBool, Ordering};

use clap::Args;
use pv_recorder::RecorderBuilder;
#[cfg(not(debug_assertions))]
use crate::pv_recorder_utils::_get_pv_recorder_lib;
use rustpotter::{WakewordDetectorBuilder};

#[derive(Args, Debug)]
/// Record wav audio sample with spec 16000hz 16bit 1 channel int
#[clap()]
pub struct RecordCommand {
    #[clap()]
    /// Generated record path
    output_path: String,
    #[clap(short, long, default_value_t = 0)]
    /// Input device index used for record
    device_index: usize,
}
pub fn record(command: RecordCommand) -> Result<(), String> {
    let mut detector_builder = WakewordDetectorBuilder::new();
    detector_builder.set_sample_rate(16000);
    let detector = detector_builder.build();
    let mut recorder_builder = RecorderBuilder::new();
    recorder_builder.frame_length(detector.get_samples_per_frame() as i32);
    recorder_builder.device_index(command.device_index as i32);
    recorder_builder.log_overflow(false);
    #[cfg(not(debug_assertions))]
    let lib_temp_path = _get_pv_recorder_lib();
    #[cfg(not(debug_assertions))]
    recorder_builder.library_path(lib_temp_path.to_path_buf().as_path());
    let recorder = recorder_builder
        .device_index(command.device_index as i32)
        .init()
        .expect("Failed to initialize recorder");
    static LISTENING: AtomicBool = AtomicBool::new(false);
    ctrlc::set_handler(|| {
        LISTENING.store(false, Ordering::SeqCst);
    })
    .expect("Unable to setup signal handler");
    println!("Start recording...");
    recorder.start().expect("Failed to start audio recording");
    LISTENING.store(true, Ordering::SeqCst);
    let mut audio_data = Vec::new();
    let mut frame_buffer = vec![0; recorder.frame_length()];
    while LISTENING.load(Ordering::SeqCst) {
        recorder
            .read(&mut frame_buffer)
            .expect("Failed to read audio frame");
        audio_data.extend_from_slice(&frame_buffer);
    }

    println!("Stop recording...");
    recorder.stop().expect("Failed to stop audio recording");
    println!("Creating wav sample {}", command.output_path);
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: 16000,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut writer = hound::WavWriter::create(command.output_path, spec).unwrap();
    for sample in audio_data {
        writer.write_sample(sample).unwrap();
    }
    println!("Done");
    #[cfg(all(not(debug_assertions), not(target_os = "windows")))]
    lib_temp_path.close().expect("Unable to remove temp file");
    Ok(())
}

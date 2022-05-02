use std::sync::atomic::{AtomicBool, Ordering};

use clap::Args;
use pv_recorder::RecorderBuilder;
use rustpotter::detector;
#[cfg(not(debug_assertions))]
use crate::pv_recorder_utils::_get_pv_recorder_lib;
#[derive(Args, Debug)]
/// Spot keyword in audio
#[clap()]
pub struct SpotCommand {
    #[clap(min_values = 1, required = true)]
    /// Model path list
    model_path: Vec<String>,
    #[clap(short, long, default_value_t = 0.5)]
    /// Default detection threshold, only applies to models without threshold
    threshold: f32,
    #[clap(short, long, default_value_t = 0)]
    /// Input device index used for record
    device_index: usize,
    #[clap(short = 'a', long)]
    /// Enables template averaging
    average_templates: bool,
}

pub fn spot(command: SpotCommand) -> Result<(), String> {
    println!("Spotting using models: {:?}!", command.model_path);
    let mut detector_builder = detector::FeatureDetectorBuilder::new();
    detector_builder.set_threshold(command.threshold);
    detector_builder.set_sample_rate(16000);
    let mut word_detector = detector_builder.build();
    for path in command.model_path {
        let result = word_detector.add_keyword_from_model(path, command.average_templates, true);
        if result.is_err() {
            clap::Error::raw(clap::ErrorKind::InvalidValue, result.unwrap_err() + "\n").exit();
        }
    }
    let mut recorder_builder = RecorderBuilder::new();
    recorder_builder.frame_length((word_detector.get_samples_per_frame()) as i32);
    recorder_builder.buffer_size_msec(word_detector.get_samples_per_frame() as i32 * 2);
    recorder_builder.device_index(command.device_index as i32);
    recorder_builder.log_overflow(false);
    #[cfg(not(debug_assertions))]
    let lib_temp_file = _get_pv_recorder_lib();
    #[cfg(not(debug_assertions))]
    recorder_builder.library_path(lib_temp_file.path());
    let recorder = recorder_builder
        .init()
        .expect("Failed to initialize recorder");
    static LISTENING: AtomicBool = AtomicBool::new(false);
    ctrlc::set_handler(|| {
        LISTENING.store(false, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl-C handler");
    recorder.start().expect("Failed to start audio recording");
    LISTENING.store(true, Ordering::SeqCst);
    while LISTENING.load(Ordering::SeqCst) {
        let mut frame_buffer = vec![0; recorder.frame_length()];
        recorder
            .read(&mut frame_buffer)
            .expect("Failed to read audio frame");
        let detections = word_detector.process_pcm_signed(&frame_buffer);
        for detection in detections {
            println!(
                "Detected '{}' with score {}!",
                detection.wakeword, detection.score
            )
        }
    }
    println!("Stopped by user request");
    #[cfg(not(debug_assertions))]
    lib_temp_file.close().expect("Unable to remove temp file");
    Ok(())
}

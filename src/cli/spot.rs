use std::sync::atomic::{AtomicBool, Ordering};

#[cfg(not(debug_assertions))]
use crate::pv_recorder_utils::_get_pv_recorder_lib;
use crate::utils::enable_rustpotter_log;
use clap::Args;
use pv_recorder::RecorderBuilder;
use rustpotter::{VadMode, WakewordDetectorBuilder};
#[derive(Args, Debug)]
/// Spot wakeword processing wav audio with spec 16000hz 16bit 1 channel int
#[clap()]
pub struct SpotCommand {
    #[clap(min_values = 1, required = true)]
    /// Model path list
    model_path: Vec<String>,
    #[clap(short, long, default_value_t = 0.5)]
    /// Default detection threshold, only applies to models without threshold
    threshold: f32,
    #[clap(short, long)]
    /// Default detection averaged threshold, only applies to models without averaged threshold, defaults to threshold/2.
    averaged_threshold: Option<f32>,
    #[clap(short, long, default_value_t = 0)]
    /// Input device index used for record
    device_index: usize,
    #[clap(short = 'e', long)]
    /// Enables eager mode
    eager_mode: bool,
    #[clap(long)]
    /// Unless enabled the comparison against multiple wakewords run in separate threads, not applies when single wakeword
    single_thread: bool,
    #[clap(short = 'v', long, possible_values = ["low-bitrate", "quality", "aggressive", "very-aggressive"])]
    /// Enables using a vad detector to reduce computation on absence of voice sound
    vad_mode: Option<String>,
    #[clap(long, default_value_t = 3)]
    /// Seconds to disable the vad detector after voice is detected
    vad_delay: u16,
    #[clap(long, default_value_t = 0.5)]
    /// Voice/silence ratio in the last second to consider voice detected
    vad_sensitivity: f32,
    #[clap(long)]
    /// Enables rustpotter debug log
    debug: bool,
}

pub fn spot(command: SpotCommand) -> Result<(), String> {
    println!("Spotting using models: {:?}!", command.model_path);
    if command.debug {
        enable_rustpotter_log();
    }
    let mut detector_builder = WakewordDetectorBuilder::new();
    if command.averaged_threshold.is_some() {
        detector_builder.set_averaged_threshold(command.averaged_threshold.unwrap());
    }
    if command.vad_mode.is_some() {
        detector_builder
            .set_vad_mode(get_vad_mode(&command.vad_mode.unwrap()))
            .set_vad_delay(command.vad_delay)
            .set_vad_sensitivity(command.vad_sensitivity);
    }
    let mut word_detector = detector_builder
        .set_threshold(command.threshold)
        .set_sample_rate(16000)
        .set_eager_mode(command.eager_mode)
        .set_single_thread(command.single_thread)
        .build();
    for path in command.model_path {
        let result = word_detector.add_wakeword_from_model_file(path, true);
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
    let lib_temp_path = _get_pv_recorder_lib();
    #[cfg(not(debug_assertions))]
    recorder_builder.library_path(lib_temp_path.to_path_buf().as_path());
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
        let detections = word_detector.process_i16(&frame_buffer);
        for detection in detections {
            println!(
                "Detected '{}' with score {}!",
                detection.wakeword, detection.score
            )
        }
    }
    println!("Stopped by user request");
    #[cfg(all(not(debug_assertions), not(target_os = "windows")))]
    lib_temp_path.close().expect("Unable to remove temp file");
    Ok(())
}
fn get_vad_mode(name: &str) -> VadMode {
    match name {
        "low-bitrate" => VadMode::LowBitrate,
        "quality" => VadMode::Quality,
        "aggressive" => VadMode::Aggressive,
        "very-aggressive" => VadMode::VeryAggressive,
        _ => clap::Error::raw(clap::ErrorKind::InvalidValue, "Unsupported vad mode.\n").exit(),
    }
}

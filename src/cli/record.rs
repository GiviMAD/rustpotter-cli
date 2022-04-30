use std::sync::atomic::{AtomicBool, Ordering};

use clap::Args;
use pv_recorder::RecorderBuilder;

use rustpotter::detector;

#[derive(Args, Debug)]
/// Record audio sample
#[clap()]
pub struct RecordCommand {
    #[clap()]
    /// Generated record path
    output_path: String,
    #[clap(short, long, default_value_t = 0)]
    /// Input device index used for record
    device_index: usize,
    /// Sample frame length ms
    #[clap(short='l', long, default_value_t = 30)]
    frame_length_ms: usize,
}
pub fn record(command: RecordCommand) {
    let mut detector_builder = detector::FeatureDetectorBuilder::new();
    detector_builder.set_sample_rate(16000);
    let detector = detector_builder.build();
    let mut recorder_builder = RecorderBuilder::new();
    recorder_builder.frame_length(detector.get_samples_per_frame() as i32);
    recorder_builder.device_index(command.device_index as i32);
    recorder_builder.log_overflow(false);
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
}

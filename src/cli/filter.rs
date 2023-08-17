use clap::Args;
use hound::{Sample, SampleFormat, WavReader};
use rustpotter::{
    BandPassFilter, Endianness, GainNormalizerFilter, WAVEncoder, WavFmt, SampleType,
    DETECTOR_INTERNAL_SAMPLE_RATE, MFCCS_EXTRACTOR_FRAME_LENGTH_MS,
};
use std::{fs::File, io::BufReader, path::Path};

#[derive(Args, Debug)]
/// Apply the audio filters to a file
#[clap()]
pub struct FilterCommand {
    #[clap()]
    /// Wav record to apply filters to.
    sample_path: String,
    #[clap(short = 'g', long)]
    /// Enables a gain-normalizer audio filter.
    gain_normalizer: bool,
    #[clap(long, default_value_t = 0.1)]
    /// Min gain applied by the gain-normalizer filter.
    min_gain: f32,
    #[clap(long, default_value_t = 1.)]
    /// Max gain applied by the gain-normalizer filter.
    max_gain: f32,
    #[clap(long, default_value_t = 0.005)]
    /// Set the rms level reference used by the gain normalizer filter.
    gain_ref: f32,
    #[clap(short, long)]
    /// Enables a band-pass audio filter.
    band_pass: bool,
    #[clap(long, default_value_t = 80.)]
    /// Band-pass audio filter low cutoff.
    low_cutoff: f32,
    #[clap(long, default_value_t = 400.)]
    /// Band-pass audio filter high cutoff.
    high_cutoff: f32,
}
pub fn filter(command: FilterCommand) -> Result<(), String> {
    if !command.band_pass && !command.gain_normalizer {
        clap::Error::raw(
            clap::error::ErrorKind::ValueValidation,
            "You need to enable at least one audio filter.\n",
        )
        .exit();
    };
    let path = Path::new(&command.sample_path);
    let mut filtered_filename = path.file_stem().unwrap().to_owned();
    if command.gain_normalizer {
        filtered_filename.push("-gain");
        filtered_filename.push(command.gain_ref.to_string());
    }
    if command.band_pass {
        filtered_filename.push("-bandpass");
        filtered_filename.push(command.low_cutoff.to_string());
        filtered_filename.push("_");
        filtered_filename.push(command.high_cutoff.to_string());
    }
    filtered_filename.push(".wav");
    println!("Creating new file {}", filtered_filename.to_str().unwrap(),);
    // Read wav file
    let file_reader =
        BufReader::new(File::open(command.sample_path).map_err(|err| err.to_string())?);
    let mut wav_reader = WavReader::new(file_reader).map_err(|err| err.to_string())?;
    let wav_spec = WavFmt {
        sample_rate: wav_reader.spec().sample_rate as usize,
        sample_format: wav_reader.spec().sample_format,
        bits_per_sample: wav_reader.spec().bits_per_sample,
        channels: wav_reader.spec().channels,
        endianness: Endianness::Little,
    };
    let mut encoder = WAVEncoder::new(
        &wav_spec,
        MFCCS_EXTRACTOR_FRAME_LENGTH_MS,
        DETECTOR_INTERNAL_SAMPLE_RATE,
    )
    .unwrap();
    let internal_spec = hound::WavSpec {
        sample_rate: DETECTOR_INTERNAL_SAMPLE_RATE as u32,
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Float,
        channels: 1,
    };
    let mut writer = hound::WavWriter::create(filtered_filename, internal_spec).unwrap();
    let mut gain_filter = GainNormalizerFilter::new(0.1, 1., Some(command.gain_ref));
    let mut bandpass_filter = BandPassFilter::new(
        DETECTOR_INTERNAL_SAMPLE_RATE as f32,
        command.low_cutoff,
        command.high_cutoff,
    );
    if wav_reader.spec().sample_format == SampleFormat::Float {
        get_encoded_chucks::<f32>(&mut wav_reader, &mut encoder)
    } else {
        match wav_spec.bits_per_sample {
            8 => get_encoded_chucks::<i8>(&mut wav_reader, &mut encoder),
            16 => get_encoded_chucks::<i16>(&mut wav_reader, &mut encoder),
            32 => get_encoded_chucks::<i32>(&mut wav_reader, &mut encoder),
            _ => panic!("Unsupported wav format"),
        }
    }
    .into_iter()
    .map(|mut chunk| {
        if command.gain_normalizer {
            let rms_level = GainNormalizerFilter::get_rms_level(&chunk);
            gain_filter.filter(&mut chunk, rms_level);
        }
        if command.band_pass {
            bandpass_filter.filter(&mut chunk);
        }
        chunk
    })
    .for_each(|encoded_chunk| {
        for sample in encoded_chunk {
            writer.write_sample(sample).ok();
        }
    });
    writer.finalize().expect("Unable to save file");
    Ok(())
}

fn get_encoded_chucks<T: Sample + SampleType>(
    wav_reader: &mut WavReader<BufReader<File>>,
    encoder: &mut WAVEncoder,
) -> Vec<Vec<f32>> {
    wav_reader
        .samples::<T>()
        .map(|chunk| *chunk.as_ref().unwrap())
        .collect::<Vec<_>>()
        .chunks_exact(encoder.get_input_frame_length())
        .map(|chuck| encoder.rencode_and_resample(chuck.to_vec()))
        .collect::<Vec<Vec<f32>>>()
}

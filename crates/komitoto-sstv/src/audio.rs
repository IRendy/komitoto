use crate::error::SstvError;
use symphonia::core::audio::{AudioBufferRef, Signal};
use symphonia::core::codecs::DecoderOptions;
use symphonia::core::errors::Error as SymphoniaError;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;
use symphonia::default::get_probe;

/// Read a WAV file into PCM samples (f32, [-1.0, 1.0]).
pub fn read_wav(path: &str) -> Result<(Vec<f32>, u32), SstvError> {
    let mut reader = hound::WavReader::open(path)?;
    let spec = reader.spec();
    let sample_rate = spec.sample_rate;

    let samples: Vec<f32> = match spec.sample_format {
        hound::SampleFormat::Int => {
            let bits = spec.bits_per_sample;
            if bits <= 16 {
                reader
                    .samples::<i16>()
                    .map(|s| s.map(|v| v as f32 / i16::MAX as f32))
                    .collect::<Result<Vec<f32>, _>>()?
            } else {
                reader
                    .samples::<i32>()
                    .map(|s| s.map(|v| v as f32 / i32::MAX as f32))
                    .collect::<Result<Vec<f32>, _>>()?
            }
        }
        hound::SampleFormat::Float => reader
            .samples::<f32>()
            .map(|s| s.map(|v| v.clamp(-1.0, 1.0)))
            .collect::<Result<Vec<f32>, _>>()?,
    };

    Ok((samples, sample_rate))
}

/// Write PCM samples (f32, [-1.0, 1.0]) to a 16-bit PCM WAV file.
pub fn write_wav(path: &str, samples: &[f32], sample_rate: u32) -> Result<(), SstvError> {
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut writer = hound::WavWriter::create(path, spec)?;
    let amplitude = i16::MAX as f32;
    for &sample in samples {
        let val = (sample * amplitude).clamp(i16::MIN as f32, i16::MAX as f32);
        writer.write_sample(val as i16)?;
    }
    writer.finalize()?;
    Ok(())
}

/// Read an MP3 file into PCM samples (f32, [-1.0, 1.0]).
pub fn read_mp3(path: &str) -> Result<(Vec<f32>, u32), SstvError> {
    let file = std::fs::File::open(path)?;
    let mss = MediaSourceStream::new(Box::new(file), Default::default());

    let probed = get_probe().format(
        &Hint::new(),
        mss,
        &FormatOptions::default(),
        &MetadataOptions::default(),
    )?;
    let mut format = probed.format;

    let track = format.default_track().ok_or_else(|| {
        SstvError::DecodingError("MP3 file has no audio track".into())
    })?;
    let sample_rate = track.codec_params.sample_rate.unwrap_or(0);

    let mut decoder = symphonia::default::get_codecs()
        .make(&track.codec_params, &DecoderOptions::default())?;

    let mut samples = Vec::new();

    while let Ok(packet) = format.next_packet() {
        let decoded = decoder.decode(&packet)?;
        let spec = decoded.spec();
        let num_channels = spec.channels.count();

        // Get the frame count from the first channel's actual length
        let frames = match &decoded {
            AudioBufferRef::F32(buf) => buf.chan(0).len(),
            AudioBufferRef::U8(buf) => buf.chan(0).len(),
            AudioBufferRef::U16(buf) => buf.chan(0).len(),
            AudioBufferRef::U24(buf) => buf.chan(0).len(),
            AudioBufferRef::U32(buf) => buf.chan(0).len(),
            AudioBufferRef::S8(buf) => buf.chan(0).len(),
            AudioBufferRef::S16(buf) => buf.chan(0).len(),
            AudioBufferRef::S24(buf) => buf.chan(0).len(),
            AudioBufferRef::S32(buf) => buf.chan(0).len(),
            AudioBufferRef::F64(buf) => buf.chan(0).len(),
        };

        for f in 0..frames {
            let mut sum = 0.0f32;
            match &decoded {
                AudioBufferRef::F32(buf) => {
                    for c in 0..num_channels {
                        sum += buf.chan(c)[f];
                    }
                }
                AudioBufferRef::U8(buf) => {
                    for c in 0..num_channels {
                        sum += buf.chan(c)[f] as f32 / 128.0 - 1.0;
                    }
                }
                AudioBufferRef::U16(buf) => {
                    for c in 0..num_channels {
                        sum += buf.chan(c)[f] as f32 / 32768.0;
                    }
                }
                AudioBufferRef::U24(buf) => {
                    for c in 0..num_channels {
                        sum += (buf.chan(c)[f].inner() as f32) / 8388608.0;
                    }
                }
                AudioBufferRef::U32(buf) => {
                    for c in 0..num_channels {
                        sum += buf.chan(c)[f] as f32 / i32::MAX as f32;
                    }
                }
                AudioBufferRef::S8(buf) => {
                    for c in 0..num_channels {
                        sum += buf.chan(c)[f] as f32 / 128.0;
                    }
                }
                AudioBufferRef::S16(buf) => {
                    for c in 0..num_channels {
                        sum += buf.chan(c)[f] as f32 / i16::MAX as f32;
                    }
                }
                AudioBufferRef::S24(buf) => {
                    for c in 0..num_channels {
                        sum += (buf.chan(c)[f].inner() as f32) / 8388608.0;
                    }
                }
                AudioBufferRef::S32(buf) => {
                    for c in 0..num_channels {
                        sum += buf.chan(c)[f] as f32 / i32::MAX as f32;
                    }
                }
                AudioBufferRef::F64(buf) => {
                    for c in 0..num_channels {
                        sum += buf.chan(c)[f] as f32;
                    }
                }
            }
            samples.push(sum / num_channels as f32);
        }
    }

    if sample_rate == 0 {
        return Err(SstvError::DecodingError(
            "Could not determine MP3 sample rate".into(),
        ));
    }

    Ok((samples, sample_rate))
}

/// Read an audio file (auto-detect format by extension).
/// Supports .wav and .mp3.
pub fn read_audio(path: &str) -> Result<(Vec<f32>, u32), SstvError> {
    let lower = path.to_lowercase();
    if lower.ends_with(".mp3") {
        read_mp3(path)
    } else if lower.ends_with(".wav") {
        read_wav(path)
    } else {
        Err(SstvError::DecodingError(format!(
            "Unsupported audio format: {} (supported: wav, mp3)",
            path
        )))
    }
}

/// Resample audio from one sample rate to another using linear interpolation.
pub fn resample(samples: &[f32], from_rate: u32, to_rate: u32) -> Vec<f32> {
    if from_rate == to_rate {
        return samples.to_vec();
    }

    let ratio = to_rate as f64 / from_rate as f64;
    let new_len = (samples.len() as f64 * ratio).round() as usize;
    let mut out = Vec::with_capacity(new_len);

    for i in 0..new_len {
        let src_pos = i as f64 / ratio;
        let idx = src_pos as usize;
        let frac = src_pos - idx as f64;

        let a = samples.get(idx).copied().unwrap_or(0.0);
        let b = samples.get(idx + 1).copied().unwrap_or(a);
        out.push(a + (b - a) * frac as f32);
    }

    out
}

impl From<SymphoniaError> for SstvError {
    fn from(e: SymphoniaError) -> Self {
        SstvError::DecodingError(format!("Audio decode error: {}", e))
    }
}

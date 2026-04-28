use crate::error::SstvError;

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

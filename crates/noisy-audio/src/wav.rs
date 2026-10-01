use crate::Result;
use std::path::Path;

pub const WAV_DEFAULT_RATE: u32 = 48_000;
/// Writes a mono WAV file with 32-bit float samples in [-1, 1]
pub fn write_wav(path: &Path, samples: &[f32], sample_rate: u32) -> Result<()> {
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate,
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Float,
    };
    let mut w = hound::WavWriter::create(path, spec)?;
    for &s in samples {
        w.write_sample(s)?;
        // If for some reason I change the amplitude to be > 1.0, I should clamp it to avoid distortion
    }
    w.finalize()?;
    Ok(())
}

/// Reads channel 0 as f32 in [-1, 1], plus the sample rate
pub fn read_wav(path: &Path) -> Result<(Vec<f32>, u32)> {
    let mut r = hound::WavReader::open(path)?;
    let spec = r.spec();
    let all: Vec<f32> = match spec.sample_format {
        hound::SampleFormat::Float => r.samples::<f32>().collect::<std::result::Result<_, _>>()?,
        hound::SampleFormat::Int => {
            let scale = (1i64 << (spec.bits_per_sample - 1)) as f32;
            r.samples::<i32>()
                .map(|s| s.map(|v| v as f32 / scale))
                .collect::<std::result::Result<_, _>>()?
        }
    };
    Ok((
        all.into_iter().step_by(spec.channels as usize).collect(),
        spec.sample_rate,
    ))
}

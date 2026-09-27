/// RMS of the signal around its mean, so a microphone's DC offset reads as silence.
pub fn rms(samples: &[f32]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }
    let mean = samples.iter().sum::<f32>() / samples.len() as f32;
    (samples.iter().map(|x| (x - mean) * (x - mean)).sum::<f32>() / samples.len() as f32).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rms_ignores_dc_offset() {
        // A silent mic that sits at a constant offset, like Mic1 (-0.074).
        assert!(rms(&[-0.074; 480]) < 1e-6);
    }
}

use std::f32::consts::PI;

/// Power of one frequency in a window of samples.
pub struct Goertzel {
    coeff: f32,
}

impl Goertzel {
    pub fn new(freq: f32, sample_rate: u32) -> Self {
        Self {
            coeff: 2.0 * (2.0 * PI * freq / sample_rate as f32).cos(),
        }
    }
    /// Power of the frequency in the given samples.
    pub fn power(&self, samples: &[f32]) -> f32 {
        let (mut s1, mut s2) = (0.0f32, 0.0f32);
        for &x in samples {
            let s = self.coeff * s1 - s2 + x;
            s2 = s1;
            s1 = s;
        }
        s1 * s1 + s2 * s2 - self.coeff * s1 * s2
    }
}

/// Hann window
pub fn hann(buffer: &mut [f32]) {
    let n = buffer.len();
    if n <= 1 {
        return;
    }
    let step = 2.0 * PI / (n as f32 - 1.0);
    for (i, x) in buffer.iter_mut().enumerate() {
        *x *= 0.5 * (1.0 - (i as f32 * step).cos());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    /// Test goertzel in an obvious comparison
    #[test]
    fn detects_target_frequency() {
        let sample_rate = 48_000;
        let target_freq = 1_000.0;
        let detector = Goertzel::new(target_freq, sample_rate);
        let samples_1k: Vec<f32> = (0..4800)
            .map(|n| (2.0 * PI * 1_000.0 * n as f32 / sample_rate as f32).sin())
            .collect();
        let samples_3k: Vec<f32> = (0..4800)
            .map(|n| (2.0 * PI * 3_000.0 * n as f32 / sample_rate as f32).sin())
            .collect();
        let power_1k = detector.power(&samples_1k);
        let power_3k = detector.power(&samples_3k);
        assert!(power_1k > power_3k);
    }
}

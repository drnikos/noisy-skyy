use crate::{config::ModemConfig, detect::goertzel::*};
use std::f32::consts::PI;

pub struct Demodulator {
    zero: Goertzel,
    one: Goertzel,
}

impl Demodulator {
    pub fn new(cfg: &ModemConfig, sample_rate: u32) -> Self {
        Self {
            zero: Goertzel::new(cfg.zero_freq, sample_rate),
            one: Goertzel::new(cfg.one_freq, sample_rate),
        }
    }

    pub fn decide(&self, window: &mut [f32]) -> u8 {
        hann(window); // Not sure if its necessary
        if self.one.power(window) > self.zero.power(window) {
            1
        } else {
            0
        }
    }
}
/// Takes a slice of bits and outputs a vector of samples
pub fn modulate(bits: &[u8], cfg: &ModemConfig, sample_rate: u32) -> Vec<f32> {
    let samples_per_bit = cfg.samples_per_bit(sample_rate);
    let mut out = Vec::with_capacity(bits.len() * samples_per_bit);
    let mut phase = 0.0f32;

    // Precompute phase steps
    let zerostep = 2.0 * PI * cfg.zero_freq / sample_rate as f32;
    let onestep = 2.0 * PI * cfg.one_freq / sample_rate as f32;

    for &bit in bits {
        let step = if bit == 0 { zerostep } else { onestep };
        for _ in 0..samples_per_bit {
            out.push(phase.sin() * cfg.amplitude);
            phase = (phase + step) % (2.0 * PI);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_bit_is_samples_per_bit_long_and_within_amplitude() {
        let cfg = ModemConfig::default();
        let s = modulate(&[0, 1], &cfg, 48_000);
        assert_eq!(s.len(), 2 * 480); // Just a sanity check, 480 samples per bit at 48kHz and 10ms per bit
        assert!(s.iter().all(|x| x.abs() <= cfg.amplitude)); //Multiplication with sin should not exceed amplitude
    }
}

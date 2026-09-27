#[derive(Debug, Clone)]
pub struct ModemConfig {
    /// "0" bit (Hz)
    pub zero_freq: f32,
    /// "1" bit (Hz)
    pub one_freq: f32,
    /// Duration of each bit (ms)
    pub bit_duration_ms: u64,
    pub amplitude: f32,
    pub silence_threshold: f32,
}

impl Default for ModemConfig {
    fn default() -> Self {
        Self {
            zero_freq: 15_900.0,
            one_freq: 16_900.0,
            bit_duration_ms: 10,
            amplitude: 0.5,
            silence_threshold: 0.04,
        }
    }
}

impl ModemConfig {
    pub fn samples_per_bit(&self, sample_rate: u32) -> usize {
        (sample_rate as f32 * (self.bit_duration_ms as f32 / 1000.0)).round() as usize
    }
}

use crate::{compress, config::ModemConfig, error::CoreError, frame, modulation::bfsk};

pub fn frame_bits(payload: &[u8]) -> Result<Vec<u8>, CoreError> {
    let compressed = compress::compress(payload)?;
    Ok(frame::build_frame(&compressed))
}
/// Produces the samples to be played for a given payload
pub fn samples(payload: &[u8], cfg: &ModemConfig, sample_rate: u32) -> Result<Vec<f32>, CoreError> {
    let bits = frame_bits(payload)?;
    Ok(bfsk::modulate(&bits, cfg, sample_rate))
}

use crate::{compress, config::ModemConfig, frame, modulation::bfsk};
use std::io;

pub fn frame_bits(payload: &[u8]) -> io::Result<Vec<u8>> {
    let compressed = compress::compress(payload)?;
    Ok(frame::build_frame(&compressed))
}
/// Produces the samples to be played for a given payload
pub fn samples(payload: &[u8], cfg: &ModemConfig, sample_rate: u32) -> io::Result<Vec<f32>> {
    let bits = frame_bits(payload)?;
    Ok(bfsk::modulate(&bits, cfg, sample_rate))
}

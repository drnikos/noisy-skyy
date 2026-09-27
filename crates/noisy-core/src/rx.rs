//! RX pipeline: audio samples in, events out. Push-based, no I/O.
use crate::{
    compress,
    config::ModemConfig,
    detect::energy::rms,
    frame::{Deframed, Deframer},
    modulation::bfsk::Demodulator,
    sync::preamble::PreambleDetector,
};

pub enum RxEvent {
    SyncFound,
    Frame(Vec<u8>),
    BadFrame { raw: Vec<u8>, error: std::io::Error },
}

enum State {
    Searching,
    Payload,
    EndFlag,
}

pub struct Receiver {
    state: State,
    window: Vec<f32>,
    filled: usize,
    silence_threshold: f32,
    demod: Demodulator,
    preamble: PreambleDetector,
    deframer: Deframer,
    bytes: Vec<u8>,
}

impl Receiver {
    pub fn new(cfg: &ModemConfig, sample_rate: u32) -> Self {
        Self {
            state: State::Searching,
            window: vec![0.0; cfg.samples_per_bit(sample_rate)],
            filled: 0,
            silence_threshold: cfg.silence_threshold,
            demod: Demodulator::new(cfg, sample_rate),
            preamble: PreambleDetector::default(),
            deframer: Deframer::default(),
            bytes: Vec::new(),
        }
    }

    pub fn push(&mut self, samples: &[f32], events: &mut Vec<RxEvent>) {
        for &s in samples {
            self.window[self.filled] = s;
            self.filled += 1;
            if self.filled == self.window.len() {
                self.filled = 0;
                if rms(&self.window) < self.silence_threshold {
                    continue;
                }
                let bit = self.demod.decide(&mut self.window);
                self.on_bit(bit, events);
            }
        }
    }

    fn on_bit(&mut self, bit: u8, events: &mut Vec<RxEvent>) {
        match self.state {
            State::Searching => {
                if self.preamble.push_bit(bit) {
                    self.deframer = Deframer::default();
                    self.state = State::Payload;
                    events.push(RxEvent::SyncFound);
                }
            }
            State::Payload => match self.deframer.push_bit(bit) {
                Deframed::Byte(b) => self.bytes.push(b),
                Deframed::End => self.state = State::EndFlag,
                Deframed::Nothing => {}
            },
            State::EndFlag => {
                let raw = std::mem::take(&mut self.bytes);
                events.push(match compress::decompress(&raw) {
                    Ok(data) => RxEvent::Frame(data),
                    Err(error) => RxEvent::BadFrame { raw, error },
                });
                self.state = State::Searching;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn roundtrip_in_memory() {
        let cfg = ModemConfig::default();
        let samples = crate::tx::samples(b"Zdravstvuyte", &cfg, 48_000).unwrap();
        let mut rx = Receiver::new(&cfg, 48_000);
        let mut events = Vec::new();
        rx.push(&samples, &mut events);

        assert!(
            events
                .iter()
                .any(|e| matches!(e, RxEvent::Frame(d) if d == b"Zdravstvuyte"))
        );
    }

    // Test if quite signal is ignored
    #[test]
    fn quiet_preamble_is_ignored() {
        use crate::{modulation::bfsk, sync::preamble::PREAMBLE_ARRAY};
        let quiet = ModemConfig {
            amplitude: 0.1,
            ..ModemConfig::default()
        };
        let samples = bfsk::modulate(&PREAMBLE_ARRAY, &quiet, 48_000);
        let mut rx = Receiver::new(&ModemConfig::default(), 48_000);
        let mut events = Vec::new();
        rx.push(&samples, &mut events);
        assert!(events.is_empty());
    }
}

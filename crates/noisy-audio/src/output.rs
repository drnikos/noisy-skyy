use crate::{AudioError, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::time::Duration;
use tracing::{debug, error};

pub struct Output {
    device: cpal::Device,
    config: cpal::StreamConfig,
}

impl Output {
    pub fn open_default() -> Result<Self> {
        let device = cpal::default_host()
            .default_output_device()
            .ok_or(AudioError::NoDevice("output"))?;
        let config: cpal::StreamConfig = device.default_output_config()?.into();
        debug!(
            sample_rate = config.sample_rate,
            channels = config.channels,
            "opened output device"
        );
        Ok(Self { device, config })
    }

    pub fn sample_rate(&self) -> u32 {
        self.config.sample_rate
    }

    /// Plays samples (mono) on every channel and returns when they are done.
    pub fn play_blocking(&self, samples: Vec<f32>) -> Result<()> {
        let channels = self.config.channels as usize;
        let duration = Duration::from_secs_f32(samples.len() as f32 / self.sample_rate() as f32)
            + Duration::from_millis(100);
        let mut next = samples.into_iter();
        let stream = self.device.build_output_stream(
            self.config,
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                for frame in data.chunks_mut(channels) {
                    frame.fill(next.next().unwrap_or(0.0)); //Fill with silence if no more samples exist
                }
            },
            |err| error!("output stream error: {err}"),
            None,
        )?;
        stream.play()?;
        std::thread::sleep(duration);
        Ok(())
    }
}

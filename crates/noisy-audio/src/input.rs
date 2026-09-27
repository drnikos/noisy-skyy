use crate::Result;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

pub struct Input {
    device: cpal::Device,
    config: cpal::StreamConfig,
}

impl Input {
    pub fn open_default() -> Result<Self> {
        let device = cpal::default_host()
            .default_input_device()
            .ok_or("no input device found")?;
        let config = device.default_input_config()?.into();
        Ok(Self { device, config })
    }
    pub fn sample_rate(&self) -> u32 {
        self.config.sample_rate
    }

    pub fn run(&self, mut on_samples: impl FnMut(&[f32]) + Send + 'static) -> Result<()> {
        let channels = self.config.channels as usize;
        let mut mono = Vec::new();
        let stream = self.device.build_input_stream(
            self.config,
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                mono.clear();
                mono.extend(data.iter().step_by(channels).copied());
                on_samples(&mono);
            },
            |err| eprintln!("input stream error: {err}"),
            None,
        )?;
        stream.play()?;
        loop {
            std::thread::park();
        }
    }
}

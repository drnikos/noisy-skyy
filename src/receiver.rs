use crate::constants::*;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use ctrlc::set_handler;
use std::f32::consts::PI;
use std::sync::{Arc, Mutex};

#[derive(Debug, PartialEq)]
enum DecodeStage {
    PrePreamble,
    DataCollection,
}

struct AudioInput {
    device: cpal::Device,
    config: cpal::StreamConfig,
}

struct DecoderStats {
    stage: DecodeStage,
    buffer: Vec<f32>,
    samples_per_bit: u32,
    channels: u8,
    current_remainder_sample: usize,
    preamble_sync_register: u64,
    preamble_asu64: u64,
    bit_index: usize,
    bit_buffer: u8, //To assemble Bit
    goertzel_c: [f32; 2],
}

impl DecoderStats {
    fn new(samples_per_bit: u32, channels: u8, sample_rate: u32) -> Self {
        let mut target = 0u64;
        for &bit in PREAMBLE_ARRAY.iter() {
            target = (target << 1) | (bit as u64);
        }
        Self {
            stage: DecodeStage::PrePreamble,
            buffer: vec![0.0; samples_per_bit as usize],
            samples_per_bit,
            channels,
            current_remainder_sample: 0,
            preamble_sync_register: 0,
            preamble_asu64: target,
            bit_index: 0,
            bit_buffer: 0,
            goertzel_c: [
                2.0 * (2.0 * PI * ZERO_FREQ / sample_rate as f32).cos(),
                2.0 * (2.0 * PI * ONE_FREQ / sample_rate as f32).cos(),
            ],
        }
    }

    fn process_sample(&mut self, sample: f32) {
        self.buffer[self.current_remainder_sample as usize] = sample;
        self.current_remainder_sample += 1;
        if self.current_remainder_sample == self.buffer.len() {
            self.current_remainder_sample = 0;
            hann_smoothing(&mut self.buffer);
            //Process bit
            if let Some(bit) = goertzel(&self.buffer, &self) {
                self.process_bit(bit);
            }
        }
    }

    fn process_bit(&mut self, bit: u8) {
        match self.stage {
            DecodeStage::PrePreamble => {
                self.preamble_sync_register = (self.preamble_sync_register << 1) | (bit as u64);
                if (self.preamble_sync_register & PREAMBLE_MASK) == self.preamble_asu64 {
                    println!("PREAMBLE FOUND!");
                    self.stage = DecodeStage::DataCollection;
                    self.bit_index = 0;
                    self.bit_buffer = 0;
                }
            }
            DecodeStage::DataCollection => {
                self.bit_buffer = (self.bit_buffer << 1) | bit;
                self.bit_index += 1;
                if self.bit_index == 8 {
                    print!("{}", self.bit_buffer as char);
                    self.bit_index = 0;
                    self.bit_buffer = 0;
                }
            }
        }
    }
}

//Select input device
fn configure_input_device() -> Result<AudioInput, Box<dyn std::error::Error>> {
    let host = cpal::default_host();
    let device = host
        .default_input_device()
        .ok_or("Error finding Input device")?;
    let config = device.default_input_config()?.into();
    Ok(AudioInput { device, config })
}

fn goertzel(stream: &[f32], coef: &DecoderStats) -> Option<u8> {
    if stream.is_empty() {
        eprintln!("Found stream length 0!");
        return None;
    }
    let mut best = -1.0f32;
    let mut res = 0u8;
    for (ind, _) in [ZERO_FREQ, ONE_FREQ].iter().enumerate() {
        let mut vm2 = 0f32;
        let mut vm1 = 0.0f32;
        for j in stream.iter() {
            let v = coef.goertzel_c[ind] * vm1 - vm2 + *j;
            vm2 = vm1;
            vm1 = v;
        }
        let power = vm1 * vm1 + vm2 * vm2 - coef.goertzel_c[ind] * vm2 * vm1;
        if power > best {
            best = power;
            res = ind as u8;
        }
    }

    Some(res)
}

//Hann Window multiplier to smooth the buffer (Doesnt work yet)
fn hann_smoothing(buffer: &mut [f32]) {
    let n = buffer.len();
    if n <= 1 {
        return;
    }
    let mut counter = 0.0f32;
    let std_phase = 2.0 * std::f32::consts::PI / (n as f32 - 1.0);
    for i in buffer.iter_mut() {
        *i = *i * 0.5 * (1.0 - (counter * std_phase).cos());
        counter += 1.0;
    }
}

///Receive the input and decode it to data
pub fn receive() -> Result<(), Box<dyn std::error::Error>> {
    set_handler(|| {
        std::process::exit(0);
    })
    .unwrap();
    let input = configure_input_device()?;
    let channels = input.config.channels as u8;
    let samples_per_bit = (input.config.sample_rate as u64 * BIT_DURATION_MS / 1000) as u32;

    let stats = Arc::new(Mutex::new(DecoderStats::new(
        samples_per_bit,
        channels,
        input.config.sample_rate,
    )));
    let stats_for_stream = stats.clone();

    let stream = input.device.build_input_stream(
        input.config,
        move |data: &[f32], _: &cpal::InputCallbackInfo| {
            let mut stats_callback = stats_for_stream.lock().unwrap();

            for i in data.iter().step_by(channels as usize) {
                stats_callback.process_sample(*i);
            }
        },
        move |err| {
            eprintln!("Something went wrong, {err}");
        },
        None,
    )?;
    stream.play()?;
    std::thread::sleep(std::time::Duration::from_secs(u64::MAX));
    Ok(())
}

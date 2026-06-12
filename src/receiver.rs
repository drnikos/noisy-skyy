use std::io::Write;

use crate::constants::*;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use ctrlc::set_handler;
use rustfft::{FftPlanner, num_complex::Complex};

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
    planner: FftPlanner<f32>, //Will probably replace with Goertzel Algorithm :)
    buffer: Vec<f32>,
    samples_per_bit: u32,
    channels: u8,
    current_remainder_sample: usize,
    preamble_sync_register: u64,
    preamble_asu64: u64,
    bit_index: usize,
    bit_buffer: u8, //To assemble Bit
}

impl DecoderStats {
    fn new(samples_per_bit: u32, channels: u8) -> Self {
        let mut target = 0u64;
        for &byte in PREAMBLE_ARRAY.iter() {
            target = (target << 1) | (byte as u64);
        }
        Self {
            stage: DecodeStage::PrePreamble,
            planner: FftPlanner::new(),
            buffer: vec![0.0; samples_per_bit as usize],
            samples_per_bit,
            channels,
            current_remainder_sample: 0,
            preamble_sync_register: 0,
            preamble_asu64: target,
            bit_index: 0,
            bit_buffer: 0,
        }
    }

    fn process_sample(&mut self, sample: f32, sample_rate: u32) {
        self.buffer[self.current_remainder_sample as usize] = sample;
        self.current_remainder_sample += 1;
        if self.current_remainder_sample == self.buffer.len() {
            self.current_remainder_sample = 0;
            //Process bit
            if let Some(freq) = fft_window(&self.buffer, sample_rate, &mut self.planner) {
                if let Some(bit) = decode_to_bits(freq) {
                    self.process_bit(bit);
                }
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
                    std::io::stdout().flush().unwrap();
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
//Translate a frequency to data bits
fn decode_to_bits(key_freq: f32) -> Option<u8> {
    if key_freq > ONE_FREQ - FREQ_TOLERANCE && key_freq < ONE_FREQ + FREQ_TOLERANCE {
        Some(1)
    } else if key_freq > ZERO_FREQ - FREQ_TOLERANCE && key_freq < ZERO_FREQ + FREQ_TOLERANCE {
        Some(0)
    } else {
        //eprintln!("Didn't get valid freq range, got {key_freq}!");
        None
    }
}

//Perform Fast Fourier Transform and return the leading frequency
fn fft_window(stream: &[f32], sample_rate: u32, planner: &mut FftPlanner<f32>) -> Option<f32> {
    let stream_len = stream.len();
    let mut stream_fft: Vec<Complex<f32>> =
        stream.iter().map(|&x| Complex { re: x, im: 0.0 }).collect();
    let fft = planner.plan_fft_forward(stream_len);
    fft.process(&mut stream_fft);
    let mut max_index = 1;
    let mut max_mag = stream_fft[1].norm_sqr();
    for i in 2..stream_len / 2 {
        let mag = stream_fft[i].norm_sqr();
        if mag > max_mag {
            max_mag = mag;
            max_index = i;
        }
    }

    Some(max_index as f32 * sample_rate as f32 / stream_len as f32)
}

//Hann Window multiplier to smooth the buffer before FFT(Doesnt work yet)
// fn hann_smoothing(buffer: &mut [f32]) {
//     let n = buffer.len();
//     if n <= 1 {
//         return;
//     }
//     let mut counter = 0.0f32;
//     let std_phase = 2.0 * std::f32::consts::PI / (n as f32 - 1.0);
//     for i in buffer.iter_mut() {
//         *i = *i * 0.5 * (1.0 - (counter * std_phase).cos());
//         counter += 1.0;
//     }
// }

///Receive the input and decode it to data
pub fn receive() -> Result<(), Box<dyn std::error::Error>> {
    set_handler(|| {
        std::process::exit(0);
    })
    .unwrap();
    let input = configure_input_device()?;
    let channels = input.config.channels as u8;
    let samples_per_bit = (input.config.sample_rate as u64 * BIT_DURATION_MS / 1000) as u32;

    let mut stats = DecoderStats::new(samples_per_bit, channels);

    let stream = input.device.build_input_stream(
        &input.config,
        move |data: &[f32], _: &cpal::InputCallbackInfo| {
            for i in data.iter().step_by(channels as usize) {
                stats.process_sample(*i, input.config.sample_rate);
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

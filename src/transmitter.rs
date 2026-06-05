use crate::constants::*;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::f32::consts::PI;
//Select output device
fn configure_output_device()
-> Result<(cpal::Device, cpal::StreamConfig), Box<dyn std::error::Error>> {
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .expect("Couldn't find any available output device");
    let config = device.default_output_config()?.into();
    Ok((device, config))
}
///Transmit the data provided
pub fn transmit(bitstream: Vec<u8>) -> Result<(), Box<dyn std::error::Error>> {
    let (device, config) = configure_output_device()?;
    let sample_rate = config.sample_rate as f32;
    let channels = config.channels as usize;

    let total_bits = bitstream.len();
    let samples_per_bit: u64 = sample_rate as u64 * BIT_DURATION_MS / 1000;
    let mut samples_in_bit = 0u64;
    let mut bit_index = 0usize;
    let mut phase = 0.0f32;
    let stream = device.build_output_stream(
        &config,
        //Each Frame(Set)
        move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
            //Each Group in Frame
            for split_data in data.chunks_mut(channels) {
                let (payload, current_freq) = if bit_index < total_bits {
                    match bitstream.get(bit_index) {
                        Some(&0) => (phase.sin() * AMPLITUDE, ZERO_FREQ),
                        Some(&1) => (phase.sin() * AMPLITUDE, ONE_FREQ),
                        _ => (0.0, 0.0),
                    }
                } else {
                    (0.0, 0.0)
                };

                //Each sample in Group
                split_data.fill(payload);
                samples_in_bit += 1;
                bit_index = if samples_in_bit == samples_per_bit {
                    samples_in_bit = 0;
                    bit_index + 1
                } else {
                    bit_index
                };
                if current_freq > 0.0 {
                    phase = (phase + 2.0 * PI * current_freq / sample_rate) % (2.0 * PI)
                }
            }
        },
        move |err| {
            eprintln!("Something went wrong, {err}");
        },
        None,
    )?;
    stream.play().unwrap();
    std::thread::sleep(std::time::Duration::from_millis(
        total_bits as u64 * BIT_DURATION_MS + 50, //Plus a small buffer just to be sure
    ));

    Ok(())
}

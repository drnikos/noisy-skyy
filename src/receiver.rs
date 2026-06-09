use crate::constants::*;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use ctrlc::set_handler;
use rustfft::{FftPlanner, num_complex::Complex};
//Select input device
fn configure_input_device() -> Result<(cpal::Device, cpal::StreamConfig), Box<dyn std::error::Error>>
{
    let host = cpal::default_host();
    let device = host
        .default_input_device()
        .expect("Couldn't find any available input device");
    let config = device.default_input_config()?.into();
    Ok((device, config))
}
//Translate a frequency to data bits
fn decode_to_bits(key_freq: f32) -> Option<u8> {
    if key_freq > ONE_FREQ - FREQ_TOLERANCE && key_freq < ONE_FREQ + FREQ_TOLERANCE {
        Some(1)
    } else if key_freq > ZERO_FREQ - FREQ_TOLERANCE && key_freq < ZERO_FREQ + FREQ_TOLERANCE {
        Some(0)
    } else {
        eprintln!("Didn't get valid freq range, got {key_freq}!");
        None
    }
}
fn fft_window(stream: &Vec<f32>, sample_rate: u32) -> Option<f32> {
    let stream_len = stream.len();
    let mut stream_fft: Vec<Complex<f32>> =
        stream.iter().map(|&x| Complex { re: x, im: 0.0 }).collect();
    let mut planner: FftPlanner<f32> = FftPlanner::new();
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
    let freq = max_index as f32 * sample_rate as f32 / stream_len as f32;

    Some(freq)
}

fn convert_to_byte(buffer: &Vec<u8>) -> u8 {
    let mut x = 0u8;
    for i in 0..8 {
        x = 2 * x + buffer[i];
    }
    x
}

///Receive the input and decode it to data
pub fn receive() -> Result<(), Box<dyn std::error::Error>> {
    set_handler(|| {
        std::process::exit(0);
    })
    .unwrap();
    let (device, config) = configure_input_device()?;
    let samples_per_bit = config.sample_rate as u64 * BIT_DURATION_MS / 1000; // Calculated to use as fft window
    let mut current_remainder_sample = 0u64;
    let channels = config.channels as usize;
    let mut buffer = vec![0.0; samples_per_bit as usize];
    let mut bit_buffer: Vec<u8> = vec![0; 8];
    let mut bit_counter = 0u8; //To make bytes
    let mut found_pramble = false;
    let mut sync_check_buffer = Vec::new();
    let mut preamble_as_vec = Vec::new();
    for i in PREAMBLE.chars() {
        preamble_as_vec.push(i as u8 - 48);
    }
    println!("{samples_per_bit}");
    let stream = device.build_input_stream(
        &config,
        move |data: &[f32], _: &cpal::InputCallbackInfo| {
            for i in data.iter().step_by(channels) {
                buffer[current_remainder_sample as usize] = *i;
                current_remainder_sample += 1;
                if current_remainder_sample == samples_per_bit {
                    if let Some(i) = fft_window(&buffer, config.sample_rate) {
                        if let Some(bit) = decode_to_bits(i) {
                            sync_check_buffer.push(bit);
                            if sync_check_buffer.len() > PREAMBLE.len() {
                                sync_check_buffer.remove(0);
                            }
                            if !found_pramble {
                                if sync_check_buffer == preamble_as_vec {
                                    found_pramble = true;
                                    println!("Preamble detected!");
                                    bit_counter = 0;
                                }
                            } else {
                                bit_buffer[bit_counter as usize] = bit;
                                bit_counter += 1;
                                if bit_counter == 8 {
                                    let x = convert_to_byte(&bit_buffer);
                                    print!("{}", x as char);
                                    bit_counter = 0;
                                }
                                print!("{bit}");
                            }
                        };
                    } else {
                        eprintln!("Error decoding");
                    }
                    current_remainder_sample = 0;
                }
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

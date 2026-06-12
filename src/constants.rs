//Signal Transmission Frequencies
pub const ZERO_FREQ: f32 = 15900.0;
pub const ONE_FREQ: f32 = 16900.0;

pub const BIT_DURATION_MS: u64 = 20;
pub const FREQ_TOLERANCE: f32 = 300.0;
pub const PREAMBLE: &str = "10110111000";
const PREAMBLE_LEN: usize = PREAMBLE.len();
pub const AMPLITUDE: f32 = 0.5;
const SILENCE_BARRIER: f32 = 0.01;

//Convert PREAMBLE string to a byte array
pub const PREAMBLE_ARRAY: [u8; PREAMBLE_LEN] = {
    const fn preamble_array_gen() -> [u8; PREAMBLE_LEN] {
        let mut i = 0;
        let mut res = [0; PREAMBLE_LEN];
        let bytes = PREAMBLE.as_bytes();

        while i < PREAMBLE_LEN {
            res[i] = match bytes[i] {
                b'0' => 0,
                b'1' => 1,
                _ => panic!("Invalid character in PREAMBLE"),
            };
            i += 1;
        }
        res
    }
    preamble_array_gen()
};
pub const PREAMBLE_MASK: u64 = (1u64 << PREAMBLE_LEN) - 1;

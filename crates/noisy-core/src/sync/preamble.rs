pub const PREAMBLE: &str = "11100010010";
pub const PREAMBLE_LEN: usize = PREAMBLE.len();

/// PREAMBLE  as a bit array
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

pub struct PreambleDetector {
    register: u64,
    target: u64,
}

impl PreambleDetector {
    pub fn new() -> Self {
        let target = PREAMBLE_ARRAY
            .iter()
            .fold(0u64, |acc, &bit| (acc << 1) | (bit as u64));
        Self {
            register: 0,
            target,
        }
    }
    /// Push a bit into the detector. Returns true if the preamble has been detected.
    pub fn push_bit(&mut self, bit: u8) -> bool {
        self.register = (self.register << 1) | (bit as u64);
        (self.register & PREAMBLE_MASK) == self.target
    }
}

impl Default for PreambleDetector {
    fn default() -> Self {
        Self::new()
    }
}

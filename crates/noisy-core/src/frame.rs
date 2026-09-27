pub const PREAMBLE: &str = "11100010010";
const PREAMBLE_LEN: usize = PREAMBLE.len();
pub const END_FLAG: [u8; 8] = [0, 1, 1, 1, 1, 1, 1, 0];

/// PREAMBLE  as a bit array
const PREAMBLE_ARRAY: [u8; PREAMBLE_LEN] = {
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

fn byte_to_bits(bytes: &[u8]) -> Vec<u8> {
    let mut res = Vec::with_capacity(bytes.len() * 8);
    for &byte in bytes {
        for i in (0..8).rev() {
            res.push((byte >> i) & 1);
        }
    }
    res
}

fn bit_stuffing(datastream: &[u8]) -> Vec<u8> {
    let mut res_bitstream: Vec<u8> = Vec::new();
    let mut counter = 0;
    for bit in datastream.iter() {
        res_bitstream.push(*bit);
        if *bit == 1 {
            counter += 1;
            if counter == 5 {
                res_bitstream.push(0);
                counter = 0;
            }
        } else {
            counter = 0;
        }
    }
    res_bitstream
}

pub fn build_frame(data: &[u8]) -> Vec<u8> {
    let stuffed_bits = bit_stuffing(&byte_to_bits(data));

    let mut res = Vec::with_capacity(PREAMBLE_LEN + stuffed_bits.len() + END_FLAG.len());
    res.extend_from_slice(&PREAMBLE_ARRAY);
    res.extend(stuffed_bits);
    res.extend_from_slice(&END_FLAG);
    res
}

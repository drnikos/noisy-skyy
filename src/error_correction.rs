use crate::constants::{ENDING_FLAG, PREAMBLE_ARRAY};

pub struct ErrorCorrectionToggles {
    pub preamble: bool,
    pub ending_flag: bool,
    pub crc: bool,
    pub reed_solomon: bool,
}

impl ErrorCorrectionToggles {
    fn put_preamble(&self, data: &[u8]) -> Vec<u8> {
        let mut res = Vec::from(PREAMBLE_ARRAY);
        for i in data.iter() {
            res.push(*i);
        }
        res
    }

    pub fn setup(&self, initial_data: &str) -> Vec<u8> {
        let initial_bits = string_to_binary(initial_data);
        let stuffed_bits = bit_stuffing(&initial_bits);
        let mut res = Vec::new();
        if self.preamble {
            for i in PREAMBLE_ARRAY {
                res.push(i);
            }

            for j in stuffed_bits {
                res.push(j);
            }
            for _ in 0..ENDING_FLAG.len() {
                res.push(1)
            }
        };

        res
    }
}

fn string_to_binary(data: &str) -> Vec<u8> {
    let mut res = Vec::new();

    for i in data.bytes() {
        for j in (0..8).rev() {
            res.push(((i >> j) & 1) as u8);
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

use crate::sync::preamble::*;

pub const END_FLAG: [u8; 8] = [0, 1, 1, 1, 1, 1, 1, 0];

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

pub enum Deframed {
    Nothing,
    Byte(u8),
    End,
}

#[derive(Default)]
pub struct Deframer {
    ones: u8,
    byte: u8,
    nbits: u8,
}

impl Deframer {
    /// Deframes the stream, removing bit stuffing and detecting end flag.
    /// Returnes the next byte if available, or Deframed::End if the end flag was detected.
    pub fn push_bit(&mut self, bit: u8) -> Deframed {
        if self.ones == 5 {
            self.ones = 0;
            return if bit == 1 {
                Deframed::End
            } else {
                Deframed::Nothing
            };
        }
        if bit == 1 {
            self.ones += 1
        } else {
            self.ones = 0
        }
        self.byte = (self.byte << 1) | bit;
        self.nbits += 1;
        if self.nbits == 8 {
            let b = self.byte;
            self.byte = 0;
            self.nbits = 0;
            Deframed::Byte(b)
        } else {
            Deframed::Nothing
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn helper(message: &[u8]) {
        let frame = build_frame(message);

        // Remove the preamble
        let frame = &frame[PREAMBLE_LEN..];

        let mut deframer = Deframer::default();
        let mut output = Vec::new();

        for bit in frame {
            match deframer.push_bit(*bit) {
                Deframed::Byte(b) => output.push(b),
                Deframed::End => break,
                Deframed::Nothing => {}
            }
        }
        println!("Output: {:?}", output);
        println!("Message: {:?}", message);
        assert_eq!(output.len(), message.len());
        assert_eq!(output, message);
    }

    #[test]
    fn deframe_message() {
        let message = b"Hello, world!";
        helper(message);
    }
    #[test]
    fn detect_stuffing() {
        let message = [0xff; 4];
        helper(&message);
    }
}

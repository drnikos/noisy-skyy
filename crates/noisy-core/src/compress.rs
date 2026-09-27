use std::io;

const LEVEL: i32 = 5;

pub fn compress(data: &[u8]) -> io::Result<Vec<u8>> {
    zstd::encode_all(data, LEVEL)
}

pub fn decompress(data: &[u8]) -> io::Result<Vec<u8>> {
    zstd::decode_all(data)
}

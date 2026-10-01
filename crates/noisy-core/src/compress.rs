use crate::error::CoreError;

const LEVEL: i32 = 5;

pub fn compress(data: &[u8]) -> Result<Vec<u8>, CoreError> {
    zstd::encode_all(data, LEVEL).map_err(CoreError::Compress)
}

pub fn decompress(data: &[u8]) -> Result<Vec<u8>, CoreError> {
    zstd::decode_all(data).map_err(CoreError::Decompress)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn garbage_is_a_decompress_error() {
        assert!(matches!(
            decompress(&[1, 2, 3]),
            Err(CoreError::Decompress(_))
        ));
    }
}

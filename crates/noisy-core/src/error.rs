use thiserror::Error;

#[derive(Error, Debug)]
pub enum CoreError {
    #[error("Compression failed")]
    Compress(#[source] std::io::Error),
    #[error("Decompression failed")]
    Decompress(#[source] std::io::Error),
}

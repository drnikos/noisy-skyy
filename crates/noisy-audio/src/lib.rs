pub mod input;
pub mod output;
pub mod wav;

// For anyhow
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

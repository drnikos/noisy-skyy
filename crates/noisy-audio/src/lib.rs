#![deny(clippy::unwrap_used, clippy::expect_used)]

pub mod error;
pub mod input;
pub mod output;
pub mod wav;

pub use error::{AudioError, Result};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum AudioError {
    #[error("no {0} device found")]
    NoDevice(&'static str),
    #[error("audio device error")]
    Cpal(#[from] cpal::Error),
    #[error("WAV file error")]
    Wav(#[from] hound::Error),
}

pub type Result<T> = std::result::Result<T, AudioError>;

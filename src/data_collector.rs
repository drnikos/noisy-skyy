// I made this file so that if I want to do more editing in data in the future it will be easier and more organized

use crate::error_correction;
use std::{fs, io, path::Path};
fn reader(path: &Path) -> Result<Vec<u8>, std::io::Error> {
    let res = fs::read(path)?;
    Ok(res)
}

pub fn setup(
    path: &Path,
    error_toggles: error_correction::ErrorCorrectionToggles,
) -> Result<Vec<u8>, std::io::Error> {
    let initial_file = reader(path)?;
    let compressed_data = zstd::encode_all(&initial_file[..], 5)?;
    let error_corrected = error_toggles.setup(&compressed_data);
    Ok(error_corrected)
}

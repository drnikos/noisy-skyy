// I made this file so that if I want to do more editing in data in the future it will be easier and more organized

use crate::error_correction;
use std::{fs, path::Path};
fn reader(path: &Path) -> Result<String, std::io::Error> {
    let res = fs::read_to_string(path)?;
    Ok(res)
}

pub fn setup(
    path: &Path,
    error_toggles: error_correction::ErrorCorrectionToggles,
) -> Result<Vec<u8>, std::io::Error> {
    let initial_file = reader(path)?;
    let error_corrected = error_toggles.setup(&initial_file);

    Ok(error_corrected)
}

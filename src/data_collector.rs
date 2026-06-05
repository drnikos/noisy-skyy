use std::{fs, path::PathBuf};

fn convert_to_binary(data: &str) -> Result<Vec<u8>, std::io::Error> {
    let mut res = Vec::new();
    for i in data.bytes() {
        for j in (0..8).rev() {
            res.push(i >> j & 1);
        }
    }
    for i in res.iter() {
        print!("{}", *i);
    }
    println!();
    Ok(res)
}

///Converts the source data to a vector of bits
pub fn str2b(path: &PathBuf) -> Result<Vec<u8>, std::io::Error> {
    let contents = fs::read_to_string(path)?;
    convert_to_binary(&contents)
}

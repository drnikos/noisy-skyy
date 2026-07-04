use crate::error_correction::ErrorCorrectionToggles;
use clap::{Args, Parser};
use std::path::PathBuf;

mod constants;
mod data_collector;
mod error_correction;
mod receiver;
mod transmitter;

#[derive(Parser)]
#[command(version, about, long_about = None)]

struct Cli {
    /// Turn debugging information on
    #[arg(short, long, action = clap::ArgAction::Count)]
    debug: u8,

    #[command(flatten)]
    mode: Mode, //Require user to provide either the listen flag or source argument

    #[arg(short, long, value_name = "PREAMBLE")]
    preamble: bool,
}
#[derive(Args)]
#[group(required = true, multiple = false)]
struct Mode {
    /// Turn on listening mode (receiver)
    #[arg(short, long)]
    listen: bool,

    ///Specify the path of the source file of which you wish to transmit it's contents
    #[arg(short, long, value_name = "SOURCE_PATH")]
    source: Option<PathBuf>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let errcortog = ErrorCorrectionToggles {
        crc: false,
        preamble: cli.preamble,
        ending_flag: false,
        reed_solomon: false,
    };

    if cli.mode.listen {
        println!("Listening mode");
        receiver::receive()?;
    } else {
        if let Some(path) = cli.mode.source {
            let stream = data_collector::setup(&path, errcortog)?;
            transmitter::transmit(stream)?;
        }
    }
    Ok(())
}

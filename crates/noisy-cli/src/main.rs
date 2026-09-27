use anyhow::{Context, Result, anyhow};
use clap::{Parser, Subcommand};
use noisy_audio::input::Input;
use noisy_audio::output::Output;
use noisy_core::config::ModemConfig;
use noisy_core::rx::{Receiver, RxEvent};
use std::io::Write;
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(name = "noisy-skyy", version, about = "Data over ultrasound")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Transmit a file through the speaker
    Tx { file: PathBuf },
    /// Receive a file through the microphone
    Rx,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let cfg = ModemConfig::default();
    match cli.command {
        Command::Tx { file } => transmit_file(&file, &cfg),
        Command::Rx => receive(&cfg),
    }
}

fn transmit_file(path: &Path, cfg: &ModemConfig) -> Result<()> {
    let payload =
        std::fs::read(path).with_context(|| format!("Failed to read {}!", path.display()))?;
    let out = Output::open_default().map_err(|e| anyhow!(e))?;
    let samples = noisy_core::tx::samples(&payload, cfg, out.sample_rate())?;
    out.play_blocking(samples).map_err(|e| anyhow!(e))?;
    Ok(())
}

fn receive(cfg: &ModemConfig) -> Result<()> {
    let input = Input::open_default().map_err(|e| anyhow!(e))?;
    let mut rx = Receiver::new(cfg, input.sample_rate());
    let mut events = Vec::new();
    eprintln!("Listening...");
    input
        .run(move |samples| {
            rx.push(samples, &mut events);
            for event in events.drain(..) {
                match event {
                    RxEvent::SyncFound => eprintln!("preamble found"),
                    RxEvent::Frame(data) => {
                        let mut out = std::io::stdout().lock();
                        let _ = out.write_all(&data);
                        let _ = out.flush();
                    }
                    RxEvent::BadFrame { raw, error } => {
                        eprintln!(
                            "decompress failed ({error}): {} bytes {raw:02X?}",
                            raw.len()
                        )
                    }
                }
            }
        })
        .map_err(|e| anyhow!(e))
}

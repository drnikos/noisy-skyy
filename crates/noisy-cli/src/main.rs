use anyhow::{Context, Result, anyhow};
use clap::{Parser, Subcommand};
use noisy_audio::output::Output;
use noisy_core::config::ModemConfig;
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
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let cfg = ModemConfig::default();
    match cli.command {
        Command::Tx { file } => transmit_file(&file, &cfg),
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

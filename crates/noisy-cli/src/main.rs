use anyhow::{Context, Result, anyhow, bail};
use clap::{Parser, Subcommand};
use noisy_audio::input::Input;
use noisy_audio::output::Output;
use noisy_audio::wav::{WAV_DEFAULT_RATE, read_wav, write_wav};
use noisy_core::config::ModemConfig;
use noisy_core::rx::{Receiver, RxEvent};
use noisy_core::tx;
use std::fs::File;
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
    /// Transmit a file through the speaker (or into a WAV file)
    Tx {
        file: PathBuf,
        /// Write the signal to this WAV file instead of playing it
        #[arg(long, value_name = "OUT.wav")]
        wav: Option<PathBuf>,
    },
    /// Receive through the microphone (or from a WAV file)
    Rx {
        /// Decode this WAV file instead of listening
        #[arg(long, value_name = "IN.wav")]
        wav: Option<PathBuf>,
        /// Write received data here instead of stdout
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
}

/// Where received bytes go: a file or stdout.
type Sink = Box<dyn Write + Send>;

fn main() -> Result<()> {
    let cli = Cli::parse();
    let cfg = ModemConfig::default();
    match cli.command {
        Command::Tx { file, wav } => transmit(&file, wav.as_deref(), &cfg),
        Command::Rx { wav, output } => {
            let out = open_sink(output.as_deref())?;
            match wav {
                Some(path) => receive_wav(&path, out, &cfg),
                None => receive_live(out, &cfg),
            }
        }
    }
}

fn transmit(file: &Path, wav: Option<&Path>, cfg: &ModemConfig) -> Result<()> {
    let payload =
        std::fs::read(file).with_context(|| format!("Failed to read {}!", file.display()))?;
    match wav {
        Some(path) => {
            let samples = tx::samples(&payload, cfg, WAV_DEFAULT_RATE)?;
            write_wav(path, &samples, WAV_DEFAULT_RATE).map_err(|e| anyhow!(e))?;
        }
        None => {
            let speaker = Output::open_default().map_err(|e| anyhow!(e))?;
            let samples = tx::samples(&payload, cfg, speaker.sample_rate())?;
            speaker.play_blocking(samples).map_err(|e| anyhow!(e))?;
        }
    }
    Ok(())
}

fn open_sink(path: Option<&Path>) -> Result<Sink> {
    Ok(match path {
        Some(p) => {
            Box::new(File::create(p).with_context(|| format!("Failed to create {}!", p.display()))?)
        }
        None => Box::new(std::io::stdout()),
    })
}

/// Decodes a whole WAV file at once. Fails if no frame was found, so `&&` chains stop.
fn receive_wav(path: &Path, mut out: Sink, cfg: &ModemConfig) -> Result<()> {
    let (samples, sample_rate) = read_wav(path).map_err(|e| anyhow!(e))?;
    let mut rx = Receiver::new(cfg, sample_rate);
    let mut events = Vec::new();
    rx.push(&samples, &mut events);

    let mut frames = 0;
    for event in events {
        if matches!(event, RxEvent::Frame(_)) {
            frames += 1;
        }
        handle(event, &mut out)?;
    }
    if frames == 0 {
        bail!("no frame decoded from {}", path.display());
    }
    Ok(())
}

fn receive_live(mut out: Sink, cfg: &ModemConfig) -> Result<()> {
    let input = Input::open_default().map_err(|e| anyhow!(e))?;
    let mut rx = Receiver::new(cfg, input.sample_rate());
    let mut events = Vec::new();
    eprintln!("Listening...");
    input
        .run(move |samples| {
            rx.push(samples, &mut events);
            for event in events.drain(..) {
                if let Err(e) = handle(event, &mut out) {
                    eprintln!("Failed to write output: {e}");
                }
            }
        })
        .map_err(|e| anyhow!(e))
}

/// Shared by live and WAV mode: status to stderr, data to `out`.
fn handle(event: RxEvent, out: &mut dyn Write) -> std::io::Result<()> {
    match event {
        RxEvent::SyncFound => eprintln!("preamble found"),
        RxEvent::Frame(data) => {
            out.write_all(&data)?;
            out.flush()?;
        }
        RxEvent::BadFrame { raw, error } => {
            eprintln!(
                "decompress failed ({error}): {} bytes {raw:02X?}",
                raw.len()
            )
        }
    }
    Ok(())
}

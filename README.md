![noisy-skyy banner](assets/banner.png)

[![CI](https://github.com/drnikos/noisy-skyy/actions/workflows/ci.yml/badge.svg)](https://github.com/drnikos/noisy-skyy/actions/workflows/ci.yml)


> Send files between computers over near-ultrasonic sound, using only a speaker and a microphone.

**Status: experimental.** 
It works between laptops a short distance apart, in a quiet room, at about 100 bit/s.
There is no error correction or integrity check yet.

## Usage

You need git and rustup. On Linux you also need the ALSA headers:
`sudo apt install libasound2-dev` (Debian/Ubuntu) or `sudo pacman -S alsa-lib` (Arch).

Clone the repo:
```sh
git clone https://github.com/drnikos/noisy-skyy && cd noisy-skyy
```

See all commands and flags:
```sh
cargo run --release -p noisy-cli -- --help
```

Start a receiver that writes what it receives to `received.bin`:
```sh
cargo run --release -p noisy-cli -- rx -o received.bin
```

Send a file from another machine:
```sh
cargo run --release -p noisy-cli -- tx file.bin
```

For debugging without a sound card, write the signal to a WAV file and decode it back:
```sh
cargo run -p noisy-cli -- tx file.bin --wav signal.wav
cargo run -p noisy-cli -- rx --wav signal.wav -o out.bin
cmp file.bin out.bin
```

Add `-v` to see the receiver's state changes, or `-vv` to see every bit and its signal level.

## Some Limitations
- Without CRC implemented yet,  bit errors can produce wrong data without an error.
- The silence threshold is a fixed level, tuned my laptop microphone.

## License
MIT
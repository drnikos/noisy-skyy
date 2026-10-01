## 2026-09-27: Split into a workspace
The old style of code was a mess that was hard to maintain and add/extend features, so I decided to split the project into multiple crates,
while maintaining the behaviour of the old code.
Also, moved the math processing from inside cpal callback


## 2026-09-27: False preambles with nothing playing
The listener kept printing "preamble found" with only room noise.
Cause: `Receiver::new` set `silence_threshold: 0.0` instead of reading the config,
so every 10 ms window of noise became a random bit. An 11-bit preamble matches
random bits with probability 1/2048 per bit; at 100 bit/s that's a false sync
about every 20 s, followed by a few garbage bytes that zstd rejects.
So, I increased the silence_threshold to partially fix the issue


## 2026-09-27: Live receiver never found the preamble (DC offset)
After fixing the threshold, live rx stopped hearing anything, while WAV roundtrips worked.
Recorded the mic during tx and measured. I figured that my laptop 
DC offset is about -0.074, and `rms()` included it:
- silence: 0.0736, during tx: 0.0741 (indistinguishable)
- threshold 0.08 → everything gated (never decodes)
- threshold 0.05 → gate always open (false preambles)
Without DC: silence 0.0005, signal 0.008 . So, putting threshold to  0.002 improved the situation a lot.
Also, found out that 16–17 kHz is about 15–20 dB stronger than 18-19 kHz


## 2026-09-27: Corrupted text came out as "success"
Live rx printed "debwg mgssqww" instead of "debug message" with no error.
Probably need to add CRC

## 2026-10-01: Errors and tracing
I choose to use `thiserror` enums in the libraries, `anyhow` in the CLI, deny`unwrap`  in
the libraries. Also, added tracing, in which `-vv` prints level and bit for every window, which would have
shown the DC offset immediately (level=0.074 in every window).
s

use noisy_core::{
    config::ModemConfig,
    rx::{Receiver, RxEvent},
    tx,
};

const RATE: u32 = 48_000;
const MSG: &[u8] = b"hello from the workspace, a longer message to be sure";

/// Deterministic uniform noise in [-amp, amp] (LCG, no deps).
fn noise(n: usize, amp: f32, seed: u64) -> Vec<f32> {
    let mut x = seed;
    (0..n)
        .map(|_| {
            x = x
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            ((x >> 40) as f32 / (1u64 << 24) as f32 * 2.0 - 1.0) * amp
        })
        .collect()
}

/// TX -> prepend `offset` silent samples -> add noise -> RX. True if MSG comes back.
fn roundtrip(offset: usize, noise_amp: f32) -> bool {
    let cfg = ModemConfig::default();
    let mut signal = vec![0.0; offset];
    signal.extend(tx::samples(MSG, &cfg, RATE).unwrap());
    let signal_len = signal.len();
    for (s, n) in signal.iter_mut().zip(noise(signal_len, noise_amp, 1)) {
        *s += n;
    }
    let mut rx = Receiver::new(&cfg, RATE);
    let mut events = Vec::new();
    rx.push(&signal, &mut events);
    events
        .iter()
        .any(|e| matches!(e, RxEvent::Frame(d) if d == MSG))
}

#[test]
fn aligned_noisy_channel() {
    assert!(roundtrip(0, 0.5));
}

#[test]
#[ignore = "needs symbol timing recovery (Phase 1)"]
fn half_bit_offset_noisy_channel() {
    assert!(roundtrip(240, 0.5)); // 240 = half of 480 samples per bit at 48 kHz
}

//! Analyze the Martin M1 encoded WAV file: measure header frequencies
//! and decode the VIS code using Goertzel.

use komitoto_sstv::audio;
use komitoto_sstv::dsp;
use komitoto_sstv::spec;

const SAMPLE_RATE: u32 = spec::SAMPLE_RATE;

/// Convert a duration in milliseconds to a sample count.
fn ms_to_samples(ms: f64) -> usize {
    ((ms / 1000.0) * SAMPLE_RATE as f64).round() as usize
}

/// Extract a window of samples starting at `start_ms` for `duration_ms`.
fn extract_window(samples: &[f32], start_ms: f64, duration_ms: f64) -> &[f32] {
    let start = ms_to_samples(start_ms);
    let len = ms_to_samples(duration_ms);
    let end = (start + len).min(samples.len());
    &samples[start..end]
}

/// Run Goertzel on a window and return the magnitude-squared energy.
fn measure_energy(samples: &[f32], freq_hz: f64) -> f64 {
    dsp::goertzel(samples, freq_hz, SAMPLE_RATE)
}

/// Format energy as a relative bar for quick visual comparison.
fn energy_bar(energy: f64, max_energy: f64) -> String {
    let ratio = if max_energy > 0.0 { energy / max_energy } else { 0.0 };
    let bars = (ratio * 40.0).round() as usize;
    format!("{:<40} ({:.0})", "#".repeat(bars.min(40)), energy)
}

fn main() {
    let wav_path = "sstv_test_results/martinm1_encoded.wav";

    println!("=== SSTV WAV Header & VIS Code Analyzer ===\n");
    println!("Reading WAV file: {}", wav_path);

    let (samples, sr) = audio::read_wav(wav_path).expect("Failed to read WAV file");
    println!("  Sample rate : {} Hz", sr);
    println!("  Total samples: {}", samples.len());
    println!("  Duration    : {:.3} s\n", samples.len() as f64 / sr as f64);

    assert_eq!(sr, SAMPLE_RATE, "Unexpected sample rate");

    // ─── 1. Leader 1 (0 – 300 ms): 1900 Hz ─────────────────────────────
    println!("--- Header Tones ---\n");

    let leader1 = extract_window(&samples, 0.0, 300.0);
    let e_1900_l1 = measure_energy(leader1, spec::FREQ_LEADER);
    let e_1200_l1 = measure_energy(leader1, spec::FREQ_SYNC);
    let e_1100_l1 = measure_energy(leader1, spec::FREQ_VIS_BIT_0);
    let e_1300_l1 = measure_energy(leader1, spec::FREQ_VIS_BIT_1);
    let max_l1 = e_1900_l1.max(e_1200_l1).max(e_1100_l1).max(e_1300_l1);

    println!("Leader 1 (0-300 ms) — expected 1900 Hz:");
    println!("  1900 Hz: {}", energy_bar(e_1900_l1, max_l1));
    println!("  1200 Hz: {}", energy_bar(e_1200_l1, max_l1));
    println!("  1100 Hz: {}", energy_bar(e_1100_l1, max_l1));
    println!("  1300 Hz: {}", energy_bar(e_1300_l1, max_l1));
    println!();

    // ─── 2. Break (300 – 310 ms): 1200 Hz ──────────────────────────────
    let brk = extract_window(&samples, 300.0, 10.0);
    let e_1900_brk = measure_energy(brk, spec::FREQ_LEADER);
    let e_1200_brk = measure_energy(brk, spec::FREQ_SYNC);
    let max_brk = e_1900_brk.max(e_1200_brk);
    println!("Break (300-310 ms) — expected 1200 Hz:");
    println!("  1900 Hz: {}", energy_bar(e_1900_brk, max_brk));
    println!("  1200 Hz: {}", energy_bar(e_1200_brk, max_brk));
    println!();

    // ─── 3. Leader 2 (310 – 610 ms): 1900 Hz ───────────────────────────
    let leader2 = extract_window(&samples, 310.0, 300.0);
    let e_1900_l2 = measure_energy(leader2, spec::FREQ_LEADER);
    let e_1200_l2 = measure_energy(leader2, spec::FREQ_SYNC);
    let max_l2 = e_1900_l2.max(e_1200_l2);
    println!("Leader 2 (310-610 ms) — expected 1900 Hz:");
    println!("  1900 Hz: {}", energy_bar(e_1900_l2, max_l2));
    println!("  1200 Hz: {}", energy_bar(e_1200_l2, max_l2));
    println!();

    // ─── 4. Start bit (610 – 640 ms): 1200 Hz ───────────────────────────
    let start_bit = extract_window(&samples, 610.0, 30.0);
    let e_1200_start = measure_energy(start_bit, spec::FREQ_SYNC);
    let e_1100_start = measure_energy(start_bit, spec::FREQ_VIS_BIT_0);
    let e_1300_start = measure_energy(start_bit, spec::FREQ_VIS_BIT_1);
    let max_start = e_1200_start.max(e_1100_start).max(e_1300_start);
    println!("Start bit (610-640 ms) — expected 1200 Hz:");
    println!("  1200 Hz: {}", energy_bar(e_1200_start, max_start));
    println!("  1100 Hz: {}", energy_bar(e_1100_start, max_start));
    println!("  1300 Hz: {}", energy_bar(e_1300_start, max_start));
    println!();

    // ─── 5. VIS data bits (8 x 30 ms, starting at 640 ms) ──────────────
    // Encoder sends LSB first: bit=1 -> 1100 Hz (FREQ_VIS_BIT_0), bit=0 -> 1300 Hz (FREQ_VIS_BIT_1)
    println!("--- VIS Code Bits (LSB first) ---\n");

    let mut vis_code: u8 = 0;
    for bit_idx in 0..8u32 {
        let start_ms = 640.0 + bit_idx as f64 * 30.0;
        let end_ms = start_ms + 30.0;
        let window = extract_window(&samples, start_ms, 30.0);

        let e_1100 = measure_energy(window, spec::FREQ_VIS_BIT_0); // bit = 1
        let e_1300 = measure_energy(window, spec::FREQ_VIS_BIT_1); // bit = 0
        let max_bit = e_1100.max(e_1300);

        let bit_val: u8 = if e_1100 > e_1300 { 1 } else { 0 };
        vis_code |= bit_val << bit_idx;

        println!(
            "  Bit {} ({:.0}-{:.0} ms): 1100Hz={}  1300Hz={}  -> {}",
            bit_idx, start_ms, end_ms,
            energy_bar(e_1100, max_bit),
            energy_bar(e_1300, max_bit),
            bit_val,
        );
    }
    println!();

    // ─── 6. Stop bit (880 – 910 ms): 1200 Hz ────────────────────────────
    let stop_bit = extract_window(&samples, 880.0, 30.0);
    let e_1200_stop = measure_energy(stop_bit, spec::FREQ_SYNC);
    let e_1100_stop = measure_energy(stop_bit, spec::FREQ_VIS_BIT_0);
    let e_1300_stop = measure_energy(stop_bit, spec::FREQ_VIS_BIT_1);
    let max_stop = e_1200_stop.max(e_1100_stop).max(e_1300_stop);
    println!("Stop bit (880-910 ms) — expected 1200 Hz:");
    println!("  1200 Hz: {}", energy_bar(e_1200_stop, max_stop));
    println!("  1100 Hz: {}", energy_bar(e_1100_stop, max_stop));
    println!("  1300 Hz: {}", energy_bar(e_1300_stop, max_stop));
    println!();

    // ─── 7. Summary ─────────────────────────────────────────────────────
    println!("=== Summary ===\n");
    println!(
        "  Decoded VIS code : 0x{:02X} ({:08b} binary) = {} decimal",
        vis_code, vis_code, vis_code
    );
    println!(
        "  Expected VIS code: 0x{:02X} ({:08b} binary) = {} decimal  (Martin M1)",
        spec::MartinM1Spec::VIS_CODE,
        spec::MartinM1Spec::VIS_CODE,
        spec::MartinM1Spec::VIS_CODE,
    );

    if vis_code == spec::MartinM1Spec::VIS_CODE {
        println!("\n  VIS code MATCHES Martin M1!");
    } else {
        println!("\n  VIS code DOES NOT match Martin M1.");
    }
}

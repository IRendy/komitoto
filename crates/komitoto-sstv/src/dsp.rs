use std::f64::consts::PI;

/// Generate a sine wave at a given frequency.
/// Returns `duration_ms` milliseconds of audio at `sample_rate` Hz.
pub fn generate_tone(frequency: f64, duration_ms: f64, sample_rate: u32) -> Vec<f32> {
    let num_samples = ((duration_ms / 1000.0) * sample_rate as f64).round() as usize;
    let mut samples = Vec::with_capacity(num_samples);
    for i in 0..num_samples {
        let t = i as f64 / sample_rate as f64;
        samples.push((2.0 * PI * frequency * t).sin() as f32);
    }
    samples
}

/// Generate a sine wave with phase continuity from a given starting phase.
pub fn generate_tone_phase(
    frequency: f64,
    duration_ms: f64,
    sample_rate: u32,
    start_phase: f64,
) -> (Vec<f32>, f64) {
    let num_samples = ((duration_ms / 1000.0) * sample_rate as f64).round() as usize;
    let mut samples = Vec::with_capacity(num_samples);
    let mut phase = start_phase;
    let phase_step = 2.0 * PI * frequency / sample_rate as f64;
    for _ in 0..num_samples {
        samples.push(phase.sin() as f32);
        phase += phase_step;
    }
    (samples, phase)
}

/// Goertzel algorithm: detect energy at a specific frequency in a sample block.
/// Returns the magnitude squared of the DFT at `frequency`.
pub fn goertzel(samples: &[f32], frequency: f64, sample_rate: u32) -> f64 {
    let n = samples.len();
    if n == 0 {
        return 0.0;
    }
    let k = (frequency * n as f64 / sample_rate as f64).round() as usize;
    let theta = 2.0 * PI * k as f64 / n as f64;
    let coeff = 2.0 * theta.cos();

    let mut s_prev2 = 0.0f64;
    let mut s_prev1 = 0.0f64;
    let mut s: f64;

    for &sample in samples {
        s = sample as f64 + coeff * s_prev1 - s_prev2;
        s_prev2 = s_prev1;
        s_prev1 = s;
    }

    s_prev2 * s_prev2 + s_prev1 * s_prev1 - coeff * s_prev1 * s_prev2
}

/// FM demodulate a block of SSTV audio to get instantaneous frequency over time.
///
/// Uses quadrature mixer approach:
/// 1. Mix signal down to baseband using center frequency (1900 Hz)
/// 2. Low-pass filter to remove double-frequency component
/// 3. Compute instantaneous frequency from phase differences
///
/// This is necessary because per-pixel Goertzel fails when samples-per-pixel
/// is too low (e.g., Martin M1 has only ~5 samples/pixel, causing both
/// 1500 Hz and 2300 Hz to map to the same DFT bin k=1).
pub fn fm_demodulate(samples: &[f32], sample_rate: u32) -> Vec<f64> {
    let sr = sample_rate as f64;
    let n = samples.len();
    if n < 2 {
        return vec![crate::spec::FREQ_BLACK; n];
    }

    // Center of SSTV frequency range: (1500 + 2300) / 2 = 1900 Hz
    let center_freq = 1900.0f64;

    // Step 1: Mix down to baseband
    let mut i_bb = vec![0.0f64; n];
    let mut q_bb = vec![0.0f64; n];
    for k in 0..n {
        let t = k as f64 / sr;
        let phase = 2.0 * PI * center_freq * t;
        i_bb[k] = samples[k] as f64 * phase.cos();
        q_bb[k] = samples[k] as f64 * (-phase.sin());
    }

    // Step 2: Low-pass filter to remove 2*f_center component (3800 Hz)
    // Use a Blackman-windowed sinc FIR for better stopband rejection.
    // The baseband signal occupies DC to ±400 Hz, so cutoff = 800 Hz.
    // Double-frequency image is at ±3800 Hz — well into stopband.
    let cutoff_hz = 800.0;
    let mut lpf_taps = design_lpf_taps(sr, cutoff_hz, 15);
    // Truncate filter if signal is shorter than filter length
    if lpf_taps.len() > n {
        lpf_taps.truncate(n);
    }
    let i_filt = fir_filter(&i_bb, &lpf_taps);
    let q_filt = fir_filter(&q_bb, &lpf_taps);

    // Step 3: Compute instantaneous frequency from phase differentiation
    let mut freq = vec![crate::spec::FREQ_BLACK; n];
    let mut prev_phase = f64::atan2(q_filt[0], i_filt[0]);

    for k in 1..n {
        let phase = f64::atan2(q_filt[k], i_filt[k]);
        let mut dphase = phase - prev_phase;

        // Unwrap phase to [-π, π]
        while dphase > PI {
            dphase -= 2.0 * PI;
        }
        while dphase < -PI {
            dphase += 2.0 * PI;
        }

        freq[k] = dphase * sr / (2.0 * PI) + center_freq;
        prev_phase = phase;
    }

    // Step 4: Smooth the raw frequency signal to reduce sample-level noise.
    // A short moving average (~20 samples ≈ 2ms) smooths out jitter without
    // blurring pixel boundaries (each pixel is typically 5-20 samples).
    let smooth_len = 21usize.min(n);
    freq = moving_average(&freq, smooth_len);

    freq
}

/// Design a Blackman-windowed sinc low-pass FIR filter.
/// Returns the filter coefficients (taps), always odd-length for zero-phase.
fn design_lpf_taps(sr: f64, cutoff_hz: f64, min_taps: usize) -> Vec<f64> {
    // Ensure odd length for symmetric filter
    let mut taps_len = min_taps | 1; // make odd
    if taps_len < 3 {
        taps_len = 3;
    }
    let half = taps_len / 2;
    let fc_norm = cutoff_hz / sr; // normalized cutoff

    let mut coeffs = vec![0.0f64; taps_len];
    for i in 0..taps_len {
        let n = i as f64 - half as f64;
        // Sinc function
        let sinc = if n.abs() < 1e-10 {
            1.0
        } else {
            (PI * 2.0 * fc_norm * n).sin() / (PI * n)
        };
        // Blackman window
        let w = 0.42 - 0.5 * (2.0 * PI * i as f64 / (taps_len - 1) as f64).cos()
            + 0.08 * (4.0 * PI * i as f64 / (taps_len - 1) as f64).cos();
        coeffs[i] = sinc * w;
    }

    // Normalize so DC gain = 1
    let sum: f64 = coeffs.iter().sum();
    if sum > 0.0 {
        for c in coeffs.iter_mut() {
            *c /= sum;
        }
    }

    coeffs
}

/// Apply a FIR filter with zero-phase (centered) convolution.
fn fir_filter(data: &[f64], coeffs: &[f64]) -> Vec<f64> {
    let n = data.len();
    let half = coeffs.len() / 2;
    let mut result = vec![0.0f64; n];

    for k in 0..n {
        let mut acc = 0.0f64;
        for (j, &c) in coeffs.iter().enumerate() {
            let idx = k as isize - half as isize + j as isize;
            if idx >= 0 && (idx as usize) < n {
                acc += c * data[idx as usize];
            }
        }
        result[k] = acc;
    }

    result
}

/// Simple centered moving average for smoothing.
fn moving_average(data: &[f64], taps: usize) -> Vec<f64> {
    let n = data.len();
    let mut result = vec![0.0f64; n];
    let half = taps / 2;

    for k in 0..n {
        let start = k.saturating_sub(half);
        let end = (k + half + 1).min(n);
        let count = end - start;
        let sum: f64 = data[start..end].iter().sum();
        result[k] = sum / count as f64;
    }

    result
}

/// Frequency to SSTV brightness mapping: 1500 Hz (black/0) to 2300 Hz (white/255).
pub fn frequency_to_brightness(freq: f64) -> u8 {
    let normalized = ((freq - 1500.0) / (2300.0 - 1500.0)).clamp(0.0, 1.0);
    (normalized * 255.0).round() as u8
}

/// Brightness to SSTV frequency mapping: 0 -> 1500 Hz, 255 -> 2300 Hz.
pub fn brightness_to_frequency(value: u8) -> f64 {
    1500.0 + (value as f64 / 255.0) * (2300.0 - 1500.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_goertzel_detects_tone() {
        let sr = 11025u32;
        let freq = 1200.0f64;
        let tone = generate_tone(freq, 50.0, sr);
        let energy = goertzel(&tone, freq, sr);
        let energy_wrong = goertzel(&tone, 2000.0, sr);
        assert!(energy > energy_wrong * 2.0);
    }

    #[test]
    fn test_brightness_frequency_roundtrip() {
        for v in 0..=255u8 {
            let f = brightness_to_frequency(v);
            let b = frequency_to_brightness(f);
            assert!((b as i32 - v as i32).abs() <= 1);
        }
    }
}

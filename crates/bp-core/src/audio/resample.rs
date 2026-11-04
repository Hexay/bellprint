use super::Audio;
use std::f64::consts::PI;

/// Kernel half-width in zero crossings of the sinc.
const ZEROS: f64 = 32.0;
const STEPS: f64 = 512.0;

/// Hann-windowed sinc g(z) = sinc(z) · hann(z / ZEROS) on z ∈ [0, ZEROS], every 1/STEPS.
fn kernel_table() -> Vec<f64> {
    (0..=(ZEROS * STEPS) as usize + 1)
        .map(|i| {
            let z = i as f64 / STEPS;
            let sinc = if z == 0.0 { 1.0 } else { (PI * z).sin() / (PI * z) };
            if z > ZEROS { 0.0 } else { sinc * (0.5 + 0.5 * (PI * z / ZEROS).cos()) }
        })
        .collect()
}

fn lookup(table: &[f64], z: f64) -> f64 {
    let p = z.abs() * STEPS;
    let i = p as usize;
    if i + 1 >= table.len() {
        return 0.0;
    }
    let f = p - i as f64;
    table[i] * (1.0 - f) + table[i + 1] * f
}

/// Band-limited (windowed-sinc) resampling to `rate`, cut off at 95 % of the lower Nyquist.
pub fn resample(audio: &Audio, rate: u32) -> Audio {
    if rate == audio.sample_rate {
        return audio.clone();
    }
    let ratio = audio.sample_rate as f64 / rate as f64;
    // Cutoff as a fraction of the input Nyquist; the kernel in input samples is fc · g(fc · u).
    let fc = 0.95 * (1.0 / ratio).min(1.0);
    let table = kernel_table();
    let half = (ZEROS / fc).ceil() as i64;
    let x = &audio.samples;
    let n_out = (x.len() as f64 / ratio).floor() as usize;
    let samples = (0..n_out)
        .map(|j| {
            let t = j as f64 * ratio;
            let c = t.floor() as i64;
            let lo = (c - half + 1).max(0);
            let hi = (c + half).min(x.len() as i64 - 1);
            let acc: f64 = (lo..=hi).map(|k| x[k as usize] as f64 * lookup(&table, fc * (t - k as f64))).sum();
            (fc * acc) as f32
        })
        .collect();
    Audio { samples, sample_rate: rate }
}

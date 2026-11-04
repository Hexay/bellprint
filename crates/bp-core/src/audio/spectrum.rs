use super::Audio;
use super::longterm::{Aggregate, Frames};
use realfft::RealFftPlanner;
use serde::{Deserialize, Serialize};

/// A spectral line in the clip's long-term spectrum.
#[derive(Debug, Clone, Copy, Serialize)]
pub struct Peak {
    pub hz: f64,
    /// dB above the local spectral floor.
    pub prominence_db: f64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct PeakParams {
    pub window_s: f64,
    pub min_hz: f64,
    pub max_hz: f64,
    pub min_prominence_db: f64,
    pub max_peaks: usize,
    /// Half-width of the moving median that defines the floor.
    pub floor_half_width_cents: f64,
    pub aggregate: Aggregate,
    /// Below this many lines `aggregate` has missed the bells (masked by loud intermittent noise, or
    /// sounding in too few frames); the `sparse_fallback` aggregates are tried in order and the one
    /// giving the most lines is kept.
    pub sparse_min_lines: usize,
    pub sparse_fallback: Vec<Aggregate>,
}

impl Default for PeakParams {
    fn default() -> Self {
        Self {
            // 32768 samples at 44.1 kHz, the length tuned in research/extraction_ab.md §12.
            window_s: 32768.0 / 44100.0,
            min_hz: 60.0,
            // 98.7 % of Dove nominals lie below this; lines above it helped wrong rings more than right ones.
            max_hz: 2500.0,
            // The matcher's background model absorbs weak chance lines; a higher cut drops faint real nominals.
            // The mean's floor is smoother than a percentile's, so 3 dB clears it (research/extraction_ab.md §12).
            min_prominence_db: 3.0,
            // At 3 dB many corpus clips fill 80; no sim clip does.
            max_peaks: 120,
            floor_half_width_cents: 150.0,
            // Nominals die within seconds of each blow while the hum rings on, so a percentile favours the
            // hum. See research/extraction_ab.md §10.
            aggregate: Aggregate::Mean,
            // Sim clips give >= 22 lines; the percentile rejects noise that swamps the mean.
            sparse_min_lines: 20,
            sparse_fallback: vec![Aggregate::Percentile(75.0), Aggregate::Percentile(90.0)],
        }
    }
}

/// Hann-windowed, zero-padded power spectra of fixed-length segments.
pub struct Fft {
    win: usize,
    plan: std::sync::Arc<dyn realfft::RealToComplex<f64>>,
    hann: Vec<f64>,
    input: Vec<f64>,
    out: Vec<realfft::num_complex::Complex<f64>>,
    pub bin_hz: f64,
}

impl Fft {
    pub fn new(win: usize, pad: usize, sample_rate: u32) -> Self {
        let n = pad.max(win);
        let plan = RealFftPlanner::<f64>::new().plan_fft_forward(n);
        let hann = (0..win).map(|i| (std::f64::consts::PI * i as f64 / win as f64).sin().powi(2)).collect();
        let (input, out) = (plan.make_input_vec(), plan.make_output_vec());
        Self { win, plan, hann, input, out, bin_hz: sample_rate as f64 / n as f64 }
    }

    /// Power spectrum of `samples[start..start + win]`; samples outside the slice count as zero.
    pub fn power(&mut self, samples: &[f32], start: i64) -> Vec<f64> {
        self.input.iter_mut().for_each(|x| *x = 0.0);
        for i in 0..self.win {
            let k = start + i as i64;
            if k >= 0 && (k as usize) < samples.len() {
                self.input[i] = samples[k as usize] as f64 * self.hann[i];
            }
        }
        self.plan.process(&mut self.input, &mut self.out).expect("fft sizes match");
        self.out.iter().map(|c| c.norm_sqr()).collect()
    }
}

/// Sub-bin position of the peak at bin `k` by parabolic interpolation on dB.
pub fn interp_bin(db: &[f64], k: usize) -> f64 {
    let (a, b, c) = (db[k - 1], db[k], db[k + 1]);
    let denom = a - 2.0 * b + c;
    k as f64 + if denom.abs() > 1e-12 { 0.5 * (a - c) / denom } else { 0.0 }
}

fn floor_db(db: &[f64], k: usize, bin_hz: f64, half_cents: f64) -> f64 {
    let f = k as f64 * bin_hz;
    let r = 2f64.powf(half_cents / 1200.0);
    let lo = ((f / r) / bin_hz) as usize;
    let hi = (((f * r) / bin_hz).ceil() as usize).min(db.len() - 1);
    let mut w: Vec<f64> = db[lo..=hi].to_vec();
    let mid = w.len() / 2;
    *w.select_nth_unstable_by(mid, f64::total_cmp).1
}

/// Prominent spectral lines of the long-term spectrum, strongest first.
pub fn find_peaks(audio: &Audio, p: &PeakParams) -> Vec<Peak> {
    let frames = Frames::new(audio, p.window_s, p.max_hz);
    let mut peaks = pick_peaks(&frames.aggregate(p.aggregate), frames.bin_hz, p);
    for &agg in &p.sparse_fallback {
        if peaks.len() >= p.sparse_min_lines {
            break;
        }
        let alt = pick_peaks(&frames.aggregate(agg), frames.bin_hz, p);
        if alt.len() > peaks.len() {
            peaks = alt;
        }
    }
    peaks
}

fn pick_peaks(power: &[f64], bin_hz: f64, p: &PeakParams) -> Vec<Peak> {
    let db: Vec<f64> = power.iter().map(|x| 10.0 * (x + 1e-20).log10()).collect();
    let lo = ((p.min_hz / bin_hz) as usize).max(3);
    let hi = ((p.max_hz / bin_hz) as usize).min(db.len() - 3);
    let mut peaks = Vec::new();
    for k in lo..hi {
        let v = db[k];
        if !(v > db[k - 1] && v >= db[k + 1] && v > db[k - 2] && v >= db[k + 2]) {
            continue;
        }
        let prom = v - floor_db(&db, k, bin_hz, p.floor_half_width_cents);
        if prom < p.min_prominence_db {
            continue;
        }
        peaks.push(Peak { hz: interp_bin(&db, k) * bin_hz, prominence_db: prom });
    }
    peaks.sort_by(|a, b| b.prominence_db.total_cmp(&a.prominence_db));
    peaks.truncate(p.max_peaks);
    peaks
}

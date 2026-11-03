use super::Audio;
use super::onsets::{self, OnsetParams};
use super::spectrum::{Fft, Peak, interp_bin};

#[derive(Debug, Clone)]
pub struct BlowParams {
    pub onsets: OnsetParams,
    /// Analysis window after each blow (skips the clapper transient) and before it.
    pub delay_s: f64,
    pub window_s: f64,
    pub min_hz: f64,
    pub max_hz: f64,
    /// dB by which a line must rise across the blow to count as struck.
    pub min_rise_db: f64,
    pub peaks_per_blow: usize,
    pub cluster_sigma_cents: f64,
    /// A line must recur in this share of blows (and at least twice).
    pub min_support: f64,
}

impl Default for BlowParams {
    fn default() -> Self {
        Self {
            onsets: OnsetParams::default(),
            delay_s: 0.02,
            window_s: 0.17,
            min_hz: 60.0,
            max_hz: 6000.0,
            // A re-struck bell is still ringing from its last blow, so its nominal rises only ~5-15 dB.
            min_rise_db: 5.0,
            peaks_per_blow: 10,
            cluster_sigma_cents: 10.0,
            min_support: 0.06,
        }
    }
}

/// (cents, rise dB) of the lines that appear at one blow.
fn blow_peaks(fft: &mut Fft, samples: &[f32], start: i64, pre_start: i64, p: &BlowParams) -> Vec<(f64, f64)> {
    let db = |v: Vec<f64>| v.into_iter().map(|x| 10.0 * (x + 1e-20).log10()).collect::<Vec<f64>>();
    let post = db(fft.power(samples, start));
    let pre = db(fft.power(samples, pre_start));
    let lo = ((p.min_hz / fft.bin_hz) as usize).max(2);
    let hi = ((p.max_hz / fft.bin_hz) as usize).min(post.len() - 3);
    let mut out: Vec<(f64, f64)> = (lo..hi)
        .filter(|&k| post[k] > post[k - 1] && post[k] >= post[k + 1] && post[k] > post[k - 2] && post[k] >= post[k + 2])
        .filter(|&k| post[k] - pre[k] >= p.min_rise_db)
        .map(|k| (1200.0 * (interp_bin(&post, k) * fft.bin_hz).log2(), post[k]))
        .collect();
    // Keep the loudest struck lines, not the biggest risers: fast-decaying upper partials always rise
    // most, while a sustaining nominal rises only a little over its previous blow.
    out.sort_by(|a, b| b.1.total_cmp(&a.1));
    out.truncate(p.peaks_per_blow);
    let top = out.first().map_or(0.0, |x| x.1);
    out.into_iter().map(|(c, level)| (c, (level - top + 40.0).max(1.0))).collect()
}

/// Groups per-blow lines into recurring lines by kernel density over cents.
fn cluster(points: &[(f64, f64)], blows: usize, p: &BlowParams) -> Vec<Peak> {
    if points.is_empty() {
        return Vec::new();
    }
    let s = p.cluster_sigma_cents;
    let lo = points.iter().map(|x| x.0).fold(f64::INFINITY, f64::min) - 3.0 * s;
    let hi = points.iter().map(|x| x.0).fold(f64::NEG_INFINITY, f64::max) + 3.0 * s;
    let n = (hi - lo).ceil() as usize + 1;
    let mut dens = vec![0.0; n];
    for &(c, _) in points {
        let (a, b) = (((c - 3.0 * s - lo).max(0.0)) as usize, ((c + 3.0 * s - lo) as usize).min(n - 1));
        for (i, d) in dens.iter_mut().enumerate().take(b + 1).skip(a) {
            *d += (-0.5 * ((lo + i as f64 - c) / s).powi(2)).exp();
        }
    }
    let need = (p.min_support * blows as f64).max(2.0);
    let mut out = Vec::new();
    for i in 1..n - 1 {
        if !(dens[i] > dens[i - 1] && dens[i] >= dens[i + 1]) {
            continue;
        }
        let centre = lo + i as f64;
        let members: Vec<&(f64, f64)> = points.iter().filter(|x| (x.0 - centre).abs() <= 2.0 * s).collect();
        if (members.len() as f64) < need {
            continue;
        }
        let wsum: f64 = members.iter().map(|x| x.1).sum();
        let cents = members.iter().map(|x| x.0 * x.1).sum::<f64>() / wsum;
        let mut rises: Vec<f64> = members.iter().map(|x| x.1).collect();
        rises.sort_by(f64::total_cmp);
        out.push(Peak { hz: 2f64.powf(cents / 1200.0), prominence_db: rises[rises.len() / 2] });
    }
    out.sort_by(|a, b| b.prominence_db.total_cmp(&a.prominence_db));
    out
}

/// Lines that appear at blows and recur across them, plus the number of blows found.
/// Steady tones (engines, hum) never rise at a blow, so they drop out here.
pub fn struck_lines(audio: &Audio, p: &BlowParams) -> (Vec<Peak>, usize) {
    let times = onsets::detect(audio, &p.onsets);
    let sr = audio.sample_rate as f64;
    let win = (p.window_s * sr) as usize;
    let mut fft = Fft::new(win, (win * 4).next_power_of_two(), audio.sample_rate);
    let points: Vec<(f64, f64)> = times
        .iter()
        .flat_map(|&t| {
            let start = ((t + p.delay_s) * sr) as i64;
            let pre = ((t - 0.01) * sr) as i64 - win as i64;
            blow_peaks(&mut fft, &audio.samples, start, pre, p)
        })
        .collect();
    (cluster(&points, times.len(), p), times.len())
}

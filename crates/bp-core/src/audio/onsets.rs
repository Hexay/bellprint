use super::Audio;
use super::spectrum::Fft;

#[derive(Debug, Clone)]
pub struct OnsetParams {
    pub min_hz: f64,
    pub max_hz: f64,
    /// Minimum gap between blows; rounds on 12 bells strike ~0.18 s apart.
    pub min_gap_s: f64,
    /// A frame is an onset when its flux exceeds `k` times the local mean flux.
    pub k: f64,
}

impl Default for OnsetParams {
    fn default() -> Self {
        Self { min_hz: 300.0, max_hz: 5000.0, min_gap_s: 0.12, k: 1.4 }
    }
}

const HOP_S: f64 = 0.01;
const WIN_S: f64 = 0.046;

/// Band-limited log spectral flux with a 3-bin max filter on the previous frame (SuperFlux-style),
/// so vibrato and beating doublets don't register as onsets.
fn flux(audio: &Audio, p: &OnsetParams) -> Vec<f64> {
    let sr = audio.sample_rate;
    let hop = (HOP_S * sr as f64) as usize;
    let win = (WIN_S * sr as f64) as usize;
    let mut fft = Fft::new(win, win.next_power_of_two(), sr);
    let (lo, hi) = ((p.min_hz / fft.bin_hz) as usize, (p.max_hz / fft.bin_hz) as usize);
    let mut prev: Option<Vec<f64>> = None;
    let mut out = Vec::new();
    let mut start = 0;
    while start + win <= audio.samples.len() {
        let pw = fft.power(&audio.samples, start as i64);
        let hi = hi.min(pw.len() - 2);
        let cur: Vec<f64> = pw[lo..=hi].iter().map(|x| (1.0 + 1e4 * x).ln()).collect();
        out.push(prev.as_ref().map_or(0.0, |pv| {
            (0..cur.len())
                .map(|i| {
                    let m = pv[i.saturating_sub(1)].max(pv[i]).max(pv[(i + 1).min(pv.len() - 1)]);
                    (cur[i] - m).max(0.0)
                })
                .sum()
        }));
        prev = Some(cur);
        start += hop;
    }
    out
}

/// Blow times in seconds: the end of the frame where energy jumps, so the strike is at or just before it.
pub fn detect(audio: &Audio, p: &OnsetParams) -> Vec<f64> {
    let f = flux(audio, p);
    let half_ctx = (0.3 / HOP_S) as usize;
    let gap = (p.min_gap_s / HOP_S) as usize;
    let mut sorted = f.clone();
    sorted.sort_by(f64::total_cmp);
    let floor = sorted.get(sorted.len() / 2).copied().unwrap_or(0.0);
    let mut out: Vec<usize> = Vec::new();
    for t in 1..f.len().saturating_sub(1) {
        let (a, b) = (t.saturating_sub(half_ctx), (t + half_ctx).min(f.len() - 1));
        let local_mean = f[a..=b].iter().sum::<f64>() / (b - a + 1) as f64;
        let is_max = (t.saturating_sub(5)..=(t + 5).min(f.len() - 1)).all(|u| f[u] <= f[t]);
        if is_max && f[t] > p.k * local_mean && f[t] > 1.5 * floor {
            match out.last() {
                Some(&last) if t - last < gap => {}
                _ => out.push(t),
            }
        }
    }
    out.into_iter().map(|t| t as f64 * HOP_S + WIN_S).collect()
}

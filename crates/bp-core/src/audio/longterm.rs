use super::Audio;
use super::spectrum::Fft;
use serde::Deserialize;

/// How per-frame power spectra combine into the long-term spectrum. In TOML: `aggregate = "mean"` or
/// `aggregate = { percentile = 75.0 }`.
#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Aggregate {
    /// Welch mean.
    Mean,
    /// Per-bin percentile (0-100). Ignores loud intermittent sounds (speech, traffic) that are absent
    /// from most frames.
    Percentile(f64),
}

/// Power spectra of every Hann frame (75 % overlap) of a clip, up to a frequency limit.
pub struct Frames {
    rows: Vec<Vec<f32>>,
    pub bin_hz: f64,
}

impl Frames {
    /// Frames of `window_s` up to `max_hz`, plus margin for the floor estimate.
    pub fn new(audio: &Audio, window_s: f64, max_hz: f64) -> Self {
        // Not rounded to a power of two: that made the window, and the ranks, depend on the sample
        // rate. Even a 1 % length change moves ranks (research/extraction_ab.md §13).
        let n = ((window_s * audio.sample_rate as f64).round() as usize).max(8);
        let mut fft = Fft::new(n, n, audio.sample_rate);
        let keep = ((max_hz * 1.2 * 2.0 / audio.sample_rate as f64 * n as f64) as usize + 4).min(n / 2 + 1);
        let mut rows = Vec::new();
        let mut start = 0;
        loop {
            rows.push(fft.power(&audio.samples, start as i64)[..keep].iter().map(|&x| x as f32).collect());
            start += n / 4;
            if start + n > audio.samples.len() {
                break;
            }
        }
        Self { rows, bin_hz: fft.bin_hz }
    }

    /// The long-term power spectrum under `agg`.
    pub fn aggregate(&self, agg: Aggregate) -> Vec<f64> {
        let frames = self.rows.len();
        let bins = self.rows[0].len();
        let Aggregate::Percentile(pct) = agg else {
            return (0..bins).map(|b| self.rows.iter().map(|r| r[b] as f64).sum::<f64>() / frames as f64).collect();
        };
        let k = ((pct / 100.0 * (frames - 1) as f64).round() as usize).min(frames - 1);
        let mut col = vec![0f32; frames];
        (0..bins)
            .map(|b| {
                col.iter_mut().zip(&self.rows).for_each(|(c, r)| *c = r[b]);
                *col.select_nth_unstable_by(k, f32::total_cmp).1 as f64
            })
            .collect()
    }
}

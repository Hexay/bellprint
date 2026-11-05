use super::Audio;
use super::spectrum::Fft;
use rayon::prelude::*;
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
    /// Experimental: mean with each frame weighted by the rise in broadband energy over the last
    /// `ONSET_SPAN` hops, so frames just after a blow dominate (research/extraction_ab.md §14).
    OnsetWeighted,
    /// Experimental: mean of frames each divided by its own median power in the bell band, so loud
    /// noisy frames don't set the floor (research/extraction_ab.md §14).
    FrameNormalized,
}

/// Band whose per-frame level drives the onset and normalisation weights.
const LEVEL_BAND_HZ: (f64, f64) = (300.0, 2500.0);
/// A blow stays in the 0.74 s window for 4 hops of a quarter window.
const ONSET_SPAN: usize = 4;

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
        let keep = ((max_hz * 1.2 * 2.0 / audio.sample_rate as f64 * n as f64) as usize + 4).min(n / 2 + 1);
        let mut starts = vec![0];
        while starts[starts.len() - 1] + n / 4 + n <= audio.samples.len() {
            starts.push(starts[starts.len() - 1] + n / 4);
        }
        let rows = starts
            .par_iter()
            .map_init(
                || Fft::new(n, n, audio.sample_rate),
                |fft, &start| fft.power(&audio.samples, start as i64)[..keep].iter().map(|&x| x as f32).collect(),
            )
            .collect();
        Self { rows, bin_hz: Fft::bin_hz(n, n, audio.sample_rate) }
    }

    /// The long-term power spectrum under `agg`.
    pub fn aggregate(&self, agg: Aggregate) -> Vec<f64> {
        match agg {
            Aggregate::Mean => self.weighted_mean(&vec![1.0; self.rows.len()]),
            Aggregate::Percentile(pct) => self.percentile(pct),
            Aggregate::OnsetWeighted => self.weighted_mean(&self.onset_weights()),
            Aggregate::FrameNormalized => {
                let w: Vec<f64> = self.band_levels(median).iter().map(|l| 1.0 / l).collect();
                self.weighted_mean(&w)
            }
        }
    }

    fn percentile(&self, pct: f64) -> Vec<f64> {
        let frames = self.rows.len();
        let k = ((pct / 100.0 * (frames - 1) as f64).round() as usize).min(frames - 1);
        let mut col = vec![0f32; frames];
        (0..self.rows[0].len())
            .map(|b| {
                col.iter_mut().zip(&self.rows).for_each(|(c, r)| *c = r[b]);
                *col.select_nth_unstable_by(k, f32::total_cmp).1 as f64
            })
            .collect()
    }

    /// Per-bin mean with frame weights; all-zero weights fall back to the plain mean.
    fn weighted_mean(&self, w: &[f64]) -> Vec<f64> {
        let total: f64 = w.iter().sum();
        let (w, total) = if total > 0.0 { (w.to_vec(), total) } else { (vec![1.0; w.len()], w.len() as f64) };
        let mut out = vec![0.0; self.rows[0].len()];
        for (r, &wt) in self.rows.iter().zip(&w) {
            out.iter_mut().zip(r).for_each(|(o, &x)| *o += wt * x as f64);
        }
        out.iter_mut().for_each(|o| *o /= total);
        out
    }

    /// `level` of each frame's power in `LEVEL_BAND_HZ` (floored above zero).
    fn band_levels(&self, level: impl Fn(&mut [f32]) -> f64) -> Vec<f64> {
        let at = |hz: f64| ((hz / self.bin_hz) as usize).min(self.rows[0].len());
        let (lo, hi) = (at(LEVEL_BAND_HZ.0), at(LEVEL_BAND_HZ.1).max(at(LEVEL_BAND_HZ.0) + 1));
        self.rows.iter().map(|r| level(&mut r[lo..hi.min(r.len())].to_vec()).max(1e-30)).collect()
    }

    /// Summed positive rise in log band energy over the current and previous `ONSET_SPAN - 1` frames.
    fn onset_weights(&self) -> Vec<f64> {
        let e: Vec<f64> = self.band_levels(|band| band.iter().map(|&x| x as f64).sum()).iter().map(|x| x.ln()).collect();
        let rise: Vec<f64> = (0..e.len()).map(|t| if t == 0 { 0.0 } else { (e[t] - e[t - 1]).max(0.0) }).collect();
        (0..e.len()).map(|t| rise[t.saturating_sub(ONSET_SPAN - 1)..=t].iter().sum()).collect()
    }
}

fn median(v: &mut [f32]) -> f64 {
    if v.is_empty() {
        return 0.0;
    }
    let mid = v.len() / 2;
    *v.select_nth_unstable_by(mid, f32::total_cmp).1 as f64
}

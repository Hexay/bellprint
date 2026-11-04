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

/// Calls `each` with the power spectrum of every Hann frame (75 % overlap); returns Hz per bin.
fn for_each_frame(audio: &Audio, window_s: f64, mut each: impl FnMut(Vec<f64>)) -> f64 {
    let n = ((window_s * audio.sample_rate as f64) as usize).next_power_of_two();
    let mut fft = Fft::new(n, n, audio.sample_rate);
    let mut start = 0;
    loop {
        each(fft.power(&audio.samples, start as i64));
        start += n / 4;
        if start + n > audio.samples.len() {
            break;
        }
    }
    fft.bin_hz
}

/// Long-term spectrum up to `max_hz` (plus margin for the floor estimate). Returns (power per bin, Hz per bin).
pub fn long_term_power(audio: &Audio, window_s: f64, max_hz: f64, agg: Aggregate) -> (Vec<f64>, f64) {
    let keep = |len: usize| ((max_hz * 1.2 * 2.0 / audio.sample_rate as f64 * len as f64) as usize + 4).min(len);
    let (mut sum, mut frames, mut stored): (Vec<f64>, usize, Vec<Vec<f32>>) = (Vec::new(), 0, Vec::new());
    let bin_hz = for_each_frame(audio, window_s, |mut p| {
        p.truncate(keep(p.len()));
        frames += 1;
        match agg {
            Aggregate::Mean => {
                sum.resize(p.len(), 0.0);
                sum.iter_mut().zip(p).for_each(|(a, x)| *a += x);
            }
            Aggregate::Percentile(_) => stored.push(p.into_iter().map(|x| x as f32).collect()),
        }
    });
    let Aggregate::Percentile(pct) = agg else {
        sum.iter_mut().for_each(|a| *a /= frames as f64);
        return (sum, bin_hz);
    };
    let k = ((pct / 100.0 * (frames - 1) as f64).round() as usize).min(frames - 1);
    let mut col = vec![0f32; frames];
    let out = (0..stored[0].len())
        .map(|b| {
            col.iter_mut().zip(&stored).for_each(|(c, f)| *c = f[b]);
            *col.select_nth_unstable_by(k, f32::total_cmp).1 as f64
        })
        .collect();
    (out, bin_hz)
}

use super::Audio;
use super::onsets::{self, OnsetParams};
use super::spectrum::{PeakParams, find_peaks};
use serde::Serialize;

#[derive(Debug, Clone)]
pub struct WindowParams {
    pub window_s: f64,
    pub hop_s: f64,
    /// Clips no longer than this are used whole.
    pub min_clip_s: f64,
    /// How many of the best windows to keep.
    pub keep: usize,
    /// Plausible gaps between consecutive blows of a ring (rounds on 3–16 bells, or slow tolling).
    pub min_gap_s: f64,
    pub max_gap_s: f64,
}

impl Default for WindowParams {
    fn default() -> Self {
        Self { window_s: 10.0, hop_s: 5.0, min_clip_s: 20.0, keep: 3, min_gap_s: 0.15, max_gap_s: 4.0 }
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
pub struct Window {
    pub start_s: f64,
    pub end_s: f64,
    pub score: f64,
}

/// Bell-likeness of one window: blows at a ringing-like rate, times how tonal the window is
/// (mean prominence of its strongest lines). Speech and traffic have onsets but few steady lines;
/// a drone has lines but no blows.
fn score(audio: &Audio, onset_times: &[f64], w: &Window, p: &WindowParams) -> f64 {
    let inside: Vec<f64> = onset_times.iter().copied().filter(|t| (w.start_s..w.end_s).contains(t)).collect();
    let ringing_gaps = inside.windows(2).filter(|g| (p.min_gap_s..=p.max_gap_s).contains(&(g[1] - g[0]))).count();
    if ringing_gaps < 2 {
        return 0.0;
    }
    let peaks = find_peaks(&audio.slice(w.start_s, Some(w.end_s)), &PeakParams { max_peaks: 15, ..Default::default() });
    let tonal = peaks.iter().map(|x| x.prominence_db).sum::<f64>() / 15.0;
    (ringing_gaps as f64).sqrt() * tonal
}

/// The best-scoring non-overlapping windows, best first. Empty when the clip is short enough to use whole.
pub fn ringing_windows(audio: &Audio, p: &WindowParams) -> Vec<Window> {
    let dur = audio.duration_s();
    if dur <= p.min_clip_s {
        return Vec::new();
    }
    let onset_times = onsets::detect(audio, &OnsetParams::default());
    let mut all: Vec<Window> = (0..)
        .map(|i| i as f64 * p.hop_s)
        .take_while(|s| s + p.window_s <= dur + p.hop_s / 2.0)
        .map(|s| {
            let mut w = Window { start_s: s, end_s: (s + p.window_s).min(dur), score: 0.0 };
            w.score = score(audio, &onset_times, &w, p);
            w
        })
        .collect();
    all.sort_by(|a, b| b.score.total_cmp(&a.score));
    let mut kept: Vec<Window> = Vec::new();
    for w in all.into_iter().filter(|w| w.score > 0.0) {
        if kept.len() == p.keep {
            break;
        }
        if kept.iter().all(|k| w.end_s <= k.start_s || w.start_s >= k.end_s) {
            kept.push(w);
        }
    }
    kept
}

/// The windows' audio joined end to end, in time order.
pub fn concat(audio: &Audio, windows: &[Window]) -> Audio {
    let mut ws = windows.to_vec();
    ws.sort_by(|a, b| a.start_s.total_cmp(&b.start_s));
    let samples = ws.iter().flat_map(|w| audio.slice(w.start_s, Some(w.end_s)).samples).collect();
    Audio { samples, sample_rate: audio.sample_rate }
}

use crate::audio::Audio;
use crate::audio::blows::{BlowParams, struck_lines};
use crate::audio::spectrum::{Peak, PeakParams, find_peaks};
use crate::audio::window::{Window, WindowParams, concat, ringing_windows};
use crate::catalog::Catalog;
use crate::geo::Region;
use crate::matcher::model::cents;
use crate::matcher::{Match, MatchParams, rank_rings};
use serde::{Deserialize, Serialize};

/// How spectral lines are taken from the clip.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Extraction {
    /// Peaks of the clip's long-term (per-bin percentile) spectrum: everything tonal, bells or not.
    #[default]
    LongTerm,
    /// Experimental: lines that rise at blows and recur, snapped to the long-term spectrum.
    /// Currently loses to `LongTerm` on the sim benchmark.
    Blows,
}

#[derive(Debug, Clone)]
pub struct LocateOptions {
    pub region: Option<Region>,
    pub extraction: Extraction,
    /// Analyse only the most bell-like windows of long clips. Off by default: on the real corpus it
    /// lost to the whole clip (8 vs 10 top-1 of 89).
    pub auto_window: bool,
    pub window: WindowParams,
    pub peaks: PeakParams,
    pub blows: BlowParams,
    pub matcher: MatchParams,
    pub top: usize,
}

impl Default for LocateOptions {
    fn default() -> Self {
        Self {
            region: None,
            extraction: Extraction::default(),
            auto_window: false,
            window: WindowParams::default(),
            peaks: PeakParams::default(),
            blows: BlowParams::default(),
            matcher: MatchParams::default(),
            top: 10,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Report {
    pub duration_s: f64,
    /// Windows analysed; empty means the whole clip.
    pub windows: Vec<Window>,
    pub region: Option<Region>,
    /// Extraction actually used (blows falls back to long-term when too few struck lines are found).
    pub extraction: Extraction,
    pub blows: usize,
    /// Band the lines were searched in.
    pub band_hz: (f64, f64),
    pub peaks: Vec<Peak>,
    pub matches: Vec<Match>,
}

const SNAP_CENTS: f64 = 15.0;
const MIN_STRUCK_LINES: usize = 3;

/// Replaces each struck line's frequency with the nearest long-term peak, whose longer window is more precise.
fn snap(struck: Vec<Peak>, long: &[Peak]) -> Vec<Peak> {
    struck
        .into_iter()
        .map(|s| {
            let near = long
                .iter()
                .map(|l| (l, (cents(l.hz) - cents(s.hz)).abs()))
                .filter(|x| x.1 <= SNAP_CENTS)
                .min_by(|a, b| a.1.total_cmp(&b.1));
            near.map_or(s, |(l, _)| Peak { hz: l.hz, ..s })
        })
        .collect()
}

/// Spectral lines per `o.extraction`, the extraction used, and the blow count.
pub fn extract_lines(audio: &Audio, o: &LocateOptions) -> (Vec<Peak>, Extraction, usize) {
    let long = find_peaks(audio, &o.peaks);
    if o.extraction == Extraction::LongTerm {
        return (long, Extraction::LongTerm, 0);
    }
    let (struck, blows) = struck_lines(audio, &o.blows);
    if struck.len() < MIN_STRUCK_LINES {
        return (long, Extraction::LongTerm, blows);
    }
    (snap(struck, &long), Extraction::Blows, blows)
}

pub fn locate_audio(catalog: &Catalog, audio: &Audio, o: &LocateOptions) -> Report {
    let windows = if o.auto_window { ringing_windows(audio, &o.window) } else { Vec::new() };
    let clip = if windows.is_empty() { audio.clone() } else { concat(audio, &windows) };
    let (peaks, extraction, blows) = extract_lines(&clip, o);
    let band = match extraction {
        Extraction::LongTerm => (o.peaks.min_hz, o.peaks.max_hz),
        Extraction::Blows => (o.blows.min_hz, o.blows.max_hz),
    };
    let matches = rank_rings(catalog, &peaks, band, o.region.as_ref(), &o.matcher, o.top.max(1));
    Report { duration_s: audio.duration_s(), windows, region: o.region, extraction, blows, band_hz: band, peaks, matches }
}

/// 1-based rank of the first match in `tower_id`, if it made the list.
pub fn tower_rank(report: &Report, tower_id: u32) -> Option<usize> {
    rank_of(&report.matches, tower_id)
}

pub fn rank_of(matches: &[Match], tower_id: u32) -> Option<usize> {
    matches.iter().position(|m| m.ring.tower_id == tower_id).map(|i| i + 1)
}

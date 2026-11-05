use super::Case;
use crate::audio::spectrum::Peak;
use crate::catalog::Catalog;
use crate::locate::{LocateOptions, Report, rank_of};
use crate::matcher::{Match, rank_rings, top_confidence};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

/// One evaluated clip's extracted lines and its truth: enough to re-rank it without the audio, so a
/// matcher change can be A/B'd in seconds.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Clip {
    pub id: String,
    pub tower_id: u32,
    pub bells: usize,
    pub note: String,
    pub band_hz: (f64, f64),
    /// (hz, prominence dB) per line.
    pub lines: Vec<(f64, f64)>,
    /// Sim only: the nominals actually rendered (Dove error, retune and speed applied), for measuring
    /// line precision against truth.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub truth_hz: Vec<f64>,
}

impl Clip {
    pub fn new(id: String, tower_id: u32, bells: usize, report: &Report, note: String) -> Self {
        let lines = report.peaks.iter().map(|p| (p.hz, p.prominence_db)).collect();
        Self { id, tower_id, bells, note, band_hz: report.band_hz, lines, truth_hz: Vec::new() }
    }

    pub fn peaks(&self) -> Vec<Peak> {
        self.lines.iter().map(|&(hz, prominence_db)| Peak { hz, prominence_db }).collect()
    }

    pub fn case(&self, matches: &[Match]) -> Case {
        let top = matches.first();
        let truth = matches.iter().find(|m| m.ring.tower_id == self.tower_id);
        Case {
            id: self.id.clone(),
            tower_id: self.tower_id,
            bells: self.bells,
            rank: rank_of(matches, self.tower_id),
            top_tower_id: top.map(|m| m.ring.tower_id),
            top_place: top.map_or(String::new(), |m| format!("{} ({})", m.ring.place, m.ring.dedication)),
            top_probability: top.map_or(0.0, |m| m.probability),
            top_margin: top.map_or(0.0, |m| m.margin),
            top_shift_cents: top.map(|m| m.shift_cents),
            true_shift_cents: truth.map(|m| m.shift_cents),
            true_gap: top.zip(truth).map(|(t, m)| t.score - m.score),
            confidence: top_confidence(matches),
            peaks: self.lines.len(),
            note: self.note.clone(),
        }
    }
}

/// Re-ranks cached clips with `opts.matcher`; extraction options are ignored.
pub fn replay(catalog: &Catalog, clips: &[Clip], opts: &LocateOptions) -> Vec<Case> {
    clips
        .par_iter()
        .map(|c| {
            let region = opts.location.region.as_ref();
            c.case(&rank_rings(catalog, &c.peaks(), c.band_hz, region, &opts.matcher, opts.top.max(1)))
        })
        .collect()
}

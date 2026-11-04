mod confidence;
pub mod model;
mod score;

pub use confidence::{Confidence, HIGH_MARGIN_NATS, MEDIUM_MARGIN_NATS, top_confidence};
use crate::audio::spectrum::Peak;
use crate::catalog::{Catalog, Ring};
use crate::geo::Region;
pub use model::MatchParams;
use rayon::prelude::*;
use score::{Lines, RingModel, RingScore, score_ring};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Match {
    pub ring: Ring,
    /// Log-likelihood-ratio style score; only differences between candidates mean anything.
    pub score: f64,
    /// Score lead over the next-ranked candidate (nats); the top match's margin says how unique it is.
    pub margin: f64,
    /// Softmax of scores over the shortlist.
    pub probability: f64,
    pub shift_cents: f64,
    pub bells_heard: usize,
    pub distance_km: Option<f64>,
}

/// Best score of the whole ring or of any contiguous run of it that drops up to `max_bells_dropped`
/// bells from the front or back, each run charged `subset_log_prior`.
fn score_ring_or_subset(ring: &Ring, lines: &Lines, p: &MatchParams, grid: &[(f64, f64)], full: bool) -> RingScore {
    let (hz, years) = (&ring.nominals, &ring.cast_years);
    let n = hz.len();
    let mut best = score_ring(&RingModel::new(hz, years, p), lines, p, grid, full);
    for d in 1..=p.max_bells_dropped.min(n.saturating_sub(p.min_subset_bells)) {
        for (run, run_years) in [(&hz[d..], &years[d..]), (&hz[..n - d], &years[..n - d])] {
            let mut s = score_ring(&RingModel::new(run, run_years, p), lines, p, grid, full);
            s.score += p.subset_log_prior;
            if s.score > best.score {
                best = s;
            }
        }
    }
    best
}

/// Ranks catalog rings by how well they explain `peaks`, found in `band_hz`, best first. `region`
/// restricts the candidates.
pub fn rank_rings(
    catalog: &Catalog,
    peaks: &[Peak],
    band_hz: (f64, f64),
    region: Option<&Region>,
    p: &MatchParams,
    top: usize,
) -> Vec<Match> {
    let lines = Lines::from_peaks(peaks, p.full_strength_db, band_hz);
    let grid = model::shift_grid(p);
    let candidates: Vec<&Ring> =
        catalog.rings.iter().filter(|r| region.is_none_or(|g| g.contains(r.lat, r.lon))).collect();

    let mut coarse: Vec<(usize, f64)> = candidates
        .par_iter()
        .enumerate()
        .map(|(i, r)| (i, score_ring_or_subset(r, &lines, p, &grid, false).score))
        .collect();
    coarse.sort_by(|a, b| b.1.total_cmp(&a.1));
    coarse.truncate(p.shortlist);

    let mut full: Vec<(usize, RingScore)> = coarse
        .par_iter()
        .map(|&(i, _)| (i, score_ring_or_subset(candidates[i], &lines, p, &grid, true)))
        .collect();
    full.sort_by(|a, b| b.1.score.total_cmp(&a.1.score));

    let top_score = full.first().map_or(0.0, |x| x.1.score);
    let z: f64 = full.iter().map(|x| (x.1.score - top_score).exp()).sum();
    let next: Vec<f64> = full.iter().skip(1).map(|x| x.1.score).chain([f64::NEG_INFINITY]).collect();
    full.into_iter()
        .zip(next)
        .take(top)
        .map(|((i, s), next)| {
            let ring = candidates[i].clone();
            Match {
                distance_km: region.map(|g| g.distance_km(ring.lat, ring.lon)),
                ring,
                score: s.score,
                margin: s.score - next,
                probability: (s.score - top_score).exp() / z,
                shift_cents: s.shift_cents,
                bells_heard: s.bells_heard,
            }
        })
        .collect()
}

pub mod model;
mod score;

use crate::audio::spectrum::Peak;
use crate::catalog::{Catalog, Ring};
use crate::geo::Region;
pub use model::MatchParams;
use rayon::prelude::*;
use score::{Lines, RingModel, score_ring};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Match {
    pub ring: Ring,
    /// Log-likelihood-ratio style score; only differences between candidates mean anything.
    pub score: f64,
    /// Softmax of scores over the shortlist.
    pub probability: f64,
    pub shift_cents: f64,
    pub bells_heard: usize,
    pub distance_km: Option<f64>,
}

/// Ranks catalog rings by how well they explain `peaks`, best first. `region` restricts the candidates.
pub fn rank_rings(
    catalog: &Catalog,
    peaks: &[Peak],
    region: Option<&Region>,
    p: &MatchParams,
    top: usize,
) -> Vec<Match> {
    let lines = Lines::from_peaks(peaks, p.full_strength_db);
    let grid = model::shift_grid(p);
    let candidates: Vec<&Ring> =
        catalog.rings.iter().filter(|r| region.is_none_or(|g| g.contains(r.lat, r.lon))).collect();

    let mut coarse: Vec<(usize, f64)> = candidates
        .par_iter()
        .enumerate()
        .map(|(i, r)| (i, score_ring(&RingModel::new(&r.nominals, p), &lines, p, &grid, false).score))
        .collect();
    coarse.sort_by(|a, b| b.1.total_cmp(&a.1));
    coarse.truncate(p.shortlist);

    let mut full: Vec<(usize, score::RingScore)> = coarse
        .par_iter()
        .map(|&(i, _)| (i, score_ring(&RingModel::new(&candidates[i].nominals, p), &lines, p, &grid, true)))
        .collect();
    full.sort_by(|a, b| b.1.score.total_cmp(&a.1.score));

    let top_score = full.first().map_or(0.0, |x| x.1.score);
    let z: f64 = full.iter().map(|x| (x.1.score - top_score).exp()).sum();
    full.into_iter()
        .take(top)
        .map(|(i, s)| {
            let ring = candidates[i].clone();
            Match {
                distance_km: region.map(|g| g.distance_km(ring.lat, ring.lon)),
                ring,
                score: s.score,
                probability: (s.score - top_score).exp() / z,
                shift_cents: s.shift_cents,
                bells_heard: s.bells_heard,
            }
        })
        .collect()
}

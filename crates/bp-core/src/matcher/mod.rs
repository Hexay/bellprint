mod confidence;
pub mod model;
mod nominal;
mod partials;
mod score;

pub use confidence::{Confidence, HIGH_MARGIN_NATS, MEDIUM_MARGIN_NATS, top_confidence};
use crate::audio::spectrum::Peak;
use crate::catalog::{Catalog, Ring};
use crate::geo::Region;
pub use model::{MatchParams, NominalShape};
use rayon::prelude::*;
use partials::RingPartials;
use score::{BellTable, Lines, RingScore, score_ring};
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

/// The bell sets scored for `ring`, in groups with their log prior: the whole ring and, with
/// `without_optional_log_prior`, the ring less its optional bells. A group lists its bells first, then
/// each contiguous run of them that drops up to `max_bells_dropped` from the back or front.
fn bell_groups(ring: &Ring, p: &MatchParams) -> Vec<(Vec<Vec<usize>>, f64)> {
    let all: Vec<usize> = (0..ring.nominals.len()).collect();
    let mut keeps = vec![(all.clone(), 0.0)];
    if let Some(prior) = p.without_optional_log_prior
        && ring.optional.contains(&true)
    {
        let core: Vec<usize> = all.into_iter().filter(|&i| !ring.optional[i]).collect();
        if !core.is_empty() {
            keeps.push((core, prior));
        }
    }
    keeps
        .into_iter()
        .map(|(keep, prior)| {
            let n = keep.len();
            let mut runs = vec![keep.clone()];
            for d in 1..=p.max_bells_dropped.min(n.saturating_sub(p.min_subset_bells)) {
                runs.extend([d..n, 0..n - d].map(|r| keep[r].to_vec()));
            }
            (runs, prior)
        })
        .collect()
}

/// Best score over `bell_groups`, runs charged `subset_log_prior` and groups their prior. `full`
/// adds how well each run explains every observed line.
fn score_ring_or_subset(ring: &Ring, bells: &BellTable, lines: &Lines, p: &MatchParams, grid: &[(f64, f64)], full: bool) -> RingScore {
    let groups = bell_groups(ring, p);
    let terms = bells.ring(ring);
    let sets: Vec<&[usize]> = groups.iter().flat_map(|g| &g.0).map(Vec::as_slice).collect();
    let bases: Vec<_> = sets.iter().map(|b| terms.base(b, grid)).collect();
    let line_terms = full.then(|| {
        let partials = RingPartials::new(ring, p);
        let base: Vec<&[f64]> = bases.iter().map(|b| b.0.as_slice()).collect();
        partials.line_terms(&partials.runs(&sets, ring.nominals.len()), lines, grid, &base, p)
    });
    let mut line_terms = line_terms.iter().flatten().map(Vec::as_slice);
    let mut bases = bases.iter();
    let mut best = RingScore { score: f64::NEG_INFINITY, shift_cents: 0.0, bells_heard: 0 };
    for (runs, prior) in &groups {
        let mut group = score_ring(bases.next().unwrap(), line_terms.next(), grid);
        for _ in &runs[1..] {
            let mut s = score_ring(bases.next().unwrap(), line_terms.next(), grid);
            s.score += p.subset_log_prior;
            if s.score > group.score {
                group = s;
            }
        }
        group.score += prior;
        if group.score > best.score {
            best = group;
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
    let bells = BellTable::new(&candidates, &lines, &grid, p);

    let mut coarse: Vec<(usize, f64)> = candidates
        .par_iter()
        .enumerate()
        .map(|(i, r)| (i, score_ring_or_subset(r, &bells, &lines, p, &grid, false).score))
        .collect();
    coarse.sort_by(|a, b| b.1.total_cmp(&a.1));
    coarse.truncate(p.shortlist);

    let mut full: Vec<(usize, RingScore)> = coarse
        .par_iter()
        .map(|&(i, _)| (i, score_ring_or_subset(candidates[i], &bells, &lines, p, &grid, true)))
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

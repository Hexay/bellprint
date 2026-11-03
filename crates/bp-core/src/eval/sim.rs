use super::Case;
use crate::catalog::{Catalog, Ring};
use crate::locate::{LocateOptions, Report, locate_audio, tower_rank};
use crate::sim::{SimParams, simulate};
use rand::rngs::StdRng;
use rand::seq::IndexedRandom;
use rand::SeedableRng;
use rayon::prelude::*;

#[derive(Debug, Clone)]
pub struct SimEvalConfig {
    pub n: usize,
    pub seed: u64,
    pub sim: SimParams,
    /// Only rings whose every bell has a Dove nominal (the sim needs them all).
    pub complete_only: bool,
    pub min_bells: usize,
    pub country: Option<String>,
}

pub(super) fn case(id: String, tower_id: u32, bells: usize, report: &Report, note: String) -> Case {
    let top = report.matches.first();
    Case {
        id,
        tower_id,
        bells,
        rank: tower_rank(report, tower_id),
        top_tower_id: top.map(|m| m.ring.tower_id),
        top_place: top.map_or(String::new(), |m| format!("{} ({})", m.ring.place, m.ring.dedication)),
        top_probability: top.map_or(0.0, |m| m.probability),
        peaks: report.peaks.len(),
        note,
    }
}

/// `cfg.n` rings drawn (with replacement) from the eligible pool, each with its own sim seed.
pub fn pick_rings<'a>(catalog: &'a Catalog, cfg: &SimEvalConfig) -> Vec<(&'a Ring, u64)> {
    let pool: Vec<&Ring> = catalog
        .rings
        .iter()
        .filter(|r| !cfg.complete_only || r.complete())
        .filter(|r| r.nominals.len() >= cfg.min_bells)
        .filter(|r| cfg.country.as_ref().is_none_or(|c| &r.country == c))
        .collect();
    let mut rng = StdRng::seed_from_u64(cfg.seed);
    (0..cfg.n).filter_map(|i| pool.choose(&mut rng).map(|r| (*r, cfg.seed.wrapping_add(i as u64)))).collect()
}

/// Simulates `cfg.n` random rings and locates each against the whole catalog.
pub fn eval_sim(catalog: &Catalog, cfg: &SimEvalConfig, opts: &LocateOptions) -> Vec<Case> {
    pick_rings(catalog, cfg)
        .par_iter()
        .map(|&(ring, seed)| {
            let (audio, truth) = simulate(ring, &cfg.sim, seed);
            let report = locate_audio(catalog, &audio, opts);
            let note = format!("speed {:+.0}c", truth.speed_cents);
            case(format!("sim-{}-{seed}", ring.ring_id), ring.tower_id, ring.nominals.len(), &report, note)
        })
        .collect()
}

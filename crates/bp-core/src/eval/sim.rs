use super::{Case, Clip};
use crate::catalog::{Catalog, Ring};
use crate::locate::{LocateOptions, locate_audio};
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
pub fn eval_sim(catalog: &Catalog, cfg: &SimEvalConfig, opts: &LocateOptions) -> Vec<(Case, Clip)> {
    pick_rings(catalog, cfg)
        .par_iter()
        .map(|&(ring, seed)| {
            let (audio, truth) = simulate(ring, &cfg.sim, seed);
            let report = locate_audio(catalog, &audio, opts);
            let note = format!("speed {:+.0}c", truth.speed_cents);
            let id = format!("sim-{}-{seed}", ring.ring_id);
            let clip = Clip::new(id, ring.tower_id, ring.nominals.len(), &report, note);
            (clip.case(&report.matches), clip)
        })
        .collect()
}

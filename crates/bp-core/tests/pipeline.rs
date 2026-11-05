use bp_core::catalog::Catalog;
use bp_core::locate::{LocateOptions, locate_audio, tower_rank};
use bp_core::matcher::{MatchParams, NominalShape};
use bp_core::sim::{SimParams, simulate};
use std::sync::OnceLock;

fn catalog() -> &'static Catalog {
    static C: OnceLock<Catalog> = OnceLock::new();
    C.get_or_init(Catalog::embedded)
}

#[test]
fn catalog_has_ab_kettleby() {
    let c = catalog();
    assert!(c.rings.len() > 6000, "rings: {}", c.rings.len());
    let r = c.ring(2575).expect("Ab Kettleby ring");
    assert_eq!(r.place, "Ab Kettleby");
    assert_eq!(r.nominals, vec![1317.0, 1173.0, 1064.0, 1001.0, 879.0, 783.0]);
}

fn clean() -> SimParams {
    SimParams { snr_db: 30.0, speed_sd_cents: 0.0, dove_error_sd_cents: 0.0, retune_prob: 0.0, ..Default::default() }
}

fn top_place(report: &bp_core::locate::Report) -> Option<&String> {
    report.matches.first().map(|m| &m.ring.place)
}

#[test]
fn clean_sim_ranks_its_own_tower_first() {
    let c = catalog();
    let ring = c.ring(2575).unwrap();
    let (audio, _) = simulate(ring, &clean(), 7);
    let report = locate_audio(c, &audio, &LocateOptions { top: 20, ..Default::default() });
    assert_eq!(tower_rank(&report, ring.tower_id), Some(1), "top: {:?}", top_place(&report));
}

/// Trebles above the analysed band can't be heard and must not count as missed.
#[test]
fn light_ring_with_trebles_above_the_band_still_ranks() {
    let c = catalog();
    let max_hz = LocateOptions::default().peaks.max_hz;
    let ring = c
        .rings
        .iter()
        .find(|r| r.nominals.len() >= 6 && r.complete() && r.nominals[0] > max_hz * 1.05 && r.nominals[2] < max_hz)
        .unwrap();
    let (audio, _) = simulate(ring, &clean(), 7);
    let report = locate_audio(c, &audio, &LocateOptions { top: 20, ..Default::default() });
    let rank = tower_rank(&report, ring.tower_id);
    assert!(rank.is_some_and(|r| r <= 3), "{} {:?}: rank {rank:?}", ring.place, ring.nominals);
}

fn eight(index: usize) -> &'static bp_core::catalog::Ring {
    catalog().rings.iter().filter(|r| r.nominals.len() == 8 && r.complete()).nth(index).unwrap()
}

fn bells(ring: &bp_core::catalog::Ring, keep: std::ops::Range<usize>) -> bp_core::catalog::Ring {
    bp_core::catalog::Ring {
        nominals: ring.nominals[keep.clone()].to_vec(),
        cast_years: ring.cast_years[keep.clone()].to_vec(),
        optional: ring.optional[keep].to_vec(),
        ..ring.clone()
    }
}

fn rank_of(audio: &bp_core::audio::Audio, tower_id: u32, matcher: MatchParams) -> usize {
    let report = locate_audio(catalog(), audio, &LocateOptions { top: 50, matcher, ..Default::default() });
    tower_rank(&report, tower_id).unwrap_or(usize::MAX)
}

fn no_subsets() -> MatchParams {
    MatchParams { subset_log_prior: f64::NEG_INFINITY, ..Default::default() }
}

/// Fixtures picked from a sweep (research/matcher.md, Ring subsets) where the default matcher
/// needs the subset to reach the top 3.
#[test]
fn default_matcher_needs_subsets_for_six_of_eight() {
    for (index, keep, seed) in [(1420, 2..8, 1), (1420, 2..8, 7), (1420, 0..6, 7), (1278, 0..6, 7)] {
        let ring = eight(index);
        let (audio, _) = simulate(&bells(ring, keep.clone()), &clean(), seed);
        let with = rank_of(&audio, ring.tower_id, MatchParams::default());
        let without = rank_of(&audio, ring.tower_id, no_subsets());
        assert!(with <= 3 && with < without, "{} {keep:?} seed {seed}: rank {with} with subsets, {without} without", ring.place);
    }
}

/// The back 6 of a common 8 is shared by many 6-bell rings, so only a clear rise is asked for.
#[test]
fn ringing_the_back_six_of_eight_still_finds_the_eight() {
    let ring = eight(0);
    let (audio, _) = simulate(&bells(ring, 2..8), &clean(), 7);
    let rank = |matcher: MatchParams| rank_of(&audio, ring.tower_id, matcher);
    // The heavy-tailed nominal term credits the silent trebles at chance lines, so the subset's effect
    // on rank shows only with the Gaussian term.
    let gaussian = MatchParams { nominal_shape: NominalShape::Gaussian, nominal_reach_sigmas: 3.0, ..Default::default() };
    let with = rank(gaussian.clone());
    let without = rank(MatchParams { subset_log_prior: f64::NEG_INFINITY, ..gaussian });
    assert!(with <= 15 && with < without, "{}: rank {with} with subsets, {without} without", ring.place);
    let default = rank(MatchParams::default());
    assert!(default <= 15, "{}: rank {default} with the default matcher", ring.place);
}

#[test]
fn semitone_and_extra_treble_are_optional() {
    let r = catalog().ring(4173).expect("Wakefield ring");
    let marked: Vec<f64> = r.nominals.iter().zip(&r.optional).filter(|x| *x.1).map(|x| *x.0).collect();
    assert_eq!(marked, vec![1735.0, 920.5]);
}

#[test]
fn ringing_without_the_semitone_still_finds_the_ring() {
    let c = catalog();
    let ring = c.ring(4173).expect("Wakefield ring");
    let keep: Vec<usize> = (0..ring.nominals.len()).filter(|&i| !ring.optional[i]).collect();
    let diatonic = bp_core::catalog::Ring {
        nominals: keep.iter().map(|&i| ring.nominals[i]).collect(),
        cast_years: keep.iter().map(|&i| ring.cast_years[i]).collect(),
        optional: vec![false; keep.len()],
        ..ring.clone()
    };
    let (audio, _) = simulate(&diatonic, &clean(), 7);
    let rank = |matcher: MatchParams| {
        let report = locate_audio(c, &audio, &LocateOptions { top: 50, matcher, ..Default::default() });
        tower_rank(&report, ring.tower_id).unwrap_or(usize::MAX)
    };
    let with = rank(MatchParams::default());
    let without = rank(MatchParams { without_optional_log_prior: None, ..Default::default() });
    assert!(with == 1 && with <= without, "rank {with} with the drop, {without} without");
}

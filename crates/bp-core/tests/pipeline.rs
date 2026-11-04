use bp_core::catalog::Catalog;
use bp_core::locate::{LocateOptions, locate_audio, tower_rank};
use bp_core::matcher::MatchParams;
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

/// The back 6 of a common 8 is shared by many 6-bell rings, so only a clear rise is asked for.
#[test]
fn ringing_the_back_six_of_eight_still_finds_the_eight() {
    let c = catalog();
    let ring = c.rings.iter().find(|r| r.nominals.len() == 8 && r.complete()).unwrap();
    let back6 = bp_core::catalog::Ring { nominals: ring.nominals[2..].to_vec(), ..ring.clone() };
    let (audio, _) = simulate(&back6, &clean(), 7);
    let rank = |matcher: MatchParams| {
        let report = locate_audio(c, &audio, &LocateOptions { top: 50, matcher, ..Default::default() });
        tower_rank(&report, ring.tower_id).unwrap_or(usize::MAX)
    };
    let with = rank(MatchParams::default());
    let without = rank(MatchParams { subset_log_prior: f64::NEG_INFINITY, ..Default::default() });
    assert!(with <= 10 && with < without, "{}: rank {with} with subsets, {without} without", ring.place);
}

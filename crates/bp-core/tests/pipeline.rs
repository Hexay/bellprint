use bp_core::catalog::Catalog;
use bp_core::locate::{LocateOptions, locate_audio, tower_rank};
use bp_core::sim::{SimParams, simulate};
use std::path::Path;
use std::sync::OnceLock;

fn catalog() -> &'static Catalog {
    static C: OnceLock<Catalog> = OnceLock::new();
    C.get_or_init(|| {
        Catalog::load(&Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/dove")).expect("data/dove present")
    })
}

#[test]
fn catalog_has_ab_kettleby() {
    let c = catalog();
    assert!(c.rings.len() > 6000, "rings: {}", c.rings.len());
    let r = c.ring(2575).expect("Ab Kettleby ring");
    assert_eq!(r.place, "Ab Kettleby");
    assert_eq!(r.nominals, vec![1317.0, 1173.0, 1064.0, 1001.0, 879.0, 783.0]);
}

#[test]
fn clean_sim_ranks_its_own_tower_first() {
    let c = catalog();
    let ring = c.ring(2575).unwrap();
    let p = SimParams { snr_db: 30.0, speed_sd_cents: 0.0, dove_error_sd_cents: 0.0, retune_prob: 0.0, ..Default::default() };
    let (audio, _) = simulate(ring, &p, 7);
    let report = locate_audio(c, &audio, &LocateOptions { top: 20, ..Default::default() });
    assert_eq!(tower_rank(&report, ring.tower_id), Some(1), "top: {:?}", report.matches.first().map(|m| &m.ring.place));
}

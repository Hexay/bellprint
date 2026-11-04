use bp_core::catalog::Catalog;
use bp_core::locate::{LocateOptions, locate_audio};
use bp_core::matcher::{Confidence, HIGH_MARGIN_NATS, MEDIUM_MARGIN_NATS};
use bp_core::sim::{SimParams, simulate};

#[test]
fn margin_thresholds_are_inclusive_lower_bounds() {
    assert_eq!(Confidence::from_margin(f64::INFINITY), Confidence::High);
    assert_eq!(Confidence::from_margin(HIGH_MARGIN_NATS), Confidence::High);
    assert_eq!(Confidence::from_margin(HIGH_MARGIN_NATS - 1e-9), Confidence::Medium);
    assert_eq!(Confidence::from_margin(MEDIUM_MARGIN_NATS), Confidence::Medium);
    assert_eq!(Confidence::from_margin(MEDIUM_MARGIN_NATS - 1e-9), Confidence::Low);
    assert_eq!(Confidence::from_margin(0.0), Confidence::Low);
}

#[test]
fn report_labels_the_top_match_by_its_margin() {
    let c = Catalog::embedded();
    let ring = c.ring(2575).unwrap();
    let sim = SimParams { snr_db: 30.0, speed_sd_cents: 0.0, ..Default::default() };
    let (audio, _) = simulate(ring, &sim, 7);
    let report = locate_audio(&c, &audio, &LocateOptions { top: 5, ..Default::default() });
    let top = report.matches.first().expect("a match");
    assert_eq!(report.confidence, Some(Confidence::from_margin(top.margin)));
    let json = serde_json::to_value(&report).unwrap();
    assert_eq!(json["confidence"], Confidence::from_margin(top.margin).label());
}

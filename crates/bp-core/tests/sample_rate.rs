use bp_core::audio::Audio;
use bp_core::audio::longterm::Frames;
use bp_core::audio::resample::resample;
use bp_core::audio::spectrum::{Peak, PeakParams};
use bp_core::catalog::Catalog;
use bp_core::locate::{LocateOptions, locate_audio};
use bp_core::sim::{SimParams, simulate};

#[test]
fn bin_width_holds_across_sample_rates() {
    let window_s = PeakParams::default().window_s;
    for sr in [8000, 22050, 44100, 48000, 96000] {
        let bin_hz = Frames::new(&sine(500.0, sr, 2.0), window_s, 2500.0).bin_hz;
        assert!((bin_hz * 32768.0 / 44100.0 - 1.0).abs() < 1e-3, "{sr} Hz: {bin_hz:.4} Hz bins");
    }
}

/// Fraction of `a`'s lines at least `min_db` prominent that `b` has within 3 cents.
fn shared(a: &[Peak], b: &[Peak], min_db: f64) -> f64 {
    let strong: Vec<&Peak> = a.iter().filter(|p| p.prominence_db >= min_db).collect();
    let hit = strong.iter().filter(|p| b.iter().any(|q| (1200.0 * (q.hz / p.hz).log2()).abs() < 3.0)).count();
    hit as f64 / strong.len() as f64
}

#[test]
fn a_clip_resampled_to_48k_gives_the_same_lines_and_match() {
    let catalog = Catalog::embedded();
    let (audio, _) = simulate(catalog.ring(2575).unwrap(), &SimParams::default(), 11);
    let o = LocateOptions { top: 5, ..Default::default() };
    let at44 = locate_audio(&catalog, &audio, &o);
    let at48 = locate_audio(&catalog, &resample(&audio, 48000), &o);
    let (fwd, back) = (shared(&at44.peaks, &at48.peaks, 6.0), shared(&at48.peaks, &at44.peaks, 6.0));
    assert!(fwd > 0.9 && back > 0.9, "lines shared 44.1->48 {fwd:.2}, 48->44.1 {back:.2}");
    let top = |r: &bp_core::locate::Report| r.matches.first().map(|m| m.ring.ring_id);
    assert_eq!(top(&at44), top(&at48));
}

fn sine(hz: f64, sr: u32, secs: f64) -> Audio {
    let n = (sr as f64 * secs) as usize;
    let samples = (0..n).map(|i| (std::f64::consts::TAU * hz * i as f64 / sr as f64).sin() as f32 * 0.5).collect();
    Audio { samples, sample_rate: sr }
}

#[test]
fn resampling_preserves_a_sine() {
    for (from, to) in [(44100, 48000), (48000, 44100)] {
        let out = resample(&sine(1234.5, from, 1.0), to);
        let want = sine(1234.5, to, 1.0);
        assert_eq!(out.sample_rate, to);
        let mid = &out.samples[2000..out.samples.len() - 2000];
        let err = mid.iter().zip(&want.samples[2000..]).map(|(a, b)| (a - b).abs()).fold(0f32, f32::max);
        assert!(err < 1e-3, "{from} -> {to}: max error {err}");
    }
}

use bp_core::audio::Audio;
use bp_core::audio::longterm::Aggregate;
use bp_core::audio::spectrum::{PeakParams, find_peaks};
use rand::{Rng, SeedableRng};

const TONES_HZ: [f64; 8] = [310.0, 420.0, 530.0, 700.0, 1000.0, 1290.0, 1610.0, 2040.0];

/// Steady tones (one at 1 kHz) with loud broadband noise over the first fifth of the clip.
fn tone_with_burst(burst: f64) -> Audio {
    let sr = 22050;
    let n = sr as usize * 20;
    let mut rng = rand::rngs::StdRng::seed_from_u64(3);
    let samples = (0..n)
        .map(|i| {
            let t = i as f64 / sr as f64;
            let tone: f64 = TONES_HZ.iter().map(|f| 0.01 * (std::f64::consts::TAU * f * t).sin()).sum();
            let noise = if i < n / 5 { rng.random_range(-burst..burst) } else { rng.random_range(-0.001..0.001) };
            (tone + noise) as f32
        })
        .collect();
    Audio { samples, sample_rate: sr }
}

fn tone_prominence(burst: f64, p: &PeakParams) -> f64 {
    let peaks = find_peaks(&tone_with_burst(burst), p);
    peaks.iter().find(|p| (p.hz - 1000.0).abs() < 3.0).map_or(0.0, |p| p.prominence_db)
}

fn without_fallback(aggregate: Aggregate) -> PeakParams {
    PeakParams { aggregate, sparse_fallback: Vec::new(), ..Default::default() }
}

#[test]
fn percentile_spectrum_ignores_an_intermittent_noise_burst() {
    let mean = tone_prominence(0.5, &without_fallback(Aggregate::Mean));
    let p75 = tone_prominence(0.5, &without_fallback(Aggregate::Percentile(75.0)));
    assert!(p75 > mean + 10.0, "mean {mean:.1} dB, p75 {p75:.1} dB");
}

#[test]
fn sparse_fallback_recovers_a_tone_the_mean_loses() {
    let p = PeakParams::default();
    let mean = tone_prominence(5.0, &PeakParams { sparse_fallback: Vec::new(), ..p.clone() });
    let default = tone_prominence(5.0, &p);
    assert!(mean == 0.0 && default > 20.0, "mean {mean:.1} dB, default {default:.1} dB");
}

#[test]
fn aggregate_parses_from_toml() {
    let p: PeakParams = toml::from_str("aggregate = { percentile = 60.0 }").unwrap();
    assert_eq!(p.aggregate, Aggregate::Percentile(60.0));
    let p: PeakParams = toml::from_str("aggregate = \"mean\"").unwrap();
    assert_eq!(p.aggregate, Aggregate::Mean);
}

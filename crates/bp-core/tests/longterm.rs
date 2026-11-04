use bp_core::audio::Audio;
use bp_core::audio::longterm::Aggregate;
use bp_core::audio::spectrum::{PeakParams, find_peaks};
use rand::{Rng, SeedableRng};

/// A steady 1 kHz tone with loud broadband noise over the first fifth of the clip.
fn tone_with_burst() -> Audio {
    let sr = 22050;
    let n = sr as usize * 20;
    let mut rng = rand::rngs::StdRng::seed_from_u64(3);
    let samples = (0..n)
        .map(|i| {
            let tone = 0.01 * (2.0 * std::f64::consts::PI * 1000.0 * i as f64 / sr as f64).sin();
            let noise = if i < n / 5 { rng.random_range(-0.5..0.5) } else { rng.random_range(-0.001..0.001) };
            (tone + noise) as f32
        })
        .collect();
    Audio { samples, sample_rate: sr }
}

fn tone_prominence(aggregate: Aggregate) -> f64 {
    let peaks = find_peaks(&tone_with_burst(), &PeakParams { aggregate, ..Default::default() });
    peaks.iter().find(|p| (p.hz - 1000.0).abs() < 3.0).map_or(0.0, |p| p.prominence_db)
}

#[test]
fn percentile_spectrum_ignores_an_intermittent_noise_burst() {
    let (mean, p75) = (tone_prominence(Aggregate::Mean), tone_prominence(Aggregate::Percentile(75.0)));
    assert!(p75 > mean + 10.0, "mean {mean:.1} dB, p75 {p75:.1} dB");
}

#[test]
fn aggregate_parses_from_toml() {
    let p: PeakParams = toml::from_str("aggregate = { percentile = 60.0 }").unwrap();
    assert_eq!(p.aggregate, Aggregate::Percentile(60.0));
    let p: PeakParams = toml::from_str("aggregate = \"mean\"").unwrap();
    assert_eq!(p.aggregate, Aggregate::Mean);
}

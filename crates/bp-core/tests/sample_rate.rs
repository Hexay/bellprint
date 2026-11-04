use bp_core::audio::Audio;
use bp_core::audio::resample::resample;

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

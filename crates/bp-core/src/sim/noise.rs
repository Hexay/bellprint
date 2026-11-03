use rand::Rng;

/// Pink-ish background (Kellet's filter) plus slow wind-like rumble, unit RMS.
pub fn background<R: Rng>(n: usize, rng: &mut R) -> Vec<f64> {
    let (mut b0, mut b1, mut b2, mut rumble) = (0.0, 0.0, 0.0, 0.0);
    let mut out: Vec<f64> = (0..n)
        .map(|_| {
            let w: f64 = rng.random_range(-1.0..1.0);
            b0 = 0.99765 * b0 + w * 0.0990460;
            b1 = 0.96300 * b1 + w * 0.2965164;
            b2 = 0.57000 * b2 + w * 1.0526913;
            rumble = 0.9995 * rumble + w * 0.02;
            b0 + b1 + b2 + w * 0.1848 + rumble
        })
        .collect();
    let rms = (out.iter().map(|x| x * x).sum::<f64>() / n.max(1) as f64).sqrt().max(1e-12);
    out.iter_mut().for_each(|x| *x /= rms);
    out
}

/// Steady tones unrelated to the bells (engines, hum, birds) that the matcher must not explain away.
pub fn distractors<R: Rng>(n: usize, sample_rate: u32, count: usize, rng: &mut R) -> Vec<f64> {
    let mut out = vec![0.0; n];
    for _ in 0..count {
        let hz: f64 = 2f64.powf(rng.random_range(6.5..12.5));
        let amp = rng.random_range(0.1..0.5);
        let w = std::f64::consts::TAU * hz / sample_rate as f64;
        let phase: f64 = rng.random_range(0.0..std::f64::consts::TAU);
        for (i, o) in out.iter_mut().enumerate() {
            *o += amp * (w * i as f64 + phase).sin();
        }
    }
    out
}

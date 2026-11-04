mod bell;
mod noise;
pub mod ringing;

use crate::audio::Audio;
use crate::catalog::Ring;
use bell::{BellVoice, Tuning};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use rand_distr::{Distribution, Normal};
pub use ringing::Pattern;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimParams {
    pub duration_s: f64,
    pub sample_rate: u32,
    pub pattern: Pattern,
    /// Bell signal RMS over background RMS.
    pub snr_db: f64,
    /// SD of the global pitch error (playback speed, resampling), cents.
    pub speed_sd_cents: f64,
    /// SD of each bell's true nominal around Dove's figure, cents.
    pub dove_error_sd_cents: f64,
    /// Chance the whole ring was retuned since Dove's measurement, by up to ±30 cents.
    pub retune_prob: f64,
    pub distractor_tones: usize,
}

impl Default for SimParams {
    fn default() -> Self {
        Self {
            duration_s: 8.0,
            sample_rate: 44100,
            pattern: Pattern::Rounds,
            snr_db: 10.0,
            speed_sd_cents: 8.0,
            dove_error_sd_cents: 3.0,
            retune_prob: 0.05,
            distractor_tones: 3,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimTruth {
    pub ring_id: u32,
    pub tower_id: u32,
    pub seed: u64,
    pub speed_cents: f64,
    pub true_nominals: Vec<f64>,
    pub params: SimParams,
}

/// Renders a recording of `ring` from its Dove nominals with every error source in `p`.
pub fn simulate(ring: &Ring, p: &SimParams, seed: u64) -> (Audio, SimTruth) {
    let mut rng = StdRng::seed_from_u64(seed);
    let speed_cents = Normal::new(0.0, p.speed_sd_cents.max(1e-9)).unwrap().sample(&mut rng);
    let retune = if rng.random_bool(p.retune_prob) { rng.random_range(-30.0..30.0) } else { 0.0 };
    let dove_err = Normal::new(0.0, p.dove_error_sd_cents.max(1e-9)).unwrap();
    let true_nominals: Vec<f64> = ring
        .nominals
        .iter()
        .map(|&h| h * 2f64.powf((speed_cents + retune + dove_err.sample(&mut rng)) / 1200.0))
        .collect();

    let sr = p.sample_rate;
    let n = (p.duration_s * sr as f64) as usize;
    let tenor = *true_nominals.last().expect("ring has a nominal");
    let strike_len = 15.0;
    let voices: Vec<Vec<f32>> = true_nominals
        .iter()
        .zip(&ring.cast_years)
        .map(|(&h, &year)| {
            let tuning = Tuning::random_for(year, &mut rng);
            BellVoice::random(h, tuning, &mut rng).render(sr, strike_len, &mut rng)
        })
        .collect();
    let mut sig = vec![0.0f64; n];
    for (b, t) in ringing::strikes(true_nominals.len(), p.pattern, tenor, p.duration_s, &mut rng) {
        let gain = rng.random_range(0.7..1.0);
        let start = (t * sr as f64).round() as i64;
        for (i, &v) in voices[b].iter().enumerate() {
            let k = start + i as i64;
            if k >= n as i64 {
                break;
            }
            if k >= 0 {
                sig[k as usize] += gain * v as f64;
            }
        }
    }
    let rms = (sig.iter().map(|x| x * x).sum::<f64>() / n.max(1) as f64).sqrt().max(1e-12);
    let noise_rms = rms / 10f64.powf(p.snr_db / 20.0);
    let bg = noise::background(n, &mut rng);
    let tones = noise::distractors(n, sr, p.distractor_tones, &mut rng);
    let mix: Vec<f64> = (0..n).map(|i| sig[i] + noise_rms * (bg[i] + tones[i])).collect();
    let peak = mix.iter().fold(0.0f64, |m, x| m.max(x.abs())).max(1e-12);
    let audio = Audio { samples: mix.iter().map(|x| (0.9 * x / peak) as f32).collect(), sample_rate: sr };
    let truth = SimTruth {
        ring_id: ring.ring_id,
        tower_id: ring.tower_id,
        seed,
        speed_cents,
        true_nominals,
        params: p.clone(),
    };
    (audio, truth)
}

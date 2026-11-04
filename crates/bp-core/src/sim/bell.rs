use rand::Rng;
use rand_distr::{Distribution, LogNormal, Normal};

/// One sinusoidal component of a struck bell.
struct Component {
    hz: f64,
    amp: f64,
    t60_s: f64,
}

/// A synthetic bell with its own randomised partial structure.
/// Deliberately independent of `matcher::model`: intervals are drawn here in cents from typical
/// measurements of old and modern English bells, never from the matcher's ratios or tolerances.
pub struct BellVoice {
    components: Vec<Component>,
}

/// Tuning style, which sets where the hum and prime fall (research/acoustics.md §2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tuning {
    TrueHarmonic,
    OldStyle,
}

impl Tuning {
    /// Taylor tuned true-harmonic from 1896, Whitechapel from the 1920s; bells cast between are a mix,
    /// as are bells with no recorded date.
    pub fn random_for<R: Rng>(cast_year: Option<u16>, rng: &mut R) -> Self {
        let p_true = match cast_year {
            Some(y) if y >= 1926 => 1.0,
            Some(y) if y < 1896 => 0.0,
            _ => 0.5,
        };
        if rng.random_bool(p_true) { Self::TrueHarmonic } else { Self::OldStyle }
    }

    /// (cents, sd) of hum and prime from the ideal -2400 and -1200: founder means pooled by n.
    fn hum_prime(self) -> [(f64, f64); 2] {
        match self {
            Self::TrueHarmonic => [(0.0, 10.0), (-2.0, 10.0)],
            Self::OldStyle => [(97.0, 68.0), (-89.0, 93.0)],
        }
    }
}

/// (cents above the nominal, sd of that interval in cents, relative amplitude); hum and prime come from
/// `Tuning`.
const SPEC: [(f64, f64, f64); 9] = [
    (-2400.0, 0.0, 0.5),   // hum
    (-1200.0, 0.0, 0.4),   // prime
    (-900.0, 15.0, 0.6),   // tierce: minor third above the prime
    (-500.0, 40.0, 0.15),  // quint
    (0.0, 0.0, 1.0),       // nominal
    (680.0, 25.0, 0.45),   // superquint: flat of a fifth on old bells, near 1.5 on modern Taylor
    (1230.0, 45.0, 0.35),  // octave nominal: 1140–1280 c commonly measured
    (1540.0, 40.0, 0.2),   // upper partials: inharmonic clutter
    (1870.0, 50.0, 0.15),
];

fn t60_for(hz: f64, size: f64) -> f64 {
    (12.0 * size * (500.0 / hz).powf(0.9)).clamp(0.4, 60.0)
}

impl BellVoice {
    pub fn random<R: Rng>(nominal_hz: f64, tuning: Tuning, rng: &mut R) -> Self {
        let amp_jitter = LogNormal::new(0.0, 0.4).unwrap();
        let size = LogNormal::new(0.0, 0.25).unwrap().sample(rng);
        let low = tuning.hum_prime();
        let mut components = Vec::new();
        for (i, &(c, sd, a)) in SPEC.iter().enumerate() {
            let (c, sd) = low.get(i).map_or((c, sd), |&(m, s)| (c + m, s));
            let cents = c + if sd > 0.0 { Normal::new(0.0, sd).unwrap().sample(rng) } else { 0.0 };
            let hz = nominal_hz * 2f64.powf(cents / 1200.0);
            let amp = a * amp_jitter.sample(rng);
            // Real partials are doublets from slight asymmetry: two close lines that beat.
            let split = rng.random_range(0.0..1.5);
            let share = rng.random_range(0.55..1.0);
            let t60_s = t60_for(hz, size);
            components.push(Component { hz: hz - split / 2.0, amp: amp * share, t60_s });
            components.push(Component { hz: hz + split / 2.0, amp: amp * (1.0 - share), t60_s });
        }
        Self { components }
    }

    /// One strike, `dur_s` long, before loudness scaling.
    pub fn render<R: Rng>(&self, sample_rate: u32, dur_s: f64, rng: &mut R) -> Vec<f32> {
        let sr = sample_rate as f64;
        let n = (dur_s * sr) as usize;
        let mut out = vec![0.0f64; n];
        for c in self.components.iter().filter(|c| c.hz < sr * 0.45) {
            let decay = (-6.9078 / (c.t60_s * sr)).exp();
            let (dc, ds) = ((std::f64::consts::TAU * c.hz / sr).cos(), (std::f64::consts::TAU * c.hz / sr).sin());
            let phase: f64 = rng.random_range(0.0..std::f64::consts::TAU);
            let (mut re, mut im) = (phase.cos() * c.amp, phase.sin() * c.amp);
            let stop = ((c.t60_s * sr) as usize).min(n);
            for o in out.iter_mut().take(stop) {
                *o += im;
                let r = (re * dc - im * ds) * decay;
                im = (re * ds + im * dc) * decay;
                re = r;
            }
        }
        // Clapper impact: a few ms of decaying noise.
        let click = (0.006 * sr) as usize;
        for (i, o) in out.iter_mut().take(click).enumerate() {
            *o += 0.3 * rng.random_range(-1.0..1.0) * (1.0 - i as f64 / click as f64);
        }
        out.into_iter().map(|x| x as f32).collect()
    }
}

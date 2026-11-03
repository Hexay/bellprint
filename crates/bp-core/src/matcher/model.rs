/// The matcher's model of a bell: partials as ratios of the nominal, with how far real bells stray (cents).
/// Ideal true-harmonic ratios; old bells' hum and prime wander, hence the wide tolerances. See docs/METHOD.md.
pub struct Partial {
    pub name: &'static str,
    pub ratio: f64,
    pub tol_cents: f64,
}

pub const PARTIALS: [Partial; 7] = [
    Partial { name: "hum", ratio: 0.25, tol_cents: 70.0 },
    Partial { name: "prime", ratio: 0.5, tol_cents: 50.0 },
    Partial { name: "tierce", ratio: 0.5946, tol_cents: 30.0 },
    Partial { name: "quint", ratio: 0.749, tol_cents: 50.0 },
    Partial { name: "nominal", ratio: 1.0, tol_cents: 0.0 },
    Partial { name: "superquint", ratio: 1.471, tol_cents: 45.0 },
    Partial { name: "octave_nominal", ratio: 2.03, tol_cents: 55.0 },
];

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct MatchParams {
    /// Spread between a Dove nominal and the measured line (Dove error, retuning, temperature).
    pub nominal_sigma_cents: f64,
    /// Search range and prior width of a global pitch offset (playback speed, resampling).
    pub max_shift_cents: f64,
    pub shift_sigma_cents: f64,
    /// Also try the shifts that common frame-rate and sample-rate mix-ups produce.
    pub resample_shifts: bool,
    /// Probability that a given bell of the ring is audible in the clip.
    pub p_bell_heard: f64,
    /// Likelihood-ratio gain of a perfectly matched nominal vs a chance line.
    pub hit_gain: f64,
    /// Prior share of observed lines that come from something other than the bells.
    pub background_share: f64,
    /// Width of the searched band in cents: the background line density is uniform over it.
    pub band_cents: f64,
    /// Prominence (dB) at which a line counts as full strength.
    pub full_strength_db: f64,
    /// Rings kept after the cheap nominal-only pass.
    pub shortlist: usize,
}

impl Default for MatchParams {
    fn default() -> Self {
        Self {
            nominal_sigma_cents: 8.0,
            max_shift_cents: 40.0,
            shift_sigma_cents: 15.0,
            resample_shifts: true,
            p_bell_heard: 0.97,
            hit_gain: 8.0,
            background_share: 0.3,
            band_cents: 8500.0,
            full_strength_db: 24.0,
            shortlist: 400,
        }
    }
}

/// Common global pitch errors in shared video (cents, prior weight): none, 24↔25 fps, 44.1↔48 kHz, both.
/// See research/pitfalls.md.
const KNOWN_SHIFTS: [(f64, f64); 7] =
    [(0.0, 0.70), (70.7, 0.05), (-70.7, 0.05), (146.7, 0.06), (-146.7, 0.06), (217.4, 0.04), (-217.4, 0.04)];

/// Shifts to try with their log prior: a Gaussian of `shift_sigma_cents` around each known shift.
/// Without `resample_shifts` only the one around zero, out to `max_shift_cents`.
pub fn shift_grid(p: &MatchParams) -> Vec<(f64, f64)> {
    let step = (p.nominal_sigma_cents / 4.0).max(0.5);
    let known: &[(f64, f64)] = if p.resample_shifts { &KNOWN_SHIFTS } else { &KNOWN_SHIFTS[..1] };
    let mut out = Vec::new();
    for &(mu, w) in known {
        let steps = (p.max_shift_cents / step).ceil() as i64;
        for i in -steps..=steps {
            let d = i as f64 * step;
            out.push((mu + d, w.ln() - 0.5 * (d / p.shift_sigma_cents).powi(2)));
        }
    }
    out
}

pub fn cents(hz: f64) -> f64 {
    1200.0 * hz.log2()
}

pub fn ratio_cents(ratio: f64) -> f64 {
    1200.0 * ratio.log2()
}

/// Shape of the offset between a Dove nominal and the line heard (research/matcher_twins.md).
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NominalShape {
    Gaussian,
    /// Student-t of scale `nominal_sigma_cents` and `nominal_t_nu` degrees of freedom.
    StudentT,
    /// Laplace of scale `nominal_sigma_cents`.
    Laplace,
}

/// The matcher's model of a bell: partials as ratios of the nominal, with how far real bells stray (cents).
/// Near true-harmonic ratios; old bells' hum, prime and quint wander, hence their wider tolerances.
/// Tolerances follow the spread measured on the real corpus; see research/matcher.md. Too-wide partials
/// let a bigger ring's narrow nominals claim the true ring's partial lines.
pub struct Partial {
    pub name: &'static str,
    pub ratio: f64,
    pub tol_cents: f64,
    /// Tuned to the ideal on true-harmonic bells (`MatchParams::true_harmonic_from`).
    pub era_tuned: bool,
}

const fn partial(name: &'static str, ratio: f64, tol_cents: f64, era_tuned: bool) -> Partial {
    Partial { name, ratio, tol_cents, era_tuned }
}

pub const PARTIALS: [Partial; 7] = [
    partial("hum", 0.25, 40.0, true),
    partial("prime", 0.5, 30.0, true),
    partial("tierce", 0.5963, 18.0, false),
    partial("quint", 0.749, 45.0, false),
    partial("nominal", 1.0, 0.0, false),
    partial("superquint", 1.4815, 25.0, false),
    partial("octave_nominal", 2.03, 35.0, false),
];

/// Tolerance of partial `q` on a bell cast in `cast_year`.
pub fn partial_tol(q: &Partial, cast_year: Option<u16>, p: &MatchParams) -> f64 {
    match cast_year {
        Some(y) if q.era_tuned && y >= p.true_harmonic_from => p.true_harmonic_tol_cents,
        _ => q.tol_cents,
    }
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct MatchParams {
    /// Spread between a Dove nominal and the measured line (Dove error, retuning, temperature).
    pub nominal_sigma_cents: f64,
    /// Shape of that spread, and its Student-t degrees of freedom. A heavy tail (ν 1.5) won on the
    /// corpus though hit residuals look Gaussian (research/matcher_twins.md).
    pub nominal_shape: NominalShape,
    pub nominal_t_nu: f64,
    /// Share and width of a wide Gaussian mixed into the nominal term (mislisted or retuned bells).
    pub nominal_outlier_weight: f64,
    pub nominal_outlier_sigma_cents: f64,
    /// Farthest line a nominal may claim, in `nominal_sigma_cents` (at least 3 outlier sigmas).
    pub nominal_reach_sigmas: f64,
    /// Search range and prior width of a global pitch offset (playback speed, resampling).
    pub max_shift_cents: f64,
    pub shift_sigma_cents: f64,
    /// Share and width of a narrow core in the shift prior around each known shift: most clips play at
    /// true speed, so true rings align within a few cents (research/matcher_twins.md).
    pub shift_core_weight: f64,
    pub shift_core_sigma_cents: f64,
    /// Also try the shifts that common frame-rate and sample-rate mix-ups produce.
    pub resample_shifts: bool,
    /// Probability that a given bell of the ring is audible in the clip: measured ≈ 0.7 on the real
    /// corpus, tuned jointly with `subset_log_prior` (research/matcher.md). Each nominal scores ln(1 - p + p * LR), LR = Gaussian pdf of its
    /// nearest-line offset / local line density.
    pub p_bell_heard: f64,
    /// `p_bell_heard` of semitone bells and extra trebles (Dove "6b", "2#", "0"), which bands often
    /// leave out; `None` treats them like any bell.
    pub p_optional_heard: Option<f64>,
    /// Also score each ring without its optional bells, charged this log prior; `None` doesn't.
    pub without_optional_log_prior: Option<f64>,
    /// Prior share of observed lines that come from something other than the bells.
    pub background_share: f64,
    /// Width of the searched band in cents: the background line density is uniform over it.
    pub band_cents: f64,
    /// Prominence (dB) at which a line counts as full strength.
    pub full_strength_db: f64,
    /// Rings kept after the cheap nominal-only pass.
    pub shortlist: usize,
    /// Log prior of a band ringing only a contiguous run of the ring (back 8 of 10, front 6 of 8)
    /// rather than every bell. Without it each silent bell costs ln(1 - p_bell_heard).
    pub subset_log_prior: f64,
    /// Most bells a subset may leave out, from either end of the ring.
    pub max_bells_dropped: usize,
    /// Fewest bells a subset may keep.
    pub min_subset_bells: usize,
    /// Cast year from which hum and prime are taken as true-harmonic, with `true_harmonic_tol_cents`
    /// in place of the all-era tolerance (research/acoustics.md §2, research/matcher.md).
    pub true_harmonic_from: u16,
    pub true_harmonic_tol_cents: f64,
}

impl Default for MatchParams {
    fn default() -> Self {
        Self {
            nominal_sigma_cents: 5.0,
            nominal_shape: NominalShape::StudentT,
            nominal_t_nu: 1.5,
            nominal_outlier_weight: 0.0,
            nominal_outlier_sigma_cents: 15.0,
            nominal_reach_sigmas: 5.0,
            max_shift_cents: 40.0,
            shift_sigma_cents: 15.0,
            shift_core_weight: 0.7,
            shift_core_sigma_cents: 6.0,
            resample_shifts: true,
            p_bell_heard: 0.55,
            p_optional_heard: None,
            without_optional_log_prior: Some(-0.5),
            background_share: 0.5,
            band_cents: 8500.0,
            full_strength_db: 24.0,
            shortlist: 400,
            subset_log_prior: -2.0,
            max_bells_dropped: 4,
            min_subset_bells: 5,
            true_harmonic_from: 1896,
            true_harmonic_tol_cents: 10.0,
        }
    }
}

/// Common global pitch errors in shared video (cents, prior weight): none, 24↔25 fps, 44.1↔48 kHz, both.
/// See research/pitfalls.md.
const KNOWN_SHIFTS: [(f64, f64); 7] =
    [(0.0, 0.70), (70.7, 0.05), (-70.7, 0.05), (146.7, 0.06), (-146.7, 0.06), (217.4, 0.04), (-217.4, 0.04)];

/// Shifts to try with their log prior: around each known shift, a Gaussian of `shift_sigma_cents`
/// mixed with a narrow core, scaled so the broad part alone peaks at the known shift's weight.
/// Without `resample_shifts` only the one around zero, out to `max_shift_cents`.
pub fn shift_grid(p: &MatchParams) -> Vec<(f64, f64)> {
    let step = (p.nominal_sigma_cents / 4.0).max(0.5);
    let known: &[(f64, f64)] = if p.resample_shifts { &KNOWN_SHIFTS } else { &KNOWN_SHIFTS[..1] };
    let (c, sc, sb) = (p.shift_core_weight, p.shift_core_sigma_cents, p.shift_sigma_cents);
    let mut out = Vec::new();
    for &(mu, w) in known {
        let steps = (p.max_shift_cents / step).ceil() as i64;
        for i in -steps..=steps {
            let d = i as f64 * step;
            let core = c * (sb / sc) * (-0.5 * (d / sc).powi(2)).exp();
            let broad = (1.0 - c) * (-0.5 * (d / sb).powi(2)).exp();
            out.push((mu + d, w.ln() + (core + broad).ln()));
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

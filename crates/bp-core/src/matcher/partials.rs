use super::model::{MatchParams, PARTIALS, cents, partial_tol, ratio_cents};
use super::score::{Lines, advance};
use crate::catalog::Ring;

struct Partial {
    cents: f64,
    sigma: f64,
    /// `sigma * sqrt(2π)`, the Gaussian's normaliser.
    norm: f64,
    bell: usize,
}

/// Sets of a ring's bells ("runs") whose partials are each scored as an equal-weight Gaussian mixture.
pub(super) struct Runs {
    /// Per bell: bit r set when run r has it.
    bell_mask: Vec<u64>,
    /// Partials per run.
    count: Vec<f64>,
    /// Distinct 3 × widest-partial-sigma of the runs, with the runs that have each: a run ignores
    /// partials farther than this from a line.
    reaches: Vec<(f64, u64)>,
    /// Per run: an earlier run of the same size, whose line term it reuses when their sums agree.
    twin: Vec<Option<usize>>,
}

/// Every predicted partial of every bell of a ring, in cents, sorted.
pub(super) struct RingPartials {
    partials: Vec<Partial>,
}

impl RingPartials {
    pub fn new(ring: &Ring, p: &MatchParams) -> Self {
        let mut partials: Vec<Partial> = (0..ring.nominals.len())
            .flat_map(|bell| {
                let (n, year) = (cents(ring.nominals[bell]), ring.cast_years[bell]);
                PARTIALS.iter().map(move |q| {
                    let sigma = partial_tol(q, year, p).hypot(p.nominal_sigma_cents);
                    Partial { cents: n + ratio_cents(q.ratio), sigma, norm: sigma * std::f64::consts::TAU.sqrt(), bell }
                })
            })
            .collect();
        // Stable, so ties stay in bell order as in a sort of any one run's partials.
        partials.sort_by(|a, b| a.cents.total_cmp(&b.cents));
        Self { partials }
    }

    /// `sets` of this ring's bells (at most 64), each scored as its own mixture.
    pub fn runs(&self, sets: &[&[usize]], n_bells: usize) -> Runs {
        assert!(sets.len() <= 64, "run masks are u64");
        let mut bell_mask = vec![0u64; n_bells];
        let mut reaches: Vec<(f64, u64)> = Vec::new();
        for (r, bells) in sets.iter().enumerate() {
            bells.iter().for_each(|&b| bell_mask[b] |= 1 << r);
            let max_sigma = self.partials.iter().filter(|q| bells.contains(&q.bell)).map(|q| q.sigma).fold(0.0, f64::max);
            let reach = 3.0 * max_sigma;
            match reaches.iter_mut().find(|x| x.0 == reach) {
                Some(x) => x.1 |= 1 << r,
                None => reaches.push((reach, 1 << r)),
            }
        }
        let twin = (0..sets.len()).map(|r| (0..r).rev().find(|&t| sets[t].len() == sets[r].len())).collect();
        Runs { bell_mask, count: sets.iter().map(|b| (b.len() * PARTIALS.len()) as f64).collect(), reaches, twin }
    }

    /// Per run, per grid shift: each line's log ratio of (background share + run share) density
    /// against pure background, weighted by strength and summed. Big runs and broad partials spread
    /// their evidence thin (Occam). Each partial's Gaussian is computed once per line and shift and
    /// shared by the runs that reach it; every run sums the same floats in the same order as if it
    /// were scored alone.
    pub fn line_terms(&self, runs: &Runs, lines: &Lines, grid: &[(f64, f64)], p: &MatchParams) -> Vec<Vec<f64>> {
        let a = p.background_share;
        let (scale, ln_a) = ((1.0 - a) * p.band_cents, a.ln());
        let n = runs.count.len();
        let reach = runs.reaches.iter().map(|r| r.0).fold(0.0, f64::max);
        let mut out = vec![vec![0.0; grid.len()]; n];
        let (mut acc, mut sums, mut terms) = (vec![0.0; n], vec![0.0; n], vec![0.0; n]);
        for (g, &(shift, _)) in grid.iter().enumerate() {
            acc.fill(-0.0);
            let mut lo = 0;
            for (&x, &w) in lines.cents.iter().zip(&lines.strength) {
                let y = x - shift;
                lo = advance(&self.partials, lo, |q| q.cents < y - reach);
                sums.fill(0.0);
                for q in &self.partials[lo..] {
                    if q.cents - y > reach {
                        break;
                    }
                    let e = (-0.5 * ((q.cents - y) / q.sigma).powi(2)).exp() / q.norm;
                    let mut m = runs.bell_mask[q.bell];
                    for &(r, rm) in &runs.reaches {
                        if q.cents < y - r || q.cents - y > r {
                            m &= !rm;
                        }
                    }
                    while m != 0 {
                        sums[m.trailing_zeros() as usize] += e;
                        m &= m - 1;
                    }
                }
                for r in 0..n {
                    let sum = sums[r];
                    terms[r] = match runs.twin[r] {
                        Some(t) if sums[t].to_bits() == sum.to_bits() => terms[t],
                        // a + scale * 0 is exactly a.
                        _ if sum == 0.0 => w * ln_a,
                        _ => w * (a + scale * (sum / runs.count[r])).ln(),
                    };
                    acc[r] += terms[r];
                }
            }
            for (o, &s) in out.iter_mut().zip(&acc) {
                o[g] = s;
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio::spectrum::Peak;
    use crate::catalog::Catalog;
    use crate::matcher::model::shift_grid;

    /// The per-run line terms as the matcher computed them before runs shared Gaussians.
    fn reference(ring: &Ring, bells: &[usize], lines: &Lines, shift: f64, p: &MatchParams) -> f64 {
        let mut partials: Vec<(f64, f64)> = bells
            .iter()
            .flat_map(|&b| {
                let (n, year) = (cents(ring.nominals[b]), ring.cast_years[b]);
                PARTIALS.iter().map(move |q| (n + ratio_cents(q.ratio), partial_tol(q, year, p).hypot(p.nominal_sigma_cents)))
            })
            .collect();
        partials.sort_by(|a, b| a.0.total_cmp(&b.0));
        let max_sigma = partials.iter().map(|x| x.1).fold(0.0, f64::max);
        let density = |lo: usize, y: f64| {
            let mut sum = 0.0;
            for &(c, s) in &partials[lo..] {
                let d = c - y;
                if d > 3.0 * max_sigma {
                    break;
                }
                sum += (-0.5 * (d / s).powi(2)).exp() / (s * (std::f64::consts::TAU).sqrt());
            }
            sum / partials.len() as f64
        };
        let a = p.background_share;
        let mut lo = 0;
        lines
            .cents
            .iter()
            .zip(&lines.strength)
            .map(|(&x, &w)| {
                let y = x - shift;
                lo = advance(&partials, lo, |l| l.0 < y - 3.0 * max_sigma);
                w * (a + (1.0 - a) * p.band_cents * density(lo, y)).ln()
            })
            .sum()
    }

    #[test]
    fn shared_gaussians_match_per_run_scoring_bitwise() {
        let p = MatchParams::default();
        let grid = shift_grid(&p);
        let catalog = Catalog::embedded();
        let rings = catalog.rings.iter().filter(|r| r.nominals.len() >= 6).step_by(97).take(40);
        for (i, ring) in rings.enumerate() {
            // Lines on some of the ring's partials plus pseudo-random clutter.
            let mut peaks: Vec<Peak> = ring.nominals.iter().step_by(2).map(|&hz| Peak { hz: hz * 1.003, prominence_db: 12.0 }).collect();
            peaks.extend((0..60).map(|k| {
                let u = ((k * 7919 + i * 104_729) % 1000) as f64 / 1000.0;
                Peak { hz: 60.0 * 2f64.powf(5.3 * u), prominence_db: 3.0 + 30.0 * u }
            }));
            let lines = Lines::from_peaks(&peaks, p.full_strength_db, (60.0, 2500.0));
            let n = ring.nominals.len();
            let sets: Vec<Vec<usize>> = vec![(0..n).collect(), (1..n).collect(), (0..n - 1).collect(), (0..n - 2).collect(), (0..n).step_by(2).collect()];
            let partials = RingPartials::new(ring, &p);
            let refs: Vec<&[usize]> = sets.iter().map(Vec::as_slice).collect();
            let got = partials.line_terms(&partials.runs(&refs, n), &lines, &grid, &p);
            for (bells, got) in sets.iter().zip(&got) {
                for (&(shift, _), &v) in grid.iter().zip(got) {
                    assert_eq!(v.to_bits(), reference(ring, bells, &lines, shift, &p).to_bits(), "ring {}", ring.ring_id);
                }
            }
        }
    }
}

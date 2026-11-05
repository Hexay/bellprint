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

/// First index of the max, as the matcher picks it.
fn argmax(v: impl Iterator<Item = f64>) -> (usize, f64) {
    v.enumerate().fold((0, f64::NEG_INFINITY), |m, (i, x)| if x > m.1 { (i, x) } else { m })
}

/// Every shift kept is bitwise the per-run value, and every run's best shift and score are kept.
#[test]
fn pruned_shared_line_terms_match_per_run_scoring() {
    let p = MatchParams::default();
    let grid = shift_grid(&p);
    let catalog = Catalog::embedded();
    let rings = catalog.rings.iter().filter(|r| r.nominals.len() >= 6).step_by(97).take(40);
    let (mut kept, mut total) = (0, 0);
    for (i, ring) in rings.enumerate() {
        // Lines on some of the ring's partials plus pseudo-random clutter.
        let mut peaks: Vec<Peak> =
            ring.nominals.iter().step_by(2).map(|&hz| Peak { hz: hz * 1.003, prominence_db: 12.0 }).collect();
        peaks.extend((0..60).map(|k| {
            let u = ((k * 7919 + i * 104_729) % 1000) as f64 / 1000.0;
            Peak { hz: 60.0 * 2f64.powf(5.3 * u), prominence_db: 3.0 + 30.0 * u }
        }));
        let lines = Lines::from_peaks(&peaks, p.full_strength_db, (60.0, 2500.0));
        let n = ring.nominals.len();
        let sets: Vec<Vec<usize>> =
            vec![(0..n).collect(), (1..n).collect(), (0..n - 1).collect(), (0..n - 2).collect(), (0..n).step_by(2).collect()];
        // Stand-in for the nominal terms: the shift prior plus a per-run ripple.
        let base: Vec<Vec<f64>> =
            (0..sets.len()).map(|r| grid.iter().enumerate().map(|(g, x)| x.1 + ((g * (r + 3)) % 11) as f64 * 0.3).collect()).collect();
        let base_refs: Vec<&[f64]> = base.iter().map(Vec::as_slice).collect();
        let refs: Vec<&[usize]> = sets.iter().map(Vec::as_slice).collect();
        let partials = RingPartials::new(ring, &p);
        let got = partials.line_terms(&partials.runs(&refs, n), &lines, &grid, &base_refs, &p);
        for ((bells, got), base) in sets.iter().zip(&got).zip(&base) {
            let want: Vec<f64> = grid.iter().map(|&(shift, _)| reference(ring, bells, &lines, shift, &p)).collect();
            for (&v, &w) in got.iter().zip(&want) {
                assert!(v == f64::NEG_INFINITY || v.to_bits() == w.to_bits(), "ring {}", ring.ring_id);
                kept += usize::from(v != f64::NEG_INFINITY);
                total += 1;
            }
            let (g_got, s_got) = argmax(base.iter().zip(got).map(|(b, l)| b + l));
            let (g_want, s_want) = argmax(base.iter().zip(&want).map(|(b, l)| b + l));
            assert_eq!((g_got, s_got.to_bits()), (g_want, s_want.to_bits()), "ring {}", ring.ring_id);
        }
    }
    assert!(kept < total, "nothing pruned");
}

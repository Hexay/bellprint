use super::model::{MatchParams, NominalShape};

/// Normalized density (per cent) of a nominal's offset to its line, out to `reach` cents.
#[derive(Debug, Clone, Copy)]
pub(super) struct NominalPdf {
    shape: NominalShape,
    sigma: f64,
    nu: f64,
    norm: f64,
    outlier_weight: f64,
    outlier_sigma: f64,
    pub reach: f64,
}

impl NominalPdf {
    pub fn new(p: &MatchParams) -> Self {
        let (s, nu) = (p.nominal_sigma_cents, p.nominal_t_nu);
        let norm = match p.nominal_shape {
            NominalShape::Gaussian => 1.0 / (s * std::f64::consts::TAU.sqrt()),
            NominalShape::StudentT => {
                (ln_gamma((nu + 1.0) / 2.0) - ln_gamma(nu / 2.0)).exp() / (s * (nu * std::f64::consts::PI).sqrt())
            }
            NominalShape::Laplace => 1.0 / (2.0 * s),
        };
        let w = p.nominal_outlier_weight;
        let wide = if w > 0.0 { p.nominal_outlier_sigma_cents } else { 0.0 };
        Self {
            shape: p.nominal_shape,
            sigma: s,
            nu,
            norm,
            outlier_weight: w,
            outlier_sigma: wide,
            reach: (p.nominal_reach_sigmas * s).max(3.0 * wide),
        }
    }

    /// A plain Gaussian of `sigma`, reaching 3 sigma.
    pub fn wide(sigma: f64) -> Self {
        Self::new(&MatchParams {
            nominal_sigma_cents: sigma,
            nominal_shape: NominalShape::Gaussian,
            nominal_outlier_weight: 0.0,
            nominal_reach_sigmas: 3.0,
            ..Default::default()
        })
    }

    /// `scale * pdf(d)`, multiplied in the order the Gaussian-only matcher used, so its floats are unchanged.
    pub fn scaled(&self, scale: f64, d: f64) -> f64 {
        let z = d / self.sigma;
        let kernel = match self.shape {
            NominalShape::Gaussian => (-0.5 * z.powi(2)).exp(),
            NominalShape::StudentT => (1.0 + z * z / self.nu).powf(-(self.nu + 1.0) / 2.0),
            NominalShape::Laplace => (-z.abs()).exp(),
        };
        if self.outlier_weight == 0.0 {
            return scale * self.norm * kernel;
        }
        let zw = d / self.outlier_sigma;
        let wide = (-0.5 * zw * zw).exp() / (self.outlier_sigma * std::f64::consts::TAU.sqrt());
        scale * ((1.0 - self.outlier_weight) * self.norm * kernel + self.outlier_weight * wide)
    }
}

/// Lanczos (g = 7, n = 9), ~1e-15 relative for x > 0.5.
fn ln_gamma(x: f64) -> f64 {
    const C: [f64; 9] = [
        0.999_999_999_999_809_9,
        676.520_368_121_885_1,
        -1_259.139_216_722_402_8,
        771.323_428_777_653_1,
        -176.615_029_162_140_6,
        12.507_343_278_686_905,
        -0.138_571_095_265_720_12,
        9.984_369_578_019_572e-6,
        1.505_632_735_149_311_6e-7,
    ];
    let x = x - 1.0;
    let t = x + 7.5;
    let a = C[1..].iter().enumerate().fold(C[0], |a, (i, &c)| a + c / (x + i as f64 + 1.0));
    0.5 * std::f64::consts::TAU.ln() + (x + 0.5) * t.ln() - t + a.ln()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn integral(p: &MatchParams) -> f64 {
        let pdf = NominalPdf::new(p);
        let h = 0.01;
        (-200_000..=200_000).map(|i| pdf.scaled(h, i as f64 * h)).sum()
    }

    #[test]
    fn shapes_integrate_to_one() {
        for shape in [NominalShape::Gaussian, NominalShape::StudentT, NominalShape::Laplace] {
            let p = MatchParams { nominal_shape: shape, nominal_t_nu: 5.0, ..Default::default() };
            assert!((integral(&p) - 1.0).abs() < 1e-3, "{shape:?}");
        }
        let p = MatchParams { nominal_outlier_weight: 0.2, ..Default::default() };
        assert!((integral(&p) - 1.0).abs() < 1e-3);
    }

    #[test]
    fn ln_gamma_matches_factorials() {
        assert!((ln_gamma(5.0) - 24f64.ln()).abs() < 1e-12);
        assert!((ln_gamma(0.5) - std::f64::consts::PI.sqrt().ln()).abs() < 1e-12);
    }
}

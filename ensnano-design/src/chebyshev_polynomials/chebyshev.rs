/// A linear combinations of Chebyshev's polynomials of first kind, defined on an closed interval
/// of ℝ
#[derive(Debug, Clone)]
pub struct ChebyshevPolynomial {
    /// The coefficients of the linear combination.
    ///
    /// `self` represents the polynomials `\sum_{i < coeffs.len()} coeffs[i] T_i` where `T_i` is
    /// the `i-th` Chebyshev's polynomial of first kind.
    ///
    /// If coeffs is empty, `self` represents the null polynomial.
    pub coeffs: Vec<f64>,
    pub(crate) definition_interval: [f64; 2],
}

/// A linear combinations of Chebyshev's polynomials of the second kind, defined on an closed interval
/// of ℝ
pub struct SecondKindChebyshevPolynomial {
    /// The coefficients of the linear combination.
    ///
    /// `self` represents the polynomials `\sum_{i < coeffs.len()} coeffs[i] U_i` where `U_i` is
    /// the `i-th` Chebyshev's polynomial of first kind.
    ///
    /// If coeffs is empty, `self` represents the null polynomial.
    pub coeffs: Vec<f64>,
    pub(crate) definition_interval: [f64; 2],
}

impl ChebyshevPolynomial {
    /// Evaluate `self` at `t`.
    #[allow(non_snake_case)]
    pub fn evaluate(&self, t: f64) -> f64 {
        if self.coeffs.is_empty() {
            0.
        } else if self.coeffs.len() == 1 {
            self.coeffs[0]
        } else {
            let a = self.definition_interval[0];
            let b = self.definition_interval[1];
            let u = (2. * t - (a + b)) / (b - a);

            let mut T_previous = 1.;
            let mut T = u;
            let mut v = self.coeffs[0] + u * self.coeffs[1];
            for coeff in self.coeffs.iter().skip(2) {
                let T_next = 2. * u * T - T_previous;
                T_previous = T;
                T = T_next;
                v += T * coeff;
            }
            v
        }
    }

    /// Return the interval on which `self` is defined.
    pub fn definition_interval(&self) -> [f64; 2] {
        self.definition_interval
    }

    pub fn from_coeffs_interval(coeffs: Vec<f64>, definition_interval: [f64; 2]) -> Self {
        Self {
            coeffs,
            definition_interval,
        }
    }
}

impl SecondKindChebyshevPolynomial {
    /// Evaluate `self` at `t`.
    #[allow(non_snake_case)]
    pub fn evaluate(&self, t: f64) -> f64 {
        if self.coeffs.is_empty() {
            0.
        } else if self.coeffs.len() == 1 {
            self.coeffs[0]
        } else {
            let a = self.definition_interval[0];
            let b = self.definition_interval[1];
            let u = (2. * t - (a + b)) / (b - a);

            let mut U_previous = 1.;
            let mut U = 2. * u;
            let mut v = self.coeffs[0] + 2. * u * self.coeffs[1];
            for coeff in self.coeffs.iter().skip(2) {
                let U_next = 2. * u * U - U_previous;
                U_previous = U;
                U = U_next;
                v += U * coeff;
            }
            v
        }
    }

    /// Return the interval on which `self` is defined.
    pub fn definition_interval(&self) -> [f64; 2] {
        self.definition_interval
    }

    pub fn from_coeffs_interval(coeffs: Vec<f64>, definition_interval: [f64; 2]) -> Self {
        Self {
            coeffs,
            definition_interval,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_tn(n: usize) -> ChebyshevPolynomial {
        let mut polynomial = ChebyshevPolynomial {
            coeffs: vec![0.; n + 1],
            definition_interval: [-1., 1.],
        };
        polynomial.coeffs[n] = 1.;
        polynomial
    }

    fn get_un(n: usize) -> SecondKindChebyshevPolynomial {
        let mut polynomial = SecondKindChebyshevPolynomial {
            coeffs: vec![0.; n + 1],
            definition_interval: [-1., 1.],
        };
        polynomial.coeffs[n] = 1.;
        polynomial
    }

    #[test]
    /// Check that the equation `T_n(cos theta) = cos(n theta)` is verified
    fn trigonometric_property() {
        for n in 0..10 {
            let polynomial = get_tn(n);
            let theta = 1.234;

            let expected = (n as f64 * theta).cos();
            let result = polynomial.evaluate(theta.cos());
            assert!((expected - result).abs() < 1e-5, "Failed for n = {n}");
        }
    }

    #[test]
    /// Check that the equation `U_{n-1}(cos theta)(sin theta) = sin(n theta)` is verified
    fn trigonometric_property_second_kind() {
        for n in 1..10 {
            let polynomial = get_un(n - 1);
            let theta = 1.234;

            let expected = (n as f64 * theta).sin();
            let result = polynomial.evaluate(theta.cos()) * theta.sin();
            assert!((expected - result).abs() < 1e-5, "Failed for n = {n}");
        }
    }
}

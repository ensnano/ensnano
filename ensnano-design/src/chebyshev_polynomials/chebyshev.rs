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

    /// Return the derivative of `self`
    pub fn derivated(self) -> SecondKindChebyshevPolynomial {
        if self.coeffs.is_empty() {
            SecondKindChebyshevPolynomial {
                coeffs: vec![],
                definition_interval: self.definition_interval,
            }
        } else {
            let derivative_coeffs: Vec<f64> = self
                .coeffs
                .into_iter()
                .enumerate()
                .skip(1)
                .map(|(n, c)| n as f64 * c)
                .collect();
            SecondKindChebyshevPolynomial {
                coeffs: derivative_coeffs,
                definition_interval: self.definition_interval,
            }
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
    /// Check that the equation `T_n(cos theta) = cos(n theta)` is verrified
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
    /// Check that the equation `U_{n-1}(cos theta)(sin theta) = sin(n theta)` is verrified
    fn trigonometric_property_second_kind() {
        for n in 1..10 {
            let polynomial = get_un(n - 1);
            let theta = 1.234;

            let expected = (n as f64 * theta).sin();
            let result = polynomial.evaluate(theta.cos()) * theta.sin();
            assert!((expected - result).abs() < 1e-5, "Failed for n = {n}");
        }
    }

    #[test]
    /// Check that the relation between Tn and its derivative is verrified
    ///
    /// The relation is
    /// 2 * T_n  = 1 / (n+1) (d/dx) T_{n + 1} - 1 / (n - 1 ) ( d/dx) T_{n-1}(x)
    fn test_derivative() {
        for n in 2..10 {
            let polynomial_plus = get_tn(n + 1).derivated();
            let polynomial_minus = get_tn(n - 1).derivated();
            let theta = 1.234;

            let expected = (n as f64 * theta).cos() * 2.;
            let result = polynomial_plus.evaluate(theta.cos()) / (n as f64 + 1.)
                - polynomial_minus.evaluate(theta.cos()) / (n as f64 - 1.);
            assert!(
                (expected - result).abs() < 1e-5,
                "Failed for n = {n}, expected {:.3}, got {:.3}",
                expected,
                result
            );
        }
    }
}

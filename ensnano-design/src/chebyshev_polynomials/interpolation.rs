//! Methods to build Chebychev's polynomials that approximate functions.

use super::chebyshev::*;
use rayon::prelude::*;

const DEGREE_MIN: usize = 1;
const DEGREE_MAX: usize = 100;

/// Interpolate a blackbox function on interval [a, b].
///
/// Return a Chebychev's Polynomial P so that for p in points_for_error_eval,
/// |P(p) - f(p)| < error_max
pub fn interpolate_fun<F>(
    f: Box<F>,
    a: f64,
    b: f64,
    error_max: f64,
    points_for_error_eval: Vec<f64>,
) -> ChebyshevPolynomial
where
    F: Fn(f64) -> f64 + Send + Sync,
{
    FunctionInterpolator::init(a, b, f).fit(error_max, points_for_error_eval)
}

/// Interpolate a function that is known only at a given set of points.
///
/// Return a Chebychev's polynomial P so that for p in points,
/// |P(p) - f(p)| < error_max.
///
/// If points is empty, return a null polynomial,
/// If points has length 1, return a constant polynomial equal to the value of the function at this
/// point
pub fn interpolate_points(point_values: Vec<(f64, f64)>, error_max: f64) -> ChebyshevPolynomial {
    if point_values.is_empty() {
        ChebyshevPolynomial {
            coeffs: vec![],
            definition_interval: [-1., 1.],
        }
    } else if point_values.len() == 1 {
        ChebyshevPolynomial {
            coeffs: vec![point_values[0].1],
            definition_interval: [-1., 1.],
        }
    } else {
        let function = LinearInterpolator::init(point_values.clone()).into_function();
        let points_for_error_eval: Vec<f64> = point_values.into_iter().map(|p| p.0).collect();
        let min = points_for_error_eval
            .iter()
            .min_by(|x, y| x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap();
        let max = points_for_error_eval
            .iter()
            .max_by(|x, y| x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap();
        interpolate_fun(function, *min, *max, error_max, points_for_error_eval)
    }
}

struct LinearInterpolator {
    point_values: Vec<(f64, f64)>,
    diffs: Vec<f64>,
}

impl LinearInterpolator {
    fn init(mut point_values: Vec<(f64, f64)>) -> Self {
        point_values.sort_by(|a, b| (a.0).partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
        let mut diffs = vec![0.; point_values.len() - 1];
        for i in 0..diffs.len() {
            diffs[i] = 1. / (point_values[i + 1].0 - point_values[i].0)
        }

        Self {
            point_values,
            diffs,
        }
    }

    fn interpolate(&self, t: f64) -> f64 {
        let mut i = 0;
        let mut j = self.point_values.len() - 1;
        while j > i + 1 {
            let k = (i + j) / 2;
            if self.point_values[k].0 < t {
                i = k;
            } else {
                j = k;
            }
        }
        let u = (t - self.point_values[i].0) * self.diffs[i];
        (1. - u) * self.point_values[i].1 + u * self.point_values[j].1
    }

    fn into_function(self) -> Box<impl Fn(f64) -> f64> {
        Box::new(move |x| self.interpolate(x))
    }
}

struct FunctionInterpolator<F: Fn(f64) -> f64 + Send + Sync> {
    f: Box<F>,
    bottom_interval: f64,
    top_interval: f64,
    polynomial: ChebyshevPolynomial,
    space: Vec<Option<Vec<f64>>>,
    matrix_standard: Vec<Option<Vec<Vec<f64>>>>,
}

impl<F: Fn(f64) -> f64 + Send + Sync> FunctionInterpolator<F> {
    fn init(a: f64, b: f64, f: Box<F>) -> Self {
        Self {
            f,
            bottom_interval: a.min(b),
            top_interval: b.max(a),
            polynomial: ChebyshevPolynomial {
                coeffs: vec![],
                definition_interval: [a, b],
            },
            space: vec![None; DEGREE_MAX + 1],
            matrix_standard: vec![None; DEGREE_MAX + 1],
        }
    }

    fn error_max(&self, points: &[f64]) -> f64 {
        points
            .par_iter()
            .map(|x| (self.polynomial.evaluate(*x) - (self.f)(*x)).abs())
            .max_by(|a, b| {
                if a < b {
                    std::cmp::Ordering::Less
                } else {
                    std::cmp::Ordering::Greater
                }
            })
            .unwrap_or(std::f64::INFINITY)
    }

    fn fit(mut self, error_max: f64, points_for_error_eval: Vec<f64>) -> ChebyshevPolynomial {
        let mut err = std::f64::INFINITY;
        let mut best_degree = 0;

        for d in DEGREE_MIN..=DEGREE_MAX {
            self.fit_degree(d);
            let err_d = self.error_max(&points_for_error_eval);
            if err_d < error_max {
                return self.polynomial;
            } else if err_d < err {
                err = err_d;
                best_degree = d;
            }
        }
        self.fit_degree(best_degree);
        self.polynomial
    }

    fn fit_degree(&mut self, degree: usize) {
        self.polynomial.coeffs = vec![0.; degree + 1];

        let space = self.space(degree).to_vec();

        let mut interpolation_points = space.to_vec();
        let mut interpolation_values = space.to_vec();

        for i in 0..interpolation_points.len() {
            interpolation_points[i] = (space[i] * (self.top_interval - self.bottom_interval)
                + (self.top_interval + self.bottom_interval))
                / 2.;
            interpolation_values[i] = (self.f)(interpolation_points[i]);
        }

        let matrix = self.matrix_standard[degree]
            .get_or_insert_with(|| Self::init_matrix(&Self::init_space(degree), degree));

        (0..=degree).for_each(|i| {
            let mut c = 0f64;
            (0..=degree).for_each(|j| {
                c += matrix[i][j] * interpolation_values[j];
            });
            self.polynomial.coeffs[i] = 2. * c / (degree as f64 + 1.);
        });
        self.polynomial.coeffs[0] /= 2.
    }

    fn space(&mut self, degree: usize) -> &[f64] {
        self.space[degree]
            .get_or_insert_with(|| Self::init_space(degree))
            .as_slice()
    }

    fn init_space(degree: usize) -> Vec<f64> {
        use std::f64::consts::FRAC_PI_2;

        (0..=degree)
            .map(|i| -((2. * i as f64 + 1.) * FRAC_PI_2 / (degree as f64 + 1.)).cos())
            .collect()
    }

    fn init_matrix(points: &[f64], degree: usize) -> Vec<Vec<f64>> {
        let mut ret = vec![vec![1.; points.len()]; degree + 1];

        (0..points.len()).for_each(|i| {
            ret[0][1] = 1.;
            ret[1][i] = points[i];
        });

        for j in 2..=degree {
            (0..points.len())
                .for_each(|i| ret[j][i] = 2. * points[i] * ret[j - 1][i] - ret[j - 2][i]);
        }
        ret
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn coefficient_similar_to_python_version() {
        let mut interpolator = FunctionInterpolator::init(1., 10., Box::new(|x: f64| x.cos()));
        interpolator.fit_degree(5);

        let expected = vec![
            -0.22719234311477124,
            -0.32587863529649186,
            -0.307952845728761,
            -0.602712800537089,
            0.4808995259781222,
            0.3171216008909075,
        ];
        for (x, y) in expected.iter().zip(interpolator.polynomial.coeffs.iter()) {
            assert!(
                (x - y).abs() < 1e-6,
                "expected {:?}, actual {:?}",
                expected,
                interpolator.polynomial.coeffs
            )
        }
    }

    fn fun_to_interpolate(x: f64) -> f64 {
        x.cos() + x.sin() * 2.
    }

    #[test]
    fn interpolate_blackbox() {
        let f = Box::new(fun_to_interpolate);
        let points_for_error_eval = (0..1_000).map(|n| 20. * n as f64 / 1_000. - 10.).collect();
        let poly = interpolate_fun(f, -10., 10., 1e-4, points_for_error_eval);

        for x in (0..10_000).map(|n| 20. * n as f64 / 10_000. - 10.) {
            let expected = fun_to_interpolate(x);
            let interpolated = poly.evaluate(x);
            assert!(
                (expected - interpolated).abs() < 1e-3,
                "x = {x}\n expected {expected} \n got {interpolated}"
            )
        }
    }

    #[test]
    fn interpolate_points_value() {
        let points_value = (0..1_000)
            .map(|n| {
                let x = 20. * n as f64 / 1_000. - 10.;
                let y = fun_to_interpolate(x);
                (x, y)
            })
            .collect();
        let poly = interpolate_points(points_value, 1e-4);

        for x in (0..10_000).map(|n| 20. * n as f64 / 10_000. - 10.) {
            let expected = fun_to_interpolate(x);
            let interpolated = poly.evaluate(x);
            assert!(
                (expected - interpolated).abs() < 1e-3,
                "x = {x}\n expected {expected} \n got {interpolated}"
            )
        }
    }

    #[test]
    fn interpolate_unsorted_points_value() {
        let mut points_value: Vec<(f64, f64)> = (0..1_000)
            .map(|n| {
                let x = 10. * (n as f64).sin();
                let y = fun_to_interpolate(x);
                (x, y)
            })
            .collect();
        points_value.push((-10., fun_to_interpolate(-10.)));
        points_value.push((10., fun_to_interpolate(10.)));
        let poly = interpolate_points(points_value, 1e-4);

        for x in (0..10_000).map(|n| 20. * n as f64 / 10_000. - 10.) {
            let expected = fun_to_interpolate(x);
            let interpolated = poly.evaluate(x);
            assert!(
                (expected - interpolated).abs() < 5e-3,
                "x = {x}\n expected {expected} \n got {interpolated}"
            )
        }
    }
}

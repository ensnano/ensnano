use ensnano_design::{
    chebyshev_polynomials::{self, ChebyshevPolynomial},
    curves::{CurveDescriptor, revolution::InterpolationDescriptor, torus::CurveDescriptor2D},
};
use ensnano_utils::surfaces::{RevolutionSurfaceSystemDescriptor, RootedRevolutionSurface};
use std::f64::consts::TAU;
use ultraviolet::{DVec3, Similarity3};

const NB_POINT_INTERPOLATION: usize = 100_000;
const INTERPOLATION_ERROR: f64 = 1e-4;
const T_MAX: f64 = 1.;

#[derive(Clone)]
pub(super) struct CloseSurfaceTopology {
    nb_segment: usize,
    nb_section_per_segment: usize,
    prev_section: Vec<usize>,
    next_section: Vec<usize>,
    other_spring_end: Vec<usize>,
    target: RootedRevolutionSurface,
    idx_range: Vec<usize>,
    target_scaffold_length: usize,
    interpolator: ChebyshevPolynomial,
}

impl CloseSurfaceTopology {
    pub(super) fn new(desc: RevolutionSurfaceSystemDescriptor) -> Self {
        // let nb_segment = 2 * desc.target.rooting_parameters.nb_helix_per_half_section; // NS: Obsolete
        let nb_segment = desc.target.rooting_parameters.nb_helices;
        let nb_section_per_segment = desc.simulation_parameters.nb_section_per_segment;
        let total_nb_section = nb_segment * nb_section_per_segment;

        let target = &desc.target;
        let next_section: Vec<usize> = (0..total_nb_section)
            .map(|n| {
                if n % nb_section_per_segment == nb_section_per_segment - 1 {
                    let segment = n / nb_section_per_segment;
                    let next_segment = (segment as isize + target.total_shift())
                        .rem_euclid(nb_segment as isize)
                        as usize;
                    next_segment * nb_section_per_segment
                } else {
                    n + 1
                }
            })
            .collect();

        let prev_section: Vec<usize> = (0..total_nb_section)
            .map(|n| {
                if n % nb_section_per_segment == 0 {
                    let segment = n / nb_section_per_segment;
                    let prev_segment = (segment as isize - target.total_shift())
                        .rem_euclid(nb_segment as isize)
                        as usize;
                    prev_segment * nb_section_per_segment + nb_section_per_segment - 1
                } else {
                    n - 1
                }
            })
            .collect();

        let other_spring_end: Vec<usize> = (0..total_nb_section)
            .map(|n| (n + nb_section_per_segment) % total_nb_section)
            .collect();

        let idx_range: Vec<usize> = (0..total_nb_section).collect();

        let interpolator = interpolator_inverse_curvilinear_abscissa(target.curve_2d());

        Self {
            nb_segment,
            nb_section_per_segment,
            prev_section,
            next_section,
            target: target.clone(),
            other_spring_end,
            idx_range,
            target_scaffold_length: desc.scaffold_len_target,
            interpolator,
        }
    }

    pub(super) fn nb_balls(&self) -> usize {
        self.nb_section_per_segment * self.nb_segment
    }

    pub(super) fn predecessor(&self, ball_id: usize) -> usize {
        self.prev_section[ball_id]
    }

    pub(super) fn number_of_sections_per_segment(&self) -> usize {
        self.nb_section_per_segment
    }

    pub(super) fn balls_with_successor(&self) -> &[usize] {
        &self.idx_range
    }
    pub(super) fn successor(&self, ball_id: usize) -> usize {
        self.next_section[ball_id]
    }

    pub(super) fn balls_with_predecessor_and_successor(&self) -> &[usize] {
        &self.idx_range
    }

    pub(super) fn balls_involved_in_spring(&self) -> &[usize] {
        &self.idx_range
    }

    pub(super) fn other_spring_end(&self, ball_id: usize) -> usize {
        self.other_spring_end[ball_id]
    }

    pub(super) fn surface_position(&self, revolution_angle: f64, theta: f64) -> DVec3 {
        self.target.position(revolution_angle, theta)
    }

    pub(super) fn revolution_angle_ball(&self, ball_id: usize) -> f64 {
        (ball_id % self.nb_section_per_segment) as f64 * TAU / (self.nb_section_per_segment as f64)
    }

    pub(super) fn theta_ball_init(&self) -> Vec<f64> {
        let total_nb_segment = self.nb_segment * self.nb_section_per_segment;
        let mut ret = Vec::with_capacity(total_nb_segment);

        for segment_idx in 0..self.nb_segment {
            let theta_init = segment_idx as f64 / self.nb_segment as f64;
            // let delta_theta = self.target.rooting_parameters.shift_per_turn as f64 // NS: obsolete
            // / (self.target.rooting_parameters.nb_helix_per_half_section as f64 * 2.);
            let delta_theta = self.target.rooting_parameters.winding as f64
                / self.target.rooting_parameters.nb_helices as f64;

            for section_idx in 0..self.nb_section_per_segment {
                let a = section_idx as f64 / self.nb_section_per_segment as f64;

                let theta_section = theta_init + a * delta_theta;
                ret.push(
                    theta_section.div_euclid(1.)
                        + self.interpolator.evaluate(theta_section.rem_euclid(1.)),
                );
            }
        }
        ret
    }

    pub(super) fn dpos_dtheta(&self, revolution_angle: f64, section_t: f64) -> DVec3 {
        self.target.dpos_dtheta(revolution_angle, section_t)
    }

    pub(super) fn d2pos_dtheta2(&self, revolution_angle: f64, section_t: f64) -> DVec3 {
        self.target.d2pos_dtheta2(revolution_angle, section_t)
    }

    pub(super) fn rescale_radius(
        &mut self,
        objective_number_of_nts: usize,
        actual_number_of_nt: usize,
    ) {
        self.target
            .rescale_radius(objective_number_of_nts, actual_number_of_nt);
    }

    pub(super) fn rescale_section(&mut self, scaling_factor: f64) {
        self.target.rescale_section(scaling_factor);
    }

    pub(super) fn axis(&self, revolution_angle: f64) -> DVec3 {
        self.target.axis(revolution_angle)
    }

    pub(super) fn to_curve_descriptor(
        &self,
        thetas: Vec<f64>,
        finished: bool,
        all_spirals_len: Option<&Vec<usize>>,
    ) -> Vec<CurveDescriptor> {
        let mut ret = Vec::new();

        let mut final_lengths: Vec<isize> = all_spirals_len
            .clone()
            .unwrap_or(&Vec::<usize>::new())
            .iter()
            .map(|x| *x as isize)
            .collect();
        if finished || final_lengths.len() > 0 {
            assert!(!all_spirals_len.is_none());
            // update the length to match the scaffold length
            let nb_spirals = final_lengths.len();
            let current_total_len = final_lengths.iter().sum::<isize>();
            let scale_len = self.target_scaffold_length as f64 / current_total_len as f64;
            final_lengths = final_lengths
                .iter()
                .map(|x| (*x as f64 * scale_len).round() as isize)
                .collect();
            let diff_len =
                self.target_scaffold_length as isize - final_lengths.iter().sum::<isize>();
            if diff_len != 0 {
                let delta = if diff_len > 0 { 1 } else { -1 };
                let mut indices_by_decreasing_length = final_lengths
                    .iter()
                    .enumerate()
                    .map(|(i, x)| (i, *x))
                    .collect::<Vec<(usize, isize)>>();
                indices_by_decreasing_length.sort_by_key(|(_, l)| -l);
                let indices_by_decreasing_length = indices_by_decreasing_length
                    .iter()
                    .map(|(i, _)| *i)
                    .collect::<Vec<usize>>();
                for i in 0..diff_len.abs() {
                    final_lengths[indices_by_decreasing_length[i as usize % nb_spirals]] += delta;
                }
            }
        }

        let nb_segment_per_helix = self.nb_segment / self.target.nb_spirals();
        for i in 0..self.target.nb_spirals() {
            let mut interpolations = Vec::new();
            let segment_indices = (0..nb_segment_per_helix).map(|n| {
                (i as isize + (n as isize * self.target.total_shift()))
                    .rem_euclid(self.nb_segment as isize)
            });
            let theta_0 = thetas[i * self.nb_section_per_segment];
            for s_idx in segment_indices {
                let start = s_idx as usize * self.nb_section_per_segment;
                let end = start + self.nb_section_per_segment - 1;
                let mut segment_thetas = thetas[start..=end].to_vec();
                let mut next_value = thetas[self.next_section[end]]
                    + self.target.section_fraction_rotation_per_revolution();

                let last_value = segment_thetas.last().unwrap();
                while next_value >= 0.5 + last_value {
                    next_value -= 1.;
                }
                while next_value <= last_value - 0.5 {
                    next_value += 1.;
                }
                segment_thetas.push(next_value);
                let s = (0..=self.nb_section_per_segment)
                    .map(|x| x as f64 / self.nb_section_per_segment as f64);
                let pv: Vec<_> = s.zip(segment_thetas.into_iter()).collect();
                let polynomials = chebyshev_polynomials::interpolate_points(pv, 1e-4);
                let interval = polynomials.definition_interval();
                interpolations.push(InterpolationDescriptor::Chebyshev {
                    coeffs: polynomials.coeffs,
                    interval,
                });
            }

            let objective_number_of_nts = if finished || final_lengths.len() > 0 {
                Some(final_lengths[i] as usize)
            } else {
                None
            };
            ret.push((
                self.target
                    .curve_descriptor(interpolations, objective_number_of_nts),
                theta_0,
            ));
        }
        ret.sort_by_key(|(_, k)| ordered_float::OrderedFloat::from(*k));

        ret.into_iter()
            .enumerate()
            .map(|(d_id, (mut desc, _))| {
                desc.known_helix_id_in_shape = Some(d_id);
                CurveDescriptor::InterpolatedCurve(desc)
            })
            .collect()
    }

    pub(super) fn fixed_points(&self) -> &[usize] {
        &[]
    }

    pub(super) fn get_frame(&self) -> Similarity3 {
        self.target.get_frame()
    }
}

fn interpolator_inverse_curvilinear_abscissa(curve: &CurveDescriptor2D) -> ChebyshevPolynomial {
    let mut abscissa = 0.;

    let mut point = curve.point(0.);

    let mut ts = Vec::with_capacity(NB_POINT_INTERPOLATION);
    let mut abscissas = Vec::with_capacity(NB_POINT_INTERPOLATION);
    ts.push(0.);
    abscissas.push(abscissa);
    for n in 1..=NB_POINT_INTERPOLATION {
        let t = T_MAX * n as f64 / NB_POINT_INTERPOLATION as f64;
        let next_point = curve.point(t);
        abscissa += (point - next_point).mag();
        abscissas.push(abscissa);
        point = next_point;
        ts.push(t);
    }

    let perimeter = *abscissas.last().unwrap();

    for x in &mut abscissas {
        *x /= perimeter;
    }

    log::info!("Interpolating inverse...");
    let abscissa_t = abscissas.iter().copied().zip(ts.iter().copied()).collect();
    chebyshev_polynomials::interpolate_points(abscissa_t, INTERPOLATION_ERROR)
}

pub mod bezier;
mod chebyshev;
mod circle_curve;
mod discretization;
mod legacy;
pub mod revolution;
pub mod sphere_concentric_circle;
pub mod sphere_like_spiral;
pub mod spiral_cylinder;
pub mod supertwist;
pub mod time_nucl_map;
pub mod torus;
pub mod torus_concentric_circle;
pub mod tube_spiral;
pub mod twist;

use crate::{
    bezier_plane::{BezierPathData, BezierPathId},
    curves::{
        bezier::{
            BezierEnd, CubicBezierConstructor, InstantiatedPiecewiseBezier,
            instantiator::PieceWiseBezierInstantiator,
        },
        revolution::InterpolatedCurveDescriptor,
        sphere_concentric_circle::{
            PillConcentricStadiumDescriptor, PillTennisBallSeamDescriptor,
            SphereConcentricCircleDescriptor, SphereTennisBallSeamDescriptor,
        },
        sphere_like_spiral::SphereLikeSpiralDescriptor,
        spiral_cylinder::SpiralCylinderDescriptor,
        supertwist::SuperTwist,
        torus::{Torus, TwistedTorusDescriptor},
        torus_concentric_circle::{
            EllipticTorusConcentricCircleDescriptor, TorusConcentricCircleDescriptor,
        },
        tube_spiral::TubeSpiralDescriptor,
        twist::Twist,
    },
    grid::{Edge, GridPosition, grid_collection::FreeGrids},
    helices::{AdditionalHelix2D, Helix},
    parameters::HelixParameters,
    utils::{is_false, vec_to_dvec},
};
use bezier::TranslatedPiecewiseBezier;
use chebyshev::{PolynomialCoordinates, PolynomialCoordinates_};
use rand::prelude::*;
use revolution::InterpolationDescriptor;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    f64::consts::{PI, TAU},
    sync::Arc,
};
use time_nucl_map::AbscissaConverter;
use torus::TwistedTorus;
use ultraviolet::{DMat3, DVec3, Isometry2, Rotor3, Vec2, Vec3};

/// To compute curvilinear abscissa over long distances
const DELTA_MAX: f64 = 256.0;

const EPSILON: f64 = 1e-6;
const EPSILON_DERIVATIVE: f64 = 1e-6;

/// Types that implements this trait represents curves.
pub trait Curved {
    /// A function that maps a `0.0 <= t <= Self::t_max` to a point in Space.
    fn position(&self, t: f64) -> DVec3;

    /// The upper bound of the definition domain of `Self::position`.
    ///
    /// By default this is 1.0, but for curves that are infinite
    /// this value may be overridden to allow the helix to have more nucleotides
    fn t_max(&self) -> f64 {
        1.0
    }

    /// The lower bound of the definition domain of `Self::position`.
    ///
    /// By default this is 0.0, but for curves that are infinite
    /// this value may be overridden to allow the helix to have more nucleotides
    fn t_min(&self) -> f64 {
        0.0
    }

    /// The derivative of `Self::position` with respect to time.
    ///
    /// If no implementation is provided, a default implementation is available using numeric
    /// derivation.
    fn speed(&self, t: f64) -> DVec3 {
        (self.position(t + EPSILON_DERIVATIVE / 2.) - self.position(t - EPSILON_DERIVATIVE / 2.))
            / EPSILON_DERIVATIVE
    }

    /// The second derivative of `Self::position` with respect to time.
    ///
    /// If no implementation is provided, a default implementation is provided using numeric
    /// derivation.
    fn acceleration(&self, t: f64) -> DVec3 {
        ((self.position(t + EPSILON_DERIVATIVE) + self.position(t - EPSILON_DERIVATIVE))
            - 2. * self.position(t))
            / (EPSILON_DERIVATIVE * EPSILON_DERIVATIVE)
    }

    /// The curvature of the curve at point `t`.
    ///
    /// This is the radius of the osculating circle of the curve at the point `t`.
    /// See `https://en.wikipedia.org/wiki/Curvature`
    fn curvature(&self, t: f64) -> f64 {
        let speed = self.speed(t);
        let numerator = speed.cross(self.acceleration(t)).mag();
        let denominator = speed.mag().powi(3);
        numerator / denominator
    }

    /// The torsion of the curve at point `t`.
    ///
    /// See `https://en.wikipedia.org/wiki/Torsion_of_a_curve`
    fn torsion(&self, t: f64) -> f64 {
        let eps: f64 = 1e-3;
        let p0 = self.position(t);
        let p1 = self.position(t + eps);
        let p2 = self.position(t + 2. * eps);
        let p3 = self.position(t + 3. * eps);
        let dp0 = (p1 - p0) / eps;
        let dp1 = (p2 - p1) / eps;
        let dp2 = (p3 - p2) / eps;
        let d2p0 = (dp1 - dp0) / eps;
        let d2p1 = (dp2 - dp1) / eps;
        let d3p = (d2p1 - d2p0) / eps;
        let c = dp0.cross(d2p0);
        d3p.dot(c) / c.mag_sq()
    }

    fn absolute_torsion(&self, t: f64) -> f64 {
        self.torsion(t).abs()
    }

    /// The bounds of the curve
    fn bounds(&self) -> CurveBounds;

    /// Curved for which there exists a closed formula for the curvilinear abscissa can override
    /// this method.
    fn curvilinear_abscissa(&self, _t: f64) -> Option<f64> {
        None
    }

    /// Curved for which there exists a closed formula for the inverse curvilinear abscissa can override
    /// this method.
    fn inverse_curvilinear_abscissa(&self, _x: f64) -> Option<f64> {
        None
    }

    /// If the rise along the curve is not the same than for straight helices, this method should
    /// be overridden
    fn rise_ratio(&self) -> Option<f64> {
        None
    }

    fn theta_shift(&self, helix_parameters: &HelixParameters) -> Option<f64> {
        if let Some(real_z_ratio) = self.rise_ratio() {
            let r = helix_parameters.helix_radius as f64;
            let z = helix_parameters.rise as f64;
            let real_z = z * real_z_ratio;
            let d1 = helix_parameters.dist_ac() as f64;
            let cos_ret = 1.0 - (d1 * d1 - real_z * real_z) / (r * r * 2.0);
            if cos_ret.abs() > 1.0 {
                None
            } else {
                Some(cos_ret.acos())
            }
        } else {
            None
        }
    }

    /// This method can be overridden to express the fact that a translation should be applied to
    /// every point of the curve. For each point of the curve, the translation is expressed in the
    /// coordinate of the frame associated to the point.
    fn translation(&self) -> Option<DVec3> {
        None
    }

    /// This method can be overridden to express the fact that a specific frame should be used to
    /// position nucleotides around the first point of the curve.
    fn initial_frame(&self) -> Option<DMat3> {
        None
    }

    /// This method can be overridden to express the fact that the curve is closed.
    /// In that case, return `Some(t)` if the curve is closed with period `t`.
    fn full_turn_at_t(&self) -> Option<f64> {
        None
    }

    /// This method can be overridden to express the fact that the curve is closed and should
    /// contain a specific number of nucleotide between `self.t_min()` and `self.full_turn_at_t()`.
    fn nucl_pos_full_turn(&self) -> Option<isize> {
        None
    }

    /// This method can be overridden to express the fact that the curve should contain a specific
    /// number of nucleotides between `self.t_min()` and `self.t_max()`.
    fn objective_nb_nt(&self) -> Option<usize> {
        None
    }

    /// This method can be overridden to express the fact that a curve needs to be represented by
    /// several helices segments in 2D.
    /// If that is the case, return the index of the corresponding segment for t. This methods must
    /// be increasing.
    fn subdivision_for_t(&self, _t: f64) -> Option<usize> {
        None
    }

    /// This method can be overridden to express the fact that a curve will be the only member of
    /// its synchronization group.
    /// In that case, the abscissa converter can be stored directly in the curve.
    fn is_time_maps_singleton(&self) -> bool {
        false
    }

    /// This method can be overridden to express the fact the a curve is a portion of a surface.
    /// In that case return the information about the surface at the point corresponding to time t
    fn surface_info_time(&self, _t: f64, _helix_id: usize) -> Option<SurfaceInfo> {
        None
    }

    /// This method can be overridden to express the fact the a curve is a portion of a surface.
    /// In that case return the information about the surface at the specified point
    fn surface_info(&self, _point: SurfacePoint) -> Option<SurfaceInfo> {
        None
    }

    /// This method can be overridden to specify the additional isometry associated to each segment
    /// of the helix.
    fn additional_isometry(&self, _segment_idx: usize) -> Option<Isometry2> {
        None
    }

    /// This method can be overridden to indicate that the curve must be discretized quickly,
    /// even at the cost of precision.
    fn discretize_quickly(&self) -> bool {
        false
    }

    /// Return true if the discretization algorithm should precompute polynomials for the
    /// curvilinear abscissa
    fn pre_compute_polynomials(&self) -> bool {
        false
    }

    fn legacy(&self) -> bool {
        false
    }

    fn abscissa_converter(&self) -> Option<AbscissaConverter> {
        None
    }
    /// Choose the iterative frame algorithm used to discretize
    fn use_original_iterative_frame_algorithm(&self) -> bool {
        false
    }
}

/// The bounds of the curve. This describe the interval in which t can be taken
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CurveBounds {
    /// t ∈ [t_min, t_max]
    Finite,
    /// t ∈ ]-∞, +∞[
    BiInfinite,
}

#[derive(Debug, Clone)]
pub struct SurfacePoint {
    pub revolution_angle: f64,
    pub abscissa_along_section: f64,
    pub helix_id: usize,
    pub section_rotation_angle: f64,
    pub reversed_direction: bool,
}

#[derive(Debug)]
pub struct SurfaceInfo {
    pub point: SurfacePoint,
    pub section_tangent: Vec2,
    /// A frame where the up vector is normal to the revolution plane, and the right vector is
    /// tangent to the revolution circle
    pub local_frame: Rotor3,
    pub position: Vec3,
}

#[derive(Clone)]
/// A discretized Curve, with precomputed curve position, and an orthogonal frame moving along the
/// curve.
pub struct Curve {
    /// The object describing the curve.
    pub geometry: Arc<dyn Curved + Sync + Send>,
    /// The precomputed points along the curve for the forward strand
    pub(crate) positions_forward: Vec<DVec3>,
    /// The precomputed points along the curve for the backward strand
    pub(crate) positions_backward: Vec<DVec3>,
    /// The precomputed orthogonal frames moving along the curve for the forward strand
    axis_forward: Vec<DMat3>,
    /// The precomputed orthogonal frames moving along the curve for the backward strand
    axis_backward: Vec<DMat3>,
    /// The precomputed values of the curve's curvature
    curvature: Vec<f64>,
    /// The precomputed values of the curve's torsion
    torsion: Vec<f64>,
    /// The index in positions that was reached when t became non-negative
    nucl_t0: usize,
    /// The time point at which nucleotides where positioned
    t_nucl: Arc<Vec<f64>>,
    nucl_pos_full_turn: Option<f64>,
    /// The first nucleotide of each additional helix segment needed to represent the curve.
    additional_segment_left: Vec<usize>,
    pub abscissa_converter: Option<AbscissaConverter>,
}

impl Curve {
    pub fn new<T: Curved + 'static + Sync + Send>(
        geometry: T,
        helix_parameters: &HelixParameters,
    ) -> Self {
        let abscissa_converter = geometry.abscissa_converter();
        let mut ret = Self {
            geometry: Arc::new(geometry),
            positions_forward: Vec::new(),
            positions_backward: Vec::new(),
            axis_forward: Vec::new(),
            axis_backward: Vec::new(),
            curvature: Vec::new(),
            torsion: Vec::new(),
            nucl_t0: 0,
            t_nucl: Arc::new(Vec::new()),
            nucl_pos_full_turn: None,
            additional_segment_left: Vec::new(),
            abscissa_converter,
        };
        let len_segment = ret.geometry.rise_ratio().unwrap_or(1.0) * helix_parameters.rise as f64;
        ret.discretize(len_segment, helix_parameters.inclination as f64);
        ret
    }

    fn compute_length<T: Curved + 'static + Sync + Send>(geometry: T) -> f64 {
        if let Some((x0, x1)) = geometry
            .curvilinear_abscissa(geometry.t_min())
            .zip(geometry.curvilinear_abscissa(geometry.t_max()))
        {
            let ret = x1 - x0;
            println!("length by curvilinear_abscissa = {ret}");
            return x1 - x0;
        }
        quadrature::integrate(
            |x| geometry.speed(x).mag(),
            geometry.t_min(),
            geometry.t_max(),
            1e-5,
        )
        .integral
    }

    fn path<T: Curved + 'static + Sync + Send>(geometry: T) -> Vec<DVec3> {
        let nb_point = 10_000;
        (0..nb_point)
            .map(|n| {
                geometry.position(geometry.t_min() + n as f64 * geometry.t_max() / nb_point as f64)
            })
            .collect()
    }

    pub fn nb_points(&self) -> usize {
        self.positions_forward
            .len()
            .min(self.positions_backward.len())
    }

    pub fn axis_pos(&self, n: isize, forward: bool) -> Option<DVec3> {
        let idx = self.idx_conversion(n)?;
        if forward {
            self.positions_forward.get(idx).copied()
        } else {
            self.positions_backward.get(idx).copied()
        }
    }

    pub fn nucl_time(&self, n: isize) -> Option<f64> {
        let idx = self.idx_conversion(n)?;
        self.t_nucl.get(idx).copied()
    }

    pub fn idx_conversion(&self, n: isize) -> Option<usize> {
        if n >= 0 {
            Some(n as usize + self.nucl_t0)
        } else {
            let nb_neg = self.nucl_t0;
            (((-n) as usize) <= nb_neg).then(|| nb_neg - ((-n) as usize))
        }
    }

    pub fn nucl_pos(
        &self,
        n: isize,
        forward: bool,
        theta: f64,
        helix_parameters: &HelixParameters,
    ) -> Option<DVec3> {
        if self.geometry.legacy() {
            return self.legacy_nucl_pos(n, forward, theta, helix_parameters);
        }

        let idx = self.idx_conversion(n)?;
        let theta = if let Some(real_theta) = self.geometry.theta_shift(helix_parameters) {
            let base_theta = TAU / helix_parameters.bases_per_turn as f64;
            (base_theta - real_theta) * n as f64 + theta
        } else if let Some(pos_full_turn) = self.nucl_pos_full_turn {
            let additional_angle = self
                .axis_forward
                .get(pos_full_turn.round() as usize + 1)
                .or_else(|| self.axis_forward.last())
                .zip(self.axis_forward.first())
                .map_or(0., |(f1, f2)| {
                    let y = f2[0].dot(f1[1]);
                    let x = f2[0].dot(f1[0]);
                    y.atan2(x)
                });
            let final_angle =
                pos_full_turn * TAU / -helix_parameters.bases_per_turn as f64 + additional_angle;
            let rem = final_angle.rem_euclid(TAU);

            let mut full_delta = -rem;
            full_delta = full_delta.rem_euclid(TAU);
            if full_delta > PI {
                full_delta -= TAU;
            }

            theta + full_delta / pos_full_turn * n as f64
        } else {
            theta
        };
        let axis = if forward {
            &self.axis_forward
        } else {
            &self.axis_backward
        };
        let positions = if forward {
            &self.positions_forward
        } else {
            &self.positions_backward
        };
        if let Some(matrix) = axis.get(idx).copied() {
            let mut ret = matrix
                * DVec3::new(
                    -theta.cos() * helix_parameters.helix_radius as f64,
                    theta.sin() * helix_parameters.helix_radius as f64,
                    0.0,
                );
            ret += positions[idx];
            Some(ret)
        } else {
            None
        }
    }

    pub fn axis_at_pos(&self, position: isize, forward: bool) -> Option<DMat3> {
        let idx = self.idx_conversion(position)?;
        let axis = if forward {
            &self.axis_forward
        } else {
            &self.axis_backward
        };
        axis.get(idx).copied()
    }

    pub fn curvature_at_pos(&self, position: isize) -> Option<f64> {
        let idx = self.idx_conversion(position)?;
        self.curvature.get(idx).copied()
    }

    pub fn torsion_at_pos(&self, position: isize) -> Option<f64> {
        let idx = self.idx_conversion(position)?;
        self.torsion.get(idx).copied()
    }

    pub fn points(&self) -> &[DVec3] {
        &self.positions_forward
    }

    pub fn range(&self) -> std::ops::RangeInclusive<isize> {
        let min = (-(self.nucl_t0 as isize)).max(-100);
        let max = (min + self.nb_points() as isize - 1).min(100);
        min..=max
    }

    pub fn nucl_t0(&self) -> usize {
        self.nucl_t0
    }

    pub fn update_additional_segments(&self, segments: &mut Vec<AdditionalHelix2D>) {
        segments.truncate(self.additional_segment_left.len());
        let mut iter = self
            .additional_segment_left
            .iter()
            .enumerate()
            .map(|(segment_idx, s)| AdditionalHelix2D {
                left: *s as isize - self.nucl_t0 as isize,
                additional_isometry: self.geometry.additional_isometry(segment_idx),
                additional_symmetry: None,
            });

        for s in segments.iter_mut() {
            if let Some(i) = iter.next() {
                s.left = i.left;
            }
        }
        segments.extend(iter);
    }

    /// If `true`, then this means that the position and orientation of the helix are encoded in
    /// the `CurveDescriptor`, and that the `position` and `orientation` fields of the helix should
    /// be ignored.
    pub fn has_its_own_encoded_frame(&self) -> bool {
        self.geometry.translation().is_some()
    }
}

pub fn perpendicular_basis(point: DVec3) -> DMat3 {
    let norm = point.mag();

    if norm < EPSILON {
        return DMat3::identity();
    }

    let axis_z = point.normalized();

    let mut axis_x = if axis_z.x.abs() >= 0.9 {
        DVec3::unit_y()
    } else {
        DVec3::unit_x()
    };

    let axis_y = axis_z.cross(axis_x).normalized();
    axis_x = axis_y.cross(axis_z).normalized();

    DMat3::new(axis_x, axis_y, axis_z)
}

#[derive(Clone, Debug, Serialize, Deserialize)]
/// A descriptor of the curve that can be serialized
pub enum CurveDescriptor {
    Bezier(CubicBezierConstructor),
    SphereLikeSpiral(SphereLikeSpiralDescriptor),
    SpiralCylinder(SpiralCylinderDescriptor),
    TubeSpiral(TubeSpiralDescriptor),
    SphereConcentricCircle(SphereConcentricCircleDescriptor),
    SphereTennisBallSeam(SphereTennisBallSeamDescriptor),
    PillConcentricStadium(PillConcentricStadiumDescriptor),
    PillTennisBallSeam(PillTennisBallSeamDescriptor),
    Twist(Twist),
    Torus(Torus),
    TorusConcentricCircle(TorusConcentricCircleDescriptor),
    EllipticTorusConcentricCircle(EllipticTorusConcentricCircleDescriptor),
    TwistedTorus(TwistedTorusDescriptor),
    PiecewiseBezier {
        #[serde(skip_serializing_if = "Option::is_none", default)]
        t_min: Option<f64>,
        #[serde(skip_serializing_if = "Option::is_none", default)]
        t_max: Option<f64>,
        points: Vec<BezierEnd>,
    },
    TranslatedPath {
        path_id: BezierPathId,
        translation: Vec3,
        #[serde(default, skip_serializing_if = "is_false")]
        legacy: bool,
    },
    SuperTwist(SuperTwist),
    InterpolatedCurve(InterpolatedCurveDescriptor),
    Chebyshev(PolynomialCoordinates),
}

const NO_BEZIER: &[BezierEnd] = &[];

impl CurveDescriptor {
    pub fn grid_positions_involved(&self) -> impl Iterator<Item = &GridPosition> {
        let points = if let Self::PiecewiseBezier { points, .. } = self {
            points.as_slice()
        } else {
            NO_BEZIER
        };
        points.iter().map(|p| &p.position)
    }
    pub fn set_t_min(&mut self, new_t_min: f64) -> bool {
        match self {
            Self::PiecewiseBezier { t_min, .. } => {
                if matches!(*t_min, Some(x) if x <= new_t_min) {
                    false
                } else {
                    *t_min = Some(new_t_min);
                    true
                }
            }
            Self::Twist(twist) => {
                if matches!(twist.t_min, Some(x) if x <= new_t_min) {
                    false
                } else {
                    twist.t_min = Some(new_t_min);
                    true
                }
            }
            _ => false,
        }
    }

    pub fn set_t_max(&mut self, new_t_max: f64) -> bool {
        match self {
            Self::PiecewiseBezier { t_max, .. } => {
                if matches!(*t_max, Some(x) if x >= new_t_max) {
                    false
                } else {
                    *t_max = Some(new_t_max);
                    true
                }
            }
            Self::Twist(twist) => {
                if matches!(twist.t_max, Some(x) if x >= new_t_max) {
                    false
                } else {
                    twist.t_max = Some(new_t_max);
                    true
                }
            }
            _ => false,
        }
    }

    pub fn t_min(&self) -> Option<f64> {
        match self {
            Self::PiecewiseBezier { t_min, .. } => *t_min,
            Self::Twist(twist) => twist.t_min,
            _ => None,
        }
    }

    pub fn t_max(&self) -> Option<f64> {
        match self {
            Self::PiecewiseBezier { t_max, .. } => *t_max,
            Self::Twist(twist) => twist.t_max,
            _ => None,
        }
    }

    pub fn translate(&self, edge: Edge, grid_reader: &dyn CurveInstantiator) -> Option<Self> {
        match self {
            Self::PiecewiseBezier {
                points,
                t_max,
                t_min,
            } => {
                log::debug!("translating {points:?}");
                let translated_points: Option<Vec<_>> = points
                    .clone()
                    .into_iter()
                    .map(|p| {
                        let ret = p.clone().translated_by(edge, grid_reader);
                        log::debug!("{p:?} -> {ret:?}");
                        ret
                    })
                    .collect();
                Some(Self::PiecewiseBezier {
                    points: translated_points?,
                    t_max: *t_max,
                    t_min: *t_min,
                })
            }
            _ => None,
        }
    }

    pub fn compute_length(&self) -> Option<f64> {
        let desc = InstantiatedCurveDescriptor::try_instantiate(Arc::new(self.clone()))?;
        desc.instance.try_length(&HelixParameters::GEARY_2014_DNA)
    }

    pub fn path(&self) -> Option<Vec<DVec3>> {
        let desc = InstantiatedCurveDescriptor::try_instantiate(Arc::new(self.clone()))?;
        desc.instance.try_path(&HelixParameters::GEARY_2014_DNA)
    }
}

#[derive(Clone, Debug)]
/// A descriptor of the the curve where all reference to design element have been resolved.
/// For example, GridPosition are replaced by their actual position in space.
pub struct InstantiatedCurveDescriptor {
    pub source: Arc<CurveDescriptor>,
    instance: InstantiatedCurveDescriptor_,
}

/// A type that is capable of converting Design object to concrete 3D position.
///
/// This is used to instantiate curves that reference design objects.
pub trait CurveInstantiator {
    fn concrete_grid_position(&self, position: GridPosition) -> Vec3;
    fn source(&self) -> FreeGrids;
    fn source_paths(&self) -> Option<BezierPathData>;
    fn translate_by_edge(&self, position: GridPosition, edge: Edge) -> Option<GridPosition>;
}

impl InstantiatedCurveDescriptor {
    /// Reads the design data to resolve the reference to elements of the design
    pub fn instantiate(desc: Arc<CurveDescriptor>, grid_reader: &dyn CurveInstantiator) -> Self {
        let instance = match desc.as_ref() {
            CurveDescriptor::Bezier(b) => InstantiatedCurveDescriptor_::Bezier(b.clone()),
            CurveDescriptor::SphereLikeSpiral(s) => {
                InstantiatedCurveDescriptor_::SphereLikeSpiral(s.clone())
            }
            CurveDescriptor::TubeSpiral(t) => InstantiatedCurveDescriptor_::TubeSpiral(t.clone()),
            CurveDescriptor::SphereConcentricCircle(t) => {
                InstantiatedCurveDescriptor_::SphereConcentricCircle(t.clone())
            }
            CurveDescriptor::SphereTennisBallSeam(t) => {
                InstantiatedCurveDescriptor_::SphereTennisBallSeam(t.clone())
            }
            CurveDescriptor::PillConcentricStadium(t) => {
                InstantiatedCurveDescriptor_::PillConcentricStadium(t.clone())
            }
            CurveDescriptor::PillTennisBallSeam(t) => {
                InstantiatedCurveDescriptor_::PillTennisBallSeam(t.clone())
            }
            CurveDescriptor::SpiralCylinder(t) => {
                InstantiatedCurveDescriptor_::SpiralCylinder(t.clone())
            }
            CurveDescriptor::Twist(t) => InstantiatedCurveDescriptor_::Twist(t.clone()),
            CurveDescriptor::Torus(t) => InstantiatedCurveDescriptor_::Torus(t.clone()),
            CurveDescriptor::TorusConcentricCircle(t) => {
                InstantiatedCurveDescriptor_::TorusConcentricCircle(t.clone())
            }
            CurveDescriptor::EllipticTorusConcentricCircle(t) => {
                InstantiatedCurveDescriptor_::EllipticTorusConcentricCircle(t.clone())
            }
            CurveDescriptor::SuperTwist(t) => InstantiatedCurveDescriptor_::SuperTwist(t.clone()),
            CurveDescriptor::TwistedTorus(t) => {
                InstantiatedCurveDescriptor_::TwistedTorus(t.clone())
            }
            CurveDescriptor::PiecewiseBezier {
                points,
                t_min,
                t_max,
            } => {
                let instantiated = InstantiatedPiecewiseBezierDescriptor::instantiate(
                    points,
                    grid_reader,
                    *t_min,
                    *t_max,
                );
                InstantiatedCurveDescriptor_::PiecewiseBezier(instantiated)
            }
            CurveDescriptor::TranslatedPath {
                path_id,
                translation,
                legacy,
            } => grid_reader
                .source_paths()
                .and_then(|paths| {
                    Self::instantiate_translated_path(*path_id, *translation, paths, *legacy)
                })
                .unwrap_or_else(|| {
                    let instantiated = InstantiatedPiecewiseBezierDescriptor::instantiate(
                        &[],
                        grid_reader,
                        None,
                        None,
                    );
                    InstantiatedCurveDescriptor_::PiecewiseBezier(instantiated)
                }),
            CurveDescriptor::InterpolatedCurve(desc) => {
                InstantiatedCurveDescriptor_::InterpolatedCurve(desc.clone())
            }
            CurveDescriptor::Chebyshev(coord) => {
                InstantiatedCurveDescriptor_::Chebyshev(coord.clone().instantiated())
            }
        };
        Self {
            source: desc,
            instance,
        }
    }

    fn instantiate_translated_path(
        path_id: BezierPathId,
        translation: Vec3,
        source_path: BezierPathData,
        legacy: bool,
    ) -> Option<InstantiatedCurveDescriptor_> {
        source_path
            .instantiated_paths
            .get(&path_id)
            .and_then(|path| path.curve_descriptor.as_ref().zip(path.initial_frame()))
            .map(
                |(desc, frame)| InstantiatedCurveDescriptor_::TranslatedBezierPath {
                    path_curve: desc.clone(),
                    initial_frame: frame,
                    translation: vec_to_dvec(translation),
                    paths_data: source_path.clone(),
                    legacy,
                },
            )
    }

    pub fn try_instantiate(desc: Arc<CurveDescriptor>) -> Option<Self> {
        let instance = match desc.as_ref() {
            CurveDescriptor::Bezier(b) => Some(InstantiatedCurveDescriptor_::Bezier(b.clone())),
            CurveDescriptor::SphereLikeSpiral(s) => {
                Some(InstantiatedCurveDescriptor_::SphereLikeSpiral(s.clone()))
            }
            CurveDescriptor::TubeSpiral(s) => {
                Some(InstantiatedCurveDescriptor_::TubeSpiral(s.clone()))
            }
            CurveDescriptor::SphereConcentricCircle(s) => Some(
                InstantiatedCurveDescriptor_::SphereConcentricCircle(s.clone()),
            ),
            CurveDescriptor::SphereTennisBallSeam(s) => Some(
                InstantiatedCurveDescriptor_::SphereTennisBallSeam(s.clone()),
            ),
            CurveDescriptor::PillConcentricStadium(s) => Some(
                InstantiatedCurveDescriptor_::PillConcentricStadium(s.clone()),
            ),
            CurveDescriptor::PillTennisBallSeam(s) => {
                Some(InstantiatedCurveDescriptor_::PillTennisBallSeam(s.clone()))
            }
            CurveDescriptor::SpiralCylinder(s) => {
                Some(InstantiatedCurveDescriptor_::SpiralCylinder(s.clone()))
            }
            CurveDescriptor::Twist(t) => Some(InstantiatedCurveDescriptor_::Twist(t.clone())),
            CurveDescriptor::Torus(t) => Some(InstantiatedCurveDescriptor_::Torus(t.clone())),
            CurveDescriptor::TorusConcentricCircle(t) => Some(
                InstantiatedCurveDescriptor_::TorusConcentricCircle(t.clone()),
            ),
            CurveDescriptor::EllipticTorusConcentricCircle(t) => Some(
                InstantiatedCurveDescriptor_::EllipticTorusConcentricCircle(t.clone()),
            ),
            CurveDescriptor::SuperTwist(t) => {
                Some(InstantiatedCurveDescriptor_::SuperTwist(t.clone()))
            }
            CurveDescriptor::TwistedTorus(t) => {
                Some(InstantiatedCurveDescriptor_::TwistedTorus(t.clone()))
            }
            CurveDescriptor::PiecewiseBezier { .. } | CurveDescriptor::TranslatedPath { .. } => {
                None
            }
            CurveDescriptor::InterpolatedCurve(desc) => Some(
                InstantiatedCurveDescriptor_::InterpolatedCurve(desc.clone()),
            ),
            CurveDescriptor::Chebyshev(coord) => Some(InstantiatedCurveDescriptor_::Chebyshev(
                coord.clone().instantiated(),
            )),
        };
        instance.map(|instance| Self {
            source: desc.clone(),
            instance,
        })
    }

    /// Return true if the instantiated curve descriptor was built using these curve descriptor and
    /// grid data
    fn is_up_to_date(
        &self,
        desc: &Arc<CurveDescriptor>,
        grids: &FreeGrids,
        paths_data: &BezierPathData,
    ) -> bool {
        if Arc::ptr_eq(&self.source, desc) {
            match &self.instance {
                InstantiatedCurveDescriptor_::PiecewiseBezier(instantiated_descriptor) => {
                    FreeGrids::ptr_eq(&instantiated_descriptor.grids, grids)
                        && instantiated_descriptor
                            .paths_data
                            .as_ref()
                            .is_some_and(|data| BezierPathData::ptr_eq(paths_data, data))
                }
                InstantiatedCurveDescriptor_::TranslatedBezierPath {
                    paths_data: source_paths,
                    ..
                } => BezierPathData::ptr_eq(paths_data, source_paths),
                _ => true,
            }
        } else {
            false
        }
    }

    pub fn make_curve(
        &self,
        helix_parameters: &HelixParameters,
        cached_curve: &mut CurveCache,
    ) -> Arc<Curve> {
        InstantiatedCurveDescriptor_::clone(&self.instance)
            .into_curve(helix_parameters, cached_curve)
    }

    pub fn get_bezier_controls(&self) -> Option<CubicBezierConstructor> {
        self.instance.get_bezier_controls()
    }

    pub fn bezier_points(&self) -> Vec<Vec3> {
        match &self.instance {
            InstantiatedCurveDescriptor_::Bezier(constructor) => {
                vec![
                    constructor.start,
                    constructor.control1,
                    constructor.control2,
                    constructor.end,
                ]
            }
            InstantiatedCurveDescriptor_::PiecewiseBezier(desc) => {
                let desc = &desc.desc;
                let mut ret: Vec<_> = desc
                    .ends
                    .iter()
                    .zip(desc.ends.iter().skip(1))
                    .flat_map(|(p1, p2)| {
                        vec![
                            p1.position,
                            p1.position + p1.vector_out,
                            p2.position - p2.vector_out,
                        ]
                        .into_iter()
                    })
                    .collect();
                if let Some(last_point) = desc.ends.iter().last() {
                    ret.push(last_point.position);
                }
                ret
            }
            _ => vec![],
        }
    }
}

#[derive(Clone, Debug)]
enum InstantiatedCurveDescriptor_ {
    Bezier(CubicBezierConstructor),
    SphereLikeSpiral(SphereLikeSpiralDescriptor),
    TubeSpiral(TubeSpiralDescriptor),
    SphereConcentricCircle(SphereConcentricCircleDescriptor),
    SphereTennisBallSeam(SphereTennisBallSeamDescriptor),
    PillConcentricStadium(PillConcentricStadiumDescriptor),
    PillTennisBallSeam(PillTennisBallSeamDescriptor),
    SpiralCylinder(SpiralCylinderDescriptor),
    Twist(Twist),
    Torus(Torus),
    TorusConcentricCircle(TorusConcentricCircleDescriptor),
    EllipticTorusConcentricCircle(EllipticTorusConcentricCircleDescriptor),
    SuperTwist(SuperTwist),
    TwistedTorus(TwistedTorusDescriptor),
    PiecewiseBezier(InstantiatedPiecewiseBezierDescriptor),
    TranslatedBezierPath {
        path_curve: Arc<InstantiatedPiecewiseBezier>,
        translation: DVec3,
        initial_frame: DMat3,
        paths_data: BezierPathData,
        legacy: bool,
    },
    InterpolatedCurve(InterpolatedCurveDescriptor),
    Chebyshev(PolynomialCoordinates_),
}

/// An instantiation of a PiecewiseBezier descriptor where reference to grid positions in the
/// design have been replaced by their actual position in space using the data in `grids`.
#[derive(Clone, Debug)]
pub struct InstantiatedPiecewiseBezierDescriptor {
    /// The instantiated descriptor
    desc: InstantiatedPiecewiseBezier,
    /// The data that was used to map grid positions to space position
    grids: FreeGrids,
    /// The data that was used to map BezierVertex to grids
    paths_data: Option<BezierPathData>,
}

struct PieceWiseBezierInstantiator_<'a, 'b> {
    points: &'a [BezierEnd],
    grid_reader: &'b dyn CurveInstantiator,
}

impl PieceWiseBezierInstantiator<Vec3> for PieceWiseBezierInstantiator_<'_, '_> {
    fn nb_vertices(&self) -> usize {
        self.points.len()
    }

    fn position(&self, i: usize) -> Option<Vec3> {
        let vertex = self.points.get(i)?;
        Some(self.grid_reader.concrete_grid_position(vertex.position))
    }

    fn vector_in(&self, _i: usize) -> Option<Vec3> {
        None
    }

    fn vector_out(&self, _i: usize) -> Option<Vec3> {
        None
    }

    fn is_cyclic(&self) -> bool {
        false
    }
}

impl InstantiatedPiecewiseBezierDescriptor {
    fn instantiate(
        points: &[BezierEnd],
        grid_reader: &dyn CurveInstantiator,
        t_min: Option<f64>,
        t_max: Option<f64>,
    ) -> Self {
        let mut rng = thread_rng();
        log::debug!("Instantiating {points:?}");
        let instantiator = PieceWiseBezierInstantiator_ {
            points,
            grid_reader,
        };
        let mut desc = instantiator
            .instantiate()
            .unwrap_or_else(|| InstantiatedPiecewiseBezier {
                ends: vec![],
                t_min: None,
                t_max: None,
                is_cyclic: false,
                id: rng.r#gen(),
                discretize_quickly: false,
            });

        desc.t_max = t_max;
        desc.t_min = t_min;
        Self {
            desc,
            grids: grid_reader.source(),
            paths_data: grid_reader.source_paths(),
        }
    }
}

impl InstantiatedCurveDescriptor_ {
    pub(crate) fn into_curve(
        self,
        helix_parameters: &HelixParameters,
        cache: &mut CurveCache,
    ) -> Arc<Curve> {
        match self {
            Self::Bezier(constructor) => {
                Arc::new(Curve::new(constructor.into_bezier(), helix_parameters))
            }
            Self::SphereLikeSpiral(spiral) => Arc::new(Curve::new(
                spiral.with_helix_parameters(*helix_parameters),
                helix_parameters,
            )),
            Self::TubeSpiral(spiral) => Arc::new(Curve::new(
                spiral.with_helix_parameters(*helix_parameters),
                helix_parameters,
            )),
            Self::SpiralCylinder(spiral) => Arc::new(Curve::new(
                spiral.with_helix_parameters(*helix_parameters),
                helix_parameters,
            )),
            Self::SphereConcentricCircle(constructor) => Arc::new(Curve::new(
                constructor.with_helix_parameters(*helix_parameters),
                helix_parameters,
            )),
            Self::SphereTennisBallSeam(constructor) => Arc::new(Curve::new(
                constructor.with_helix_parameters(*helix_parameters),
                helix_parameters,
            )),
            Self::PillConcentricStadium(constructor) => Arc::new(Curve::new(
                constructor.with_helix_parameters(*helix_parameters),
                helix_parameters,
            )),
            Self::PillTennisBallSeam(constructor) => Arc::new(Curve::new(
                constructor.with_helix_parameters(*helix_parameters),
                helix_parameters,
            )),
            Self::Twist(twist) => Arc::new(Curve::new(twist, helix_parameters)),
            Self::Torus(torus) => Arc::new(Curve::new(torus, helix_parameters)),
            Self::TorusConcentricCircle(torus) => Arc::new(Curve::new(
                torus.with_helix_parameters(helix_parameters),
                helix_parameters,
            )),
            Self::EllipticTorusConcentricCircle(torus) => Arc::new(Curve::new(
                torus.with_helix_parameters(helix_parameters),
                helix_parameters,
            )),
            Self::SuperTwist(twist) => Arc::new(Curve::new(twist, helix_parameters)),
            Self::TwistedTorus(desc) => {
                if let Some(curve) = cache.0.get(&desc) {
                    curve.clone()
                } else {
                    let ret = Arc::new(Curve::new(
                        TwistedTorus::new(desc.clone(), helix_parameters),
                        helix_parameters,
                    ));
                    println!("Number of nucleotides {}", ret.nb_points());
                    cache.0.insert(desc, ret.clone());
                    ret
                }
            }
            Self::PiecewiseBezier(instantiated_descriptor) => {
                Arc::new(Curve::new(instantiated_descriptor.desc, helix_parameters))
            }
            Self::TranslatedBezierPath {
                path_curve,
                translation,
                initial_frame,
                legacy,
                ..
            } => Arc::new(Curve::new(
                TranslatedPiecewiseBezier {
                    original_curve: path_curve,
                    translation,
                    initial_frame,
                    legacy,
                },
                helix_parameters,
            )),
            Self::InterpolatedCurve(desc) => {
                Arc::new(Curve::new(desc.instantiate(true), helix_parameters))
            }
            Self::Chebyshev(coordinates) => Arc::new(Curve::new(coordinates, helix_parameters)),
        }
    }

    pub(crate) fn try_into_curve(&self, helix_parameters: &HelixParameters) -> Option<Arc<Curve>> {
        match self {
            Self::Bezier(constructor) => Some(Arc::new(Curve::new(
                constructor.clone().into_bezier(),
                helix_parameters,
            ))),
            Self::SphereLikeSpiral(spiral) => Some(Arc::new(Curve::new(
                spiral.clone().with_helix_parameters(*helix_parameters),
                helix_parameters,
            ))),
            Self::TubeSpiral(spiral) => Some(Arc::new(Curve::new(
                spiral.clone().with_helix_parameters(*helix_parameters),
                helix_parameters,
            ))),
            Self::SpiralCylinder(spiral) => Some(Arc::new(Curve::new(
                spiral.clone().with_helix_parameters(*helix_parameters),
                helix_parameters,
            ))),
            Self::SphereConcentricCircle(constructor) => Some(Arc::new(Curve::new(
                constructor.clone().with_helix_parameters(*helix_parameters),
                helix_parameters,
            ))),
            Self::SphereTennisBallSeam(constructor) => Some(Arc::new(Curve::new(
                constructor.clone().with_helix_parameters(*helix_parameters),
                helix_parameters,
            ))),
            Self::PillConcentricStadium(constructor) => Some(Arc::new(Curve::new(
                constructor.clone().with_helix_parameters(*helix_parameters),
                helix_parameters,
            ))),
            Self::PillTennisBallSeam(constructor) => Some(Arc::new(Curve::new(
                constructor.clone().with_helix_parameters(*helix_parameters),
                helix_parameters,
            ))),
            Self::Twist(twist) => Some(Arc::new(Curve::new(twist.clone(), helix_parameters))),
            Self::Torus(torus) => Some(Arc::new(Curve::new(torus.clone(), helix_parameters))),
            Self::TorusConcentricCircle(torus) => Some(Arc::new(Curve::new(
                torus.clone().with_helix_parameters(helix_parameters),
                helix_parameters,
            ))),
            Self::EllipticTorusConcentricCircle(torus) => Some(Arc::new(Curve::new(
                torus.clone().with_helix_parameters(helix_parameters),
                helix_parameters,
            ))),
            Self::SuperTwist(twist) => Some(Arc::new(Curve::new(twist.clone(), helix_parameters))),
            Self::TwistedTorus(_) | Self::PiecewiseBezier(_) => None,
            Self::TranslatedBezierPath {
                path_curve,
                translation,
                initial_frame,
                legacy,
                ..
            } => Some(Arc::new(Curve::new(
                TranslatedPiecewiseBezier {
                    original_curve: path_curve.clone(),
                    translation: *translation,
                    initial_frame: *initial_frame,
                    legacy: *legacy,
                },
                helix_parameters,
            ))),
            Self::InterpolatedCurve(desc) => Some(Arc::new(Curve::new(
                desc.clone().instantiate(true),
                helix_parameters,
            ))),
            Self::Chebyshev(coordinates) => {
                Some(Arc::new(Curve::new(coordinates.clone(), helix_parameters)))
            }
        }
    }

    fn try_length(&self, helix_parameters: &HelixParameters) -> Option<f64> {
        match self {
            Self::Bezier(constructor) => {
                Some(Curve::compute_length(constructor.clone().into_bezier()))
            }
            Self::SphereLikeSpiral(spiral) => Some(Curve::compute_length(
                spiral.clone().with_helix_parameters(*helix_parameters),
            )),
            Self::TubeSpiral(spiral) => Some(Curve::compute_length(
                spiral.clone().with_helix_parameters(*helix_parameters),
            )),
            Self::SpiralCylinder(spiral) => Some(Curve::compute_length(
                spiral.clone().with_helix_parameters(*helix_parameters),
            )),
            Self::SphereConcentricCircle(constructor) => Some(Curve::compute_length(
                constructor.clone().with_helix_parameters(*helix_parameters),
            )),
            Self::SphereTennisBallSeam(constructor) => Some(Curve::compute_length(
                constructor.clone().with_helix_parameters(*helix_parameters),
            )),
            Self::PillConcentricStadium(constructor) => Some(Curve::compute_length(
                constructor.clone().with_helix_parameters(*helix_parameters),
            )),
            Self::PillTennisBallSeam(constructor) => Some(Curve::compute_length(
                constructor.clone().with_helix_parameters(*helix_parameters),
            )),
            Self::Twist(twist) => Some(Curve::compute_length(twist.clone())),
            Self::Torus(torus) => Some(Curve::compute_length(torus.clone())),
            Self::TorusConcentricCircle(torus) => Some(Curve::compute_length(
                torus.clone().with_helix_parameters(helix_parameters),
            )),
            Self::EllipticTorusConcentricCircle(torus) => Some(Curve::compute_length(
                torus.clone().with_helix_parameters(helix_parameters),
            )),
            Self::SuperTwist(twist) => Some(Curve::compute_length(twist.clone())),
            Self::TwistedTorus(_) | Self::PiecewiseBezier(_) => None,
            Self::TranslatedBezierPath {
                path_curve,
                translation,
                initial_frame,
                legacy,
                ..
            } => Some(Curve::compute_length(TranslatedPiecewiseBezier {
                original_curve: path_curve.clone(),
                translation: *translation,
                initial_frame: *initial_frame,
                legacy: *legacy,
            })),
            Self::InterpolatedCurve(desc) => {
                Some(Curve::compute_length(desc.clone().instantiate(true)))
            }
            Self::Chebyshev(coord) => Some(Curve::compute_length(coord.clone())),
        }
    }

    fn try_path(&self, helix_parameters: &HelixParameters) -> Option<Vec<DVec3>> {
        match self {
            Self::Bezier(constructor) => Some(Curve::path(constructor.clone().into_bezier())),
            Self::SphereLikeSpiral(spiral) => Some(Curve::path(
                spiral.clone().with_helix_parameters(*helix_parameters),
            )),
            Self::TubeSpiral(spiral) => Some(Curve::path(
                spiral.clone().with_helix_parameters(*helix_parameters),
            )),
            Self::SpiralCylinder(spiral) => Some(Curve::path(
                spiral.clone().with_helix_parameters(*helix_parameters),
            )),
            Self::SphereConcentricCircle(constructor) => Some(Curve::path(
                constructor.clone().with_helix_parameters(*helix_parameters),
            )),
            Self::SphereTennisBallSeam(constructor) => Some(Curve::path(
                constructor.clone().with_helix_parameters(*helix_parameters),
            )),
            Self::PillConcentricStadium(constructor) => Some(Curve::path(
                constructor.clone().with_helix_parameters(*helix_parameters),
            )),
            Self::PillTennisBallSeam(constructor) => Some(Curve::path(
                constructor.clone().with_helix_parameters(*helix_parameters),
            )),
            Self::Twist(twist) => Some(Curve::path(twist.clone())),
            Self::Torus(torus) => Some(Curve::path(torus.clone())),
            Self::TorusConcentricCircle(torus) => Some(Curve::path(
                torus.clone().with_helix_parameters(helix_parameters),
            )),
            Self::EllipticTorusConcentricCircle(torus) => Some(Curve::path(
                torus.clone().with_helix_parameters(helix_parameters),
            )),
            Self::SuperTwist(twist) => Some(Curve::path(twist.clone())),
            Self::TwistedTorus(_) | Self::PiecewiseBezier(_) => None,
            Self::TranslatedBezierPath {
                path_curve,
                translation,
                initial_frame,
                legacy,
                ..
            } => Some(Curve::path(TranslatedPiecewiseBezier {
                original_curve: path_curve.clone(),
                translation: *translation,
                initial_frame: *initial_frame,
                legacy: *legacy,
            })),
            Self::InterpolatedCurve(desc) => Some(Curve::path(desc.clone().instantiate(false))),
            Self::Chebyshev(coordinates) => Some(Curve::path(coordinates.clone())),
        }
    }

    pub(crate) fn get_bezier_controls(&self) -> Option<CubicBezierConstructor> {
        if let Self::Bezier(b) = self {
            Some(b.clone())
        } else {
            None
        }
    }
}

#[derive(Default, Clone)]
/// A map from curve descriptor to instantiated curves to avoid duplication of computations
pub struct CurveCache(pub(crate) HashMap<TwistedTorusDescriptor, Arc<Curve>>);

#[derive(Clone)]
/// An instantiated curve with pre-computed nucleotides positions and orientations
pub(super) struct InstantiatedCurve {
    /// A descriptor of the instantiated curve
    pub source: Arc<InstantiatedCurveDescriptor>,
    pub curve: Arc<Curve>,
}

impl std::fmt::Debug for InstantiatedCurve {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("InstantiatedCurve")
            .field("source", &Arc::as_ptr(&self.source))
            .finish()
    }
}

impl AsRef<Curve> for InstantiatedCurve {
    fn as_ref(&self) -> &Curve {
        self.curve.as_ref()
    }
}

impl Helix {
    pub(super) fn need_curve_descriptor_update(
        &self,
        grid_data: &FreeGrids,
        paths_data: &BezierPathData,
    ) -> bool {
        if let Some(current_desc) = self.curve.as_ref() {
            self.instantiated_descriptor
                .as_ref()
                .filter(|desc| desc.is_up_to_date(current_desc, grid_data, paths_data))
                .is_none()
        } else {
            // If helix should not be a curved, the descriptor is up-to-date iff there is no
            // descriptor
            self.instantiated_descriptor.is_some()
        }
    }

    pub(super) fn need_curve_update(
        &self,
        grid_data: &FreeGrids,
        paths_data: &BezierPathData,
    ) -> bool {
        self.need_curve_descriptor_update(grid_data, paths_data) || {
            self.need_curve_update_only()
        }
    }

    fn need_curve_update_only(&self) -> bool {
        let up_to_date = self
            .instantiated_curve
            .as_ref()
            .map(|c| Arc::as_ptr(&c.source))
            == self.instantiated_descriptor.as_ref().map(Arc::as_ptr);
        !up_to_date
    }

    pub fn try_update_curve(&mut self, helix_parameters: &HelixParameters) {
        if let Some(curve) = self.curve.as_ref()
            && let Some(desc) = InstantiatedCurveDescriptor::try_instantiate(curve.clone())
        {
            let desc = Arc::new(desc);
            self.instantiated_descriptor = Some(desc.clone());
            let hp = &(self.helix_parameters.unwrap_or(*helix_parameters));
            println!("helix: {} nm {} bpt", hp.rise, hp.bases_per_turn);
            if let Some(curve) = desc.as_ref().instance.try_into_curve(hp) {
                self.instantiated_curve = Some(InstantiatedCurve {
                    curve,
                    source: desc,
                });
            }
        }
    }
}

impl InterpolationDescriptor {
    pub fn instantiated(self) -> chebyshev_polynomials::ChebyshevPolynomial {
        match self {
            Self::PointsValues { points, values } => {
                let points_values = points.into_iter().zip(values).collect();
                chebyshev_polynomials::interpolate_points(points_values, 1e-4)
            }
            Self::Chebyshev { coeffs, interval } => {
                chebyshev_polynomials::ChebyshevPolynomial::from_coeffs_interval(coeffs, interval)
            }
        }
    }
}

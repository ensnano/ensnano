use crate::{
    Nucl,
    bezier_plane::BezierPathId,
    curves::{
        CurveDescriptor, InstantiatedCurve, InstantiatedCurveDescriptor, SurfaceInfo, SurfacePoint,
        bezier::{BezierControlPoint, BezierEnd, CubicBezierConstructor},
        sphere_like_spiral::SphereLikeSpiralDescriptor,
        tube_spiral::TubeSpiralDescriptor,
    },
    design_operations::DesignOperationError,
    grid::{Edge, Grid, GridAwareTranslation, GridData, GridId, HelixGridPosition},
    nucl::VirtualNucl,
    parameters::HelixParameters,
    utils::{
        serde::{f32_is_zero, is_false, is_true, isize_is_zero},
        ultraviolet::{dvec_to_vec, rotor_to_drotor, vec_to_dvec},
    },
};
use ahash::HashMap;
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    f32::consts::{FRAC_PI_2, PI, TAU},
    sync::Arc,
};
use ultraviolet::{DRotor3, DVec3, Isometry2, Mat4, Rotor3, Vec2, Vec3};

/// A structure mapping helices identifier to `Helix` objects.
#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct Helices(pub(super) Arc<BTreeMap<usize, Arc<Helix>>>);

impl Helices {
    pub fn make_mut(&mut self) -> HelicesMut<'_> {
        let new_map = BTreeMap::clone(self.0.as_ref());
        HelicesMut {
            source: self,
            new_map,
        }
    }

    pub fn get(&self, id: &usize) -> Option<&Helix> {
        self.0.get(id).map(AsRef::as_ref)
    }

    pub fn keys(&self) -> impl Iterator<Item = &usize> {
        self.0.keys()
    }

    pub fn values(&self) -> impl Iterator<Item = &'_ Helix> {
        self.0.values().map(AsRef::as_ref)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&'_ usize, &'_ Helix)> {
        self.0.iter().map(|(id, arc)| (id, arc.as_ref()))
    }

    pub fn contains_key(&self, id: &usize) -> bool {
        self.0.contains_key(id)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }
}

pub struct HelicesMut<'a> {
    source: &'a mut Helices,
    new_map: BTreeMap<usize, Arc<Helix>>,
}

impl HelicesMut<'_> {
    pub fn get_mut(&mut self, id: &usize) -> Option<&mut Helix> {
        self.new_map.get_mut(id).map(|arc| {
            // For the same reasons as above, ensure that a new helix is created so that the
            // modified helix is stored at a different address.
            // Calling Arc::make_mut directly does not work because we want a new pointer even if
            // the arc count is 1
            let new_helix = Helix::clone(arc.as_ref());
            *arc = Arc::new(new_helix);

            Arc::make_mut(arc)
        })
    }

    pub fn values_mut(&mut self) -> impl Iterator<Item = &mut Helix> {
        self.new_map.values_mut().map(|arc| {
            let new_helix = Helix::clone(arc.as_ref());
            *arc = Arc::new(new_helix);
            Arc::make_mut(arc)
        })
    }

    pub fn insert(&mut self, id: usize, helix: Helix) {
        self.new_map.insert(id, Arc::new(helix));
    }

    pub fn remove(&mut self, id: &usize) {
        self.new_map.remove(id);
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&usize, &mut Helix)> {
        self.new_map.iter_mut().map(|(id, arc)| {
            let new_helix = Helix::clone(arc.as_ref());
            *arc = Arc::new(new_helix);
            (id, Arc::make_mut(arc))
        })
    }

    /// Add an helix to the collection and return the identifier of the added helix in the
    /// collection.
    pub fn push_helix(&mut self, helix: Helix) -> usize {
        let helix_id = self.new_map.keys().last().unwrap_or(&0) + 1;
        self.insert(helix_id, helix);
        helix_id
    }

    pub fn get(&self, id: &usize) -> Option<&Helix> {
        self.new_map.get(id).map(AsRef::as_ref)
    }
}

impl AsRef<Helices> for HelicesMut<'_> {
    fn as_ref(&self) -> &Helices {
        self.source
    }
}

impl Drop for HelicesMut<'_> {
    fn drop(&mut self) {
        *self.source = Helices(Arc::new(std::mem::take(&mut self.new_map)));
    }
}

fn default_visibility() -> bool {
    true
}

/// A DNA helix. All bases of all strands must be on a helix.
///
/// The three angles are illustrated in the following image, from [the NASA website](https://www.grc.nasa.gov/www/k-12/airplane/rotations.html):
/// Angles are applied in the order yaw -> pitch -> roll
/// ![Aircraft angles](https://www.grc.nasa.gov/www/k-12/airplane/Images/rotations.gif)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Helix {
    /// Position of the origin of the helix axis.
    pub position: Vec3,

    /// Orientation of the helix.
    pub orientation: Rotor3,

    /// Helix Parameters of the helix.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub helix_parameters: Option<HelixParameters>,

    /// Indicate whether the helix should be displayed in the 3D view.
    #[serde(default = "default_visibility", skip_serializing_if = "is_true")]
    pub visible: bool,

    #[serde(default, skip_serializing_if = "is_false")]
    /// Indicate that the helix cannot move during rigid body simulations.
    pub locked_for_simulations: bool,

    /// The position of the helix on a grid. If this is None, it means that helix is not bound to
    /// any grid.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub grid_position: Option<HelixGridPosition>,

    /// Representation of the helix in 2d.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub isometry2d: Option<Isometry2>,

    /// Additional segments for representing the helix in 2d.
    #[serde(
        skip_serializing_if = "Vec::is_empty",
        default,
        alias = "additonal_isometries" // cspell: disable-line
    )]
    pub additional_isometries: Vec<AdditionalHelix2D>,

    #[serde(default = "Vec2::one")]
    /// Symmetry applied inside the representation of the helix in 2d.
    pub symmetry: Vec2,

    /// Roll of the helix. A roll equal to 0 means that the nucleotide 0 of the forward strand is
    /// at point (0., 1., 0.) in the helix's coordinate.
    #[serde(default)]
    pub roll: f32,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub curve: Option<Arc<CurveDescriptor>>,

    #[serde(
        default,
        skip,
        alias = "instanciated_descriptor", // cspell: disable-line
    )]
    pub(crate) instantiated_descriptor: Option<Arc<InstantiatedCurveDescriptor>>,

    #[serde(
        default,
        skip,
        alias = "instanciated_curve", // cspell: disable-line
    )]
    pub(crate) instantiated_curve: Option<InstantiatedCurve>,

    // TODO: remove? Seems to always be 0.0.
    #[serde(default, skip_serializing_if = "f32_is_zero")]
    pub(crate) delta_bases_per_turn: f32,

    #[serde(default, skip_serializing_if = "isize_is_zero")]
    pub initial_nt_index: isize,

    /// An optional helix whose roll is copied from and to which self transfer forces applying
    /// to its roll.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub support_helix: Option<usize>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) path_id: Option<BezierPathId>,
}

impl Helix {
    pub fn translated_by(&self, edge: Edge, grid_data: &GridData) -> Option<Self> {
        log::debug!("attempt to translate helix");
        let grid_position = self
            .grid_position
            .as_ref()
            .and_then(|gp| grid_data.translate_by_helix_and_edge(gp, &edge));
        let new_curve_descriptor = self
            .curve
            .as_ref()
            .and_then(|c| c.translate(edge, grid_data));

        if self.curve.is_some() != new_curve_descriptor.is_some() {
            None
        } else {
            Some(Self {
                instantiated_curve: None,
                instantiated_descriptor: None,
                grid_position,
                isometry2d: None,
                curve: new_curve_descriptor.map(Arc::new),
                ..self.clone()
            })
        }
    }

    pub fn new(origin: Vec3, orientation: Rotor3) -> Self {
        Self {
            position: origin,
            orientation,
            helix_parameters: None,
            isometry2d: None,
            additional_isometries: Vec::new(),
            symmetry: Vec2::one(),
            grid_position: None,
            visible: true,
            roll: 0f32,
            locked_for_simulations: false,
            curve: None,
            instantiated_curve: None,
            instantiated_descriptor: None,
            delta_bases_per_turn: 0.,
            initial_nt_index: 0,
            support_helix: None,
            path_id: None,
        }
    }

    pub fn new_on_grid(grid: &Grid, x: isize, y: isize, g_id: GridId) -> Self {
        let position = grid.position_helix(x, y);
        Self {
            position,
            helix_parameters: Some(grid.helix_parameters),
            orientation: grid.orientation,
            isometry2d: None,
            additional_isometries: Vec::new(),
            symmetry: Vec2::one(),
            grid_position: Some(HelixGridPosition {
                grid: g_id,
                x,
                y,
                axis_pos: 0,
                roll: 0f32,
            }),
            visible: true,
            roll: 0f32,
            locked_for_simulations: false,
            curve: None,
            instantiated_curve: None,
            instantiated_descriptor: None,
            delta_bases_per_turn: 0.,
            initial_nt_index: 0,
            support_helix: None,
            path_id: None,
        }
    }

    pub fn new_sphere_like_spiral(desc: SphereLikeSpiralDescriptor) -> Self {
        Self {
            position: Vec3::zero(),
            orientation: Rotor3::identity(),
            helix_parameters: None,
            isometry2d: None,
            additional_isometries: Vec::new(),
            symmetry: Vec2::one(),
            grid_position: None,
            visible: true,
            roll: 0f32,
            locked_for_simulations: false,
            curve: Some(Arc::new(CurveDescriptor::SphereLikeSpiral(desc))),
            instantiated_curve: None,
            instantiated_descriptor: None,
            delta_bases_per_turn: 0.,
            initial_nt_index: 0,
            support_helix: None,
            path_id: None,
        }
    }

    pub fn new_tube_spiral(desc: TubeSpiralDescriptor) -> Self {
        Self {
            position: Vec3::zero(),
            orientation: Rotor3::identity(),
            helix_parameters: None,
            isometry2d: None,
            additional_isometries: Vec::new(),
            symmetry: Vec2::one(),
            grid_position: None,
            visible: true,
            roll: 0f32,
            locked_for_simulations: false,
            curve: Some(Arc::new(CurveDescriptor::TubeSpiral(desc))),
            instantiated_curve: None,
            instantiated_descriptor: None,
            delta_bases_per_turn: 0.,
            initial_nt_index: 0,
            support_helix: None,
            path_id: None,
        }
    }

    pub fn new_with_curve(desc: CurveDescriptor) -> Self {
        Self {
            position: Vec3::zero(),
            orientation: Rotor3::identity(),
            helix_parameters: None,
            isometry2d: None,
            additional_isometries: Vec::new(),
            symmetry: Vec2::one(),
            grid_position: None,
            visible: true,
            roll: 0f32,
            locked_for_simulations: false,
            curve: Some(Arc::new(desc)),
            instantiated_curve: None,
            instantiated_descriptor: None,
            delta_bases_per_turn: 0.,
            initial_nt_index: 0,
            support_helix: None,
            path_id: None,
        }
    }

    pub fn piecewise_bezier_points(&self) -> Option<Vec<Vec3>> {
        if let Some(CurveDescriptor::PiecewiseBezier { .. }) = self.curve.as_ref().map(Arc::as_ref)
        {
            Some(self.bezier_points())
        } else {
            None
        }
    }

    pub fn cubic_bezier_points(&self) -> Option<Vec<Vec3>> {
        if let Some(CurveDescriptor::Bezier(_)) = self.curve.as_ref().map(Arc::as_ref) {
            Some(self.bezier_points())
        } else {
            None
        }
    }

    pub fn translate_bezier_point(
        &self,
        _bezier_point: BezierControlPoint,
        _translation: GridAwareTranslation,
    ) -> Result<(), DesignOperationError> {
        log::error!("Translation of cubic bezier point not implemented");
        Ok(())
    }

    fn bezier_points(&self) -> Vec<Vec3> {
        if let Some(desc) = self.instantiated_descriptor.as_ref() {
            desc.bezier_points()
        } else {
            vec![]
        }
    }

    pub fn new_bezier_two_points(
        grid_manager: &GridData,
        grid_pos_start: HelixGridPosition,
        grid_pos_end: HelixGridPosition,
    ) -> Result<Self, DesignOperationError> {
        let position = grid_manager
            .pos_to_space(grid_pos_start.light())
            .ok_or(DesignOperationError::GridDoesNotExist(grid_pos_start.grid))?;
        let point_start = BezierEnd {
            position: grid_pos_start.light(),
            inward_coeff: 1.,
            outward_coeff: 1.,
        };
        let point_end = BezierEnd {
            position: grid_pos_end.light(),
            inward_coeff: 1.,
            outward_coeff: 1.,
        };
        let constructor = CurveDescriptor::PiecewiseBezier {
            points: vec![point_start, point_end],
            t_max: None,
            t_min: None,
        };
        let mut ret = Self {
            position,
            orientation: Rotor3::identity(),
            helix_parameters: Some(grid_manager.helix_parameters),
            isometry2d: None,
            additional_isometries: Vec::new(),
            symmetry: Vec2::one(),
            grid_position: Some(grid_pos_start),
            visible: true,
            roll: 0f32,
            locked_for_simulations: false,
            curve: Some(Arc::new(constructor)),
            instantiated_curve: None,
            instantiated_descriptor: None,
            delta_bases_per_turn: 0.,
            initial_nt_index: 0,
            support_helix: None,
            path_id: None,
        };
        // we can use a fake cache because we don't need it for bezier curves.
        let mut fake_cache = Default::default();
        grid_manager.update_curve(&mut ret, &mut fake_cache);
        Ok(ret)
    }

    pub fn new_on_bezier_path(
        grid_manager: &GridData,
        grid_pos: HelixGridPosition,
        path_id: BezierPathId,
    ) -> Result<Self, DesignOperationError> {
        let translation = (|| {
            let grid = grid_manager.grids.get(&grid_pos.grid)?;
            let position = grid.position_helix_in_grid_coordinates(grid_pos.x, grid_pos.y);
            Some(position)
        })();

        let curve = translation
            .map(|translation| CurveDescriptor::TranslatedPath {
                path_id,
                translation,
                legacy: false,
            })
            .map(Arc::new);

        let mut ret = Self {
            position: Vec3::zero(),
            orientation: Rotor3::identity(),
            helix_parameters: None,
            isometry2d: None,
            additional_isometries: Vec::new(),
            symmetry: Vec2::one(),
            grid_position: Some(grid_pos),
            visible: true,
            roll: 0f32,
            locked_for_simulations: false,
            curve,
            instantiated_curve: None,
            instantiated_descriptor: None,
            delta_bases_per_turn: 0.,
            initial_nt_index: 0,
            support_helix: None,
            path_id: Some(path_id),
        };
        let mut fake_cache = Default::default();
        grid_manager.update_curve(&mut ret, &mut fake_cache);
        Ok(ret)
    }

    pub fn nb_bezier_nucls(&self) -> usize {
        self.instantiated_curve
            .as_ref()
            .map_or(0, |c| c.curve.as_ref().nb_points())
    }

    pub fn roll_at_pos(&self, n: isize, cst: &HelixParameters) -> f32 {
        let bases_per_turn = match self.helix_parameters {
            None => cst.bases_per_turn,
            Some(p) => p.bases_per_turn,
        } + self.delta_bases_per_turn;
        let beta = TAU / bases_per_turn;
        self.roll - n as f32 * beta // Beta is positive but helix turn clockwise when n increases
    }

    /// Angle of base number `n` around this helix.
    pub fn theta(&self, n: isize, forward: bool, cst: &HelixParameters) -> f32 {
        // The groove_angle goes from the backward strand to the forward strand
        let shift = if forward { cst.groove_angle } else { 0. };
        let bases_per_turn = match self.helix_parameters {
            None => cst.bases_per_turn,
            Some(p) => p.bases_per_turn,
        } + self.delta_bases_per_turn;
        let beta = TAU / bases_per_turn;
        self.roll
            -n as f32 * beta  // Beta is positive but helix turn clockwise when n increases
            + shift
            + FRAC_PI_2 // Add PI/2 so that when the roll is 0,
        // the backward strand is at vertical position on nucl 0
    }

    /// 3D position of a nucleotide on this helix. `n` is the position along the axis, and `forward` is true iff the 5' to 3' direction of the strand containing that nucleotide runs in the same direction as the axis of the helix.
    pub fn space_pos(&self, p: &HelixParameters, n: isize, forward: bool) -> Vec3 {
        let p = self.helix_parameters.unwrap_or(*p);
        self.shifted_space_pos(&p, n, forward, 0.0)
    }

    pub fn normal_at_pos(&self, n: isize, forward: bool) -> Vec3 {
        self.instantiated_curve
            .as_ref()
            .and_then(|c| {
                let axis = c.curve.axis_at_pos(n, forward)?;

                // THIS WORKS FOR BEZIER CURVES
                Some(dvec_to_vec(axis[2]))
                // THIS WORKS FOR T1
                // Some(dvec_to_vec(axis[2]).rotated_by(self.orientation))
            })
            .unwrap_or_else(|| Vec3::unit_x().rotated_by(self.orientation))
    }

    pub fn curvature_at_pos(&self, n: isize) -> Option<f64> {
        self.instantiated_curve
            .as_ref()
            .and_then(|c| c.curve.curvature_at_pos(n))
    }

    pub fn torsion_at_pos(&self, n: isize) -> Option<f64> {
        self.instantiated_curve
            .as_ref()
            .and_then(|c| c.curve.torsion_at_pos(n))
    }

    fn theta_n_to_space_pos(
        &self,
        p: &HelixParameters,
        n: isize,
        theta: f32,
        forward: bool,
    ) -> Vec3 {
        let p = self.helix_parameters.unwrap_or(*p);
        if let Some(curve) = self.instantiated_curve.as_ref()
            && let Some(point) = curve
                .as_ref()
                .nucl_pos(n, forward, theta as f64, &p)
                .map(dvec_to_vec)
        {
            let (position, orientation) = if curve.as_ref().has_its_own_encoded_frame() {
                (Vec3::zero(), Rotor3::identity()) // position and orientation ignored
            } else {
                (self.position, self.orientation)
            };
            return point.rotated_by(orientation) + position;
        }

        let delta_inclination = if forward { 0.0 } else { p.inclination };
        let mut ret = Vec3::new(
            n as f32 * p.rise + delta_inclination,
            theta.sin() * p.helix_radius,
            theta.cos() * p.helix_radius,
        );
        ret = self.rotate_point(ret);
        ret += self.position;
        ret
    }

    pub fn shifted_space_pos(
        &self,
        p: &HelixParameters,
        n: isize,
        forward: bool,
        shift: f32,
    ) -> Vec3 {
        let p = self.helix_parameters.unwrap_or(*p);
        //  match self.helix_parameters {
        //     None => p.clone(),
        //     Some(hp) => hp.clone(),
        // };
        let n = self.initial_nt_index + n;
        let theta = self.theta(n, forward, &p) + shift;
        self.theta_n_to_space_pos(&p, n, theta, forward)
    }

    ///Return an helix that makes an ideal cross-over with self at position n.
    #[must_use]
    pub fn ideal_neighbor(&self, n: isize, forward: bool, p: &HelixParameters) -> Self {
        let p = match self.helix_parameters {
            None => *p,
            Some(hp) => hp,
        };
        let other_helix_pos = self.position_ideal_neighbor(n, forward, &p);
        let mut new_helix = self.detached_copy_at(other_helix_pos);
        self.adjust_theta_neighbor(n, forward, &mut new_helix, &p);
        new_helix
    }

    fn detached_copy_at(&self, position: Vec3) -> Self {
        Self {
            position,
            orientation: self.orientation,
            helix_parameters: None,
            grid_position: None,
            roll: 0.,
            visible: true,
            isometry2d: None,
            additional_isometries: Vec::new(),
            symmetry: Vec2::one(),
            locked_for_simulations: false,
            curve: None,
            instantiated_curve: None,
            instantiated_descriptor: None,
            delta_bases_per_turn: 0.,
            initial_nt_index: 0,
            support_helix: None,
            path_id: None,
        }
    }

    fn position_ideal_neighbor(&self, n: isize, forward: bool, p: &HelixParameters) -> Vec3 {
        let p = match self.helix_parameters {
            None => *p,
            Some(hp) => hp,
        };
        let axis_pos = self.axis_position(&p, n, forward);
        let my_nucl_pos = self.space_pos(&p, n, forward);
        let direction = (my_nucl_pos - axis_pos).normalized();

        (2. * p.helix_radius + p.inter_helix_gap) * direction + axis_pos
    }

    fn adjust_theta_neighbor(
        &self,
        n: isize,
        forward: bool,
        new_helix: &mut Self,
        p: &HelixParameters,
    ) {
        let p = match self.helix_parameters {
            None => *p,
            Some(hp) => hp,
        };
        let theta_current = new_helix.theta(0, forward, &p);
        let theta_obj = self.theta(n, forward, &p) + PI;
        new_helix.roll = theta_obj - theta_current;
    }

    pub fn get_axis<'a>(&'a self, p: &HelixParameters) -> Axis<'a> {
        if let Some(curve) = self.instantiated_curve.as_ref() {
            let shift = self.initial_nt_index;
            let points = curve.as_ref().points();
            let (position, orientation) = if curve.as_ref().has_its_own_encoded_frame() {
                (DVec3::zero(), DRotor3::identity())
            } else {
                (
                    vec_to_dvec(self.position),
                    rotor_to_drotor(self.orientation),
                )
            };
            Axis::Curve {
                shift,
                points,
                nucl_t0: curve.as_ref().nucl_t0(),
                position,
                orientation,
            }
        } else {
            let p = self.helix_parameters.unwrap_or(*p);
            Axis::Line {
                origin: self.position,
                direction: self.axis_position(&p, 1, true) - self.position,
            }
        }
    }

    pub fn axis_position(&self, p: &HelixParameters, n: isize, forward: bool) -> Vec3 {
        // WARNING: doesn't take the inclination into account!
        let n = n + self.initial_nt_index;
        if let Some(curve) = self.instantiated_curve.as_ref().map(|s| &s.curve)
            && let Some(point) = curve.axis_pos(n, forward).map(dvec_to_vec)
        {
            let (position, orientation) = if curve.as_ref().has_its_own_encoded_frame() {
                (Vec3::zero(), Rotor3::identity())
            } else {
                (self.position, self.orientation)
            };
            return point.rotated_by(orientation) + position;
        }
        let p = self.helix_parameters.unwrap_or(*p);
        let mut ret = Vec3::new(n as f32 * p.rise, 0., 0.);

        ret = self.rotate_point(ret);
        ret += self.position;
        ret
    }

    pub fn rotate_point(&self, ret: Vec3) -> Vec3 {
        ret.rotated_by(self.orientation)
    }

    fn append_translation(&mut self, translation: Vec3) {
        self.position += translation;
    }

    fn append_rotation(&mut self, rotation: Rotor3) {
        self.orientation = rotation * self.orientation;
        self.position = rotation * self.position;
    }

    pub fn rotate_around(&mut self, rotation: Rotor3, origin: Vec3) {
        self.append_translation(-origin);
        self.append_rotation(rotation);
        self.append_translation(origin);
    }

    pub fn translate(&mut self, translation: Vec3) {
        self.append_translation(translation);
    }

    pub fn roll(&mut self, roll: f32) {
        self.roll += roll;
    }

    pub fn set_roll(&mut self, roll: f32) {
        self.roll = roll;
    }

    pub fn get_bezier_controls(&self) -> Option<CubicBezierConstructor> {
        self.instantiated_descriptor
            .as_ref()
            .and_then(|c| c.get_bezier_controls())
    }

    pub fn get_curve_range(&self) -> Option<std::ops::RangeInclusive<isize>> {
        self.instantiated_curve
            .as_ref()
            .map(|curve| curve.curve.range())
    }

    pub fn get_surface_info_nucl(&self, nucl: Nucl) -> Option<SurfaceInfo> {
        let mut surface_info = self.instantiated_curve.as_ref().and_then(|curve| {
            let curve = &curve.curve;
            let t = curve.nucl_time(nucl.position)?;
            curve.geometry.surface_info_time(t, nucl.helix)
        })?;
        surface_info.local_frame.rotate_by(self.orientation);
        surface_info.position.rotate_by(self.orientation);
        surface_info.position += self.position;
        Some(surface_info)
    }

    pub fn get_surface_info(&self, point: SurfacePoint) -> Option<SurfaceInfo> {
        let mut surface_info = self.instantiated_curve.as_ref().and_then(|curve| {
            let curve = &curve.curve;
            curve.geometry.surface_info(point)
        })?;
        surface_info.local_frame.rotate_by(self.orientation);
        surface_info.position.rotate_by(self.orientation);
        surface_info.position += self.position;
        Some(surface_info)
    }
}

#[derive(Default, Clone)]
pub struct NuclCollection {
    pub identifier: BTreeMap<Nucl, u32>,
    virtual_nucl_map: HashMap<VirtualNucl, Nucl>,
}

impl NuclCollection {
    pub fn iter_nucls_ids(&'_ self) -> impl Iterator<Item = (&'_ Nucl, &'_ u32)> {
        self.identifier.iter()
    }

    pub fn virtual_to_real(&self, virtual_nucl: &VirtualNucl) -> Option<&Nucl> {
        self.virtual_nucl_map.get(virtual_nucl)
    }

    pub fn get_identifier(&self, nucl: &Nucl) -> Option<&u32> {
        self.identifier.get(nucl)
    }

    pub fn contains_nucl(&self, nucl: &Nucl) -> bool {
        self.identifier.contains_key(nucl)
    }

    pub fn nb_nucls(&self) -> usize {
        self.identifier.len()
    }

    pub fn insert(&mut self, key: Nucl, id: u32) -> Option<u32> {
        self.identifier.insert(key, id)
    }

    pub fn insert_virtual(&mut self, virtual_nucl: VirtualNucl, nucl: Nucl) -> Option<Nucl> {
        self.virtual_nucl_map.insert(virtual_nucl, nucl)
    }
}

/// Represents the axis of an helix.
#[derive(Debug, Clone, Copy)]
pub enum Axis<'a> {
    Line {
        origin: Vec3,
        direction: Vec3,
    },
    Curve {
        shift: isize,
        points: &'a [DVec3],
        nucl_t0: usize,
        position: DVec3,
        orientation: DRotor3,
    },
}

#[derive(Debug, Clone)]
pub enum OwnedAxis {
    Line {
        origin: Vec3,
        direction: Vec3,
    },
    Curve {
        shift: isize,
        points: Vec<DVec3>,
        nucl_t0: usize,
        position: DVec3,
        orientation: DRotor3,
    },
}

impl OwnedAxis {
    pub fn borrow(&self) -> Axis<'_> {
        match self {
            Self::Line { origin, direction } => Axis::Line {
                origin: *origin,
                direction: *direction,
            },
            Self::Curve {
                shift,
                points,
                nucl_t0,
                orientation,
                position,
            } => Axis::Curve {
                shift: *shift,
                points,
                nucl_t0: *nucl_t0,
                orientation: *orientation,
                position: *position,
            },
        }
    }
}

impl Axis<'_> {
    pub fn to_owned(self) -> OwnedAxis {
        match self {
            Self::Line { origin, direction } => OwnedAxis::Line { origin, direction },
            Self::Curve {
                shift,
                points,
                nucl_t0,
                orientation,
                position,
            } => OwnedAxis::Curve {
                shift,
                points: points.to_vec(),
                nucl_t0,
                orientation,
                position,
            },
        }
    }

    #[must_use]
    pub fn transformed(&self, model_matrix: &Mat4) -> Self {
        match self {
            Self::Line {
                origin: old_origin,
                direction: old_direction,
            } => {
                let origin = model_matrix.transform_point3(*old_origin);
                let direction = model_matrix.transform_vec3(*old_direction);
                Self::Line { origin, direction }
            }
            Self::Curve { .. } => *self,
        }
    }

    pub fn direction(&self) -> Option<Vec3> {
        if let Axis::Line { direction, .. } = self {
            Some(*direction)
        } else {
            None
        }
    }
}

/// An additional 2d helix used to represent an helix in the 2d view.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdditionalHelix2D {
    /// The minimum nucleotide index of the helix.
    ///
    /// Nucleotides with small indices are represented by the previous helix.
    pub left: isize,
    /// The Isometry to be applied after applying the isometry of the main helix 2d representation
    /// to obtain this segment.
    pub additional_isometry: Option<Isometry2>,
    pub additional_symmetry: Option<Vec2>,
}

#[derive(Debug, Clone)]
pub struct HalfHBond {
    pub backbone: Vec3,
    pub center_of_mass: Vec3,
    pub base: Option<char>,
    pub backbone_color: u32,
}

#[derive(Debug, Clone)]
pub struct HBond {
    pub forward: HalfHBond,
    pub backward: HalfHBond,
}

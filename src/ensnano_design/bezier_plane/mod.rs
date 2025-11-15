mod import_from_svg;

pub use import_from_svg::*;

use super::Collection;
use super::HelixParameters;
use super::PieceWiseBezierInstantiator;
use super::collection::HasMap;
use super::curves::{BezierEndCoordinates, Curve, InstantiatedPiecewiseBezier};
use super::grid::*;
use super::utils::rotor_to_drotor;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, sync::Arc};
use ultraviolet::{DMat3, DVec3, Mat3, Rotor3, Vec2, Vec3};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BezierPlaneDescriptor {
    pub position: Vec3,
    pub orientation: Rotor3,
}

impl Default for BezierPlaneDescriptor {
    fn default() -> Self {
        Self {
            position: Vec3::zero(),
            orientation: Rotor3::identity(),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct BezierPlaneId(pub u32);

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct BezierPlanes(Arc<BTreeMap<BezierPlaneId, Arc<BezierPlaneDescriptor>>>);

impl HasMap for BezierPlanes {
    type Key = BezierPlaneId;
    type Item = BezierPlaneDescriptor;
    fn get_map(&self) -> &BTreeMap<Self::Key, Arc<Self::Item>> {
        &self.0
    }
}

impl BezierPlanes {
    pub fn make_mut(&mut self) -> BezierPlanesMut<'_> {
        let new_map = BTreeMap::clone(&self.0);
        BezierPlanesMut {
            source: self,
            new_map,
        }
    }
}

impl BezierPlaneDescriptor {
    pub fn ray_intersection(
        &self,
        origin: Vec3,
        direction: Vec3,
    ) -> Option<BezierPlaneIntersection> {
        let normal = Vec3::unit_x().rotated_by(self.orientation);
        let denom = direction.dot(normal);
        let depth = if denom.abs() < 1e-3 {
            None
        } else {
            let d = (self.position - origin).dot(normal) / denom;
            Some(d)
        }?;
        let (x, y) = {
            let intersection = origin + depth * direction;
            let vec = intersection - self.position;
            let x_dir = Vec3::unit_z().rotated_by(self.orientation);
            let y_dir = Vec3::unit_y().rotated_by(self.orientation);
            (vec.dot(x_dir), vec.dot(y_dir))
        };
        Some(BezierPlaneIntersection { x, y, depth })
    }

    pub fn space_position_of_point2d(&self, vec: Vec2) -> Vec3 {
        self.position
            + Vec3::unit_z().rotated_by(self.orientation) * vec.x
            + Vec3::unit_y().rotated_by(self.orientation) * vec.y
    }

    pub fn vec2_angle_to_vec3(&self, vec: Vec2, angle: f32) -> Vec3 {
        let z = vec.mag() * angle.tan();
        Vec3::unit_z().rotated_by(self.orientation) * vec.x
            + Vec3::unit_y().rotated_by(self.orientation) * vec.y
            + Vec3::unit_x().rotated_by(self.orientation) * z
    }
}

pub fn ray_bezier_plane_intersection<'a>(
    planes: impl Iterator<Item = (&'a BezierPlaneId, &'a BezierPlaneDescriptor)>,
    origin: Vec3,
    direction: Vec3,
) -> Option<(BezierPlaneId, BezierPlaneIntersection)> {
    let mut ret: Option<(BezierPlaneId, BezierPlaneIntersection)> = None;
    for (id, plane) in planes {
        if let Some(intersection) = plane.ray_intersection(origin, direction) {
            if let Some((best_id, inter)) = ret.as_mut() {
                if inter.depth > intersection.depth {
                    *best_id = *id;
                    *inter = intersection;
                }
            } else {
                ret = Some((*id, intersection));
            }
        }
    }
    ret
}

#[derive(Debug)]
pub struct BezierPlaneIntersection {
    pub x: f32,
    pub y: f32,
    pub depth: f32,
}

impl BezierPlaneIntersection {
    pub fn position(&self) -> Vec2 {
        Vec2::new(self.x, self.y)
    }
}

pub struct BezierPlanesMut<'a> {
    source: &'a mut BezierPlanes,
    new_map: BTreeMap<BezierPlaneId, Arc<BezierPlaneDescriptor>>,
}

impl BezierPlanesMut<'_> {
    pub fn push(&mut self, desc: BezierPlaneDescriptor) {
        let new_key = self
            .new_map
            .keys()
            .max()
            .map(|m| BezierPlaneId(m.0 + 1))
            .unwrap_or_default();
        self.new_map.insert(new_key, Arc::new(desc));
    }
}

impl Drop for BezierPlanesMut<'_> {
    fn drop(&mut self) {
        *self.source = BezierPlanes(Arc::new(std::mem::take(&mut self.new_map)));
    }
}

#[derive(
    Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Default, Hash,
)]
pub struct BezierPathId(pub u32);

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct BezierPaths(Arc<BTreeMap<BezierPathId, Arc<BezierPath>>>);

impl HasMap for BezierPaths {
    type Key = BezierPathId;
    type Item = BezierPath;
    fn get_map(&self) -> &BTreeMap<Self::Key, Arc<Self::Item>> {
        self.0.as_ref()
    }
}

pub struct BezierPathsMut<'a> {
    source: &'a mut BezierPaths,
    new_map: BTreeMap<BezierPathId, Arc<BezierPath>>,
}

impl BezierPaths {
    pub fn make_mut(&mut self) -> BezierPathsMut<'_> {
        BezierPathsMut {
            new_map: BTreeMap::clone(&self.0),
            source: self,
        }
    }
}

impl BezierPathsMut<'_> {
    pub fn create_path(&mut self, first_vertex: BezierVertex) -> BezierPathId {
        let new_key = self
            .new_map
            .keys()
            .max()
            .map(|m| BezierPathId(m.0 + 1))
            .unwrap_or_default();
        let new_path = BezierPath {
            vertices: vec![first_vertex],
            is_cyclic: false,
            grid_type: None,
        };
        self.new_map.insert(new_key, Arc::new(new_path));
        new_key
    }

    pub fn get_mut(&mut self, id: &BezierPathId) -> Option<&mut BezierPath> {
        self.new_map.get_mut(id).map(Arc::make_mut)
    }

    pub fn values_mut(&mut self) -> impl Iterator<Item = &mut BezierPath> {
        self.new_map.values_mut().map(Arc::make_mut)
    }

    pub fn push(&mut self, path: BezierPath) {
        let id = self
            .new_map
            .keys()
            .max()
            .map_or(BezierPathId(0), |BezierPathId(n)| BezierPathId(n + 1));
        self.new_map.insert(id, Arc::new(path));
    }

    #[must_use]
    pub fn remove_path(&mut self, path_id: &BezierPathId) -> Option<()> {
        self.new_map.contains_key(path_id).then(|| {
            self.new_map.remove(path_id);
        })
    }
}

impl Drop for BezierPathsMut<'_> {
    fn drop(&mut self) {
        *self.source = BezierPaths(Arc::new(std::mem::take(&mut self.new_map)));
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct BezierPath {
    vertices: Vec<BezierVertex>,
    #[serde(alias = "cyclic")]
    pub is_cyclic: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub grid_type: Option<GridTypeDescr>,
}

impl BezierPath {
    pub fn add_vertex(&mut self, vertex: BezierVertex) -> usize {
        self.vertices.push(vertex);
        self.vertices.len() - 1
    }

    pub fn get_vertex_mut(&mut self, vertex_id: usize) -> Option<&mut BezierVertex> {
        self.vertices.get_mut(vertex_id)
    }

    pub fn vertices(&self) -> &[BezierVertex] {
        &self.vertices
    }

    pub fn vertices_mut(&mut self) -> &mut [BezierVertex] {
        self.vertices.as_mut_slice()
    }

    #[must_use]
    pub fn remove_vertex(&mut self, v_id: usize) -> Option<()> {
        (self.vertices.len() > v_id).then(|| {
            self.vertices.remove(v_id);
        })
    }

    pub fn set_vector_out(&mut self, i: usize, vector_out: Vec3, planes: &BezierPlanes) {
        if let Some(v) = self.vertices_mut().get_mut(i) {
            v.set_vector_out(vector_out, planes);
        }
    }

    pub fn to_instantiated_path_2d(&self) -> Option<InstantiatedPiecewiseBezier> {
        self.instantiate()
    }
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct BezierVertex {
    pub plane_id: BezierPlaneId,
    pub position: Vec2,
    pub position_in: Option<Vec2>,
    pub position_out: Option<Vec2>,
    #[serde(default)]
    grid_translation: Vec3,
    #[serde(default)]
    angle_with_plane: f32,
}

impl BezierVertex {
    pub fn space_position(&self, planes: &BezierPlanes) -> Option<Vec3> {
        if let Some(plane) = planes.0.get(&self.plane_id) {
            Some(
                plane.position
                    + self.position.x * Vec3::unit_z().rotated_by(plane.orientation)
                    + self.position.y * Vec3::unit_y().rotated_by(plane.orientation),
            )
        } else {
            log::error!("Could not get plane");
            None
        }
    }

    fn set_vector_out(&mut self, vector_out: Vec3, planes: &BezierPlanes) {
        if let Some(plane) = planes.0.get(&self.plane_id) {
            // (x, y) = coordinates in the bezier plane
            let x = vector_out.dot(Vec3::unit_z().rotated_by(plane.orientation));
            let y = vector_out.dot(Vec3::unit_y().rotated_by(plane.orientation));
            let height = vector_out.dot(Vec3::unit_x().rotated_by(plane.orientation));

            let tangent_angle = height / Vec2::new(x, y).mag();
            let angle_with_plane = tangent_angle.atan();
            let ratio = if let Some((inward, outward)) = self
                .position_in
                .map(|x| self.position - x)
                .zip(self.position_out.map(|x| x - self.position))
            {
                inward.mag() / outward.mag()
            } else {
                1.
            };
            let vector_out = Vec2::new(x, y);
            let vector_in = ratio * vector_out;
            self.angle_with_plane = angle_with_plane;
            self.position_out = Some(self.position + vector_out);
            self.position_in = Some(self.position - vector_in);
        }
    }

    pub fn grid_position(&self, planes: &BezierPlanes) -> Option<Vec3> {
        self.space_position(planes)
            .map(|p| p + self.grid_translation)
    }

    pub fn add_translation(&mut self, translation: Vec3) {
        self.grid_translation += translation;
    }

    pub fn new(plane_id: BezierPlaneId, position: Vec2) -> Self {
        Self {
            plane_id,
            position,
            position_out: None,
            position_in: None,
            grid_translation: Vec3::zero(),
            angle_with_plane: 0.,
        }
    }
}

pub struct InstantiatedPath {
    source_planes: BezierPlanes,
    source_path: Arc<BezierPath>,
    pub(crate) curve_descriptor: Option<Arc<InstantiatedPiecewiseBezier>>,
    curve_descriptor_2d: Option<Arc<InstantiatedPiecewiseBezier>>,
    curve_2d: Option<Curve>,
    pub(crate) frames: Option<Vec<(Vec3, Rotor3)>>,
}

struct BezierInstantiator {
    source_planes: BezierPlanes,
    source_path: Arc<BezierPath>,
    path_3d: bool,
}

impl PieceWiseBezierInstantiator<Vec3> for BezierInstantiator {
    fn vector_in(&self, i: usize) -> Option<Vec3> {
        let vertex = self.source_path.vertices().get(i)?;
        vertex.position_in.and_then(|position_in| {
            let vec2 = vertex.position - position_in;
            let plane = self.source_planes.get(&vertex.plane_id)?;
            Some(plane.vec2_angle_to_vec3(vec2, vertex.angle_with_plane))
        })
    }

    fn vector_out(&self, i: usize) -> Option<Vec3> {
        let vertex = self.source_path.vertices().get(i)?;
        vertex.position_out.and_then(|position_out| {
            let vec2 = position_out - vertex.position;
            let plane = self.source_planes.get(&vertex.plane_id)?;
            Some(plane.vec2_angle_to_vec3(vec2, vertex.angle_with_plane))
        })
    }

    fn position(&self, i: usize) -> Option<Vec3> {
        let vertex = self.source_path.vertices().get(i)?;
        if self.path_3d {
            vertex.grid_position(&self.source_planes)
        } else {
            vertex.space_position(&self.source_planes)
        }
    }

    fn nb_vertices(&self) -> usize {
        self.source_path.vertices.len()
    }

    fn is_cyclic(&self) -> bool {
        self.source_path.is_cyclic
    }
}

impl PieceWiseBezierInstantiator<Vec2> for BezierPath {
    fn vector_in(&self, i: usize) -> Option<Vec2> {
        let vertex = self.vertices().get(i)?;
        vertex
            .position_in
            .map(|position_in| vertex.position - position_in)
    }

    fn vector_out(&self, i: usize) -> Option<Vec2> {
        let vertex = self.vertices().get(i)?;
        vertex
            .position_out
            .map(|position_out| position_out - vertex.position)
    }

    fn position(&self, i: usize) -> Option<Vec2> {
        self.vertices().get(i).map(|v| v.position)
    }

    fn nb_vertices(&self) -> usize {
        self.vertices.len()
    }

    fn is_cyclic(&self) -> bool {
        self.is_cyclic
    }
}

fn path_to_curve_descriptor(
    source_planes: BezierPlanes,
    source_path: Arc<BezierPath>,
    path_3d: bool,
) -> Option<InstantiatedPiecewiseBezier> {
    let instantiator = BezierInstantiator {
        source_planes,
        source_path,
        path_3d,
    };
    let mut ret =
        <BezierInstantiator as PieceWiseBezierInstantiator<Vec3>>::instantiate(&instantiator)?;

    // This descriptor is only used to draw the path of the curve on the bezier plane. It does not
    // need to be precise, but it is better if we can update it quickly.
    ret.discretize_quickly = true;
    Some(ret)
}

fn curve_descriptor_to_frame(
    source_planes: BezierPlanes,
    source_path: Arc<BezierPath>,
    desc: &InstantiatedPiecewiseBezier,
) -> Option<Vec<(Vec3, Rotor3)>> {
    source_path
        .vertices
        .iter()
        .zip(desc.ends.iter())
        .map(|(v_desc, v_instance)| {
            let up = source_planes
                .0
                .get(&v_desc.plane_id)
                .map(|p| Vec3::unit_x().rotated_by(p.orientation).normalized())?;
            let right = -v_instance.vector_out.normalized();
            let front = right.cross(up).normalized();
            let up = front.cross(right).normalized();
            let orientation = Mat3::new(right, up, front).into_rotor3();

            Some((v_instance.position, orientation))
        })
        .collect()
}

impl InstantiatedPath {
    fn new(
        source_planes: BezierPlanes,
        source_path: Arc<BezierPath>,
        helix_parameters: &HelixParameters,
    ) -> Self {
        let descriptor_2d =
            path_to_curve_descriptor(source_planes.clone(), source_path.clone(), false);
        let descriptor_3d =
            path_to_curve_descriptor(source_planes.clone(), source_path.clone(), true);
        let frames = descriptor_2d.as_ref().and_then(|desc| {
            curve_descriptor_to_frame(source_planes.clone(), source_path.clone(), desc)
        });
        let curve_2d = descriptor_2d
            .clone()
            .filter(|d| d.ends.len() >= 2) // Do not try to create a curve if there is only one vertex
            .map(|desc| Curve::new(desc, helix_parameters));
        Self {
            source_planes,
            source_path,
            curve_2d,
            curve_descriptor_2d: descriptor_2d.map(Arc::new),
            curve_descriptor: descriptor_3d.map(Arc::new),
            frames,
        }
    }

    fn updated(
        &self,
        source_planes: BezierPlanes,
        source_path: Arc<BezierPath>,
        helix_parameters: &HelixParameters,
    ) -> Option<Self> {
        self.need_update(&source_planes, &source_path)
            .then(|| Self::new(source_planes, source_path, helix_parameters))
    }

    fn need_update(&self, source_planes: &BezierPlanes, source_path: &Arc<BezierPath>) -> bool {
        !Arc::ptr_eq(&source_planes.0, &self.source_planes.0)
            || !Arc::ptr_eq(&self.source_path, source_path)
    }

    pub fn bezier_controls(&self) -> &[BezierEndCoordinates] {
        self.curve_descriptor_2d
            .as_ref()
            .map_or(&[], |c| c.ends.as_slice())
    }

    pub fn get_curve_points(&self) -> &[DVec3] {
        self.curve_2d
            .as_ref()
            .map_or(&[], |c| c.positions_forward.as_slice())
    }

    pub fn initial_frame(&self) -> Option<DMat3> {
        self.frames
            .as_ref()
            .and_then(|fs| fs.first())
            .as_ref()
            .map(|f| rotor_to_drotor(f.1).into_matrix())
            .map(|m| DMat3::new(m.cols[2], m.cols[1], m.cols[0]))
    }
}

#[derive(Clone)]
pub struct BezierPathData {
    source_planes: BezierPlanes,
    pub(crate) source_paths: BezierPaths,
    pub instantiated_paths: Arc<BTreeMap<BezierPathId, Arc<InstantiatedPath>>>,
}

impl std::fmt::Debug for BezierPathData {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("BezierPathData")
            .field(
                "instantiated_paths",
                &format_args!("{:p}", &self.instantiated_paths),
            )
            .finish()
    }
}

impl BezierPathData {
    pub fn new(
        source_planes: BezierPlanes,
        source_paths: BezierPaths,
        helix_parameters: &HelixParameters,
    ) -> Self {
        let instantiated_paths: BTreeMap<_, _> = source_paths
            .0
            .iter()
            .map(|(id, path)| {
                (
                    *id,
                    Arc::new(InstantiatedPath::new(
                        source_planes.clone(),
                        path.clone(),
                        helix_parameters,
                    )),
                )
            })
            .collect();
        Self {
            instantiated_paths: Arc::new(instantiated_paths),
            source_planes,
            source_paths,
        }
    }

    pub fn need_update(&self, source_planes: &BezierPlanes, source_paths: &BezierPaths) -> bool {
        !Arc::ptr_eq(&source_planes.0, &self.source_planes.0)
            || !Arc::ptr_eq(&self.source_paths.0, &source_paths.0)
    }

    pub fn updated(
        &self,
        source_planes: BezierPlanes,
        source_paths: BezierPaths,
        helix_parameters: &HelixParameters,
    ) -> Option<Self> {
        self.need_update(&source_planes, &source_paths).then(|| {
            let instantiated_paths: BTreeMap<_, _> = source_paths
                .0
                .iter()
                .map(|(id, source_path)| {
                    let path = if let Some(path) = self.instantiated_paths.get(id) {
                        path.updated(source_planes.clone(), source_path.clone(), helix_parameters)
                            .map_or_else(|| path.clone(), Arc::new)
                    } else {
                        Arc::new(InstantiatedPath::new(
                            source_planes.clone(),
                            source_path.clone(),
                            helix_parameters,
                        ))
                    };
                    (*id, path)
                })
                .collect();
            Self {
                instantiated_paths: Arc::new(instantiated_paths),
                source_planes,
                source_paths,
            }
        })
    }

    pub fn ptr_eq(a: &Self, b: &Self) -> bool {
        Arc::ptr_eq(&a.instantiated_paths, &b.instantiated_paths)
    }

    pub fn position_vertex_2d(&self, vertex_id: BezierVertexId) -> Option<Vec3> {
        let path = self.instantiated_paths.get(&vertex_id.path_id)?;
        path.frames
            .as_ref()
            .and_then(|f| f.get(vertex_id.vertex_id))
            .map(|f| f.0)
    }

    pub fn orientation_vertex(&self, vertex_id: BezierVertexId) -> Option<Rotor3> {
        let path = self.instantiated_paths.get(&vertex_id.path_id)?;
        path.frames
            .as_ref()
            .and_then(|f| f.get(vertex_id.vertex_id))
            .map(|f| f.1)
    }

    pub fn get_vector_out(&self, vertex_id: BezierVertexId) -> Option<Vec3> {
        let path = self.instantiated_paths.get(&vertex_id.path_id)?;
        path.curve_descriptor
            .as_ref()
            .and_then(|desc| desc.ends.get(vertex_id.vertex_id))
            .map(|end| end.vector_out)
    }

    pub fn grids(&self) -> Vec<(GridId, GridDescriptor)> {
        self.instantiated_paths
            .iter()
            .flat_map(|(path_id, path)| {
                if let Some(grid_type) = path.source_path.grid_type {
                    path.source_path
                        .vertices
                        .iter()
                        .enumerate()
                        .filter_map(|(vertex_id, v)| {
                            let vertex_id = BezierVertexId {
                                path_id: *path_id,
                                vertex_id,
                            };
                            let desc = GridDescriptor {
                                invisible: false,
                                grid_type,
                                helix_parameters: None,
                                orientation: self.orientation_vertex(vertex_id)?,
                                position: self.position_vertex_2d(vertex_id)? + v.grid_translation,
                                bezier_vertex: Some(vertex_id),
                            };
                            Some((GridId::BezierPathGrid(vertex_id), desc))
                        })
                        .collect()
                } else {
                    vec![]
                }
            })
            .collect()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash, PartialOrd, Ord)]
pub struct BezierVertexId {
    pub path_id: BezierPathId,
    pub vertex_id: usize,
}

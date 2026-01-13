//! This module defines the ensnano format.
//! All other format supported by ensnano are converted into this format and
//! run-time manipulations of designs are performed on an `ensnano::Design` structure

#[cfg(test)]
mod tests;

pub mod bezier_plane;
pub mod cadnano;
pub mod codenano;
pub mod consts;
pub mod curves;
pub mod design_operations;
pub mod domains;
pub mod drawing_style;
pub mod elements;
pub mod external_3d_objects;
pub mod grid;
pub mod group_attributes;
pub mod helices;
mod insertions;
pub mod isometry3_descriptor;
mod material_colors;
pub mod nucl;
pub mod operation;
pub mod organizer;
pub mod parameters;
pub mod scadnano;
pub mod selection;
pub mod strands;
pub mod utils;

use self::{
    bezier_plane::{BezierPathData, BezierPaths, BezierPlanes},
    curves::CurveCache,
    domains::Domain,
    elements::DesignElementKey,
    external_3d_objects::External3DObjects,
    grid::grid_collection::FreeGrids,
    grid::{GridData, GridDescriptor, GridId},
    group_attributes::GroupAttribute,
    helices::{Helices, Helix},
    isometry3_descriptor::Isometry3Descriptor,
    nucl::Nucl,
    parameters::HelixParameters,
    strands::Strands,
};
use crate::{
    grid::HelixGridPosition,
    organizer::tree::{GroupId, OrganizerTree},
    strands::Strand,
};
use serde::{Deserialize, Serialize};
use serde_with::{DefaultOnError, serde_as};
use std::{
    collections::{BTreeMap, HashMap, HashSet},
    sync::Arc,
};
use ultraviolet::{Rotor3, Similarity3, Vec3};

/// The `ensnano` Design structure.
#[derive(Serialize, Deserialize, Clone)]
pub struct Design {
    /// The collection of all helices used in this design. Helices have a
    /// position and an orientation in 3D.
    pub helices: Helices,
    /// The vector of strands.
    pub strands: Strands,
    /// Parameters of DNA geometry. This can be skipped (in JSON), or
    /// set to `None` in Rust, in which case a default set of
    /// parameters from the literature is used.
    #[serde(
        skip_serializing_if = "Option::is_none",
        default,
        rename(serialize = "dna_parameters"),
        alias = "dna_parameters"
    )]
    pub helix_parameters: Option<HelixParameters>,

    /// The strand that is the scaffold if the design is an origami
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub scaffold_id: Option<usize>,

    /// The sequence of the scaffold if the design is an origami
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub scaffold_sequence: Option<String>,

    /// The shifting of the scaffold if the design is an origami. This is used to reduce the number
    /// of anti-pattern in the staples sequences
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub scaffold_shift: Option<usize>,

    #[serde(default)]
    pub free_grids: FreeGrids,

    #[serde(default, skip_serializing, alias = "grids")]
    old_grids: Vec<GridDescriptor>,

    /// The cross-over suggestion groups
    #[serde(skip_serializing_if = "groups_is_empty", default)]
    pub groups: Arc<BTreeMap<usize, bool>>,

    /// The set of identifiers of grids whose helices must not always display their phantom
    /// helices.
    #[serde(skip_serializing_if = "HashSet::is_empty", default)]
    pub no_phantoms: Arc<HashSet<GridId>>,

    /// The set of identifiers of grids whose helices are displayed with smaller spheres for the
    /// nucleotides.
    #[serde(
        alias = "small_shperes", // cspell: disable-line
        alias = "no_spheres",
        rename(serialize = "no_spheres"),
        skip_serializing_if = "HashSet::is_empty",
        default
    )]
    pub small_spheres: Arc<HashSet<GridId>>,

    /// The set of nucleotides that must not move during physical simulations
    #[serde(skip_serializing_if = "HashSet::is_empty", default)]
    pub anchors: HashSet<Nucl>,

    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub organizer_tree: Option<Arc<OrganizerTree<DesignElementKey>>>,

    #[serde(default)]
    pub ensnano_version: String,

    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub group_attributes: HashMap<GroupId, GroupAttribute>,

    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    cameras: BTreeMap<CameraId, Camera>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    favorite_camera: Option<CameraId>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    saved_camera: Option<Camera>,

    #[serde(default, skip_serializing_if = "HashSet::is_empty")]
    pub checked_xovers: HashSet<usize>,

    /// True if the colors of the scaffold's nucleotides should make a rainbow
    #[serde(default)]
    pub rainbow_scaffold: bool,

    #[serde(
        skip,
        alias = "instanciated_grid_data", // cspell: disable-line
    )]
    instantiated_grid_data: Option<GridData>,

    #[serde(skip, default)]
    cached_curve: Arc<CurveCache>,

    #[serde(default)]
    pub bezier_planes: BezierPlanes,

    #[serde(default)]
    pub bezier_paths: BezierPaths,

    #[serde(
        skip,
        alias = "instanciated_paths", // cspell: disable-line
    )]
    instantiated_paths: Option<BezierPathData>,

    #[serde(default)]
    pub external_3d_objects: External3DObjects,

    #[serde(skip)]
    pub additional_structure: Option<Arc<dyn AdditionalStructure>>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub clone_isometries: Option<Vec<Isometry3Descriptor>>,
}

impl Design {
    /// If self is up-to-date return an `UpToDateDesign` reference to self.
    ///
    /// If this methods returns `None`, one needs to call `Design::get_up_to_date` to get an
    /// `UpToDateDesign` reference to the data.
    /// Having an option to not mutate the design is meant to prevent unnecessary run-time cloning
    /// of the design
    pub fn try_get_up_to_date(&self) -> Option<UpToDateDesign<'_>> {
        let paths_data = self
            .instantiated_paths
            .as_ref()
            .filter(|data| !data.need_update(&self.bezier_planes, &self.bezier_paths))?;
        if let Some(data) = self.instantiated_grid_data.as_ref() {
            data.is_up_to_date(self).then_some(UpToDateDesign {
                design: self,
                grid_data: data,
                paths_data,
            })
        } else {
            None
        }
    }

    /// Update self if necessary and returns an up-to-date reference to self.
    pub fn get_up_to_date(&mut self) -> UpToDateDesign<'_> {
        let helix_parameters = self
            .helix_parameters
            .as_ref()
            .unwrap_or(&HelixParameters::DEFAULT);
        if let Some(paths_data) = self.instantiated_paths.as_ref() {
            if let Some(new_data) = paths_data.updated(
                self.bezier_planes.clone(),
                self.bezier_paths.clone(),
                helix_parameters,
            ) {
                self.instantiated_paths = Some(new_data);
            }
        } else {
            self.instantiated_paths = Some(BezierPathData::new(
                self.bezier_planes.clone(),
                self.bezier_paths.clone(),
                helix_parameters,
            ));
        }
        if self.needs_update() {
            let grid_data = GridData::new_by_updating_design(self);
            self.instantiated_grid_data = Some(grid_data);
        }
        UpToDateDesign {
            design: self,
            grid_data: self.instantiated_grid_data.as_ref().unwrap(),
            paths_data: self.instantiated_paths.as_ref().unwrap(),
        }
    }

    pub fn get_up_to_date_paths(&mut self) -> &BezierPathData {
        let helix_parameters = self
            .helix_parameters
            .as_ref()
            .unwrap_or(&HelixParameters::DEFAULT);
        if let Some(paths_data) = self.instantiated_paths.as_ref() {
            if let Some(new_data) = paths_data.updated(
                self.bezier_planes.clone(),
                self.bezier_paths.clone(),
                helix_parameters,
            ) {
                self.instantiated_paths = Some(new_data);
            }
        } else {
            self.instantiated_paths = Some(BezierPathData::new(
                self.bezier_planes.clone(),
                self.bezier_paths.clone(),
                helix_parameters,
            ));
        }
        self.instantiated_paths.as_ref().unwrap()
    }

    fn needs_update(&self) -> bool {
        if let Some(data) = self.instantiated_grid_data.as_ref() {
            !data.is_up_to_date(self)
        } else {
            true
        }
    }

    pub fn new() -> Self {
        Self {
            helices: Default::default(),
            strands: Default::default(),
            helix_parameters: Some(HelixParameters::DEFAULT),
            free_grids: Default::default(),
            scaffold_id: None,
            scaffold_sequence: None,
            scaffold_shift: None,
            groups: Default::default(),
            small_spheres: Default::default(),
            no_phantoms: Default::default(),
            anchors: Default::default(),
            organizer_tree: None,
            ensnano_version: ensnano_version(),
            group_attributes: Default::default(),
            cameras: Default::default(),
            favorite_camera: None,
            saved_camera: None,
            checked_xovers: Default::default(),
            rainbow_scaffold: false,
            instantiated_grid_data: None,
            cached_curve: Default::default(),
            bezier_planes: Default::default(),
            bezier_paths: Default::default(),
            old_grids: Vec::new(),
            instantiated_paths: None,
            external_3d_objects: Default::default(),
            additional_structure: None,
            clone_isometries: Some(Vec::new()),
        }
    }

    pub fn update_version(&mut self) {
        // The conversion from the old grid data structure to the new one can be made regardless of
        // the version.
        let grids = std::mem::take(&mut self.old_grids);
        let mut grids_mut = self.free_grids.make_mut();
        for g in grids {
            grids_mut.push(g);
        }
        drop(grids_mut);

        if version_compare::compare(&self.ensnano_version, "0.5.0") == Ok(version_compare::Cmp::Lt)
        {
            // For legacy reason, the version of curved design must be set to a value >= 0.5.0
            for h in self.helices.values() {
                if h.curve.is_some() {
                    self.ensnano_version = "0.5.0".to_owned();
                    break;
                }
            }
        }

        if self.ensnano_version.is_empty() {
            // Version < 0.2.0 had no version identifier, and the DNA parameters where different.
            // The groove_angle was negative, and the roll was going in the opposite direction
            if let Some(helix_parameters) = self.helix_parameters.as_mut() {
                helix_parameters.groove_angle *= -1.;
            } else {
                self.helix_parameters = Some(Default::default());
            }
            mutate_all_helices(self, |h| h.roll *= -1.);
            self.ensnano_version = ensnano_version();
        }
    }

    /// Return a list of tuples (n1, n2, M) where n1 and n2 are nucleotides that are not on the same
    /// helix and whose distance is at most `epsilon` and M is the middle of the segment between
    /// the two positions of n1 and n2.
    pub fn get_pairs_of_close_nucleotides(&self, epsilon: f32) -> Vec<(Nucl, Nucl, Vec3)> {
        let mut ret = Vec::new();
        let mut nucls = Vec::new();
        let helix_parameters = self.helix_parameters.unwrap_or_default();
        for s in self.strands.values() {
            for d in &s.domains {
                if let Domain::HelixDomain(interval) = d {
                    for i in interval.iter() {
                        let nucl = Nucl {
                            helix: interval.helix,
                            forward: interval.forward,
                            position: i,
                        };
                        if let Some(h) = self.helices.get(&interval.helix) {
                            let space_position =
                                h.space_pos(&helix_parameters, nucl.position, nucl.forward);
                            nucls.push((nucl, space_position));
                        }
                    }
                }
            }
        }
        for (n_id, n1) in nucls.iter().enumerate() {
            for n2 in nucls.iter().skip(n_id + 1) {
                if n1.0.helix != n2.0.helix && (n1.1 - n2.1).mag() < epsilon {
                    ret.push((n1.0, n2.0, ((n1.1 + n2.1) / 2.)));
                }
            }
        }
        ret
    }

    pub fn add_camera(
        &mut self,
        position: Vec3,
        orientation: Rotor3,
        pivot_position: Option<Vec3>,
    ) {
        let camera_id = self
            .cameras
            .keys()
            .max()
            .map_or(CameraId(1), |id| CameraId(id.0 + 1));
        let new_camera = Camera {
            position,
            orientation,
            name: format!("Camera {}", camera_id.0),
            id: camera_id,
            pivot_position,
        };
        self.cameras.insert(camera_id, new_camera);
    }

    pub fn rm_camera(&mut self, camera_id: CameraId) -> bool {
        if self.cameras.remove(&camera_id).is_some() {
            if self.favorite_camera == Some(camera_id) {
                self.favorite_camera = self.cameras.keys().min().copied();
            }
            true
        } else {
            false
        }
    }

    pub fn get_camera_mut(&mut self, camera_id: CameraId) -> Option<&mut Camera> {
        self.cameras.get_mut(&camera_id)
    }

    pub fn get_camera(&self, camera_id: CameraId) -> Option<&Camera> {
        self.cameras.get(&camera_id)
    }

    pub fn get_favorite_camera(&self) -> Option<&Camera> {
        self.favorite_camera
            .as_ref()
            .and_then(|id| self.cameras.get(id))
            .or(self.saved_camera.as_ref())
    }

    pub fn get_favorite_camera_id(&self) -> Option<CameraId> {
        self.favorite_camera
    }

    pub fn set_favorite_camera(&mut self, camera_id: CameraId) -> bool {
        if self.cameras.contains_key(&camera_id) {
            if self.favorite_camera != Some(camera_id) {
                self.favorite_camera = Some(camera_id);
            } else {
                self.favorite_camera = None;
            }
            true
        } else {
            false
        }
    }

    pub fn get_cameras(&self) -> impl Iterator<Item = (&CameraId, &Camera)> {
        self.cameras.iter()
    }

    pub fn prepare_for_save(&mut self, saving_information: SavingInformation) {
        self.saved_camera = saving_information.camera;
    }

    pub fn get_nucl_position(&self, nucl: Nucl) -> Option<Vec3> {
        let helix = self.helices.get(&nucl.helix)?;
        Some(helix.space_pos(
            &self.helix_parameters.unwrap_or_default(),
            nucl.position,
            nucl.forward,
        ))
    }

    pub fn get_updated_grid_data(&mut self) -> &GridData {
        self.update_curve_bounds();
        for _ in 0..3 {
            let need_update = if let Some(data) = self.instantiated_grid_data.as_ref() {
                !data.is_up_to_date(self)
            } else {
                true
            };
            if need_update {
                let updated_data = GridData::new_by_updating_design(self);
                self.instantiated_grid_data = Some(updated_data);
            }
            if !self.update_curve_bounds() {
                // we are done
                break;
            }
        }
        self.get_up_to_date().grid_data
    }

    fn update_curve_bounds(&mut self) -> bool {
        log::debug!("updating curve bounds");
        let mut new_helices = self.helices.clone();
        let mut new_helices_mut = new_helices.make_mut();
        let mut replace = false;
        let helix_parameters = self.helix_parameters.unwrap_or_default();
        for (h_id, h) in self.helices.iter() {
            log::debug!("Helix {h_id}");
            if let Some((n_min, n_max)) =
                self.strands.get_used_bounds_for_helix(*h_id, &self.helices)
            {
                log::debug!("bounds {n_min} {n_max}");
                if let Some(curve) = h.instantiated_curve.as_ref() {
                    if let Some(t_min) = curve
                        .curve
                        .left_extension_to_have_nucl(n_min, &helix_parameters)
                    {
                        log::debug!("t_min {t_min}");
                        if let Some(h_mut) = new_helices_mut.get_mut(h_id) {
                            replace |= h_mut
                                .curve
                                .as_mut()
                                .is_some_and(|c| Arc::make_mut(c).set_t_min(t_min));
                        }
                    }
                    if let Some(t_max) = curve
                        .curve
                        .right_extension_to_have_nucl(n_max, &helix_parameters)
                    {
                        log::debug!("t_max {t_max}");
                        if let Some(h_mut) = new_helices_mut.get_mut(h_id) {
                            replace |= h_mut
                                .curve
                                .as_mut()
                                .is_some_and(|c| Arc::make_mut(c).set_t_max(t_max));
                        }
                    }
                }
            }
        }
        drop(new_helices_mut);
        if replace {
            self.helices = new_helices;
            true
        } else {
            false
        }
    }

    pub fn mut_strand_and_data(&mut self) -> MutStrandAndData<'_> {
        self.get_updated_grid_data();
        MutStrandAndData {
            strands: &mut self.strands,
            grid_data: self.instantiated_grid_data.as_ref().unwrap(),
            helices: &self.helices,
            helix_parameters: self.helix_parameters.unwrap_or_default(),
        }
    }

    pub fn set_helices(&mut self, helices: BTreeMap<usize, Arc<Helix>>) {
        self.helices = Helices(Arc::new(helices));
    }
}

pub trait MainDesignReaderExt {
    fn get_grid_position_of_helix(&self, h_id: usize) -> Option<HelixGridPosition>;
    fn get_xover_id(&self, pair: &(Nucl, Nucl)) -> Option<usize>;
    fn get_xover_with_id(&self, id: usize) -> Option<(Nucl, Nucl)>;
    fn get_strand_with_id(&self, id: usize) -> Option<&Strand>;
    fn get_helix_grid(&self, h_id: usize) -> Option<GridId>;
    fn get_domain_ends(&self, s_id: usize) -> Option<Vec<Nucl>>;
}

pub trait AdditionalStructure: Send + Sync {
    fn frame(&self) -> Similarity3;
    fn position(&self) -> Vec<Vec3>;
    fn right(&self) -> Vec<(usize, usize)>;
    fn next(&self) -> Vec<(usize, usize)>;
    fn nt_paths(&self) -> Option<Vec<Vec<Vec3>>>;
    fn current_length(&self) -> Option<usize>;
    fn number_of_sections(&self) -> usize;
}

/// An immutable reference to a design whose helices paths and grid data are guaranteed to be up-to
/// date.
pub struct UpToDateDesign<'a> {
    pub design: &'a Design,
    pub grid_data: &'a GridData,
    pub paths_data: &'a BezierPathData,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct CameraId(u64);

/// A saved camera position. This can be use to register interesting point of views of the design.
#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Camera {
    pub position: Vec3,
    pub orientation: Rotor3,
    pub name: String,
    pub id: CameraId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[serde_as(deserialize_as = "DefaultOnError")]
    pub pivot_position: Option<Vec3>,
}

pub fn ensnano_version() -> String {
    std::env!("CARGO_PKG_VERSION").to_owned()
}

fn groups_is_empty<K, V>(groups: &Arc<BTreeMap<K, V>>) -> bool {
    groups.as_ref().is_empty()
}

impl Default for Design {
    fn default() -> Self {
        Self::new()
    }
}

/// A structure that wraps a mutable reference to the design's strands along with a read only
/// access to the grid and helices.
pub struct MutStrandAndData<'a> {
    pub strands: &'a mut Strands,
    pub grid_data: &'a GridData,
    pub helices: &'a Helices,
    pub helix_parameters: HelixParameters,
}

pub struct SavingInformation {
    pub camera: Option<Camera>,
}

/// Apply a mutating function to the value wrapped in an `Arc<Helix>`. This will make `helix_ptr`
/// point to a new helix on which the update has been applied.
pub fn mutate_in_arc<F, Obj: Clone>(obj_ptr: &mut Arc<Obj>, mut mutation: F)
where
    F: FnMut(&mut Obj),
{
    let mut new_obj = Obj::clone(obj_ptr);
    mutation(&mut new_obj);
    *obj_ptr = Arc::new(new_obj);
}

/// Apply a mutating function to all the helices of a design.
pub fn mutate_all_helices<F>(design: &mut Design, mutation: F)
where
    F: FnMut(&mut Helix) + Clone,
{
    let mut new_helices_map = BTreeMap::clone(design.helices.0.as_ref());
    for h in new_helices_map.values_mut() {
        mutate_in_arc(h, mutation.clone());
    }
    design.helices = Helices(Arc::new(new_helices_map));
}

pub fn mutate_one_helix<F>(design: &mut Design, h_id: usize, mutation: F) -> Option<()>
where
    F: FnMut(&mut Helix) + Clone,
{
    let mut new_helices_map = BTreeMap::clone(design.helices.0.as_ref());
    new_helices_map
        .get_mut(&h_id)
        .map(|h| mutate_in_arc(h, mutation))?;
    design.helices = Helices(Arc::new(new_helices_map));
    Some(())
}

/*
ENSnano, a 3d graphical application for DNA nanostructures.
    Copyright (C) 2021  Nicolas Levy <nicolaspierrelevy@gmail.com> and Nicolas Schabanel <nicolas.schabanel@ens-lyon.fr>

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with this program.  If not, see <https://www.gnu.org/licenses/>.
*/
use super::super::maths_3d::{Basis3D, UnalignedBoundaries};
use super::super::view::{
    ConeInstance, Ellipsoid, Instanciable, RawDnaInstance, Sheet2D, SlicedTubeInstance,
    SphereInstance, TubeInstance, TubeLidInstance,
};
use super::super::GridInstance;
use super::{ultraviolet, LetterInstance, SceneElement};
use crate::rotor_utils::SafeRotor;
use crate::sausage_rosary::SausageRosary;
use ensnano_design::grid::{GridId, GridObject, GridPosition};
use ensnano_design::{grid::HelixGridPosition, Nucl};
use ensnano_design::{
    AdditionalStructure, BezierPathId, BezierPlaneDescriptor, BezierPlaneId, BezierVertex,
    Collection, CubicBezierConstructor, CurveDescriptor, External3DObjects, HelixParameters,
    InstanciatedPath,
};
pub use ensnano_design::{SurfaceInfo, SurfacePoint};
use ensnano_interactor::consts::*;
use ensnano_interactor::{
    graphics::{LoopoutBond, LoopoutNucl},
    phantom_helix_encoder_bond, phantom_helix_encoder_nucl, BezierControlPoint, ObjectType,
    PhantomElement, Referential, PHANTOM_RANGE,
};
use ensnano_utils::instance::Instance;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::f32::consts::PI;
use std::iter::Zip;
use std::rc::Rc;
use std::sync::Arc;
use ultraviolet::{Mat4, Rotor3, Vec2, Vec3, Vec4};

mod bezier_paths;

use crate::SceneElement::DesignElement;

/// An object that handles the 3d graphical representation of a `Design`
pub struct Design3D<R: DesignReader> {
    pub design_reader: R,
    id: u32,
    symbol_map: HashMap<char, usize>,
    /// indicate if all helices must be on axis (helices_off_axis = false)
    pub all_helices_on_axis: bool,
}

impl<R: DesignReader> Design3D<R> {
    pub fn new(design_reader: R, id: u32) -> Self {
        let mut symbol_map = HashMap::new();
        for (s_id, s) in PRINTABLE_CHARS.iter().enumerate() {
            symbol_map.insert(*s, s_id);
        }
        Self {
            design_reader,
            id,
            symbol_map,
            all_helices_on_axis: false,
        }
    }

    /// Convert a list of ids into a list of instances
    pub fn id_to_raw_instances(&self, ids: Vec<u32>) -> Vec<RawDnaInstance> {
        let mut ret = Vec::new();
        for id in ids.iter() {
            if let Some(instances) = self.make_raw_instances(*id) {
                ret.extend(instances)
            }
        }
        ret
    }

    /// Return the list of raw sphere instances to be displayed to represent the design
    pub fn get_spheres_raw(&self, show_insertion_representents: bool) -> Rc<Vec<RawDnaInstance>> {
        let visible_nucls_ids = self.design_reader.get_all_visible_nucl_ids();
        let mut ret = self.id_to_raw_instances(visible_nucls_ids);
        if !show_insertion_representents {
            for loopout_nucl in self.design_reader.get_all_loopout_nucl() {
                ret.push(
                    SphereInstance {
                        position: loopout_nucl.position,
                        color: Instance::unclear_color_from_u32(loopout_nucl.color),
                        id: loopout_nucl.repr_bond_identifier,
                        radius: SPHERE_RADIUS,
                    }
                    .to_raw_instance(),
                );
            }
        }
        if let Some(additional_structure) = self.design_reader.get_additional_structure() {
            let transformation = additional_structure.frame();
            for p in additional_structure.position() {
                ret.push(
                    SphereInstance {
                        position: transformation.transform_vec(p),
                        color: Instance::unclear_color_from_u32(SURFACE_PIVOT_SPHERE_COLOR),
                        id: u32::MAX,
                        radius: SPHERE_RADIUS,
                    }
                    .to_raw_instance(),
                );
            }
            if let Some(path) = additional_structure.nt_path() {
                for p in path {
                    ret.push(
                        SphereInstance {
                            position: transformation.transform_vec(p),
                            color: Instance::unclear_color_from_u32(PIVOT_SPHERE_COLOR),
                            id: u32::MAX,
                            radius: SPHERE_RADIUS,
                        }
                        .to_raw_instance(),
                    );
                }
            }
        }
        Rc::new(ret)
    }

    pub fn get_pasted_strand(&self) -> (Vec<RawDnaInstance>, Vec<RawDnaInstance>) {
        let mut spheres = Vec::new();
        let mut tubes = Vec::new();
        let positions = self.design_reader.get_pasted_position();
        for (positions, pastable) in positions {
            let mut previous_position = None;
            let color = if pastable {
                CANDIDATE_COLOR
            } else {
                SELECTED_COLOR
            };
            let color_vec4 = Instance::color_from_au32(color);
            for position in positions.iter() {
                let sphere = SphereInstance {
                    position: *position,
                    color: color_vec4,
                    id: 0,
                    radius: SPHERE_RADIUS,
                }
                .to_raw_instance();
                spheres.push(sphere);
                if let Some(prev) = previous_position {
                    let tube = create_dna_bond(prev, *position, color, 0, true);
                    tubes.push(tube.to_raw_instance());
                }
                previous_position = Some(*position);
            }
        }
        (spheres, tubes)
    }

    pub fn get_letter_instances(
        &self,
        show_insertion_representents: bool,
    ) -> Vec<Vec<LetterInstance>> {
        let ids = self.design_reader.get_all_nucl_ids();
        let mut vecs = vec![Vec::new(); NB_PRINTABLE_CHARS];
        for id in ids {
            let pos = self.design_reader.get_symbol_position(id);
            let symbol = self.design_reader.get_symbol(id);
            if let Some((pos, symbol)) = pos.zip(symbol) {
                if let Some(id) = self.symbol_map.get(&symbol) {
                    let instance = LetterInstance {
                        position: pos,
                        color: ultraviolet::Vec4::new(0., 0., 0., 1.),
                        design_id: self.id,
                        scale: 1.,
                        shift: Vec3::zero(),
                    };
                    vecs[*id].push(instance);
                }
            }
        }
        if !show_insertion_representents {
            for loopout_nucl in self.design_reader.get_all_loopout_nucl() {
                if let Some(symbol) = loopout_nucl.basis {
                    let pos = loopout_nucl.position;
                    if let Some(id) = self.symbol_map.get(&symbol) {
                        let instance = LetterInstance {
                            position: pos,
                            color: ultraviolet::Vec4::new(0., 0., 0., 1.),
                            design_id: self.id,
                            scale: 1.,
                            shift: Vec3::zero(),
                        };
                        vecs[*id].push(instance);
                    }
                }
            }
        }
        vecs
    }

    pub fn get_cones_raw(&self, show_insertion_representents: bool) -> Vec<RawDnaInstance> {
        let mut ids = self.design_reader.get_all_visible_bond_ids();
        if !show_insertion_representents {
            ids.retain(|id| self.design_reader.get_insertion_length(*id) == 0);
        }
        let filter = |_n: &Nucl| true;
        let vec: Vec<_> = ids
            .iter()
            .flat_map(|id| self.make_cone_from_bond(*id, &filter))
            .collect();
        vec
    }

    /// Return the list of tube instances to be displayed to represent the design
    pub fn get_tubes_raw(&self, show_insertion_representents: bool) -> Rc<Vec<RawDnaInstance>> {
        let mut visible_bonds_ids = self.design_reader.get_all_visible_bond_ids();
        if !show_insertion_representents {
            visible_bonds_ids.retain(|id| self.design_reader.get_insertion_length(*id) == 0);
        }
        // let expected_length = if self.thick_helices {
        //     // normal representation of an helix
        //     self.design_reader.get_expected_bond_length()
        // } else {
        //     // axis representation of an helix
        //     HelixParameters::INTER_CENTER_GAP
        // };
        let mut ret: Vec<_> = self
            .id_to_raw_instances(visible_bonds_ids)
            // .into_iter()
            // .map(|x| x.with_expected_length(expected_length)) // inutile désormais
            // .collect()
            ;
        if !show_insertion_representents {
            for loopout_bond in self.design_reader.get_all_loopout_bonds() {
                ret.push(
                    create_dna_bond(
                        loopout_bond.position_prime5,
                        loopout_bond.position_prime3,
                        loopout_bond.color,
                        loopout_bond.repr_bond_identifier,
                        false,
                    )
                    .to_raw_instance(), // .with_expected_length(expected_length),
                )
            }
        }

        if let Some(additional_structure) = self.design_reader.get_additional_structure() {
            let transformation = additional_structure.frame();
            let positions = additional_structure.position();
            for (me, next) in additional_structure.right().into_iter() {
                let pos_left = transformation.transform_vec(positions[me]);
                let pos_right = transformation.transform_vec(positions[next]);
                ret.push(
                    create_dna_bond(pos_left, pos_right, REGULAR_H_BOND_COLOR, u32::MAX, false)
                        .to_raw_instance(),
                )
            }
            for (me, other) in additional_structure.next().into_iter() {
                let pos_left = transformation.transform_vec(positions[me]);
                let pos_right = transformation.transform_vec(positions[other]);
                ret.push(
                    create_dna_bond(pos_left, pos_right, COLOR_GUANINE, u32::MAX, false)
                        .to_raw_instance(),
                )
            }
        }

        //     // extra tubes to test sliced_tubes
        //     let n = 50;
        //     let b = 7.*PI/(n as f32);
        //     let r = 3.0f32;

        //     use rand::Rng;

        //     let mut rng = rand::thread_rng();

        //     let points = (0..n+1).map(|i|
        //         match i {
        //             0 =>  Vec3::zero(),
        //             50 =>  Vec3::zero(),
        //             _ => {
        //                 let a = PI * 2. * rng.gen::<f32>(); /// i as f32 * b;
        //                 Vec3::new(
        //                 0.5,
        //                 r*(a.cos()),
        //                 r*(a.sin()))
        //             },
        // }).collect::<Vec<Vec3>>();

        //     let mut point = Vec3::zero();
        //     for ((prev, p) , next) in points.iter().cycle().skip(n).zip(&points).zip(points.iter().cycle().skip(1)) {
        //         // println!("Prev: {:?}", *prev);
        //         // println!("Curr: {:?}", *p);
        //         // println!("Next: {:?}", *next);
        //         let position = point + *p / 2.;
        //         let q = p.normalized();
        //         let rotor = Rotor3::from_rotation_between(Vec3::unit_x(), q);

        //         let rotor = Rotor3::safe_from_rotation_from_unit_x_to(q);
        //         let model =
        //         Mat4::from_translation(position) * rotor.into_matrix().into_homogeneous(); // translation à position et rotation dans la bonne position u_x -> axe du tube
        //         let rotor2 = Rotor3::safe_from_rotation_to_unit_x_from(q);

        //         ret.push(SlicedTubeInstance {
        //             position: position,
        //             rotor: rotor,
        //             color: Vec4::new(1.,0.,0.,1.), //RGBA
        //             id: 100_000,
        //             radius: 0.3,
        //             length: p.mag(),
        //             prev: prev.rotated_by(rotor2),
        //             next: next.rotated_by(rotor2),
        //         }.to_raw_instance());
        //         point += *p;
        //     }
        Rc::new(ret)
    }

    pub fn get_model_matrix(&self) -> Mat4 {
        self.design_reader.get_model_matrix()
    }

    pub fn get_bezier_grid_used_by_helix(&self, h_id: usize) -> Vec<GridId> {
        self.design_reader.get_bezier_grid_used_by_helix(h_id)
    }

    /// Convert return an instance representing the object with identifier `id` and custom
    /// color and radius.
    pub(super) fn make_instance(
        &self,
        id: u32,
        color: u32,
        mut radius: f32,
        expand_with: Option<ExpandWith>,
    ) -> Vec<RawDnaInstance> {
        let kind = self.get_object_type(id);

        let mut ret = Vec::new();
        if expand_with.is_none()
            || self.design_reader.get_insertion_length(id) == 0
            || matches!(kind, Some(ObjectType::Nucleotide(_)))
        {
            let instanciables = match kind {
                Some(ObjectType::Bond(id1, id2)) => {
                    let pos1 = self
                        .get_graphic_element_position(&SceneElement::DesignElement(self.id, id1))
                        .unwrap_or(f32::NAN * Vec3::unit_x());
                    let pos2 = self
                        .get_graphic_element_position(&SceneElement::DesignElement(self.id, id2))
                        .unwrap_or(f32::NAN * Vec3::unit_x());
                    let id = id | self.id << 24;
                    vec![create_dna_bond(pos1, pos2, color, id, true)
                        .with_radius(radius)
                        .to_raw_instance()]
                }
                Some(ObjectType::HelixCylinder(id1, id2)) => {
                    let pos1 = self
                        .get_graphic_element_axis_position(&SceneElement::DesignElement(
                            self.id, id1,
                        ))
                        .unwrap_or(f32::NAN * Vec3::unit_x());
                    let pos2 = self
                        .get_graphic_element_axis_position(&SceneElement::DesignElement(
                            self.id, id2,
                        ))
                        .unwrap_or(f32::NAN * Vec3::unit_x());
                    let id = id | self.id << 24;
                    let (lid1, tube, lid2) =
                        create_helix_cylinder(pos1, pos2, radius, color, id, true);
                    // println!("Lid1: {:?}\nTube: {:?}\nLid2: {:?}", lid1.position, tube.position, lid2.position);
                    vec![
                        lid1.to_raw_instance(),
                        tube.to_raw_instance(),
                        lid2.to_raw_instance(),
                    ]
                }
                Some(ObjectType::Nucleotide(id)) => {
                    let position = self
                        .get_graphic_element_position(&SceneElement::DesignElement(self.id, id))
                        .unwrap_or(f32::NAN * Vec3::unit_x());
                    let id = id | self.id << 24;
                    let color = Instance::color_from_au32(color);
                    // let small = self.design_reader.has_small_spheres_nucl_id(id);
                    // if radius > 1.01 && small {
                    //     radius *= 2.5;
                    // }
                    // radius = if small { radius / 3.5 } else { radius };
                    vec![SphereInstance {
                        position,
                        radius,
                        color,
                        id,
                    }
                    .to_raw_instance()]
                }
                _ => vec![],
            };
            ret.extend(instanciables.iter());
        }
        if let Some(ExpandWith::Tubes) = expand_with {
            for loopout_bond in self
                .design_reader
                .get_all_loopout_bonds()
                .iter()
                .filter(|lb| lb.repr_bond_identifier == id)
            {
                ret.push(
                    create_dna_bond(
                        loopout_bond.position_prime5,
                        loopout_bond.position_prime3,
                        color,
                        loopout_bond.repr_bond_identifier,
                        true,
                    )
                    .with_radius(radius)
                    .to_raw_instance(),
                )
            }
        }
        if let Some(ExpandWith::Spheres) = expand_with {
            for loopout_nucl in self
                .design_reader
                .get_all_loopout_nucl()
                .iter()
                .filter(|ln| ln.repr_bond_identifier == id)
            {
                ret.push(
                    SphereInstance {
                        position: loopout_nucl.position,
                        color: Instance::color_from_au32(color),
                        id: loopout_nucl.repr_bond_identifier,
                        radius,
                    }
                    .to_raw_instance(),
                );
            }
        }
        ret
    }

    fn make_checked_xover_instance(&self, id: u32, checked: bool) -> Option<RawDnaInstance> {
        let referential = Referential::Model;
        if let Some(ObjectType::Bond(n1, n2)) = self.get_object_type(id) {
            let pos1 = self.get_design_element_position(n1, referential)?;
            let pos2 = self.get_design_element_position(n2, referential)?;
            Some(create_check_bond(pos1, pos2, checked))
        } else {
            None
        }
    }

    pub fn get_all_checked_xover_instance(&self, checked: bool) -> Vec<RawDnaInstance> {
        self.design_reader
            .get_checked_xovers_ids(checked)
            .into_iter()
            .filter_map(|id| self.make_checked_xover_instance(id, checked))
            .collect()
    }

    /// Return (h bonds instances, ellipoids instances)
    pub(super) fn get_all_h_bonds(&self) -> HBondsInstances {
        let mut full_h_bonds = Vec::new();
        let mut partial_h_bonds = Vec::new();
        let mut ellipsoids = Vec::new();
        for hbond in self.design_reader.get_all_h_bonds() {
            let forward_bond = create_dna_bond(
                hbond.forward.backbone,
                hbond.forward.center_of_mass,
                hbond.forward.backbone_color,
                0,
                false,
            );
            let backward_bond = create_dna_bond(
                hbond.backward.backbone,
                hbond.backward.center_of_mass,
                hbond.backward.backbone_color,
                0,
                false,
            );
            let full_bond = create_dna_bond(
                hbond.backward.backbone,
                hbond.forward.backbone,
                REGULAR_H_BOND_COLOR,
                0,
                false,
            );
            let forward_ellipsoid = Ellipsoid {
                orientation: forward_bond.rotor,
                scale: BASIS_SCALE,
                sphere: SphereInstance {
                    position: hbond.forward.center_of_mass,
                    color: Instance::unclear_color_from_u32(basis_color(
                        hbond.forward.base.unwrap_or('?'),
                    )),
                    radius: SPHERE_RADIUS,
                    id: 0,
                },
            };
            let backward_ellipsoid = Ellipsoid {
                orientation: backward_bond.rotor,
                scale: BASIS_SCALE,
                sphere: SphereInstance {
                    position: hbond.backward.center_of_mass,
                    color: Instance::unclear_color_from_u32(basis_color(
                        hbond.backward.base.unwrap_or('?'),
                    )),
                    radius: SPHERE_RADIUS,
                    id: 0,
                },
            };
            partial_h_bonds.push(forward_bond.to_raw_instance());
            partial_h_bonds.push(backward_bond.to_raw_instance());
            ellipsoids.push(backward_ellipsoid.to_raw_instance());
            ellipsoids.push(forward_ellipsoid.to_raw_instance());
            full_h_bonds.push(full_bond.to_raw_instance());
        }
        HBondsInstances {
            partial_h_bonds,
            full_h_bonds,
            ellipsoids,
        }
    }

    fn make_cone_from_bond(
        &self,
        id: u32,
        filter: &dyn Fn(&Nucl) -> bool,
    ) -> Option<RawDnaInstance> {
        let kind = self.get_object_type(id)?;

        match kind {
            ObjectType::Bond(id1, id2) | ObjectType::SlicedBond(_, id1, id2, _)
                if self.get_with_cones(id).unwrap_or(true) =>
            {
                let pos1 =
                    self.get_graphic_element_position(&SceneElement::DesignElement(self.id, id1))?;
                let pos2 =
                    self.get_graphic_element_position(&SceneElement::DesignElement(self.id, id2))?;
                let radius = self.get_radius(id).unwrap_or(BOND_RADIUS);
                let cone_radius = (1.5 * radius).max(0.75 * SPHERE_RADIUS);
                self.design_reader.get_nucl_with_id(id1).filter(filter)?;

                let color = self.get_color(id).unwrap_or(0);
                let cone = create_prime3_cone(pos1, pos2, color, cone_radius);
                Some(cone)
            }
            _ => None,
        }
    }

    /// Auxilary function that computes the length-adjusted color of a bond
    /// return None if color does not need to be adjusted
    pub fn length_adjusted_color_and_radius_for_bond(
        &self,
        id1: u32,
        id2: u32,
    ) -> Option<(u32, f32)> {
        let real_pos1 = self
            .get_element_position(
                &SceneElement::DesignElement(self.id, id1),
                Referential::World,
            )
            .unwrap_or(f32::NAN * Vec3::unit_x());
        let real_pos2 = self
            .get_element_position(
                &SceneElement::DesignElement(self.id, id2),
                Referential::World,
            )
            .unwrap_or(f32::NAN * Vec3::unit_x());
        let length = (real_pos2 - real_pos1).mag();
        let expected_length = self.design_reader.get_expected_bond_length(); // SHOULD DEPEND ON THE HELIX_MODEL OF THE STRAND -> SHOULD BE MODIFIED LATER
        let critical_length_low = expected_length / 0.7; // TO BE MADE A CONSTANT IN CONST.RS
        let critical_length_high = 2.0 * critical_length_low; // TO BE MADE A CONSTANT IN CONST.RS
        if length < critical_length_low {
            return None;
        }
        if length < critical_length_high {
            let x = (length - critical_length_low) / (critical_length_high - critical_length_low);
            let grey = 0.25 - 0.25 * x;
            return Some((Instance::grey_u32_color_from_f32(grey), 1.0));
        }
        return Some((0x_00_00_00, 1.0));
    }

    /// Convert return an instance representing the object with identifier `id`
    pub fn make_raw_instances(&self, id: u32) -> Option<Vec<RawDnaInstance>> {
        let kind = self.get_object_type(id)?;
        let raw_instances =
            match kind {
                ObjectType::Bond(id1, id2) => {
                    let pos1 = self
                        .get_graphic_element_position(&SceneElement::DesignElement(self.id, id1))?;
                    let pos2 = self
                        .get_graphic_element_position(&SceneElement::DesignElement(self.id, id2))?;
                    let color = self.get_color(id).unwrap_or(0x00_00_00);
                    let id = id | self.id << 24;
                    // Adjust the color and rafius of the bond according to the REAL length of the bond
                    let xover_coloring = self.get_xover_coloring(id).unwrap_or(true);
                    let (color, radius_scale) = (if xover_coloring {
                        self.length_adjusted_color_and_radius_for_bond(id1, id2)
                    } else {
                        None
                    })
                    .unwrap_or((color, 1.0));
                    let radius = radius_scale * self.get_radius(id).unwrap_or(BOND_RADIUS);
                    let tube = create_dna_bond(pos1, pos2, color, id, false).with_radius(radius);
                    vec![tube.to_raw_instance()]
                }
                ObjectType::SlicedBond(prev_nucl_id, nucl1_id, nucl2_id, next_nucl_id) => {
                    let pos1 = self.get_graphic_element_position(&SceneElement::DesignElement(
                        self.id, nucl1_id,
                    ))?;
                    let pos2 = self.get_graphic_element_position(&SceneElement::DesignElement(
                        self.id, nucl2_id,
                    ))?;

                    let prev = if prev_nucl_id != nucl1_id {
                        let pos0 = self.get_graphic_element_position(
                            &SceneElement::DesignElement(self.id, prev_nucl_id),
                        )?;
                        pos1 - pos0
                    } else {
                        Vec3::zero()
                    };
                    let next = if next_nucl_id != nucl2_id {
                        let pos3 = self.get_graphic_element_position(
                            &SceneElement::DesignElement(self.id, next_nucl_id),
                        )?;
                        pos3 - pos2
                    } else {
                        Vec3::zero()
                    };
                    let position = (pos1 + pos2) / 2.;
                    let current = pos2 - pos1;
                    let normalized = current.normalized();
                    let rotor = Rotor3::safe_from_rotation_from_unit_x_to(normalized);
                    let rotor_inv = Rotor3::safe_from_rotation_to_unit_x_from(normalized);

                    let color = self.get_color(id).unwrap_or(0x00_00_00);
                    let id = id | self.id << 24;
                    // Adjust the color and rafius of the bond according to the REAL length of the bond
                    let xover_coloring = self.get_xover_coloring(id).unwrap_or(true);
                    let (color, radius_scale) = (if xover_coloring {
                        self.length_adjusted_color_and_radius_for_bond(nucl1_id, nucl2_id)
                    } else {
                        None
                    })
                    .unwrap_or((color, 1.0));
                    let radius = radius_scale * self.get_radius(id).unwrap_or(BOND_RADIUS);
                    let color = Instance::unclear_color_from_u32(color);
                    let sliced_tube = SlicedTubeInstance {
                        position,
                        rotor,
                        color,
                        id,
                        radius,
                        length: current.mag(),
                        prev: prev.rotated_by(rotor_inv),
                        next: next.rotated_by(rotor_inv),
                    };
                    let mut instances = vec![sliced_tube.to_raw_instance()];
                    if prev_nucl_id == nucl1_id {
                        // add lid
                        let lid_left = TubeLidInstance {
                            position: pos1,
                            color,
                            rotor: Rotor3::safe_from_rotation_from_unit_x_to(-normalized),
                            id,
                            radius,
                        };
                        instances.push(lid_left.to_raw_instance());
                    }
                    if next_nucl_id == nucl2_id {
                        // add lid
                        let lid_right = TubeLidInstance {
                            position: pos2,
                            color,
                            rotor,
                            id,
                            radius,
                        };
                        instances.push(lid_right.to_raw_instance());
                    }
                    instances
                }
                ObjectType::HelixCylinder(id1, id2) => {
                    let h_id = self.design_reader.get_id_of_helix_containing(id1).unwrap();

                    if self.design_reader.get_curve_descriptor(h_id).is_none() {
                        // straight helix

                        let pos1 = self.get_graphic_element_axis_position(
                            &SceneElement::DesignElement(self.id, id1),
                        )?;
                        let pos2 = self.get_graphic_element_axis_position(
                            &SceneElement::DesignElement(self.id, id2),
                        )?;
                        let color = self.get_color(id).unwrap_or(HELIX_CYLINDER_COLOR);
                        let color = Instance::add_alpha_to_clear_color_u32(color);
                        let id = id | self.id << 24;
                        // Adjust the color and rafius of the bond according to the REAL length of the bond
                        let radius = self.get_radius(id).unwrap_or(HELIX_CYLINDER_RADIUS);
                        let (lid1, tube, lid2) =
                            create_helix_cylinder(pos1, pos2, radius, color, id, true);
                        // println!("Lid1: {:?}\nTube: {:?}\nLid2: {:?}", lid1.position, tube.position, lid2.position);
                        vec![
                            lid1.to_raw_instance(),
                            tube.to_raw_instance(),
                            lid2.to_raw_instance(),
                        ]
                    } else {
                        // curved helix
                        let color = self.get_color(id).unwrap_or(HELIX_CYLINDER_COLOR);
                        let id = id | self.id << 24;
                        let radius = self.get_radius(id).unwrap_or(HELIX_CYLINDER_RADIUS);

                        let nucl1 = self.design_reader.get_nucl_with_id(id1).unwrap();
                        let nucl2 = self.design_reader.get_nucl_with_id(id2).unwrap();
                        // REQUIRE: nucl1 and nucl2 are on the forward strand and in increasing order
                        assert_eq!(
                            nucl1.helix, nucl2.helix,
                            "Helix cylinder accross different helices"
                        );
                        assert!(
                            nucl1.forward && nucl2.forward,
                            "Helix cylinder: nucleotides not along forward strand"
                        );
                        assert!(
                            nucl1.position <= nucl2.position,
                            "Helix cylinder: nucleotides in decreasing order"
                        );
                        let positions = (nucl1.position..=nucl2.position)
                            .map(|i| {
                                let nucl_id = self
                                    .get_identifier_nucl(&Nucl {
                                        helix: nucl1.helix,
                                        forward: true,
                                        position: i,
                                    })
                                    .unwrap();
                                self.get_element_axis_position(
                                    &DesignElement(self.id, nucl_id),
                                    Referential::World,
                                )
                                .unwrap()
                            })
                            .collect::<Vec<Vec3>>();
                        // println!("{:?}", positions);
                        let color = |_| -> u32 { color.clone() };
                        let (tubes, lids) = SausageRosary {
                            positions,
                            is_cyclic: false,
                        }
                        .to_raw_dna_instances(color, radius, id);
                        let mut rosary = tubes
                            .into_iter()
                            .map(|t| t.to_raw_instance())
                            .collect::<Vec<RawDnaInstance>>();
                        if let Some(lids) = lids {
                            rosary.push(lids.0.to_raw_instance());
                            rosary.push(lids.1.to_raw_instance());
                        }
                        return Some(rosary);
                    }
                }
                ObjectType::Nucleotide(id) => {
                    let position = self
                        .get_graphic_element_position(&SceneElement::DesignElement(self.id, id))?;
                    let color = self.get_color(id)?;
                    let color = Instance::unclear_color_from_u32(color);
                    let id = id | self.id << 24;
                    // let small = self.design_reader.has_small_spheres_nucl_id(id);
                    // let radius = if small {
                    //     BOND_RADIUS  // so that it's radius is the BOND_RADIUS as radius in shader is SPHERE_RADIUS
                    // } else {
                    //     SPHERE_RADIUS
                    // };
                    let radius = self.get_radius(id).unwrap_or(SPHERE_RADIUS);
                    let sphere = SphereInstance {
                        position,
                        color,
                        id,
                        radius,
                    };
                    vec![sphere.to_raw_instance()]
                }
            };
        Some(raw_instances)
    }

    pub fn get_suggested_spheres(&self) -> Vec<RawDnaInstance> {
        let suggestion = self.design_reader.get_suggestions();
        let mut ret = vec![];
        for (n1, n2) in suggestion {
            let nucl_1 = self.design_reader.get_position_of_nucl_on_helix(
                n1,
                Referential::Model,
                self.all_helices_on_axis,
            );
            let nucl_2 = self.design_reader.get_position_of_nucl_on_helix(
                n2,
                Referential::Model,
                self.all_helices_on_axis,
            );
            if let Some(position) = nucl_1 {
                let instance = SphereInstance {
                    color: Instance::color_from_au32(SUGGESTION_COLOR),
                    position,
                    id: 0,
                    radius: SELECT_SCALE_FACTOR * SPHERE_RADIUS,
                }
                .to_raw_instance();
                ret.push(instance);
            }
            if let Some(position) = nucl_2 {
                let instance = SphereInstance {
                    color: Instance::color_from_au32(SUGGESTION_COLOR),
                    position,
                    id: 0,
                    radius: SELECT_SCALE_FACTOR * SPHERE_RADIUS,
                }
                .to_raw_instance();
                ret.push(instance);
            }
        }
        ret
    }

    pub fn get_suggested_tubes(&self) -> Vec<RawDnaInstance> {
        let suggestion = self.design_reader.get_suggestions();
        let mut ret = vec![];
        for (n1, n2) in suggestion {
            let nucl_1 = self.design_reader.get_position_of_nucl_on_helix(
                n1,
                Referential::Model,
                self.all_helices_on_axis,
            );
            let nucl_2 = self.design_reader.get_position_of_nucl_on_helix(
                n2,
                Referential::Model,
                self.all_helices_on_axis,
            );
            if let Some((position1, position2)) = nucl_1.zip(nucl_2) {
                let instance = create_dna_bond(position1, position2, SUGGESTION_COLOR, 0, true)
                    .to_raw_instance();
                ret.push(instance);
            }
        }
        ret
    }

    /// Make a instance with the same postion and orientation as a phantom element.
    pub fn make_instance_phantom(
        &self,
        phantom_element: &PhantomElement,
        color: u32,
        radius: f32,
    ) -> Option<RawDnaInstance> {
        let nucl = Nucl {
            helix: phantom_element.helix_id as usize,
            position: phantom_element.position as isize,
            forward: phantom_element.forward,
        };
        let helix_id = phantom_element.helix_id;
        let i = phantom_element.position;
        let forward = phantom_element.forward;
        if phantom_element.bond {
            let nucl_1 = self.design_reader.get_position_of_nucl_on_helix(
                nucl,
                Referential::Model,
                false,
            )?;
            let nucl_2 = self.design_reader.get_position_of_nucl_on_helix(
                nucl.left(),
                Referential::Model,
                false,
            )?;
            let id = phantom_helix_encoder_bond(self.id, helix_id, i, forward);
            Some(create_dna_bond(nucl_1, nucl_2, color, id, true).to_raw_instance())
        } else {
            let nucl_coord = self.design_reader.get_position_of_nucl_on_helix(
                nucl,
                Referential::Model,
                false,
            )?;
            let id = phantom_helix_encoder_nucl(self.id, helix_id, i, forward);
            let instance = SphereInstance {
                color: Instance::color_from_au32(color),
                position: nucl_coord,
                id,
                radius,
            }
            .to_raw_instance();
            Some(instance)
        }
    }

    pub fn get_phantom_element_position(
        &self,
        phantom_element: &PhantomElement,
        referential: Referential,
        on_axis: bool,
    ) -> Option<Vec3> {
        let helix_id = phantom_element.helix_id;
        let i = phantom_element.position;
        let forward = phantom_element.forward;
        let nucl = Nucl {
            helix: helix_id as usize,
            position: i as isize,
            forward,
        };
        if phantom_element.bond {
            let nucl_1 =
                self.design_reader
                    .get_position_of_nucl_on_helix(nucl, referential, on_axis)?;
            let nucl_2 = self.design_reader.get_position_of_nucl_on_helix(
                nucl.left(),
                referential,
                on_axis,
            )?;
            Some((nucl_1 + nucl_2) / 2.)
        } else {
            self.design_reader
                .get_position_of_nucl_on_helix(nucl, referential, on_axis)
        }
    }

    pub fn make_phantom_helix_instances_raw(
        &self,
        helix_ids: &HashMap<u32, bool>,
    ) -> (Rc<Vec<RawDnaInstance>>, Rc<Vec<RawDnaInstance>>) {
        let mut spheres = Vec::new();
        let mut tubes = Vec::new();
        for (helix_id, short) in helix_ids.iter() {
            let range_phantom = if *short {
                PHANTOM_RANGE / 10
            } else {
                PHANTOM_RANGE
            } as isize;
            for forward in [false, true].iter() {
                let mut previous_nucl = None;
                let range = self
                    .design_reader
                    .get_curve_range(*helix_id as usize)
                    .unwrap_or(-range_phantom..=range_phantom);
                for i in range {
                    let i = i as i32;
                    let nucl_coord = self.design_reader.get_position_of_nucl_on_helix(
                        Nucl {
                            helix: *helix_id as usize,
                            position: i as isize,
                            forward: *forward,
                        },
                        Referential::Model,
                        false,
                    );
                    let color = 0xFFD0D0D0;
                    if nucl_coord.is_none() {
                        continue;
                    }
                    let nucl_coord = nucl_coord.unwrap();
                    let id = phantom_helix_encoder_nucl(self.id, *helix_id, i, *forward);
                    spheres.push(
                        SphereInstance {
                            position: nucl_coord,
                            color: Instance::color_from_au32(color),
                            id,
                            radius: 0.6 * SPHERE_RADIUS, // TO BE PUT IN CONST.RS
                        }
                        .to_raw_instance(),
                    );
                    if let Some(coord) = previous_nucl {
                        let id = phantom_helix_encoder_bond(self.id, *helix_id, i, *forward);
                        tubes.push(
                            create_dna_bond(nucl_coord, coord, color, id, true)
                                .with_radius(0.6 * BOND_RADIUS) // TO BE PUT IN CONST.RS
                                .to_raw_instance(),
                        );
                    }
                    previous_nucl = Some(nucl_coord);
                }
            }
        }
        (Rc::new(spheres), Rc::new(tubes))
    }

    fn get_object_type(&self, id: u32) -> Option<ObjectType> {
        self.design_reader.get_object_type(id)
    }

    pub fn get_bond(&self, id: u32) -> Option<(Nucl, Nucl)> {
        if let Some(ObjectType::Bond(n1, n2)) = self.get_object_type(id) {
            self.get_nucl(n1).zip(self.get_nucl(n2))
        } else {
            None
        }
    }

    pub fn get_element_position(
        &self,
        element: &SceneElement,
        referential: Referential,
    ) -> Option<Vec3> {
        match element {
            SceneElement::DesignElement(_, e_id) => {
                self.get_design_element_position(*e_id, referential)
            }
            SceneElement::PhantomElement(phantom) => {
                self.get_phantom_element_position(phantom, referential, false)
            }
            SceneElement::Grid(_, g_id) => self.design_reader.get_grid_position(*g_id),
            SceneElement::GridCircle(_, position) => {
                self.design_reader.get_grid_latice_position(*position)
            }
            SceneElement::BezierVertex { path_id, vertex_id } => {
                self.get_bezier_vertex_position(*path_id, *vertex_id)
            }
            _ => None,
        }
    }

    fn get_graphic_element_position(&self, element: &SceneElement) -> Option<Vec3> {
        if !self.all_helices_on_axis {
            self.get_element_graphic_position(element, Referential::World)
        } else {
            self.get_element_axis_position(element, Referential::World)
        }
    }

    fn get_graphic_element_axis_position(&self, element: &SceneElement) -> Option<Vec3> {
        self.get_element_axis_position(element, Referential::World)
    }

    pub fn get_element_axis_position(
        &self,
        element: &SceneElement,
        referential: Referential,
    ) -> Option<Vec3> {
        match element {
            SceneElement::DesignElement(_, e_id) => {
                self.get_design_element_axis_position(*e_id, referential)
            }
            SceneElement::PhantomElement(phantom) => {
                self.get_phantom_element_position(phantom, referential, true)
            }
            SceneElement::WidgetElement(_)
            | SceneElement::Grid(_, _)
            | SceneElement::BezierControl { .. }
            | SceneElement::BezierVertex { .. }
            | SceneElement::GridCircle(_, _)
            | SceneElement::PlaneCorner { .. }
            | SceneElement::BezierTangent { .. } => None,
        }
    }

    pub fn get_element_graphic_position(
        &self,
        element: &SceneElement,
        referential: Referential,
    ) -> Option<Vec3> {
        match element {
            SceneElement::DesignElement(_, e_id) => {
                self.get_design_element_graphic_position(*e_id, referential)
            }
            SceneElement::PhantomElement(phantom) => {
                self.get_phantom_element_position(phantom, referential, true)
            }
            SceneElement::WidgetElement(_)
            | SceneElement::Grid(_, _)
            | SceneElement::BezierControl { .. }
            | SceneElement::BezierVertex { .. }
            | SceneElement::GridCircle(_, _)
            | SceneElement::PlaneCorner { .. }
            | SceneElement::BezierTangent { .. } => None,
        }
    }

    pub fn get_design_element_position(&self, id: u32, referential: Referential) -> Option<Vec3> {
        self.design_reader.get_element_position(id, referential)
    }

    pub fn get_design_element_graphic_position(
        &self,
        id: u32,
        referential: Referential,
    ) -> Option<Vec3> {
        self.design_reader
            .get_element_graphic_position(id, referential)
    }

    pub fn get_design_element_axis_position(
        &self,
        id: u32,
        referential: Referential,
    ) -> Option<Vec3> {
        self.design_reader
            .get_element_axis_position(id, referential)
    }

    fn get_color(&self, id: u32) -> Option<u32> {
        self.design_reader.get_color(id)
    }

    fn get_radius(&self, id: u32) -> Option<f32> {
        self.design_reader.get_radius(id)
    }

    fn get_with_cones(&self, id: u32) -> Option<bool> {
        self.design_reader.get_with_cones(id)
    }

    fn get_xover_coloring(&self, id: u32) -> Option<bool> {
        self.design_reader.get_xover_coloring(id)
    }

    /// Return the middle point of `self` in the world coordinates
    pub fn middle_point(&self) -> Vec3 {
        let boundaries = self.boundaries();
        let middle = Vec3::new(
            (boundaries[0] + boundaries[1]) as f32 / 2.,
            (boundaries[2] + boundaries[3]) as f32 / 2.,
            (boundaries[4] + boundaries[5]) as f32 / 2.,
        );
        self.design_reader.get_model_matrix().transform_vec3(middle)
    }

    fn boundaries(&self) -> [f32; 6] {
        let mut min_x = std::f32::INFINITY;
        let mut min_y = std::f32::INFINITY;
        let mut min_z = std::f32::INFINITY;
        let mut max_x = std::f32::NEG_INFINITY;
        let mut max_y = std::f32::NEG_INFINITY;
        let mut max_z = std::f32::NEG_INFINITY;

        let ids = self.design_reader.get_all_nucl_ids();
        for id in ids {
            let coord: [f32; 3] = self
                .design_reader
                .get_element_position(id, Referential::World)
                .unwrap()
                .into();
            if coord[0] < min_x {
                min_x = coord[0];
            }
            if coord[0] > max_x {
                max_x = coord[0];
            }
            if coord[1] < min_y {
                min_y = coord[1];
            }
            if coord[1] > max_y {
                max_y = coord[1];
            }
            if coord[2] < min_z {
                min_z = coord[2];
            }
            if coord[2] > max_z {
                max_z = coord[2];
            }
        }
        for grid in self.get_grid().values() {
            let coords: [[f32; 3]; 2] = [
                grid.grid
                    .position_helix(grid.min_x as isize, grid.min_y as isize)
                    .into(),
                grid.grid
                    .position_helix(grid.max_x as isize, grid.max_y as isize)
                    .into(),
            ];
            for coord in coords.iter() {
                if coord[0] < min_x {
                    min_x = coord[0];
                }
                if coord[0] > max_x {
                    max_x = coord[0];
                }
                if coord[1] < min_y {
                    min_y = coord[1];
                }
                if coord[1] > max_y {
                    max_y = coord[1];
                }
                if coord[2] < min_z {
                    min_z = coord[2];
                }
                if coord[2] > max_z {
                    max_z = coord[2];
                }
            }
        }
        [min_x, max_x, min_y, max_y, min_z, max_z]
    }

    /// Return the list of corners of grid with no helices on them
    fn get_all_naked_grids_corners(&self) -> Vec<Vec3> {
        let mut ret = Vec::new();
        for (grid_id, grid) in self.get_grid().iter() {
            if self
                .design_reader
                .get_helices_on_grid(*grid_id)
                .map(|s| s.is_empty())
                .unwrap_or(false)
            {
                ret.push(
                    grid.grid
                        .position_helix(grid.min_x as isize, grid.min_y as isize),
                );
                ret.push(
                    grid.grid
                        .position_helix(grid.min_x as isize, grid.max_y as isize),
                );
                ret.push(
                    grid.grid
                        .position_helix(grid.max_x as isize, grid.min_y as isize),
                );
                ret.push(
                    grid.grid
                        .position_helix(grid.max_x as isize, grid.max_y as isize),
                );
            }
        }
        ret
    }

    fn get_all_points(&self) -> Vec<Vec3> {
        let ids = self.design_reader.get_all_nucl_ids();
        let mut ret: Vec<Vec3> = ids
            .iter()
            .filter_map(|id| {
                self.design_reader
                    .get_element_position(*id, Referential::World)
            })
            .collect();
        ret.extend(self.get_all_naked_grids_corners().into_iter());
        ret
    }

    fn boundaries_unaligned(&self, basis: Basis3D) -> UnalignedBoundaries {
        let mut ret = UnalignedBoundaries::from_basis(basis);
        for point in self.get_all_points().into_iter() {
            ret.add_point(point)
        }
        ret
    }

    pub fn get_fitting_camera_position(
        &self,
        basis: Basis3D,
        fovy: f32,
        ratio: f32,
    ) -> Option<Vec3> {
        let boundaries = self.boundaries_unaligned(basis);
        boundaries.fit_point(fovy, ratio)
    }

    pub fn get_all_elements(&self) -> HashSet<u32> {
        let mut ret = HashSet::new();
        for x in self.design_reader.get_all_nucl_ids().iter() {
            ret.insert(*x);
        }
        for x in self.design_reader.get_all_bond_ids().iter() {
            ret.insert(*x);
        }
        ret
    }

    pub fn get_strand(&self, element_id: u32) -> Option<usize> {
        self.design_reader.get_id_of_strand_containing(element_id)
    }

    pub fn get_helix(&self, element_id: u32) -> Option<usize> {
        self.design_reader.get_id_of_helix_containing(element_id)
    }

    pub fn get_strand_elements(&self, strand_id: u32) -> HashSet<u32> {
        self.design_reader
            .get_ids_of_elements_belonging_to_strand(strand_id as usize)
            .into_iter()
            .collect()
    }

    pub fn get_element_type(&self, e_id: u32) -> Option<ObjectType> {
        self.design_reader.get_object_type(e_id)
    }

    pub fn get_helix_elements(&self, helix_id: u32) -> HashSet<u32> {
        self.design_reader
            .get_ids_of_elements_belonging_to_helix(helix_id as usize)
            .into_iter()
            .collect()
    }

    pub fn get_helix_basis(&self, h_id: u32) -> Option<Rotor3> {
        self.design_reader.get_helix_basis(h_id)
    }

    pub fn get_identifier_nucl(&self, nucl: &Nucl) -> Option<u32> {
        self.design_reader.get_identifier_nucl(nucl)
    }

    pub fn get_identifier_bond(&self, n1: Nucl, n2: Nucl) -> Option<u32> {
        self.design_reader.get_identifier_bond(n1, n2)
    }

    pub fn get_element_identifier_from_xover_id(&self, xover_id: usize) -> Option<u32> {
        self.design_reader
            .get_xover_with_id(xover_id)
            .and_then(|(n1, n2)| self.design_reader.get_identifier_bond(n1, n2))
    }

    pub fn get_xover_id(&self, xover: &(Nucl, Nucl)) -> Option<usize> {
        self.design_reader.get_xover_id(xover)
    }

    pub fn get_xover_with_id(&self, xover_id: usize) -> Option<(Nucl, Nucl)> {
        self.design_reader.get_xover_with_id(xover_id)
    }

    pub fn can_start_builder(&self, element: &SceneElement) -> Option<Nucl> {
        match element {
            SceneElement::DesignElement(_, e_id) => self.can_start_builder_on_element(*e_id),
            SceneElement::PhantomElement(phantom_element) => {
                self.can_start_builder_on_phantom(phantom_element)
            }
            _ => None,
        }
    }

    fn can_start_builder_on_element(&self, e_id: u32) -> Option<Nucl> {
        let nucl = self.design_reader.get_nucl_with_id(e_id);
        nucl.filter(|n| self.design_reader.can_start_builder_at(n))
    }

    fn can_start_builder_on_phantom(&self, phantom_element: &PhantomElement) -> Option<Nucl> {
        let nucl = Nucl {
            helix: phantom_element.helix_id as usize,
            position: phantom_element.position as isize,
            forward: phantom_element.forward,
        };
        if self.design_reader.can_start_builder_at(&nucl) {
            Some(nucl)
        } else {
            None
        }
    }

    pub fn get_grid(&self) -> BTreeMap<GridId, GridInstance> {
        self.design_reader.get_grid_instances()
    }

    pub fn get_helices_grid(&self, g_id: GridId) -> Option<HashSet<usize>> {
        self.design_reader.get_helices_on_grid(g_id)
    }

    pub fn get_helices_grid_coord(&self, g_id: GridId) -> Vec<(isize, isize)> {
        self.design_reader
            .get_used_coordinates_on_grid(g_id)
            .unwrap_or_default()
    }

    pub fn get_helices_grid_key_coord(&self, g_id: GridId) -> Vec<((isize, isize), usize)> {
        self.design_reader
            .get_helices_grid_key_coord(g_id)
            .unwrap_or_default()
    }

    pub fn get_helix_grid(&self, position: GridPosition) -> Option<u32> {
        self.design_reader.get_helix_id_at_grid_coord(position)
    }

    pub fn get_grid_object(&self, position: GridPosition) -> Option<GridObject> {
        self.design_reader.get_grid_object(position)
    }

    pub fn get_persistent_phantom_helices(&self) -> HashSet<u32> {
        self.design_reader.get_persistent_phantom_helices_id()
    }

    pub fn get_grid_basis(&self, g_id: GridId) -> Option<Rotor3> {
        self.design_reader.get_grid_basis(g_id)
    }

    pub fn get_nucl(&self, e_id: u32) -> Option<Nucl> {
        self.design_reader.get_nucl_with_id(e_id)
    }

    pub fn get_nucl_relax(&self, e_id: u32) -> Option<Nucl> {
        self.design_reader.get_nucl_with_id_relaxed(e_id)
    }

    pub fn get_helix_grid_position(&self, h_id: u32) -> Option<HelixGridPosition> {
        self.design_reader.get_helix_grid_position(h_id)
    }

    pub fn get_nucl_position(&self, nucl: Nucl) -> Option<Vec3> {
        self.design_reader
            .get_position_of_nucl_on_helix(nucl, Referential::World, false)
    }

    pub fn pivot_sphere(position: Vec3, radius: f32) -> RawDnaInstance {
        SphereInstance {
            position,
            id: 0,
            radius: PIVOT_SCALE_FACTOR * radius,
            color: Instance::color_from_au32(PIVOT_SPHERE_COLOR),
        }
        .to_raw_instance()
    }

    pub fn surface_pivot_sphere(position: Vec3) -> RawDnaInstance {
        SphereInstance {
            position,
            id: 0,
            radius: 1.2 * SELECT_SCALE_FACTOR * SPHERE_RADIUS,
            color: Instance::color_from_au32(SURFACE_PIVOT_SPHERE_COLOR),
        }
        .to_raw_instance()
    }

    pub fn free_xover_sphere(position: Vec3) -> RawDnaInstance {
        SphereInstance {
            position,
            id: 0,
            radius: FREE_XOVER_SCALE_FACTOR * SPHERE_RADIUS,
            color: Instance::color_from_au32(FREE_XOVER_COLOR),
        }
        .to_raw_instance()
    }

    pub fn free_xover_tube(pos1: Vec3, pos2: Vec3) -> RawDnaInstance {
        create_dna_bond(pos1, pos2, FREE_XOVER_COLOR, 0, true).to_raw_instance()
    }

    pub fn has_nucl(&self, nucl: &Nucl) -> bool {
        self.design_reader.get_identifier_nucl(nucl).is_some()
    }

    pub fn both_prime3(&self, nucl1: Nucl, nucl2: Nucl) -> bool {
        let prime3_1 = self.design_reader.prime3_of_which_strand(nucl1);
        let prime3_2 = self.design_reader.prime3_of_which_strand(nucl2);
        prime3_1.and(prime3_2).is_some()
    }

    pub fn both_prime5(&self, nucl1: Nucl, nucl2: Nucl) -> bool {
        let prime5_1 = self.design_reader.prime5_of_which_strand(nucl1);
        let prime5_2 = self.design_reader.prime5_of_which_strand(nucl2);
        prime5_1.and(prime5_2).is_some()
    }

    #[allow(dead_code)]
    pub fn get_all_prime3_cone(&self) -> Vec<RawDnaInstance> {
        if self.all_helices_on_axis {
            return vec![];
        }
        let cones = self.design_reader.get_all_prime3_nucl();
        let mut ret = Vec::with_capacity(cones.len());
        for c in cones {
            ret.push(create_prime3_cone(c.0, c.1, c.2, 1.));
        }
        ret
    }

    pub fn get_surface_info_nucl(&self, nucl: Nucl) -> Option<SurfaceInfo> {
        self.design_reader.get_surface_info_nucl(nucl)
    }

    pub fn get_surface_info(&self, point: SurfacePoint) -> Option<SurfaceInfo> {
        self.design_reader.get_surface_info(point)
    }
}

fn create_dna_bond(source: Vec3, dest: Vec3, color: u32, id: u32, use_alpha: bool) -> TubeInstance {
    let color = if use_alpha {
        Instance::color_from_au32(color)
    } else {
        Instance::unclear_color_from_u32(color)
    };
    let rotor = Rotor3::from_rotation_between(Vec3::unit_x(), (dest - source).normalized());
    let position = (dest + source) / 2.;
    let length = (dest - source).mag();

    TubeInstance {
        position,
        color,
        rotor,
        id,
        radius: BOND_RADIUS,
        length,
    }
}

fn create_helix_cylinder(
    source: Vec3,
    dest: Vec3,
    radius: f32,
    color: u32,
    id: u32,
    use_alpha: bool,
) -> (TubeLidInstance, SlicedTubeInstance, TubeLidInstance) {
    let color = if use_alpha {
        Instance::color_from_au32(color)
    } else {
        Instance::unclear_color_from_u32(color)
    };
    let length = (dest - source).mag();
    let u = (dest - source).normalized();
    let rotor = Rotor3::safe_from_rotation_from_unit_x_to(u);
    let rotor2 = Rotor3::safe_from_rotation_from_unit_x_to(-u);
    let position = (dest + source) / 2.;

    (
        TubeLidInstance {
            position: source,
            color,
            rotor: rotor2,
            id,
            radius,
        },
        SlicedTubeInstance {
            position,
            color,
            rotor,
            id,
            radius,
            length,
            prev: Vec3::zero(),
            next: Vec3::zero(),
        },
        TubeLidInstance {
            position: dest,
            color,
            rotor,
            id,
            radius,
        },
    )
}

fn create_check_bond(source: Vec3, dest: Vec3, checked: bool) -> RawDnaInstance {
    let radius = (source - dest).mag() / 2.;
    let position = (source + dest) / 2.;
    let color = if checked {
        Instance::color_from_au32(CHECKED_XOVER_COLOR)
    } else {
        Instance::color_from_au32(UNCHECKED_XOVER_COLOR)
    };
    SphereInstance {
        position,
        radius,
        color,
        id: 0,
    }
    .to_raw_instance()
}

fn create_prime3_cone(source: Vec3, dest: Vec3, color: u32, radius: f32) -> RawDnaInstance {
    let color = Instance::unclear_color_from_u32(color);
    let rotor = Rotor3::from_rotation_between(Vec3::unit_x(), (dest - source).normalized());
    let radius = radius;
    let length = (2. * radius).max(1. / 3. * (dest - source).mag());
    let position = (3. * source + 2. * dest) / 5.;
    ConeInstance {
        position,
        length,
        rotor,
        color,
        id: 0,
        radius,
    }
    .to_raw_instance()
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

pub(super) enum ExpandWith {
    Spheres,
    Tubes,
}

pub trait DesignReader: 'static + ensnano_interactor::DesignReader {
    /// Return the identifier of all the visible nucleotides
    fn get_all_visible_nucl_ids(&self) -> Vec<u32>;
    /// Return the identifier of all the visible bounds
    fn get_all_visible_bond_ids(&self) -> Vec<u32>;
    fn get_all_nucl_ids(&self) -> Vec<u32>;
    fn get_all_bond_ids(&self) -> Vec<u32>;
    fn get_pasted_position(&self) -> Vec<(Vec<Vec3>, bool)>;
    /// If e_id is the identifier of a nucleotide, return the position on which the
    /// nucleotide's symbols must be displayed
    fn get_symbol_position(&self, e_id: u32) -> Option<Vec3>;
    /// If e_id is the identifier of a nucleotide, return the symbol associated to the
    /// nucleotide.
    fn get_symbol(&self, e_id: u32) -> Option<char>;
    fn get_model_matrix(&self) -> Mat4;
    /// Return true iff e_id is the identifier of a nucleotide that must be displayed with a
    /// smaller size
    fn has_small_spheres_nucl_id(&self, e_id: u32) -> bool;
    /// Return the list of pairs of nucleotides that can be linked by a cross-over
    fn get_suggestions(&self) -> Vec<(Nucl, Nucl)>;
    fn get_position_of_nucl_on_helix(
        &self,
        nucl: Nucl,
        referential: Referential,
        on_axis: bool,
    ) -> Option<Vec3>;
    fn get_object_type(&self, id: u32) -> Option<ObjectType>;
    fn get_grid_position(&self, g_id: GridId) -> Option<Vec3>;
    fn get_grid_latice_position(&self, position: GridPosition) -> Option<Vec3>;
    fn get_element_position(&self, e_id: u32, referential: Referential) -> Option<Vec3>;
    fn get_element_axis_position(&self, id: u32, referential: Referential) -> Option<Vec3>;
    fn get_element_graphic_position(&self, id: u32, referential: Referential) -> Option<Vec3>;
    fn get_color(&self, e_id: u32) -> Option<u32>;
    fn get_radius(&self, e_id: u32) -> Option<f32>;
    fn get_xover_coloring(&self, e_id: u32) -> Option<bool>;
    fn get_with_cones(&self, e_id: u32) -> Option<bool>;
    fn get_id_of_strand_containing(&self, e_id: u32) -> Option<usize>;
    fn get_id_of_helix_containing(&self, e_id: u32) -> Option<usize>;
    fn get_ids_of_all_helices(&self) -> Vec<u32>;
    fn get_ids_of_elements_belonging_to_strand(&self, s_id: usize) -> Vec<u32>;
    fn get_ids_of_elements_belonging_to_helix(&self, h_id: usize) -> Vec<u32>;
    fn get_helix_basis(&self, h_id: u32) -> Option<Rotor3>;
    fn get_basis(&self) -> Rotor3;
    fn get_identifier_nucl(&self, nucl: &Nucl) -> Option<u32>;
    fn get_identifier_bond(&self, n1: Nucl, n2: Nucl) -> Option<u32>;
    fn get_nucl_with_id(&self, e_id: u32) -> Option<Nucl>;
    /// Return the nucleotide with id e_id or the 5' end of the bond with id e_id
    fn get_nucl_with_id_relaxed(&self, e_id: u32) -> Option<Nucl>;
    fn can_start_builder_at(&self, nucl: &Nucl) -> bool;
    fn get_grid_instances(&self) -> BTreeMap<GridId, GridInstance>;
    fn get_helices_on_grid(&self, g_id: GridId) -> Option<HashSet<usize>>;
    fn get_used_coordinates_on_grid(&self, g_id: GridId) -> Option<Vec<(isize, isize)>>;
    fn get_helices_grid_key_coord(&self, g_id: GridId) -> Option<Vec<((isize, isize), usize)>>;
    fn get_helix_id_at_grid_coord(&self, position: GridPosition) -> Option<u32>;
    fn get_persistent_phantom_helices_id(&self) -> HashSet<u32>;
    fn get_grid_basis(&self, g_id: GridId) -> Option<Rotor3>;
    fn get_helix_grid_position(&self, h_id: u32) -> Option<HelixGridPosition>;
    fn prime5_of_which_strand(&self, nucl: Nucl) -> Option<usize>;
    fn prime3_of_which_strand(&self, nucl: Nucl) -> Option<usize>;
    fn get_all_prime3_nucl(&self) -> Vec<(Vec3, Vec3, u32)>;
    fn get_curve_range(&self, h_id: usize) -> Option<std::ops::RangeInclusive<isize>>;
    fn get_checked_xovers_ids(&self, checked: bool) -> Vec<u32>;
    fn get_id_of_xover_involving_nucl(&self, nucl: Nucl) -> Option<usize>;
    fn get_grid_object(&self, position: GridPosition) -> Option<GridObject>;
    fn get_position_of_bezier_control(
        &self,
        helix: usize,
        control: BezierControlPoint,
    ) -> Option<Vec3>;
    fn get_cubic_bezier_controls(&self, helix: usize) -> Option<CubicBezierConstructor>;
    fn get_piecewise_bezier_controls(&self, helix: usize) -> Option<Vec<Vec3>>;
    fn get_curve_descriptor(&self, helix: usize) -> Option<&CurveDescriptor>;
    fn get_all_h_bonds(&self) -> &[HBond];
    fn get_all_loopout_nucl(&self) -> &[LoopoutNucl];
    fn get_all_loopout_bonds(&self) -> &[LoopoutBond];
    fn get_insertion_length(&self, bond_id: u32) -> usize;
    fn get_expected_bond_length(&self) -> f32;
    fn get_bezier_planes(
        &self,
    ) -> &dyn Collection<Item = BezierPlaneDescriptor, Key = BezierPlaneId>;
    fn get_parameters(&self) -> HelixParameters;
    fn get_bezier_paths(&self) -> Option<&BTreeMap<BezierPathId, Arc<InstanciatedPath>>>;
    fn get_bezier_vertex(&self, path_id: BezierPathId, vertex_id: usize) -> Option<BezierVertex>;
    fn get_corners_of_plane(&self, plane_id: BezierPlaneId) -> [Vec2; 4];
    fn get_optimal_xover_arround(&self, source: Nucl, target: Nucl) -> Option<(Nucl, Nucl)>;
    fn get_bezier_grid_used_by_helix(&self, h_id: usize) -> Vec<GridId>;
    fn get_external_objects(&self) -> &External3DObjects;
    fn get_surface_info_nucl(&self, nucl: Nucl) -> Option<SurfaceInfo>;
    fn get_surface_info(&self, point: SurfacePoint) -> Option<SurfaceInfo>;
    fn get_additional_structure(&self) -> Option<&dyn AdditionalStructure>;
}

pub(super) struct HBondsInstances {
    pub full_h_bonds: Vec<RawDnaInstance>,
    pub partial_h_bonds: Vec<RawDnaInstance>,
    pub ellipsoids: Vec<RawDnaInstance>,
}

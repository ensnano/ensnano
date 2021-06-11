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

use super::*;
use crate::design::{ObjectType, Referential};
use crate::scene::GridInstance;
use ensnano_design::{grid::GridPosition, Nucl};
use std::collections::HashSet;
use ultraviolet::{Mat4, Rotor3, Vec3};

use crate::scene::DesignReader as Reader3D;

impl Reader3D for DesignReader {
    fn get_color(&self, e_id: u32) -> Option<u32> {
        todo!()
    }

    fn get_basis(&self) -> Rotor3 {
        todo!()
    }

    fn get_symbol(&self, e_id: u32) -> Option<char> {
        todo!()
    }

    fn get_xover_id(&self, xover: &(Nucl, Nucl)) -> Option<usize> {
        todo!()
    }

    fn get_grid_basis(&self, g_id: usize) -> Option<Rotor3> {
        todo!()
    }

    fn get_suggestions(&self) -> Vec<(Nucl, Nucl)> {
        todo!()
    }

    fn get_object_type(&self, id: u32) -> Option<ObjectType> {
        todo!()
    }

    fn get_helix_basis(&self, h_id: u32) -> Option<Rotor3> {
        todo!()
    }

    fn get_all_nucl_ids(&self) -> Vec<u32> {
        todo!()
    }

    fn get_model_matrix(&self) -> Mat4 {
        todo!()
    }

    fn get_nucl_with_id(&self, e_id: u32) -> Option<Nucl> {
        todo!()
    }

    fn get_all_bound_ids(&self) -> Vec<u32> {
        todo!()
    }

    fn get_grid_position(&self, g_id: usize) -> Option<Vec3> {
        todo!()
    }

    fn get_xover_with_id(&self, xover_id: usize) -> Option<(Nucl, Nucl)> {
        todo!()
    }

    fn get_element_5prime(&self, e_id: u32) -> Option<u32> {
        todo!()
    }

    fn get_element_3prime(&self, e_id: u32) -> Option<u32> {
        todo!()
    }

    fn get_grid_instances(&self) -> Vec<GridInstance> {
        todo!()
    }

    fn get_pasted_position(&self) -> Vec<(Vec<Vec3>, bool)> {
        todo!()
    }

    fn get_symbol_position(&self, e_id: u32) -> Option<Vec3> {
        todo!()
    }

    fn get_identifier_nucl(&self, nucl: &Nucl) -> Option<u32> {
        todo!()
    }

    fn get_position_of_nucl_on_helix(
        &self,
        nucl: Nucl,
        referential: Referential,
        on_axis: bool,
    ) -> Option<Vec3> {
        todo!()
    }

    fn get_helices_on_grid(&self, g_id: usize) -> Option<HashSet<usize>> {
        todo!()
    }

    fn get_all_prime3_nucl(&self) -> Vec<(Vec3, Vec3, u32)> {
        todo!()
    }

    fn get_element_position(&self, e_id: u32, referential: Referential) -> Option<Vec3> {
        todo!()
    }

    fn get_identifier_bound(&self, n1: &Nucl, n2: &Nucl) -> Option<u32> {
        todo!()
    }

    fn get_helix_grid_position(&self, h_id: u32) -> Option<GridPosition> {
        todo!()
    }

    fn get_all_visible_nucl_ids(&self) -> Vec<u32> {
        todo!()
    }

    fn get_grid_latice_position(&self, g_id: usize, x: isize, y: isize) -> Option<Vec3> {
        todo!()
    }

    fn get_nucl_with_id_relaxed(&self, e_id: u32) -> Option<Nucl> {
        todo!()
    }

    fn get_all_visible_bound_ids(&self) -> Vec<u32> {
        todo!()
    }

    fn get_element_axis_position(&self, id: u32, referential: Referential) -> Option<Vec3> {
        todo!()
    }

    fn get_id_of_helix_containing(&self, e_id: u32) -> Option<usize> {
        todo!()
    }

    fn get_helices_grid_key_coord(&self, g_id: usize) -> Option<Vec<((isize, isize), usize)>> {
        todo!()
    }

    fn get_helix_id_at_grid_coord(&self, g_id: usize, x: isize, y: isize) -> Option<u32> {
        todo!()
    }

    fn get_id_of_strand_containing(&self, e_id: u32) -> Option<usize> {
        todo!()
    }

    fn get_used_coordinates_on_grid(&self, g_id: usize) -> Option<Vec<(isize, isize)>> {
        todo!()
    }

    fn get_persistent_phantom_helices_id(&self) -> HashSet<u32> {
        todo!()
    }

    fn get_ids_of_elements_belonging_to_helix(&self, h_id: usize) -> Vec<u32> {
        todo!()
    }

    fn get_ids_of_elements_belonging_to_strand(&self, s_id: usize) -> Vec<u32> {
        todo!()
    }

    fn prime5_of_which_strand(&self, nucl: Nucl) -> Option<usize> {
        todo!()
    }

    fn prime3_of_which_strand(&self, nucl: Nucl) -> Option<usize> {
        todo!()
    }

    fn can_start_builder_at(&self, nucl: &Nucl) -> bool {
        todo!()
    }

    fn has_small_spheres_nucl_id(&self, e_id: u32) -> bool {
        todo!()
    }
}

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

use ensnano_design::{elements::DnaElement, CameraId};

use super::*;
use crate::gui::DesignReader as ReaderGui;
use ensnano_interactor::InsertionPoint;
use ultraviolet::Rotor3;

impl ReaderGui for DesignReader {
    fn grid_has_small_spheres(&self, g_id: usize) -> bool {
        self.presenter.content.grid_has_small_spheres(g_id)
    }

    fn grid_has_persistent_phantom(&self, g_id: usize) -> bool {
        self.presenter.content.grid_has_persistent_phantom(g_id)
    }

    fn get_grid_shift(&self, g_id: usize) -> Option<f32> {
        self.presenter.content.get_grid_shift(g_id)
    }

    fn get_strand_length(&self, s_id: usize) -> Option<usize> {
        self.presenter
            .current_design
            .strands
            .get(&s_id)
            .map(|s| s.length())
    }

    fn is_id_of_scaffold(&self, s_id: usize) -> bool {
        self.presenter.current_design.scaffold_id == Some(s_id)
    }

    fn nucl_is_anchor(&self, nucl: Nucl) -> bool {
        self.presenter.current_design.anchors.contains(&nucl)
    }

    fn length_decomposition(&self, s_id: usize) -> String {
        self.presenter.decompose_length(s_id)
    }

    fn get_dna_elements(&self) -> &[DnaElement] {
        self.presenter.content.elements.as_slice()
    }

    fn get_organizer_tree(&self) -> Option<Arc<ensnano_design::EnsnTree>> {
        RollPresenter::get_design(self.presenter.as_ref())
            .organizer_tree
            .clone()
    }

    fn strand_name(&self, s_id: usize) -> String {
        self.presenter
            .current_design
            .strands
            .get(&s_id)
            .and_then(|s| s.name.as_ref().map(|n| n.to_string()))
            .unwrap_or(String::from("Unamed strand"))
    }

    fn get_all_cameras(&self) -> Vec<(CameraId, &str)> {
        //TODO this obviously needs to be updated to relate to the real content
        self.presenter
            .current_design
            .get_cameras()
            .into_iter()
            .map(|(id, cam)| (*id, cam.name.as_str()))
            .collect()
    }

    fn get_favourite_camera(&self) -> Option<CameraId> {
        self.presenter.current_design.get_favourite_camera_id()
    }

    fn get_grid_position_and_orientation(&self, g_id: usize) -> Option<(Vec3, Rotor3)> {
        self.presenter
            .current_design
            .grids
            .get(g_id)
            .map(|g| (g.position, g.orientation))
    }

    fn get_insertion_length(&self, selection: &Selection) -> Option<usize> {
        match selection {
            Selection::Bound(_, n1, n2) => {
                let bond_id = self
                    .presenter
                    .content
                    .identifier_bound
                    .get(&(*n1, *n2))
                    .or(self.presenter.content.identifier_bound.get(&(*n2, *n1)))?;
                self.presenter
                    .content
                    .insertion_length
                    .get(&bond_id)
                    .cloned()
                    .or(Some(0))
            }
            Selection::Xover(_, xover_id) => {
                let (n1, n2) = self.presenter.junctions_ids.get_element(*xover_id)?;
                let bond_id = self
                    .presenter
                    .content
                    .identifier_bound
                    .get(&(n1, n2))
                    .or(self.presenter.content.identifier_bound.get(&(n2, n1)))?;
                self.presenter
                    .content
                    .insertion_length
                    .get(&bond_id)
                    .cloned()
                    .or(Some(0))
            }
            _ => None,
        }
    }

    fn get_insertion_point(&self, selection: &Selection) -> Option<InsertionPoint> {
        match selection {
            Selection::Bound(_, n1, _n2) => Some(InsertionPoint {
                nucl: *n1,
                nucl_is_prime5_of_insertion: true,
            }),
            Selection::Xover(_, xover_id) => {
                let (n1, _n2) = self.presenter.junctions_ids.get_element(*xover_id)?;
                Some(InsertionPoint {
                    nucl: n1,
                    nucl_is_prime5_of_insertion: true,
                })
            }
            Selection::Nucleotide(_, nucl) => {
                if let Some(s_id) = self.prime5_of_which_strand(*nucl) {
                    Some(InsertionPoint {
                        nucl: *nucl,
                        nucl_is_prime5_of_insertion: false,
                    })
                } else if let Some(s_id) = self.prime3_of_which_strand(*nucl) {
                    Some(InsertionPoint {
                        nucl: *nucl,
                        nucl_is_prime5_of_insertion: true,
                    })
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

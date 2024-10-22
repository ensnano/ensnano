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

use crate::rotor_utils::SafeRotor;
use crate::ultraviolet::{Rotor3, Vec3};
use ensnano_utils::instance::Instance;

use crate::view::{SlicedTubeInstance, TubeLidInstance};

use ensnano_interactor::consts::{HELIX_CYLINDER_COLOR, HELIX_CYLINDER_RADIUS};

pub struct SausageRosary {
    pub positions: Vec<Vec3>,
    pub is_cyclic: bool,
}

impl SausageRosary {
    pub fn to_raw_dna_instances(
        &self,
        color: impl Fn(usize) -> u32,
        radius: f32,
        id: u32,
    ) -> (
        Vec<SlicedTubeInstance>,
        Option<(TubeLidInstance, TubeLidInstance)>,
    ) {
        let n = self.positions.len();

        if n <= 1 {
            return (vec![], None);
        }

        let prev_current_next_p1_p2 = if self.is_cyclic {
            let vecs = self
                .positions
                .iter()
                .cycle()
                .skip(n - 1)
                .zip(self.positions.iter())
                .map(|(prev, point)| *point - *prev)
                .collect::<Vec<Vec3>>();
            vecs.iter()
                .zip(vecs.iter().cycle().skip(1))
                .zip(vecs.iter().cycle().skip(2))
                .zip(self.positions.iter())
                .zip(self.positions.iter().cycle().skip(1))
                .map(|((((prev, cur), next), p1), p2)| (*prev, *cur, *next, *p1, *p2))
                .collect::<Vec<(Vec3, Vec3, Vec3, Vec3, Vec3)>>()
        } else {
            let mut vecs = vec![Vec3::zero()];
            vecs.extend(
                self.positions
                    .iter()
                    .zip(self.positions.iter().skip(1))
                    .map(|(prev, point)| *point - *prev)
                    .collect::<Vec<Vec3>>(),
            );
            vecs.push(Vec3::zero());
            vecs.iter()
                .zip(vecs.iter().skip(1))
                .zip(vecs.iter().skip(2))
                .zip(self.positions.iter())
                .zip(self.positions.iter().skip(1))
                .map(|((((prev, current), next), p1), p2)| (*prev, *current, *next, *p1, *p2))
                .collect::<Vec<(Vec3, Vec3, Vec3, Vec3, Vec3)>>()
        };

        // for s in prev_current_next_p1_p2.clone().into_iter() {
        //     println!("{:?}", s);
        // }

        let mut color_iter = (0..prev_current_next_p1_p2.len()).map(|i| color(i));

        let tubes = prev_current_next_p1_p2
            .into_iter()
            .map(|(prev, current, next, p1, p2)| {
                let position = (p1 + p2) / 2.;
                let normalized = current.normalized();
                let rotor = Rotor3::safe_from_rotation_from_unit_x_to(normalized);
                let rotor_inv = Rotor3::safe_from_rotation_to_unit_x_from(normalized);
                SlicedTubeInstance {
                    position,
                    rotor,
                    color: Instance::unclear_color_from_u32(
                        color_iter.next().unwrap_or(HELIX_CYLINDER_COLOR),
                    ),
                    id,
                    radius,
                    length: current.mag(),
                    prev: prev.rotated_by(rotor_inv),
                    next: next.rotated_by(rotor_inv),
                }
            })
            .collect::<Vec<SlicedTubeInstance>>();

        if self.is_cyclic {
            return (tubes, None);
        } else {
            let (p0, p1) = (self.positions[0], self.positions[1]);
            // println!("{:?}", p1-p0);
            let u = (p0 - p1).normalized();
            let rotor = Rotor3::safe_from_rotation_from_unit_x_to(u);

            let lid_left = TubeLidInstance {
                position: p0,
                color: Instance::unclear_color_from_u32(color(0)),
                rotor,
                id,
                radius,
            };

            let (p0, p1) = (self.positions[n - 2], self.positions[n - 1]);
            let u = (p1 - p0).normalized();
            // println!("{:?}", p1-p0);
            let rotor = Rotor3::safe_from_rotation_from_unit_x_to(u);

            let lid_right = TubeLidInstance {
                position: p1,
                color: Instance::unclear_color_from_u32(color(n - 1)),
                rotor,
                id,
                radius,
            };
            return (tubes, Some((lid_left, lid_right)));
        }
    }
}

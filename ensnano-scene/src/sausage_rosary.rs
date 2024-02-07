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
    pub cyclic: bool,
}

impl SausageRosary {
    pub fn to_raw_dna_instances(
        &self,
    ) -> (
        Vec<SlicedTubeInstance>,
        Option<(TubeLidInstance, TubeLidInstance)>,
    ) {
        let n = self.positions.len();

        if n <= 1 {
            return (vec![], None);
        }

        let prev_current_next_p1_p2 = if self.cyclic {
            let vecs = self
                .positions
                .into_iter()
                .cycle()
                .skip(n - 1)
                .zip(self.positions.into_iter())
                .map(|(prev, point)| point - prev)
                .collect::<Vec<Vec3>>();
            let vecs = vecs
                .into_iter()
                .cycle()
                .skip(n - 1)
                .zip(vecs.into_iter())
                .zip(vecs.into_iter().cycle().skip(1))
                .map(|((prev, current), next)| (prev, current, next))
                .collect::<Vec<(Vec3, Vec3, Vec3)>>();
            vecs.into_iter()
                .zip(self.positions.into_iter())
                .zip(self.positions.into_iter().cycle().skip(1))
                .map(|(((prev, cur, next), p1), p2)| (prev, cur, next, p1, p2))
                .collect::<Vec<(Vec3, Vec3, Vec3, Vec3, Vec3)>>()
        } else {
            let mut vecs = vec![Vec3::zero()];
            vecs.extend(
                self.positions
                    .into_iter()
                    .zip(self.positions.into_iter().skip(1))
                    .map(|(prev, point)| point - prev)
                    .collect::<Vec<Vec3>>(),
            );
            vecs.push(Vec3::zero());
            let vecs = vecs
                .into_iter()
                .zip(vecs.into_iter().skip(1))
                .zip(vecs.into_iter().skip(2))
                .map(|((prev, current), next)| (prev, current, next))
                .collect::<Vec<(Vec3, Vec3, Vec3)>>();
            vecs.into_iter()
                .zip(self.positions.into_iter())
                .zip(self.positions.into_iter().skip(1))
                .map(|(((prev, cur, next), p1), p2)| (prev, cur, next, p1, p2))
                .collect::<Vec<(Vec3, Vec3, Vec3, Vec3, Vec3)>>()
        };

        let tubes = prev_current_next_p1_p2
            .into_iter()
            .map(|(prev, current, next, p1, p2)| {
                let pos = (p1 + p2) / 2.;
                let rotor = Rotor3::safe_from_rotation_from_unit_x_to(current);
                let rotor_inv = Rotor3::safe_from_rotation_to_unit_x_from(current);
                SlicedTubeInstance {
                    position: (p1 + p2) / 2.,
                    rotor: rotor,
                    color: Instance::unclear_color_from_u32(HELIX_CYLINDER_COLOR),
                    id: 0,
                    radius: HELIX_CYLINDER_RADIUS,
                    length: current.mag(),
                    prev: prev.rotated_by(rotor_inv),
                    next: next.rotated_by(rotor_inv),
                }
            })
            .collect::<Vec<SlicedTubeInstance>>();

        return (tubes, None);
    }
}

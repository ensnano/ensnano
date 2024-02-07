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

use crate::ultraviolet::{Mat4, Rotor3, Vec2, Vec3, Vec4};
use crate::view::{
    Instanciable, RawDnaInstance, SlicedTubeInstance, TubeLidInstance,
};

pub struct SausageRosary {
	pub positions: Vec<Vec3>,
	pub cyclic: bool, 	
}

impl SausageRosary {

	pub fn to_raw_dna_instances(&self) -> (Vec<SlicedTubeInstance>, Option<(TubeLidInstance, TubeLidInstance)>) {
        let n = self.positions.len();
        let vecs = if self.cyclic {
            let mut vecs = positions.iter().cycle().skip(n-1).zip(positions)
            .map(|prev,point| point - prev)
            .collect::<Vec<Vec3>>();
            vecs.push(vecs[0].clone());
            vecs
        } else {
            let mut vecs = vec![Vec3::zero()];
            vecs.extend(positions.iter().zip(positions.iter().skip(1))
            .map(|prev, point| point - prev)
            .collect::<Vec<Vec3>>());
            vecs.push(Vec3::zero());
            vecs
        };


        return (vec![SlicedTubeInstance::default()], None);
	}
}
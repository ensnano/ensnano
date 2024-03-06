/*
ENSnano, a 3d graphical application for DNA nanostructures.
    Copyright (C) 2024 Nicolas Schabanel <nicolas.schabanel@ens-lyon.fr>

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


use ultraviolet::{Vec2};
use std::f32::consts::PI;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum IsoPoint {
	Move(usize, f32),
	Origin,
}

impl Default for IsoPoint {
	fn default() -> Self {
		Self::Origin
	}
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct IsoGraph {
	pub points: Vec<IsoPoint>,
	pub coords: Vec<Vec2>,
}

impl IsoGraph {
	pub fn new() -> Self {
		IsoGraph::default()
	}

	pub fn from(points: Vec<IsoPoint>) -> Self {
		let mut g = IsoGraph::new();
		g.points = points.clone();
		g.compute_coords();
		g
	}

	pub fn compute_coords(&mut self) {
		let mut coords = Vec::new();
		for p in &self.points {
			coords.push(match p {
				IsoPoint::Origin => Vec2::new(0., 0.),
				IsoPoint::Move(i, angle) => {
					let a = *angle * PI / 180.;
					coords[*i] + Vec2::new(a.cos(), a.sin())
				},
			});
		}
		self.coords = coords;
	}
}


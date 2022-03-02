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
use std::collections::BTreeMap;
use std::sync::Arc;
use ultraviolet::{Rotor3, Vec3};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BezierPlaneDescriptor {
    pub position: Vec3,
    pub orientation: Rotor3,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct BezierPlaneId(usize);

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct BezierPlanes(Arc<BTreeMap<BezierPlaneId, Arc<BezierPlaneDescriptor>>>);

pub trait BezierPlaneCollection {
    fn get(&self, id: &BezierPlaneId) -> Option<&BezierPlaneDescriptor>;
    fn iter<'a>(
        &'a self,
    ) -> Box<dyn Iterator<Item = (&'a BezierPlaneId, &'a BezierPlaneDescriptor)> + 'a>;
    fn values<'a>(&'a self) -> Box<dyn Iterator<Item = &'a BezierPlaneDescriptor> + 'a>;
    fn keys<'a>(&'a self) -> Box<dyn Iterator<Item = &'a BezierPlaneId> + 'a>;
    fn len(&self) -> usize;
    fn contains_key(&self, id: &BezierPlaneId) -> bool;
}

impl BezierPlanes {
    pub fn make_mut<'a>(&'a mut self) -> BezierPlanesMut<'a> {
        let new_map = BTreeMap::clone(&self.0);
        BezierPlanesMut {
            source: self,
            new_map,
        }
    }
}

pub struct BezierPlanesMut<'a> {
    source: &'a mut BezierPlanes,
    new_map: BTreeMap<BezierPlaneId, Arc<BezierPlaneDescriptor>>,
}

impl<'a> BezierPlanesMut<'a> {
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

impl<'a> Drop for BezierPlanesMut<'a> {
    fn drop(&mut self) {
        *self.source = BezierPlanes(Arc::new(std::mem::take(&mut self.new_map)))
    }
}

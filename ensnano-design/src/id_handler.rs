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
use crate::HasMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Default, Hash)]
/// Named datastructure
pub struct NamedItem<T: Clone>(pub String, pub T);

#[derive(
    Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Default, Hash,
)]
/// Generic Identifier
pub struct HandledId(pub usize);

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
/// Collection of free grids descriptor
pub struct IdHandler<T: Clone>(pub(super) Arc<BTreeMap<HandledId, Arc<T>>>);

// impl<T> HasMap for IdHandler<T>  where T: Clone {
//     type Key = HandledId;
//     type Item = T;
//     fn get_map(&self) -> &BTreeMap<Self::Key, Arc<Self::Item>> {
//         &self.0
//     }
// }

pub trait GiveItemNamed<T> {
    fn get_id_of_one_item_named(self, name: String) -> Option<HandledId>;
}

impl<T: Clone> GiveItemNamed<T> for IdHandler<NamedItem<T>> {
    fn get_id_of_one_item_named(self, name: String) -> Option<HandledId> {
        for (k, v) in self.0.iter() {
            if v.0.eq(&name) {
                return Some(k.clone());
            }
        }
        return None;
    }
}

impl<T> IdHandler<T>
where
    T: Clone,
{
    pub fn make_mut(&mut self) -> IdHandlerMut<T> {
        IdHandlerMut {
            new_map: BTreeMap::clone(&self.0),
            source: self,
        }
    }

    pub fn from_vec(vec: Vec<T>) -> Self {
        Self(Arc::new(
            vec.into_iter()
                .enumerate()
                .map(|(id, item)| (HandledId(id), Arc::new(item)))
                .collect(),
        ))
    }

    pub fn ptr_eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

pub struct IdHandlerMut<'a, T>
where
    T: Clone,
{
    source: &'a mut IdHandler<T>,
    new_map: BTreeMap<HandledId, Arc<T>>,
}

impl<'a, T> IdHandlerMut<'a, T>
where
    T: Clone,
{
    pub fn push(&mut self, item: T) -> HandledId {
        let new_key = self
            .new_map
            .keys()
            .max()
            .map(|m| HandledId(m.0 + 1))
            .unwrap_or_default();
        self.new_map.insert(new_key, Arc::new(item));
        HandledId(new_key.0)
    }

    pub fn get_mut(&mut self, id: &HandledId) -> Option<&mut T> {
        self.new_map.get_mut(&id).map(Arc::make_mut)
    }

    pub fn remove(&mut self, id: &HandledId) -> Option<Arc<T>> {
        self.new_map.remove(&id)
    }
}

impl<'a, T> Drop for IdHandlerMut<'a, T>
where
    T: Clone,
{
    fn drop(&mut self) {
        *self.source = IdHandler(Arc::new(std::mem::take(&mut self.new_map)))
    }
}

#[cfg(test)]
mod tests {
    use super::IdHandler;

    #[test]
    fn simple_test_for_Vec() {
        let v: IdHandler<Vec<i32>> = IdHandler(());
    }
}

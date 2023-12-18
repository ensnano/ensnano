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
pub struct NamedItem<T: Clone>(pub String, pub T);

#[derive(
    Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Default, Hash,
)]
/// Generic Identifier
pub struct Id(pub usize);

#[derive(Debug, Clone, Deserialize, Serialize, Default)]

/// Collection of items with ids
pub struct IdManager<T: Clone>(pub(super) Arc<BTreeMap<Id, Arc<T>>>);

// impl<T> HasMap for IdHandler<T>  where T: Clone {
//     type Key = Id;
//     type Item = T;
//     fn get_map(&self) -> &BTreeMap<Self::Key, Arc<Self::Item>> {
//         &self.0
//     }
// }

pub trait ItemWithName<'a> {
    fn get_name(self) -> &'a str;
}

impl<'a> ItemWithName<'a> for NamedParameter {
    fn get_name(self) -> &'static str {
        return &self.name;
    }
}

pub trait IdManagerForNamedItems {
    fn get_id_of_one_item_named(self, name: String) -> Option<Id>;
    fn get_name_by_id(self, id: Id) -> Option<String>;
}

impl<T: Clone> IdManagerForNamedItems for IdManager<NamedItem<T>> {
    fn get_id_of_one_item_named(self, name: String) -> Option<Id> {
        for (k, v) in self.0.iter() {
            if v.0.eq(&name) {
                return Some(k.clone());
            }
        }
        return None;
    }

    fn get_name_by_id(self, id: Id) -> Option<String> {
        self.0.get(&id).map(|item| item.0.clone())
    }
}

impl<T: Clone> IdManager<T> {
    pub fn make_mut(&mut self) -> IdManagerMut<T> {
        IdManagerMut {
            new_map: BTreeMap::clone(&self.0),
            source: self,
        }
    }

    pub fn from_vec(vec: Vec<T>) -> Self {
        Self(Arc::new(
            vec.into_iter()
                .enumerate()
                .map(|(id, item)| (Id(id), Arc::new(item)))
                .collect(),
        ))
    }

    pub fn ptr_eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

pub struct IdManagerMut<'a, T: Clone> {
    source: &'a mut IdManager<T>,
    new_map: BTreeMap<Id, Arc<T>>,
}

impl<'a, T: Clone> IdManagerMut<'a, T> {
    pub fn push(&mut self, item: T) -> Id {
        let new_key = self
            .new_map
            .keys()
            .max()
            .map(|m| Id(m.0 + 1))
            .unwrap_or_default();
        self.new_map.insert(new_key, Arc::new(item));
        Id(new_key.0)
    }

    pub fn get_mut(&mut self, id: &Id) -> Option<&mut T> {
        self.new_map.get_mut(&id).map(Arc::make_mut)
    }

    pub fn remove(&mut self, id: &Id) -> Option<Arc<T>> {
        self.new_map.remove(&id)
    }
}

impl<'a, T> Drop for IdManagerMut<'a, T>
where
    T: Clone,
{
    fn drop(&mut self) {
        *self.source = IdManager(Arc::new(std::mem::take(&mut self.new_map)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::HelixParameters;
    use crate::NamedParameter;

    #[test]
    fn get_name_from_itemwithname_for_namedparameter() {
        let my_parameter = NamedParameter {
            name: "My Parameter Name",
            value: HelixParameters {
                z_step: 0.,
                helix_radius: 0.,
                bases_per_turn: 0.,
                groove_angle: 0.,
                inter_helix_gap: 0.,
                inclination: 0.,
            },
        };
        assert_eq!("My Parameter Name", my_parameter.get_name())
    }
    #[test]
    fn get_id_of_named_if_it_exists() {
        let cat1 = NamedItem(String::from("Otto"), "cat");
        let cat2 = NamedItem(String::from("Duchesse"), "cat");
        let dog = NamedItem(String::from("Otto"), "dog");
        let mut my_collection = BTreeMap::from([(Id(1), Arc::new(cat1)), (Id(2), Arc::new(cat2))]);
        my_collection.insert(Id(101), Arc::new(dog));
        let my_arced_collection = Arc::new(my_collection);
        let my_ided_collection = IdManager(my_arced_collection);
        assert_eq!(
            Id(1),
            my_ided_collection
                .get_id_of_one_item_named(String::from("Otto"))
                .unwrap()
        );
    }

    #[test]
    fn get_id_of_named_if_does_not_exist() {
        let cat1 = NamedItem(String::from("Otto"), "cat");
        let my_collection = BTreeMap::from([(Id(1), Arc::new(cat1))]);
        let my_arced_collection = Arc::new(my_collection);
        let my_ided_collection = IdManager(my_arced_collection);
        assert_eq!(
            None,
            my_ided_collection.get_id_of_one_item_named(String::from("Chachat"))
        );
    }
    #[test]
    fn get_name_if_exists() {
        let cat1 = NamedItem(String::from("Otto"), "cat");
        let my_collection = BTreeMap::from([(Id(1), Arc::new(cat1))]);
        let my_arced_collection = Arc::new(my_collection);
        let my_ided_collection = IdManager(my_arced_collection);
        assert_eq!(
            Some(String::from("Otto")),
            my_ided_collection.get_name_by_id(Id(1))
        );
    }

    #[test]
    fn get_name_if_does_not_exist() {
        let cat1 = NamedItem(String::from("Otto"), "cat");
        let my_collection = BTreeMap::from([(Id(1), Arc::new(cat1))]);
        let my_arced_collection = Arc::new(my_collection);
        let my_ided_collection = IdManager(my_arced_collection);
        assert_eq!(None, my_ided_collection.get_name_by_id(Id(2)));
    }

    #[test]
    fn make_mut_and_push() {
        let cat = NamedItem(String::from("Snowball"), "cat");
        let mut my_ided_collection =
            IdManager(Arc::new(BTreeMap::from([(Id(100), Arc::new(cat))])));
        {
            let mut my_mut_ided_collection = my_ided_collection.make_mut();
            my_mut_ided_collection.push(NamedItem(String::from("Ember"), "dog"));
        }
        assert_eq!("IdManager({Id(100): NamedItem(\"Snowball\", \"cat\"), Id(101): NamedItem(\"Ember\", \"dog\")})", format!("{:?}", my_ided_collection));
    }

    #[test]
    fn make_mut_and_remove() {
        let cat = NamedItem(String::from("Snowball"), "cat");
        let mut my_ided_collection = IdManager(Arc::new(BTreeMap::from([
            (Id(100), Arc::new(cat.clone())),
            (Id(20), Arc::new(cat)),
        ])));
        {
            let mut my_mut_ided_collection = my_ided_collection.make_mut();
            my_mut_ided_collection.remove(&Id(20));
            my_mut_ided_collection.remove(&Id(100));
        }
        assert_eq!("IdManager({})", format!("{:?}", my_ided_collection));
    }
}

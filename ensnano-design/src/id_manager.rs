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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Default, Hash)]
pub struct NamedItem<'a, T>(pub &'a str, pub T);

pub trait ItemWithName<'a> {
    fn get_name(&self) -> &'a str;
}

/*impl<'a> ItemWithName<'a> for NamedParameter {
    fn get_name(self) -> &'static str {
        return &self.name;
    }
}*/

impl<'a, T> ItemWithName<'a> for NamedItem<'a, T> {
    fn get_name(&self) -> &'a str {
        return &self.0;
    }
}

impl<'a, T> ItemWithName<'a> for Arc<NamedItem<'a, T>> {
    fn get_name(&self) -> &'a str {
        return &self.0;
    }
}

#[derive(
    Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Default, Hash,
)]
/// Generic Identifier
pub struct Id(pub usize);

#[derive(Debug, Clone)] //, Deserialize, Serialize, Default)]
/// Collection of items with ids
struct IdCollection<T: Clone>(pub(super) Arc<BTreeMap<Id, Arc<T>>>);

enum IdManagerError<'a> {
    NoItemWithSuchId(Id),
    NoItemWithSuchName(&'a str),
}

impl<T: Clone> IdCollection<T> {
    pub fn make_mut(&mut self) -> IdCollectionMut<T> {
        IdCollectionMut {
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

pub struct IdCollectionMut<'a, T: Clone> {
    source: &'a mut IdCollection<T>,
    new_map: BTreeMap<Id, Arc<T>>,
}

impl<'a, T: Clone> IdCollectionMut<'a, T> {
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

    pub fn remove(&mut self, id: &Id) -> Result<Arc<T>, IdManagerError> {
        self.new_map
            .remove(&id)
            .ok_or(IdManagerError::NoItemWithSuchId(Id(id.0)))
    }
}

impl<'a, T> Drop for IdCollectionMut<'a, T>
where
    T: Clone,
{
    fn drop(&mut self) {
        *self.source = IdCollection(Arc::new(std::mem::take(&mut self.new_map)))
    }
}

#[derive(Debug, Clone)] //, Deserialize, Serialize, Default)]
/// Collection of named items with ids and additional UNIQUE names given to items in function of their names
pub struct IdCollectionWithNames<'a, T: Clone> {
    id_collection: IdCollection<NamedItem<'a, T>>,
    unique_names: BTreeMap<Id, (&'a str, Id)>,
}

// in progress
impl<'a, T: Clone> From<IdCollection<NamedItem<'a, T>>> for IdCollectionWithNames<'a, T> {
    fn from(id_collection: IdCollection<NamedItem<'a, T>>) -> Self {
        let mut unique_names = BTreeMap::new();
        /*for (item_id, NamedItem(item_name, item)) in id_collection.iter() {
            let name_index = unique_names
                .into_iter()
                .filter(|(_, (name, _))| *name == item_name)
                .collect()
                .len();
            unique_names.insert(item_id, (item_name, name_index));
        }*/
        IdCollectionWithNames {
            id_collection: id_collection.clone(),
            unique_names,
        }
    }
}

pub trait IdManagerForNamedItems<'a> {
    /// Returns the id of one item with the given name if it exists
    fn get_id_by_name(self, name: &str) -> Option<Id>;
    fn get_name_by_id(self, id: Id) -> Option<&'a str>;
}

impl<'a, T: Clone> IdManagerForNamedItems<'a> for IdCollection<NamedItem<'a, T>> {
    fn get_id_by_name(self, name: &str) -> Option<Id> {
        for (k, v) in self.0.iter() {
            if v.0.eq(name) {
                return Some(k.clone());
            }
        }
        return None;
    }

    fn get_name_by_id(self, id: Id) -> Option<&'a str> {
        self.0.get(&id).map(|item| item.get_name())
    }
}

pub trait IdManagerMutForNamedItems<'a, 'b, 'c> {
    fn rename_by_id(&mut self, id: &Id, name: &'c str) -> Result<(), IdManagerError>;
}

impl<'a, 'b, 'c, T: Clone> IdManagerMutForNamedItems<'a, 'b, 'c>
    for IdCollectionMut<'a, NamedItem<'b, T>>
where
    'c: 'b,
{
    fn rename_by_id(&mut self, id: &Id, name: &'c str) -> Result<(), IdManagerError> {
        match self.get_mut(id) {
            Some(item) => {
                item.0 = name;
                Ok(())
            }
            _ => Err(IdManagerError::NoItemWithSuchId(Id(id.0))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::HelixParameters;
    use crate::NamedParameter;

    /*
        #[test]
        fn get_name_from_itemwithname_for_namedparameter() {
            let my_parameter = NamedParameter {
                name: "My Parameter Name",
                value: HelixParameters {
                    rise: 0.,
                    helix_radius: 0.,
                    bases_per_turn: 0.,
                    groove_angle: 0.,
                    inter_helix_gap: 0.,
                    inclination: 0.,
                },
            };
            assert_eq!("My Parameter Name", my_parameter.get_name())
    }*/
    #[test]
    fn get_name_of_nameditem() {
        let cat = NamedItem("Otto", "cat");
        let name = cat.get_name();
        assert_eq!("Otto", name);
    }
    #[test]
    fn get_name_of_arced_nameditem() {
        let cat = NamedItem("Otto", Arc::new("cat"));
        let name = cat.get_name();
        assert_eq!("Otto", name);
    }
    #[test]
    fn from_vec() {
        let myvec = vec![NamedItem("Bouftou", "mob")];
        let my_ided_collection = IdCollection::from_vec(myvec);
        assert_eq!(
            "IdCollection({Id(0): NamedItem(\"Bouftou\", \"mob\")})",
            format!("{:?}", my_ided_collection)
        );
    }
    #[test]
    fn get_id_of_named_if_it_exists() {
        let cat1 = NamedItem("Otto", "cat");
        let cat2 = NamedItem("Duchesse", "cat");
        let dog = NamedItem("Otto", "dog");
        let my_ided_collection = IdCollection::from_vec(vec![cat1, cat2, dog]);
        assert_eq!(Id(0), my_ided_collection.get_id_by_name("Otto").unwrap());
    }

    #[test]
    fn get_id_of_named_if_does_not_exist() {
        let cat1 = NamedItem("Otto", "cat");
        let my_ided_collection = IdCollection::from_vec(vec![cat1]);
        assert_eq!(None, my_ided_collection.get_id_by_name("Chachat"));
    }
    #[test]
    fn get_name_if_exists() {
        let cat1 = NamedItem("Otto", "cat");
        let my_ided_collection = IdCollection::from_vec(vec![cat1]);
        assert_eq!(Some("Otto"), my_ided_collection.get_name_by_id(Id(0)));
    }

    #[test]
    fn get_name_if_does_not_exist() {
        let cat1 = NamedItem("Otto", "cat");
        let my_ided_collection = IdCollection::from_vec(vec![cat1]);
        assert_eq!(None, my_ided_collection.get_name_by_id(Id(2)));
    }

    #[test]
    fn make_mut_and_push() {
        let cat = NamedItem("Snowball", "cat");
        let mut my_ided_collection = IdCollection::from_vec(vec![cat]);
        {
            let mut my_mut_ided_collection = my_ided_collection.make_mut();
            my_mut_ided_collection.push(NamedItem("Ember", "dog"));
        }
        assert_eq!("IdCollection({Id(0): NamedItem(\"Snowball\", \"cat\"), Id(1): NamedItem(\"Ember\", \"dog\")})", format!("{:?}", my_ided_collection));
    }

    #[test]
    fn make_mut_and_remove() {
        let cat = NamedItem("Snowball", "cat");
        let mut my_ided_collection = IdCollection::from_vec(vec![cat.clone(), cat]);
        {
            let mut my_mut_ided_collection = my_ided_collection.make_mut();
            my_mut_ided_collection.remove(&Id(0));
            my_mut_ided_collection.remove(&Id(1));
        }
        assert_eq!("IdCollection({})", format!("{:?}", my_ided_collection));
    }

    #[test]
    fn make_mut_and_rename() {
        let cat = NamedItem("Snowball", "cat");
        let mut my_ided_collection = IdCollection::from_vec(vec![cat.clone(), cat]);
        {
            let mut my_mut_ided_collection = my_ided_collection.make_mut();
            my_mut_ided_collection.rename_by_id(&Id(0), "Bouboule");
        }
        assert_eq!(
            "IdCollection({Id(0): NamedItem(\"Bouboule\", \"cat\"), Id(1): NamedItem(\"Snowball\", \"cat\")})"
,
            format!("{:?}", my_ided_collection)
        );
    }

    #[test]
    fn rename_when_id_not_found() {
        let cat = NamedItem("Snowball", "cat");
        let mut my_ided_collection = IdCollection::from_vec(vec![cat.clone(), cat]);

        let mut my_mut_ided_collection = my_ided_collection.make_mut();
        assert!(my_mut_ided_collection
            .rename_by_id(&Id(100), "Bouboule")
            .is_err());
    }
}

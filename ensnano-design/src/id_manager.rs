use core::fmt;
use std::ops::Deref;
use std::os::unix::prelude::OsStringExt;

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

#[derive(
    Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Default, Hash,
)]
/// Generic Identifier
pub struct Id(pub usize);

impl fmt::Display for Id {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone)] //, Deserialize, Serialize, Default)]
/// Collection of items with ids
struct IdCollectionInner<T: Clone>(BTreeMap<Id, Arc<T>>);

struct IdCollection<T: Clone>(pub(super) Arc<IdCollectionInner<T>>);

pub struct IdCollectionMut<'a, T: Clone> {
    source: &'a mut IdCollection<T>,
    new_map: IdCollectionInner<T>,
}

enum CollectionError<'a> {
    NoItemWithSuchId(Id),
    NoItemWithSuchName(&'a str),
}

impl<T: Clone> IdCollectionInner<T> {
    fn from_vec(vec: Vec<T>) -> Self {
        Self(
            vec.into_iter()
                .enumerate()
                .map(|(id, item)| (Id(id), Arc::new(item)))
                .collect(),
        )
    }

    fn push(&mut self, item: T) -> Id {
        let new_key = (self.0)
            .keys()
            .max()
            .map(|id| Id(id.0 + 1))
            .unwrap_or_default();
        (self.0).insert(new_key, Arc::new(item));
        new_key
    }

    fn remove(&mut self, id: &Id) -> Result<Arc<T>, CollectionError> {
        (self.0)
            .remove(&id)
            .ok_or(CollectionError::NoItemWithSuchId(Id(id.0)))
    }
}
/*
impl<T: Clone> IdCollection<T> {
    pub fn make_mut(&mut self) -> IdCollectionMut<T> {
        IdCollectionMut {
            new_map: IdCollectionInner(BTreeMap::clone((&self).0)), // TODO: regarder dans helices
            source: self,
        }
    }
    pub fn from_vec(vec: Vec<T>) -> Self {
        Self(Arc::new(IdCollectionInner::from_vec(vec)))
    }

    pub fn ptr_eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Default, Hash)]
///Item decorated with a name
pub struct NamedItem<'a, T>(&'a str, T);

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
        self.0
    }
}

impl<'a, T> ItemWithName<'a> for Arc<NamedItem<'a, T>> {
    fn get_name(&self) -> &'a str {
        self.0
    }
}

#[derive(Debug, Clone)]
struct UniqueName<'a> {
    name: &'a str,
    index: Id,
}

impl<'a> UniqueName<'a> {
    fn unique_name_string(&'a self) -> String {
        match &self.index {
            Id(0) => self.name.to_string(),
            _ => format!("{}_{}", self.name, self.index),
        }
    }
    fn name_string(&'a self) -> String {
        self.name.to_string()
    }
}

#[derive(Debug, Clone)] //, Deserialize, Serialize, Default)]

pub struct CollectionWithNames<'a, T: Clone>(pub(super) Arc<CollectionWithNamesInner<'a, T>>);

/// Collection of named items with ids and additional UNIQUE names given to items in function of their names
struct CollectionWithNamesInner<'a, T: Clone> {
    id_collection: IdCollectionInner<NamedItem<'a, T>>,
    unique_names: BTreeMap<Id, UniqueName<'a>>,
}

impl<'a, T: Clone> CollectionWithNames<'a, T> {
    fn get_unique_name(&self, id: Id) -> Option<String> {
        self.unique_names
            .get(&id)
            .map(|uname| uname.unique_name_string())
    }
    fn get_name(&self, id: Id) -> Option<String> {
        self.unique_names
            .get(&id)
            .map(|uname| format!("{}", uname.name_string()))
    }
    fn find_id_by_name(&self, name: &str) -> Option<Id> {
        for (id, uname) in self.unique_names.iter() {
            if uname.name.eq(name) {
                return Some(id.clone());
            }
        }
        return None;
    }
    fn from_vec(vec: Vec<NamedItem<'a, T>>) -> Self {
        let id_collection = IdCollection::from_vec(vec);
        CollectionWithNames::from(id_collection)
    }

    fn push(&mut self, item: NamedItem<'a, T>) {
        let item_id = self.id_collection.make_mut().push(item.clone());
        let item_name = item.get_name().clone();
        let name_index = self
            .unique_names
            .clone()
            .into_iter()
            .filter(|(_, uname)| uname.name == item_name)
            .collect::<Vec<_>>()
            .len();
        self.unique_names.insert(
            Id(item_id.0),
            UniqueName {
                name: item_name,
                index: Id(name_index),
            },
        );
    }
    fn rename(&mut self, id: &Id, new_name: &'c str) -> Result<(), IdManagerError> {
        match self.id_collection.get_mut(id) {
            Some(item) => {
                item.0 = name;
                Ok(())
            }
            _ => Err(IdManagerError::NoItemWithSuchId(Id(id.0))),
        }
    }
}

impl<'a, T: Clone> From<IdCollection<NamedItem<'a, T>>> for CollectionWithNames<'a, T> {
    fn from(id_collection: IdCollection<NamedItem<'a, T>>) -> Self {
        let mut unique_names: BTreeMap<Id, UniqueName> = BTreeMap::new();
        for (item_id, arc_item) in id_collection.0.iter() {
            let item_name = arc_item.0;
            let name_index = unique_names
                .clone()
                .into_iter()
                .filter(|(_, uname)| uname.name == item_name)
                .collect::<Vec<_>>()
                .len();
            unique_names.insert(
                Id(item_id.0),
                UniqueName {
                    name: item_name,
                    index: Id(name_index),
                },
            );
        }
        CollectionWithNames {
            id_collection: id_collection.clone(),
            unique_names: unique_names.clone(),
        }
    }
}

pub trait IdManagerMutForNamedItems<'a, 'b, 'c> {
    fn rename(&mut self, id: &Id, name: &'c str) -> Result<(), IdManagerError>;
}

impl<'a, 'b, 'c, T: Clone> IdManagerMutForNamedItems<'a, 'b, 'c>
    for IdCollectionMut<'a, NamedItem<'b, T>>
where
    'c: 'b,
{
    fn rename(&mut self, id: &Id, name: &'c str) -> Result<(), IdManagerError> {
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
        let collection = vec![NamedItem("Bouftou", "mob")];
        let collection = IdCollection::from_vec(myvec);
        assert_eq!(
            "IdCollection({Id(0): NamedItem(\"Bouftou\", \"mob\")})",
            format!("{:?}", collection)
        );
    }

    #[test]
    fn make_mut_and_push() {
        let cat = NamedItem("Snowball", "cat");
        let mut collection = IdCollection::from_vec(vec![cat]);
        {
            let mut collection = collection.make_mut();
            collection.push(NamedItem("Ember", "dog"));
        }
        assert_eq!("IdCollection({Id(0): NamedItem(\"Snowball\", \"cat\"), Id(1): NamedItem(\"Ember\", \"dog\")})", format!("{:?}", collection));
    }

    #[test]
    fn make_mut_and_remove() {
        let cat = NamedItem("Snowball", "cat");
        let mut collection = IdCollection::from_vec(vec![cat.clone(), cat]);
        {
            let mut collection = collection.make_mut();
            collection.remove(&Id(0));
            collection.remove(&Id(1));
        }
        assert_eq!("IdCollection({})", format!("{:?}", collection));
    }

    #[test]
    fn make_mut_and_rename() {
        let cat = NamedItem("Snowball", "cat");
        let mut collection = IdCollection::from_vec(vec![cat.clone(), cat]);
        {
            let mut collection = collection.make_mut();
            collection.rename(&Id(0), "Bouboule");
        }
        assert_eq!(
            "IdCollection({Id(0): NamedItem(\"Bouboule\", \"cat\"), Id(1): NamedItem(\"Snowball\", \"cat\")})"
,
            format!("{:?}", collection)
        );
    }

    #[test]
    fn rename_when_id_not_found() {
        let cat = NamedItem("Snowball", "cat");
        let mut collection = IdCollection::from_vec(vec![cat.clone(), cat]);

        let mut collection = collection.make_mut();
        assert!(collection.rename(&Id(100), "Bouboule").is_err());
    }

    #[test]
    fn id_collection_with_unique_names_from() {
        let cat = NamedItem("Pushok", ());

        let collection = IdCollection::from_vec(vec![cat.clone(), cat]);
        let collection = CollectionWithNames::from(collection);
        assert_eq!(
            "{Id(0): UniqueName { name: \"Pushok\", index: Id(0) }, Id(1): UniqueName { name: \"Pushok\", index: Id(1) }}"
,
            format!("{:?}", collection.unique_names)
        );
    }

    #[test]
    fn get_name_and_unique_name() {
        let cat = NamedItem("Pushok", ());
        let collection = IdCollection::from_vec(vec![cat.clone(), cat.clone()]);
        let collection = CollectionWithNames::from(collection);
        assert_eq!(
            "Some(\"Pushok_1\")",
            format!("{:?}", collection.get_unique_name(Id(1)))
        );
        assert_eq!("None", format!("{:?}", collection.get_unique_name(Id(2))));
        assert_eq!(
            "Some(\"Pushok\")",
            format!("{:?}", collection.get_name(Id(1)))
        );
        assert_eq!("None", format!("{:?}", collection.get_name(Id(2))));
    }

    #[test]
    fn id_by_name() {
        let cat = NamedItem("Pushok", ());
        let collection = CollectionWithNames::from_vec(vec![cat]);
        assert_eq!(Some(Id(0)), collection.find_id_by_name("Pushok"));
    }
    #[test]
    fn push() {
        let mut collection = CollectionWithNames::from_vec(vec![]);
        collection.push(NamedItem("Bob", ()));
        collection.push(NamedItem("Alice", ()));
        collection.push(NamedItem("Bob", ()));
        assert_eq!("Bob", collection.get_unique_name(Id(0)).unwrap().as_str());
        assert_eq!("Bob_1", collection.get_unique_name(Id(2)).unwrap().as_str());
    }
}
*/

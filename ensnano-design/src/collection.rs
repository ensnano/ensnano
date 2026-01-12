use std::{collections::BTreeMap, sync::Arc};

// pub trait Collection {
//     type Key;
//     type Item;
//     fn get(&self, id: &Self::Key) -> Option<&Self::Item>;
//     fn iter<'a>(&'a self) -> Box<dyn Iterator<Item = (&'a Self::Key, &'a Self::Item)> + 'a>;
//     fn values<'a>(&'a self) -> Box<dyn Iterator<Item = &'a Self::Item> + 'a>;
//     fn keys<'a>(&'a self) -> Box<dyn Iterator<Item = &'a Self::Key> + 'a>;
//     fn is_empty(&self) -> bool;
//     fn len(&self) -> usize;
// }

pub trait HasMap {
    type Key: Ord + Eq;
    type Item;
    fn get_map(&self) -> &BTreeMap<Self::Key, Arc<Self::Item>>;
}

// impl<T> Collection for T
// where
//     T: HasMap,
// {
//     type Key = <T as HasMap>::Key;
//     type Item = <T as HasMap>::Item;

//     fn get(&self, id: &T::Key) -> Option<&Self::Item> {
//         self.get_map().get(id).map(AsRef::as_ref)
//     }

//     fn iter<'a>(&'a self) -> Box<dyn Iterator<Item = (&'a Self::Key, &'a Self::Item)> + 'a> {
//         Box::new(self.get_map().iter().map(|(id, arc)| (id, arc.as_ref())))
//     }

//     fn keys<'a>(&'a self) -> Box<dyn Iterator<Item = &'a Self::Key> + 'a> {
//         Box::new(self.get_map().keys())
//     }

//     fn values<'a>(&'a self) -> Box<dyn Iterator<Item = &'a Self::Item> + 'a> {
//         Box::new(self.get_map().values().map(AsRef::as_ref))
//     }

//     fn is_empty(&self) -> bool {
//         self.get_map().is_empty()
//     }

//     fn len(&self) -> usize {
//         self.get_map().len()
//     }
// }

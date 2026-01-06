use ahash::RandomState;
use std::{collections::HashMap, hash::Hash};

#[derive(Clone, Debug)]
pub(crate) struct IdGenerator<K: Eq + Hash + Clone> {
    next_id: usize,
    ids: HashMap<K, usize, RandomState>,
    elements: HashMap<usize, K, RandomState>,
}

impl<K: Eq + Hash + Clone> Default for IdGenerator<K> {
    fn default() -> Self {
        Self {
            next_id: 0,
            ids: Default::default(),
            elements: Default::default(),
        }
    }
}

impl<K: Eq + Hash + Clone> IdGenerator<K> {
    pub(super) fn insert(&mut self, key: K) -> usize {
        let ret = self.next_id;
        self.elements.insert(self.next_id, key.clone());
        self.ids.insert(key, self.next_id);
        self.next_id += 1;
        ret
    }

    pub(super) fn insert_at(&mut self, key: K, id: usize) {
        self.elements.insert(id, key.clone());
        self.ids.insert(key, id);
        self.next_id = self.next_id.max(id + 1);
    }

    pub(super) fn get_element(&self, id: usize) -> Option<K> {
        self.elements.get(&id).cloned()
    }

    pub(super) fn get_id(&self, element: &K) -> Option<usize> {
        self.ids.get(element).copied()
    }

    #[cfg(test)]
    pub(super) fn remove(&mut self, id: usize) {
        let elt = self.get_element(id).expect("Removing non-existent id");
        self.ids.remove(&elt);
        self.elements.remove(&id);
    }

    #[cfg(test)]
    pub(super) fn is_empty(&self) -> bool {
        self.ids.is_empty() && self.elements.is_empty()
    }

    pub(super) fn get_all_elements(&self) -> Vec<(usize, K)> {
        self.elements.clone().into_iter().collect()
    }

    pub(super) fn copy_next_id_to(&self, next: &mut Self) {
        next.next_id = self.next_id;
    }
}

use {
    ahash::RandomState,
    std::{collections::HashMap, hash::Hash},
};

#[derive(Clone, Debug)]
pub struct IdGenerator<K: Eq + Hash + Clone> {
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
    pub fn insert(&mut self, key: K) -> usize {
        let ret = self.next_id;
        self.elements.insert(self.next_id, key.clone());
        self.ids.insert(key, self.next_id);
        self.next_id += 1;
        ret
    }

    pub fn insert_at(&mut self, key: K, id: usize) {
        self.elements.insert(id, key.clone());
        self.ids.insert(key, id);
        self.next_id = self.next_id.max(id + 1);
    }

    pub fn get_element(&self, id: usize) -> Option<K> {
        self.elements.get(&id).cloned()
    }

    pub fn get_id(&self, element: &K) -> Option<usize> {
        self.ids.get(element).copied()
    }

    #[cfg(test)]
    pub fn remove(&mut self, id: usize) {
        let elt = self.get_element(id).expect("Removing non-existent id");
        self.ids.remove(&elt);
        self.elements.remove(&id);
    }

    #[cfg(test)]
    pub fn is_empty(&self) -> bool {
        self.ids.is_empty() && self.elements.is_empty()
    }

    pub fn get_all_elements(&self) -> Vec<(usize, K)> {
        self.elements.clone().into_iter().collect()
    }

    pub fn copy_next_id_to(&self, next: &mut Self) {
        next.next_id = self.next_id;
    }
}

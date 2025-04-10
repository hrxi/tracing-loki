use std::collections::hash_map;
use std::collections::HashMap;

#[derive(Default, Debug)]
pub struct LabelMap<T> {
    map: HashMap<String, T>,
}

impl<T> LabelMap<T> {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn get_or_insert(&mut self, key: &str, builder: impl FnOnce() -> T) -> &mut T {
        // Due to borrow checker limitations around &mut, this has to either accept a cloned String
        // or do a double lookup, which is what we do here, with the expectation that it
        // ends up better than the allocation.
        if !self.map.contains_key(key) {
            self.map.insert(key.to_owned(), builder());
        }

        self.map.get_mut(key).unwrap()
    }

    pub fn values(&self) -> hash_map::Values<'_, String, T> {
        self.map.values()
    }

    pub fn values_mut(&mut self) -> hash_map::ValuesMut<'_, String, T> {
        self.map.values_mut()
    }
}

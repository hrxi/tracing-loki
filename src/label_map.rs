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

    pub fn get_or_insert(&mut self, key: String, builder: impl FnOnce() -> T) -> &mut T {
        // Due to borrow checker limitations around &mut, this has to accept a String
        // instead of a &str. I tried.
        self.map.entry(key).or_insert_with(builder)
    }

    pub fn values(&self) -> hash_map::Values<'_, String, T> {
        self.map.values()
    }

    pub fn values_mut(&mut self) -> hash_map::ValuesMut<'_, String, T> {
        self.map.values_mut()
    }
}

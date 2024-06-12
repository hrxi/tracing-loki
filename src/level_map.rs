use std::fmt;
use std::ops;
use std::slice;
use tracing_core::Level;

#[derive(Default)]
pub struct LevelMap<T> {
    map: [T; 5],
}

fn level_index(level: Level) -> usize {
    match level {
        Level::TRACE => 0,
        Level::DEBUG => 1,
        Level::INFO => 2,
        Level::WARN => 3,
        Level::ERROR => 4,
    }
}

impl<T> LevelMap<T> {
    pub fn from_fn<F: FnMut(Level) -> T>(mut f: F) -> LevelMap<T> {
        LevelMap {
            map: [
                f(Level::TRACE),
                f(Level::DEBUG),
                f(Level::INFO),
                f(Level::WARN),
                f(Level::ERROR),
            ],
        }
    }
    pub fn values(&self) -> slice::Iter<'_, T> {
        self.map.iter()
    }
    pub fn values_mut(&mut self) -> slice::IterMut<'_, T> {
        self.map.iter_mut()
    }
}

impl<T: fmt::Debug> fmt::Debug for LevelMap<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_map()
            .entry(&Level::TRACE, &self[Level::TRACE])
            .entry(&Level::DEBUG, &self[Level::DEBUG])
            .entry(&Level::INFO, &self[Level::INFO])
            .entry(&Level::WARN, &self[Level::WARN])
            .entry(&Level::ERROR, &self[Level::ERROR])
            .finish()
    }
}

impl<T> ops::Index<Level> for LevelMap<T> {
    type Output = T;
    fn index(&self, index: Level) -> &T {
        &self.map[level_index(index)]
    }
}

impl<T> ops::IndexMut<Level> for LevelMap<T> {
    fn index_mut(&mut self, index: Level) -> &mut T {
        &mut self.map[level_index(index)]
    }
}

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
    pub fn try_from_fn<E, F: FnMut(Level) -> Result<T, E>>(mut f: F) -> Result<LevelMap<T>, E> {
        Ok(LevelMap {
            map: [
                f(Level::TRACE)?,
                f(Level::DEBUG)?,
                f(Level::INFO)?,
                f(Level::WARN)?,
                f(Level::ERROR)?,
            ],
        })
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

struct LevelIter {
    next: Option<Level>,
}

impl Iterator for LevelIter {
    type Item = Level;
    fn next(&mut self) -> Option<Level> {
        let result = self.next;
        self.next = self.next.and_then(|l| {
            Some(match l {
                Level::TRACE => Level::DEBUG,
                Level::DEBUG => Level::INFO,
                Level::INFO => Level::WARN,
                Level::WARN => Level::ERROR,
                Level::ERROR => return None,
            })
        });
        result
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = match self.next {
            Some(Level::TRACE) => 5,
            Some(Level::DEBUG) => 4,
            Some(Level::INFO) => 3,
            Some(Level::WARN) => 2,
            Some(Level::ERROR) => 1,
            None => 0,
        };
        (len, Some(len))
    }
}

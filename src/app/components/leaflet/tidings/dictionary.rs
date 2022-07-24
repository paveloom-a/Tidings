//! Dictionary of (index, tidings) key-value pairs

use generational_arena::Index;
use std::collections::{hash_map::Entry, HashMap};
use std::hash::BuildHasherDefault;
use wyhash::WyHash;

/// A vector of tidings
pub type Tidings = Vec<Tiding>;

/// Tiding
pub struct Tiding {
    /// Title
    pub title: String,
}

/// Hash map type
type HashMapType = HashMap<Index, Tidings, BuildHasherDefault<WyHash>>;

/// Dictionary of (index, tidings) key-value pairs
pub(super) struct Dictionary {
    /// Inner hash map
    hash_map: HashMapType,
}

impl Dictionary {
    /// Initialize a dictionary
    pub(super) fn new() -> Self {
        let hash_map = HashMapType::default();
        Self { hash_map }
    }
    /// Insert a key-value pair into the dictionary
    pub(super) fn insert(&mut self, index: Index, tidings: Tidings) {
        match self.hash_map.entry(index) {
            Entry::Occupied(v) => {
                *v.into_mut() = tidings;
            }
            Entry::Vacant(v) => {
                v.insert(tidings);
            }
        }
    }
    /// Get tidings from the index
    pub(super) fn get(&self, index: &Index) -> Option<&[Tiding]> {
        self.hash_map.get(index).map(std::vec::Vec::as_slice)
    }
}

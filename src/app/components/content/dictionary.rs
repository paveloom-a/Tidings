//! Dictionary of (URL, tidings) key-value pairs

use std::collections::{hash_map::Entry, HashMap};
use std::hash::BuildHasherDefault;
use wyhash::WyHash;

use super::tiding::Model as Tiding;

/// Hash map type
type HashMapType = HashMap<String, Vec<Tiding>, BuildHasherDefault<WyHash>>;

/// Dictionary of (Feed URL, tidings) key-value pairs
pub(super) struct Dictionary {
    /// Inner hash map
    hash_map: HashMapType,
}

impl Dictionary {
    /// Initialize a dictionary
    pub(super) fn new() -> Self {
        Self {
            hash_map: HashMapType::default(),
        }
    }
    /// Insert a key-value pair into the dictionary
    pub(super) fn insert(&mut self, url: String, tidings: Vec<Tiding>) {
        match self.hash_map.entry(url) {
            Entry::Occupied(v) => {
                *v.into_mut() = tidings;
            }
            Entry::Vacant(v) => {
                v.insert(tidings);
            }
        }
    }
    /// Get tidings from the Feed URL
    pub(super) fn get(&self, url: &str) -> Option<&[Tiding]> {
        self.hash_map.get(url).map(std::vec::Vec::as_slice)
    }
}

//! Tree of feeds

use generational_arena::{Arena, Index};

use super::Item;

/// An alias type for a vector of (index, URL) pairs of the feeds
pub(in crate::app::components::leaflet) type IndicesUrls = Vec<(Index, Box<str>)>;

/// Node in the tree of feeds
pub enum Node {
    /// Feed
    Feed {
        /// Label
        label: String,
        /// URL
        url: String,
        /// Is the feed in the process of being updated?
        updating: bool,
    },
    /// Directory
    Directory {
        /// Label
        label: String,
        /// Children
        children: Vec<Index>,
        /// Parent
        parent: Option<Index>,
    },
}

impl Node {
    /// Create a new feed
    pub(in crate::app) fn new_feed(label: String, url: String) -> Self {
        Node::Feed {
            label,
            url,
            updating: false,
        }
    }
    /// Create a new directory
    pub(in crate::app) fn new_directory(label: String) -> Self {
        Node::Directory {
            label,
            children: vec![],
            parent: None,
        }
    }
}

/// Tree of feeds
pub(super) struct Tree {
    /// Arena
    arena: Arena<Node>,
    /// Index of the current root of the subtree
    pub(super) current: Index,
}

impl Default for Tree {
    fn default() -> Self {
        // Prepare an arena with a root node
        let mut arena = Arena::with_capacity(1);
        let node = Node::new_directory("Root".to_owned());
        let current = arena.insert(node);
        // Initialize the tree
        Self { arena, current }
    }
}

impl Tree {
    /// Insert an item into the tree, return the index
    pub(super) fn insert(&mut self, parent_index: Index, mut node: Node) -> Option<Index> {
        // If the specified parent actually exists
        self.arena.get(parent_index).is_some().then(|| {
            // If the node is a directory
            if let Node::Directory { ref mut parent, .. } = node {
                // Add the index of the parent to the node
                *parent = Some(parent_index);
            }
            // Insert the node into the arena
            let child_index = self.arena.insert(node);
            // If able to access the children of the parent node
            if let Some(&mut Node::Directory {
                ref mut children, ..
            }) = self.arena.get_mut(parent_index)
            {
                // Add the index of the node to the
                // list of children of the parent
                children.push(child_index);
            }
            // Return the index of the child
            child_index
        })
    }
    /// Get a reference to the current node
    fn current(&self) -> Option<&Node> {
        self.arena.get(self.current)
    }
    /// Get the index of the parent of the current node
    fn current_parent_index(&self) -> Option<Index> {
        if let Some(&Node::Directory { parent, .. }) = self.current() {
            return parent;
        }
        None
    }
    /// Get the vector of indices of the children of the current node
    fn current_children_indices(&self) -> Option<&Vec<Index>> {
        if let Some(&Node::Directory { ref children, .. }) = self.current() {
            Some(children)
        } else {
            None
        }
    }
    /// Return `true` if the root of the current subtree
    /// coincides with the root of the whole tree
    pub(super) fn is_root(&self) -> bool {
        self.current_parent_index().is_none()
    }
    /// Go one level up in the tree
    pub(super) fn back(&mut self) {
        // Replace the index of the current root of
        // the subtree with the index of the parent
        // of the current node
        if let Some(index) = self.current_parent_index() {
            self.current = index;
        }
    }
    /// Enter the directory, going one level down in the tree
    pub(super) fn enter_dir(&mut self, position: usize) {
        // Get the indices of the children
        if let Some(children_indices) = self.current_children_indices() {
            // Get the index of the child at the provided position
            if let Some(&index) = children_indices.get(position) {
                // Replace the index of the current root
                // of the subtree with the above index
                self.current = index;
            }
        }
    }
    /// Cast nodes at the top level of the current subtree
    /// to store items, collect and return them in a vector
    pub(super) fn items(&self) -> Option<Vec<Item>> {
        // Get the indices of the children
        if let Some(children_indices) = self.current_children_indices() {
            // For each index of a child
            children_indices
                .iter()
                .map(|&index| {
                    // Get the node
                    if let Some(node) = self.arena.get(index) {
                        // Cast the node to the object
                        if let Some(mut item) = Option::<Item>::from(node) {
                            // If that's successful, set the index of the item
                            item.set_index(index);
                            // Return the result
                            Some(item)
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .collect()
        } else {
            None
        }
    }
    /// Return a list of (index, URL) pairs of the feeds
    pub(super) fn indices_urls(&self) -> IndicesUrls {
        self.arena
            .iter()
            .filter_map(|(index, node)| match *node {
                Node::Directory { .. } => None,
                Node::Feed { ref url, .. } => Some((index, url.clone().into_boxed_str())),
            })
            .collect()
    }
    /// Set the updating status of one of the items in the tree,
    pub(super) fn set_updating(&mut self, index: Index, updating: bool) {
        if let Some(&mut Node::Feed {
            updating: ref mut inner_updating,
            ..
        }) = self.arena.get_mut(index)
        {
            *inner_updating = updating;
        }
    }
}

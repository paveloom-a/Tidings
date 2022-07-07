//! Tree of feeds

use generational_arena::{Arena, Index};

use super::Item;

/// Node in the tree of feeds
pub(super) enum Node {
    /// Directory
    Directory {
        /// Label
        label: String,
        /// Children
        children: Vec<Index>,
        /// Parent
        parent: Option<Index>,
    },
    /// Feed
    Feed {
        /// Label
        label: String,
    },
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
        let node = Node::Directory {
            label: String::from("Root"),
            children: vec![],
            parent: None,
        };
        let current = arena.insert(node);
        // Initialize the tree
        Self { arena, current }
    }
}

impl Tree {
    /// Insert an item in the tree, return the index
    pub(super) fn insert(&mut self, parent_index: Index, node: Node) -> Option<Index> {
        // If the specified parent actually exists
        if self.arena.get(parent_index).is_some() {
            // Insert the node into the arena
            let child_index = self.arena.insert(node);
            // If the parent is a node with children
            if let Some(&mut Node::Directory {
                ref mut children, ..
            }) = self.arena.get_mut(parent_index)
            {
                // Connect the parent and the child
                children.push(child_index);
            }
            // Return the index of the child
            return Some(child_index);
        }
        None
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
            // Prepare a vector for the store items
            let mut vec = vec![];
            // For each index of a child
            for &index in children_indices {
                // Cast the node to the object
                // and, if that's successful, push
                match self.arena.get(index) {
                    Some(&Node::Directory { ref label, .. }) => {
                        if let Ok(feed) = Item::new(true, label) {
                            vec.push(feed);
                        } else {
                            log::error!("Couldn't create a new feed");
                        }
                    }
                    Some(&Node::Feed { ref label, .. }) => {
                        if let Ok(feed) = Item::new(false, label) {
                            vec.push(feed);
                        } else {
                            log::error!("Couldn't create a new feed");
                        }
                    }
                    _ => (),
                }
            }
            return Some(vec);
        }
        None
    }
}

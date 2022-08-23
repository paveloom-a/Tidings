//! Source

#![allow(clippy::module_name_repetitions)]

use adw::prelude::{ActionRowExt, PreferencesRowExt};
use generational_arena::{Arena, Index};
use gtk::prelude::ListBoxRowExt;
use relm4::factory::{DynamicIndex, FactoryComponent, FactoryComponentSender};

use std::collections::{HashMap, HashSet};
use std::hash::BuildHasherDefault;
use wyhash::WyHash;

/// A type alias to the dictionary of the (URL, Vec<Index>) key-value pairs
pub(super) type URLsMap = HashMap<String, Vec<Index>, BuildHasherDefault<WyHash>>;

/// Arena source
#[derive(Debug, Clone)]
pub enum ArenaSource {
    /// Feed
    Feed {
        /// Title
        title: String,
        /// URL
        url: String,
        /// Is the feed in the process of being updated?
        updating: bool,
        /// Arena index of the parent directory
        parent_index: Index,
    },
    /// Directory
    Directory {
        /// Title
        title: String,
        /// Children arena indices
        children: Vec<Index>,
        /// Arena index of the parent directory
        parent_index: Index,
    },
    /// Root directory
    RootDirectory {
        /// Children (counting recursively down)
        children: Vec<Index>,
    },
}

impl ArenaSource {
    /// Push the index to the children
    pub(super) fn push_to_children(&mut self, index: Index) {
        match *self {
            Self::Directory {
                ref mut children, ..
            }
            | Self::RootDirectory { ref mut children } => {
                children.push(index);
            }
            Self::Feed { .. } => {}
        }
    }
    /// Get the URLs from the source recursively
    pub(super) fn urls(&self, arena: &Arena<ArenaSource>) -> HashSet<String> {
        match *self {
            // If it's a feed, return its URL
            Self::Feed { ref url, .. } => {
                // Create a new hash set
                let mut urls = HashSet::default();
                // Insert the URL into the hash set
                urls.insert(url.clone());
                // Return the hash set
                urls
            }
            // But if it's a directory,
            Self::Directory { ref children, .. } | Self::RootDirectory { ref children, .. } => {
                // Iterate over all children, collecting all their URLs
                children
                    .iter()
                    .fold(HashSet::new(), |mut acc, child_index| {
                        // If the child with this index still exists
                        if let Some(child) = arena.get(*child_index) {
                            // Extend the accumulator with their URL(s)
                            acc.extend(child.urls(arena));
                        }
                        acc
                    })
            }
        }
    }
    /// Get a dictionary of (URL, Vec<Index>) key-value pairs recursively
    pub(super) fn urls_map(&self, index: Index, arena: &Arena<ArenaSource>) -> URLsMap {
        match *self {
            // If it's a feed
            Self::Feed { ref url, .. } => {
                // Create a new hash map
                let mut urls_map = URLsMap::default();
                // Insert an entry
                urls_map.entry(url.clone()).or_insert_with(|| vec![index]);
                // Return the hash map
                urls_map
            }
            // But if it's a directory,
            Self::Directory { ref children, .. } | Self::RootDirectory { ref children, .. } => {
                // Iterate over all children, collecting all their (Index, URL) pairs
                children
                    .iter()
                    .fold(URLsMap::default(), |mut acc, child_index| {
                        // If the child with this index still exists
                        if let Some(child) = arena.get(*child_index) {
                            // Get the child hash map
                            let urls_map = child.urls_map(*child_index, arena);
                            // Merge the hash maps
                            for (key, child_vec) in urls_map {
                                acc.entry(key)
                                    .and_modify(|vec| vec.extend_from_slice(&child_vec))
                                    .or_insert(child_vec);
                            }
                        }
                        acc
                    })
            }
        }
    }
    /// Get the children of directory
    pub(super) fn children(&self) -> Option<&Vec<Index>> {
        match *self {
            Self::Directory { ref children, .. } | Self::RootDirectory { ref children, .. } => {
                Some(children)
            }
            Self::Feed { .. } => None,
        }
    }
    /// Get the title of the source
    pub(super) fn title(&self) -> String {
        match *self {
            Self::Directory { ref title, .. } => title.clone(),
            Self::Feed { .. } | Self::RootDirectory { .. } => String::from(""),
        }
    }
    /// Get the index of the parent directory
    pub(super) fn parent_index(&self) -> Option<&Index> {
        match *self {
            Self::Feed {
                ref parent_index, ..
            }
            | Self::Directory {
                ref parent_index, ..
            } => Some(parent_index),
            Self::RootDirectory { .. } => None,
        }
    }
    /// Is the source a directory?
    pub(super) fn is_dir(&self) -> bool {
        matches!(self, &Self::Directory { .. } | &Self::RootDirectory { .. })
    }
    /// Is this source a child (not necessarily a direct one) of the specified source?
    pub(super) fn is_child_of(&self, index: &Index, arena: &Arena<ArenaSource>) -> bool {
        // If this source has a parent
        if let Some(parent_index) = self.parent_index() {
            // If the parent still exists
            if let Some(parent) = arena.get(*parent_index) {
                // If the indexes match
                if index == parent_index {
                    true
                // Otherwise, try one more level
                } else {
                    parent.is_child_of(index, arena)
                }
            // Otherwise,
            } else {
                false
            }
        // Otherwise, there are no parents left, so
        } else {
            false
        }
    }
    /// Set the updating status of the source
    pub(super) fn set_updating(&mut self, status: bool) {
        match *self {
            Self::Feed {
                ref mut updating, ..
            } => {
                *updating = status;
            }
            Self::Directory { .. } | Self::RootDirectory { .. } => {}
        }
    }
    /// Create a new feed source
    pub(super) fn new_feed(title: String, url: String, parent_index: Index) -> Self {
        Self::Feed {
            title,
            url,
            updating: false,
            parent_index,
        }
    }
    /// Create a new directory source
    pub(super) fn new_directory(title: String, parent_index: Index) -> Self {
        Self::Directory {
            title,
            children: vec![],
            parent_index,
        }
    }
    /// Create a new root source
    pub(super) fn new_root() -> Self {
        Self::RootDirectory { children: vec![] }
    }
    /// Convert the arena source to the list source with the arena index
    pub(super) fn into_list_source(self, index: Index) -> Option<ListSource> {
        match self {
            Self::Feed {
                title,
                url,
                updating,
                parent_index,
            } => Some(ListSource::Feed {
                title,
                url,
                updating,
                parent_index,
                index,
            }),
            Self::Directory {
                title,
                children,
                parent_index,
            } => Some(ListSource::Directory {
                title,
                children,
                parent_index,
                index,
            }),
            Self::RootDirectory { .. } => None,
        }
    }
}

/// List source
#[derive(Debug, Clone)]
pub enum ListSource {
    /// Feed
    Feed {
        /// Title
        title: String,
        /// URL
        url: String,
        /// Is the feed in the process of being updated?
        updating: bool,
        /// Arena index of the parent directory
        parent_index: Index,
        /// Arena index of the source
        index: Index,
    },
    /// Directory
    Directory {
        /// Title
        title: String,
        /// Children arena indices
        children: Vec<Index>,
        /// Arena index of the parent directory
        parent_index: Index,
        /// Arena index of the source
        index: Index,
    },
}

impl ListSource {
    /// Get the title of the source
    fn title(&self) -> &str {
        match *self {
            Self::Feed { ref title, .. } | Self::Directory { ref title, .. } => title,
        }
    }
    /// Get the index of the source
    pub(super) fn index(&self) -> &Index {
        match *self {
            Self::Feed { ref index, .. } | Self::Directory { ref index, .. } => index,
        }
    }
    /// Is the source a directory?
    pub(super) fn is_dir(&self) -> bool {
        matches!(self, &Self::Directory { .. })
    }
}

/// Messages
#[derive(Debug)]
pub enum Msg {}

#[allow(clippy::missing_docs_in_private_items)]
#[relm4::factory(pub)]
impl FactoryComponent for ListSource {
    type CommandOutput = ();
    type Init = ListSource;
    type Input = Msg;
    type Output = super::Msg;
    type ParentMsg = super::Msg;
    type ParentWidget = gtk::ListBox;
    type Widgets = Widgets;
    view! {
        // Action Row
        adw::ActionRow {
            #[watch]
            set_title: self.title(),
            set_activatable: true,
            // Favicon
            add_prefix = &gtk::Image {
                set_icon_name: if self.is_dir() {
                    Some("inode-directory-symbolic")
                } else {
                    Some("emblem-shared-symbolic")
                },
            },
        }
    }
    fn init_model(
        source: Self::Init,
        _index: &DynamicIndex,
        _sender: FactoryComponentSender<Self>,
    ) -> Self {
        // The callers should construct the variants themselves
        source
    }
    fn update(&mut self, msg: Self::Input, _sender: FactoryComponentSender<Self>) {
        match msg {}
    }
    fn output_to_parent_msg(output: Self::Output) -> Option<super::Msg> {
        Some(output)
    }
}

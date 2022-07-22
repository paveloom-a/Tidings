//! List of items

use generational_arena::Index;
use gtk::glib::{ParamFlags, ParamSpec, ParamSpecBoolean, ParamSpecString, ParamSpecUInt64, Value};
use gtk::prelude::{ListModelExt, ObjectExt, StaticType, ToValue};
use gtk::subclass::prelude::{ObjectImpl, ObjectSubclass};
use gtk::{gio, glib};
use once_cell::sync::Lazy;

use std::cell::{Cell, RefCell};

use super::{Node, Tree};

/// Object holding the state
#[derive(Default)]
pub struct GItem {
    /// Index
    index: Cell<u64>,
    /// Generation
    generation: Cell<u64>,
    /// Label
    label: RefCell<String>,
    /// Is the feed is a directory?
    is_dir: Cell<bool>,
}

#[glib::object_subclass]
impl ObjectSubclass for GItem {
    const NAME: &'static str = "Item";
    type Type = Item;
    type ParentType = glib::Object;
}

impl ObjectImpl for GItem {
    fn properties() -> &'static [ParamSpec] {
        /// Properties
        static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
            vec![
                // Index
                ParamSpecUInt64::new(
                    // Name
                    "index",
                    // Nickname
                    "index",
                    // Short description
                    "index",
                    // Minimum value
                    0,
                    // Maximum value
                    std::u64::MAX,
                    // Default value
                    0,
                    // Flags
                    ParamFlags::READWRITE,
                ),
                // Generation
                ParamSpecUInt64::new(
                    // Name
                    "generation",
                    // Nickname
                    "generation",
                    // Short description
                    "generation",
                    // Minimum value
                    0,
                    // Maximum value
                    std::u64::MAX,
                    // Default value
                    0,
                    // Flags
                    ParamFlags::READWRITE,
                ),
                // Label
                ParamSpecString::new(
                    // Name
                    "label",
                    // Nickname
                    "label",
                    // Short description
                    "label",
                    // Default value
                    None,
                    // Flags
                    ParamFlags::READWRITE,
                ),
                // Is the feed a directory?
                ParamSpecBoolean::new(
                    // Name
                    "is-dir",
                    // Nickname
                    "is-dir",
                    // Short description
                    "is-dir",
                    // Default value
                    false,
                    // Flags
                    ParamFlags::READWRITE,
                ),
            ]
        });
        &PROPERTIES
    }
    fn set_property(&self, _obj: &Self::Type, _id: usize, value: &Value, pspec: &ParamSpec) {
        match pspec.name() {
            "index" => {
                if let Ok(index) = value.get() {
                    self.index.replace(index);
                }
            }
            "generation" => {
                if let Ok(generation) = value.get() {
                    self.generation.replace(generation);
                }
            }
            "label" => {
                if let Ok(label) = value.get() {
                    self.label.replace(label);
                }
            }
            "is-dir" => {
                if let Ok(is_dir) = value.get() {
                    self.is_dir.replace(is_dir);
                }
            }
            _ => (),
        }
    }
    fn property(&self, _obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> Value {
        match pspec.name() {
            "index" => self.index.get().to_value(),
            "generation" => self.generation.get().to_value(),
            "label" => self.label.borrow().to_value(),
            "is-dir" => self.is_dir.get().to_value(),
            _ => "".to_value(),
        }
    }
}

glib::wrapper! {
    /// Item in the list
    pub struct Item(ObjectSubclass<GItem>);
}

impl Item {
    /// Create a new feed
    pub(super) fn new(index: u64, generation: u64, label: &str, is_dir: bool) -> Option<Self> {
        glib::Object::new(&[
            ("index", &index),
            ("generation", &generation),
            ("label", &label.to_owned()),
            ("is-dir", &is_dir),
        ])
        .ok()
    }
    /// Return `true` if the feed is a directory
    pub(super) fn is_dir(&self) -> bool {
        self.property("is-dir")
    }
    /// Get a generational arena's index
    pub(super) fn index(&self) -> Option<Index> {
        // Get the raw parts
        let index: u64 = self.property("index");
        let generation: u64 = self.property("generation");
        // Return the full index
        if let Ok(index) = usize::try_from(index) {
            Some(Index::from_raw_parts(index, generation))
        } else {
            None
        }
    }
    /// Set raw parts of the generational arena's index
    pub(super) fn set_index(&mut self, index: Index) {
        // Convert the index into its raw parts
        let (index, generation) = index.into_raw_parts();
        // Set the properties
        if let Ok(index) = u64::try_from(index) {
            self.set_property("index", index);
            self.set_property("generation", generation);
        }
    }
}

impl From<&Node> for Option<Item> {
    fn from(node: &Node) -> Self {
        match *node {
            // Note that the index and the generation are set to zero.
            // These should be changed appropriately after inserting the node
            Node::Directory { ref label, .. } => Item::new(0, 0, label, true),
            Node::Feed { ref label, .. } => Item::new(0, 0, label, false),
        }
    }
}

/// List of items
pub(super) struct List {
    /// List Store
    pub(super) store: gio::ListStore,
}

impl List {
    /// Initialize a list
    pub(super) fn new() -> Self {
        Self {
            store: gio::ListStore::new(Item::static_type()),
        }
    }
    /// Append an item to the list
    pub(super) fn append(&mut self, item: &Item) {
        self.store.append(item);
    }
    /// Update the list with the
    /// items of the current subtree
    pub(super) fn update(&mut self, tree: &Tree) {
        if let Some(items) = tree.items() {
            // Replace the current list with the new list
            self.store.splice(0, self.store.n_items(), &items);
        }
    }
}

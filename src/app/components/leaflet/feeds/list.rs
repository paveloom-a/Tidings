//! List of items

use gtk::glib::{ParamFlags, ParamSpec, ParamSpecBoolean, ParamSpecString, Value};
use gtk::prelude::{ListModelExt, ObjectExt, ToValue};
use gtk::subclass::prelude::{ObjectImpl, ObjectSubclass};
use gtk::{gio, glib};
use once_cell::sync::Lazy;

use std::cell::{Cell, RefCell};

use super::{Node, Tree};

/// List of items
pub(super) type List = gio::ListStore;

/// Object holding the state
#[derive(Default)]
pub struct GItem {
    /// Is the feed is a directory?
    is_dir: Cell<bool>,
    /// Label
    label: RefCell<String>,
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
                // Label
                ParamSpecString::new(
                    // Name
                    "label",
                    // Nickname
                    "label",
                    // Short description
                    "label",
                    // Default value
                    Some(""),
                    // Flags
                    ParamFlags::READWRITE,
                ),
            ]
        });
        &PROPERTIES
    }
    fn set_property(&self, _obj: &Self::Type, _id: usize, value: &Value, pspec: &ParamSpec) {
        match pspec.name() {
            "is-dir" => {
                if let Ok(is_dir) = value.get() {
                    self.is_dir.replace(is_dir);
                }
            }
            "label" => {
                if let Ok(label) = value.get() {
                    self.label.replace(label);
                }
            }
            _ => (),
        }
    }
    fn property(&self, _obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> Value {
        match pspec.name() {
            "is-dir" => self.is_dir.get().to_value(),
            "label" => self.label.borrow().to_value(),
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
    pub(super) fn new(is_dir: bool, label: &str) -> Option<Self> {
        glib::Object::new(&[("is-dir", &is_dir), ("label", &label.to_owned())]).ok()
    }
    /// Return `true` if the feed is a directory
    pub(super) fn is_dir(&self) -> bool {
        self.property("is-dir")
    }
}

impl From<&Node> for Option<Item> {
    fn from(node: &Node) -> Self {
        match *node {
            Node::Directory { ref label, .. } => Item::new(true, label),
            Node::Feed { ref label, .. } => Item::new(false, label),
        }
    }
}

/// Update the list
pub(super) trait UpdateList {
    /// Update the list with the
    /// items of the current subtree
    fn update(&mut self, tree: &Tree);
}

impl UpdateList for List {
    fn update(&mut self, tree: &Tree) {
        if let Some(items) = tree.items() {
            // Replace the current list with the new list
            self.splice(0, self.n_items(), &items);
        }
    }
}

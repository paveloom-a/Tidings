//! List of items

use anyhow::{Context, Result};
use gtk::glib::{ParamFlags, ParamSpec, ParamSpecBoolean, ParamSpecString, Value};
use gtk::prelude::{ListModelExt, ObjectExt, ToValue};
use gtk::subclass::prelude::{ObjectImpl, ObjectSubclass};
use gtk::{gio, glib};
use once_cell::sync::Lazy;

use std::cell::{Cell, RefCell};

use super::Tree;

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
                let is_dir: bool = value.get().unwrap_or_else(|e| {
                    log::error!("Couldn't unwrap the value of the `is-dir` property");
                    log::debug!("{e}");
                    false
                });
                self.is_dir.replace(is_dir);
            }
            "label" => {
                let label: String = value.get().unwrap_or_else(|e| {
                    log::error!("Couldn't unwrap the value of the `label` property");
                    log::debug!("{e}");
                    String::from("")
                });
                self.label.replace(label);
            }
            _ => log::error!("Tried to set an unsupported property {value:?}"),
        }
    }
    fn property(&self, _obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> Value {
        match pspec.name() {
            "is-dir" => self.is_dir.get().to_value(),
            "label" => self.label.borrow().to_value(),
            _ => {
                log::error!("Tried to get an unsupported property");
                log::debug!("{pspec:?}");
                "".to_value()
            }
        }
    }
}

glib::wrapper! {
    /// Item in the list
    pub struct Item(ObjectSubclass<GItem>);
}

impl Item {
    /// Create a new feed
    pub(super) fn new(is_dir: bool, label: &str) -> Result<Self> {
        glib::Object::new(&[("is-dir", &is_dir), ("label", &label.to_owned())])
            .with_context(|| "Could not initialize a feed")
    }
    /// Return `true` if the feed is a directory
    pub(super) fn is_dir(&self) -> bool {
        self.property("is-dir")
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

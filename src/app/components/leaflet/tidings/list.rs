//! List of items

use gtk::glib::{ParamFlags, ParamSpec, ParamSpecString, Value};
use gtk::prelude::{ObjectExt, ToValue};
use gtk::subclass::prelude::{ObjectImpl, ObjectSubclass};
use gtk::{gio, glib};
use once_cell::sync::Lazy;
use std::cell::RefCell;

/// List of items
pub(super) type List = gio::ListStore;

/// Object holding the state
#[derive(Default)]
pub struct GItem {
    /// Label
    label: RefCell<String>,
}

#[glib::object_subclass]
impl ObjectSubclass for GItem {
    const NAME: &'static str = "Tiding";
    type Type = Item;
    type ParentType = glib::Object;
}

impl ObjectImpl for GItem {
    fn properties() -> &'static [ParamSpec] {
        /// Properties
        static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
            vec![
                // Label of the tiding
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
        if pspec.name() == "label" {
            if let Ok(label) = value.get() {
                self.label.replace(label);
            }
        }
    }
    fn property(&self, _obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> Value {
        match pspec.name() {
            "label" => self.label.borrow().to_value(),
            _ => "".to_value(),
        }
    }
}

glib::wrapper! {
    pub struct Item(ObjectSubclass<GItem>);
}

impl Item {
    /// Initialize a tiding from the label
    pub fn new(label: &str) -> Option<Self> {
        glib::Object::new(&[("label", &label.to_owned())]).ok()
    }
    /// Update the string
    pub fn update_string(self) {
        let label: String = self.property("label");
        self.set_property("label", format!("{}!", label));
    }
}

//! List of items

use gtk::glib::{ParamFlags, ParamSpec, ParamSpecString, Value};
use gtk::prelude::{BoxExt, Cast, ListModelExt, ObjectExt, StaticType, ToValue, WidgetExt};
use gtk::subclass::prelude::{ObjectImpl, ObjectSubclass};
use gtk::{gio, glib};
use once_cell::sync::Lazy;
use relm4::WidgetPlus;
use std::cell::RefCell;

use super::dictionary::Tiding;

/// Object holding the state
#[derive(Default)]
pub struct GItem {
    /// Title
    title: RefCell<String>,
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
                // Title of the tiding
                ParamSpecString::new(
                    // Name
                    "title",
                    // Nickname
                    "title",
                    // Short description
                    "title",
                    // Default value
                    None,
                    // Flags
                    ParamFlags::READWRITE,
                ),
            ]
        });
        &PROPERTIES
    }
    fn set_property(&self, _obj: &Self::Type, _id: usize, value: &Value, pspec: &ParamSpec) {
        if pspec.name() == "title" {
            if let Ok(title) = value.get() {
                self.title.replace(title);
            }
        }
    }
    fn property(&self, _obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> Value {
        match pspec.name() {
            "title" => self.title.borrow().to_value(),
            _ => "".to_value(),
        }
    }
}

glib::wrapper! {
    pub struct Item(ObjectSubclass<GItem>);
}

impl Item {
    /// Initialize a tiding from the title
    pub fn new(title: &str) -> Option<Self> {
        glib::Object::new(&[("title", &title.to_owned())]).ok()
    }
    /// Return the title of the item
    pub(super) fn title(&self) -> String {
        self.property("title")
    }
}

impl From<&Tiding> for Option<Item> {
    fn from(tiding: &Tiding) -> Self {
        Item::new(&tiding.title)
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
    /// Update the list with the provided tidings
    pub(super) fn update(&mut self, tidings: &[Tiding]) {
        // Collect a vector of items
        if let Some(items) = tidings
            .iter()
            .map(Option::<Item>::from)
            .collect::<Option<Vec<Item>>>()
        {
            self.store.splice(0, self.store.n_items(), &items);
        }
    }
}

/// An extension trait for the `ListItem`
pub(super) trait ListItemExt {
    /// Setup the `ListItem`
    fn setup(&self);
    /// Modify the `ListItem`
    fn modify(&self, modify_icon: fn(&gtk::Image, &Item), modify_title: fn(&gtk::Label, &Item));
}

impl ListItemExt for gtk::ListItem {
    fn setup(&self) {
        // Create a widget for the item
        let icon = gtk::Image::new();
        icon.set_icon_name(None);
        icon.set_margin_top(12);
        icon.set_margin_bottom(12);
        icon.set_margin_start(12);
        let title = gtk::Label::default();
        title.set_margin_all(12);
        title.set_ellipsize(gtk::pango::EllipsizeMode::End);
        let widget = gtk::Box::default();
        widget.append(&icon);
        widget.append(&title);
        widget.set_focusable(true);
        widget.set_margin_top(2);
        widget.set_margin_bottom(2);
        widget.set_margin_start(12);
        widget.set_margin_end(12);
        widget.set_css_classes(&["activatable", "card"]);
        // Attach the widget to the list item
        self.set_child(Some(&widget));
    }
    fn modify(&self, modify_icon: fn(&gtk::Image, &Item), modify_title: fn(&gtk::Label, &Item)) {
        // Get the GObject
        if let Some(object) = self.item() {
            // Downcast the object
            if let Ok(item) = object.downcast::<Item>() {
                // Get the widget
                if let Some(item_child) = self.child() {
                    if let Ok(widget) = item_child.downcast::<gtk::Box>() {
                        // Get the icon
                        if let Some(widget_first_child) = widget.first_child() {
                            if let Ok(icon) = widget_first_child.downcast::<gtk::Image>() {
                                // Modify the icon
                                modify_icon(&icon, &item);
                                // Get the title
                                if let Some(icon_next_sibling) = icon.next_sibling() {
                                    if let Ok(title) = icon_next_sibling.downcast::<gtk::Label>() {
                                        // Modify the title
                                        modify_title(&title, &item);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

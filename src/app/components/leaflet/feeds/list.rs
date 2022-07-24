//! List of items

use generational_arena::Index;
use gtk::glib::{ParamFlags, ParamSpec, ParamSpecBoolean, ParamSpecString, ParamSpecUInt64, Value};
use gtk::prelude::{BoxExt, Cast, ListModelExt, ObjectExt, StaticType, ToValue, WidgetExt};
use gtk::subclass::prelude::{ObjectImpl, ObjectSubclass};
use gtk::{gio, glib};
use once_cell::sync::Lazy;
use relm4::WidgetPlus;

use std::cell::{Cell, RefCell};

use super::{Node, Tree};

/// Object holding the state
#[derive(Default)]
pub struct GItem {
    /// Index
    index: Cell<u64>,
    /// Generation
    generation: Cell<u64>,
    /// Title
    title: RefCell<String>,
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
                // Title
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
            "title" => {
                if let Ok(title) = value.get() {
                    self.title.replace(title);
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
            "title" => self.title.borrow().to_value(),
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
    pub(super) fn new(index: u64, generation: u64, title: &str, is_dir: bool) -> Option<Self> {
        glib::Object::new(&[
            ("index", &index),
            ("generation", &generation),
            ("title", &title.to_owned()),
            ("is-dir", &is_dir),
        ])
        .ok()
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
    /// Return `true` if the feed is a directory
    pub(super) fn is_dir(&self) -> bool {
        self.property("is-dir")
    }
    /// Return the title of the item
    pub(super) fn title(&self) -> String {
        self.property("title")
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
            Node::Directory { ref title, .. } => Item::new(0, 0, title, true),
            Node::Feed { ref title, .. } => Item::new(0, 0, title, false),
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

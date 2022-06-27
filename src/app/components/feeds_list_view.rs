//! Feeds List View

use anyhow::{bail, Context, Result};
use gtk::glib::{ParamFlags, ParamSpec, ParamSpecBoolean, ParamSpecString, Value};
use gtk::prelude::{Cast, ListModelExt, ObjectExt, StaticType, ToValue};
use gtk::subclass::prelude::{ObjectImpl, ObjectSubclass};
use gtk::{gio, glib};
use once_cell::sync::Lazy;
use relm4::{ComponentUpdate, Sender};
use std::cell::{Cell, RefCell};
use std::rc::{Rc, Weak};

use super::{AppModel, AppMsg};

/// Object holding the state
#[derive(Default)]
pub struct GFeed {
    /// Is the feed is a directory?
    is_dir: Cell<bool>,
    /// Label
    label: RefCell<String>,
}

#[glib::object_subclass]
impl ObjectSubclass for GFeed {
    const NAME: &'static str = "Feed";
    type Type = Feed;
    type ParentType = glib::Object;
}

impl ObjectImpl for GFeed {
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
    pub struct Feed(ObjectSubclass<GFeed>);
}

impl Feed {
    /// Create a new feed
    fn new(is_dir: bool, label: &str) -> Result<Self> {
        glib::Object::new(&[("is-dir", &is_dir), ("label", &label.to_owned())])
            .with_context(|| "Could not initialize a feed")
    }
    /// Return `true` if the feed is a directory
    fn is_dir(&self) -> bool {
        self.property("is-dir")
    }
}

/// A tree of feeds
struct Tree {
    /// Root of the whole tree
    root: Rc<Node>,
    /// Root of the current subtree
    current: RefCell<Rc<Node>>,
}

impl Default for Tree {
    fn default() -> Self {
        let root = Rc::new(Node {
            is_dir: true,
            label: "All feeds".to_owned(),
            children: RefCell::new(vec![]),
            parent: RefCell::new(Weak::default()),
        });
        let current = RefCell::new(Rc::clone(&root));
        Self { root, current }
    }
}

impl Tree {
    /// Return `true` if the root of the current subtree
    /// coincides with the root of the whole tree
    fn is_root(&self) -> bool {
        self.current.borrow().parent.borrow().upgrade().is_none()
    }
    /// Go one level up in the tree
    fn back(&self) -> Result<()> {
        // Get the new subtree root in the current block
        // (we need this to avoid the `already borrowed` error at runtime)
        let node = {
            // Get the current root of the subtree
            let current = self.current.borrow();
            // Borrow the parent of the current node
            let parent = current.parent.borrow();
            // Try to get the parent of this node
            match parent.upgrade() {
                Some(node) => Rc::clone(&node),
                None => bail!("Couldn't get the parent of the current subtree root node"),
            }
        };
        // Change the current root of the subtree to this node
        *self.current.borrow_mut() = Rc::clone(&node);
        Ok(())
    }
    /// Enter the directory, going one level down in the tree
    fn enter_dir(&self, position: usize) -> Result<()> {
        // Get the new subtree root in the current block
        // (we need this to avoid the `already borrowed` error at runtime)
        let node = {
            // Get the current root of the subtree
            let current = self.current.borrow();
            // Borrow the children of the current node
            let children = current.children.borrow();
            // Try to get the child and the position
            match children.get(position) {
                Some(node) => Rc::clone(node),
                None => bail!("Couldn't get a child node in the current subtree"),
            }
        };
        // Change the current root of the subtree to this node
        *self.current.borrow_mut() = Rc::clone(&node);
        Ok(())
    }
    /// Get a vector of feeds
    fn list(&self) -> Vec<Feed> {
        let mut vec = vec![];
        for child in &*self.current.borrow().children.borrow() {
            if let Ok(feed_object) = Feed::new(child.is_dir, &child.label) {
                vec.push(feed_object);
            } else {
                log::error!("Couldn't create a new feed");
            }
        }
        vec
    }
    /// Append the child to the node
    fn append(parent: &Rc<Node>, child: &Rc<Node>) {
        // Push the child to the children
        parent.children.borrow_mut().push(Rc::clone(child));
        // Update the parent of the child node
        *child.parent.borrow_mut() = Rc::downgrade(parent);
    }
}

/// A node in the tree of feeds
struct Node {
    /// Is it a directory?
    is_dir: bool,
    /// Label
    label: String,
    /// Children nodes
    children: RefCell<Vec<Rc<Node>>>,
    /// Parent node
    parent: RefCell<Weak<Node>>,
}

/// Model
pub struct Model {
    /// Feeds tree (this struct holds all data)
    tree: Tree,
    /// List Store (this struct holds a list of
    /// items in the current subtree)
    store: gio::ListStore,
}

/// Messages
pub enum Msg {
    /// Go one level up in the tree of feeds
    Back,
    /// Enter the directory at the position,
    /// going one level down in the tree of feeds
    EnterDirectory(usize),
}

impl relm4::Model for Model {
    type Msg = Msg;
    type Widgets = Widgets;
    type Components = ();
}

impl ComponentUpdate<AppModel> for Model {
    fn init_model(_parent_model: &AppModel) -> Self {
        // Initialize the feeds tree
        let tree = Tree::default();
        // Append a fake feed
        Tree::append(
            &tree.root,
            &Rc::new(Node {
                is_dir: false,
                label: "Feed".to_owned(),
                children: RefCell::new(vec![]),
                parent: RefCell::new(Weak::default()),
            }),
        );
        // Append a fake directory with a fake feed inside
        let feed = Rc::new(Node {
            is_dir: false,
            label: "Feed inside the directory".to_owned(),
            children: RefCell::new(vec![]),
            parent: RefCell::new(Weak::default()),
        });
        let dir = Rc::new(Node {
            is_dir: true,
            label: "Directory".to_owned(),
            children: RefCell::new(vec![]),
            parent: RefCell::new(Weak::default()),
        });
        Tree::append(&tree.root, &dir);
        Tree::append(&dir, &feed);
        // Initialize the store
        let store = gio::ListStore::new(Feed::static_type());
        // Append each item from the tree to the store
        for item in tree.list() {
            store.append(&item);
        }
        // Return the model
        Self { tree, store }
    }

    fn update(
        &mut self,
        msg: Msg,
        _components: &(),
        _sender: Sender<Msg>,
        parent_sender: Sender<AppMsg>,
    ) {
        match msg {
            Msg::Back => {
                // Update the tree
                self.tree.back().unwrap_or_else(|e| {
                    log::error!("Couldn't go back in the tree");
                    log::debug!("{e}");
                });
                // Update the store
                self.store.remove_all();
                for item in self.tree.list() {
                    self.store.append(&item);
                }
                // If on the top level
                if self.tree.is_root() {
                    // Hide the back button
                    parent_sender
                        .send(AppMsg::FeedsHideBack)
                        .unwrap_or_else(|e| {
                            log::error!("Couldn't send a message to hide the Feeds Back Button");
                            log::debug!("{e}");
                        });
                }
            }
            Msg::EnterDirectory(position) => {
                // Update the tree
                self.tree.enter_dir(position).unwrap_or_else(|e| {
                    log::error!("Couldn't enter the directory at position {position}");
                    log::debug!("{e}");
                });
                // Update the store
                self.store.remove_all();
                for item in self.tree.list() {
                    self.store.append(&item);
                }
                // Show the back button
                parent_sender
                    .send(AppMsg::FeedsShowBack)
                    .unwrap_or_else(|e| {
                        log::error!("Couldn't send a message to show the Feeds Back Button");
                        log::debug!("{e}");
                    });
            }
        }
    }
}

/// Get a `ListView` from the model
fn list_view(model: &Model) -> gtk::ListView {
    let factory = gtk::SignalListItemFactory::new();
    factory.connect_setup(move |_, list_item| {
        // Create a label
        let label = gtk::Label::new(None);
        list_item.set_child(Some(&label));
        // Create expressions describing `list_item -> item -> label`
        let list_item_expression = gtk::ConstantExpression::new(list_item);
        let feed_object_expression = gtk::PropertyExpression::new(
            gtk::ListItem::static_type(),
            Some(&list_item_expression),
            "item",
        );
        let label_expression = gtk::PropertyExpression::new(
            Feed::static_type(),
            Some(&feed_object_expression),
            "label",
        );
        // Bind the labels
        label_expression.bind(&label, "label", Some(&label));
    });
    // Create a filter model
    let filter_model = gtk::FilterListModel::new(Some(&model.store), gtk::Filter::NONE);
    // Create a sort model
    let sort_model = gtk::SortListModel::new(Some(&filter_model), gtk::Sorter::NONE);
    // Create a selection model
    let selection_model = gtk::SingleSelection::new(Some(&sort_model));
    // Create a List View
    gtk::ListView::new(Some(&selection_model), Some(&factory))
}

#[allow(clippy::missing_docs_in_private_items)]
#[relm4_macros::widget(pub)]
impl relm4::Widgets<Model, AppModel> for Widgets {
    view! {
        list_view(model) -> gtk::ListView {
            set_single_click_activate: true,
            connect_activate(sender) => move |list_view, position| {
                // Get the model
                if let Some(list_model) = list_view.model() {
                    if let Some(item) = list_model.item(position) {
                        if let Ok(feed) = item.downcast::<Feed>() {
                            // If this feed is a directory
                            if feed.is_dir() {
                                if let Ok(position) = position.try_into() {
                                    // Enter it
                                    sender.send(Msg::EnterDirectory(position))
                                        .unwrap_or_else(|e| {
                                            log::error!("Couldn't send a message to enter the directory");
                                            log::debug!("{e}");
                                        });
                                } else {
                                    log::error!("Couldn't cast u32 to usize");
                                }
                            }
                        } else {
                            log::error!("Couldn't downcast the object");
                        }
                    } else {
                        log::error!("Couldn't get the item at the position {position}");
                    }
                } else {
                    log::error!("Couldn't unwrap the model");
                };
            }
        }
    }
}

//! Feeds List View

use gtk::glib::{ParamFlags, ParamSpec, ParamSpecBoolean, ParamSpecString, Value};
use gtk::prelude::{Cast, ListModelExt, ObjectExt, StaticType, ToValue};
use gtk::subclass::prelude::{ObjectImpl, ObjectSubclass};
use gtk::{gio, glib};
use once_cell::sync::Lazy;
use relm4::{send, ComponentUpdate, Sender};
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
                let is_dir: bool = value.get().expect("The value needs to be of type `bool`.");
                self.is_dir.replace(is_dir);
            }
            "label" => {
                let label: String = value
                    .get()
                    .expect("The value needs to be of type `String`.");
                self.label.replace(label);
            }
            _ => unimplemented!(),
        }
    }

    fn property(&self, _obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> Value {
        match pspec.name() {
            "is-dir" => self.is_dir.get().to_value(),
            "label" => self.label.borrow().to_value(),
            _ => unimplemented!(),
        }
    }
}

glib::wrapper! {
    pub struct Feed(ObjectSubclass<GFeed>);
}

impl Feed {
    /// Create a new feed
    fn new(is_dir: bool, label: &str) -> Self {
        glib::Object::new(&[("is-dir", &is_dir), ("label", &label.to_string())])
            .expect("Could not create a `Feed`.")
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
    fn back(&self) {
        let node = {
            let current = self.current.borrow();
            let parent = &current
                .parent
                .borrow()
                .upgrade()
                .expect("Tried to go back on the top level");
            Rc::clone(parent)
        };
        *self.current.borrow_mut() = node;
    }
    /// Enter the directory, going one level down in the tree
    fn enter_dir(&self, position: usize) {
        let node = {
            let current = self.current.borrow();
            let children = current.children.borrow();
            Rc::clone(children.get(position).expect("Couldn't get a child node"))
        };
        *self.current.borrow_mut() = node;
    }
    /// Get a vector of feeds
    fn list(&self) -> Vec<Feed> {
        let mut vec = vec![];
        for child in &*self.current.borrow().children.borrow() {
            let feed_object = Feed::new(child.is_dir, &child.label);
            vec.push(feed_object);
        }
        vec
    }
    /// Append the child to the node
    fn append(parent: &Rc<Node>, child: &Rc<Node>) {
        parent.children.borrow_mut().push(Rc::clone(child));
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
    /// Feeds tree
    tree: Tree,
    /// List Store
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
        let tree = Tree::default();

        Tree::append(
            &tree.root,
            &Rc::new(Node {
                is_dir: false,
                label: "Feed".to_owned(),
                children: RefCell::new(vec![]),
                parent: RefCell::new(Weak::default()),
            }),
        );

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

        let store = gio::ListStore::new(Feed::static_type());
        for item in tree.list() {
            store.append(&item);
        }

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
                self.tree.back();
                // Update the store
                self.store.remove_all();
                for item in self.tree.list() {
                    self.store.append(&item);
                }
                // If on the top level
                if self.tree.is_root() {
                    // Hide the back button
                    send!(parent_sender, AppMsg::FeedsHideBack);
                }
            }
            Msg::EnterDirectory(position) => {
                // Update the tree
                self.tree.enter_dir(position);
                // Update the store
                self.store.remove_all();
                for item in self.tree.list() {
                    self.store.append(&item);
                }
                // Show the back button
                send!(parent_sender, AppMsg::FeedsShowBack);
            }
        }
    }
}

/// Get a `ListView` from the model
fn list_view(model: &Model) -> gtk::ListView {
    let factory = gtk::SignalListItemFactory::new();
    factory.connect_setup(move |_, list_item| {
        // Create label
        let label = gtk::Label::new(None);
        list_item.set_child(Some(&label));

        // Create expression describing `list_item->item->label`
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

        // Bind the object's "label" to the widget's "label"
        label_expression.bind(&label, "label", Some(&label));
    });

    let filter_model = gtk::FilterListModel::new(Some(&model.store), gtk::Filter::NONE);
    let sort_model = gtk::SortListModel::new(Some(&filter_model), gtk::Sorter::NONE);
    let selection_model = gtk::SingleSelection::new(Some(&sort_model));

    gtk::ListView::new(Some(&selection_model), Some(&factory))
}

#[allow(clippy::missing_docs_in_private_items)]
#[relm4_macros::widget(pub)]
impl relm4::Widgets<Model, AppModel> for Widgets {
    view! {
        list_view(model) -> gtk::ListView {
            set_single_click_activate: true,
            connect_activate(sender) => move |list_view, position| {
                // Get `FeedObject` from model
                let list_model = list_view.model().expect("The model has to exist.");
                let feed_object: Feed = list_model
                    .item(position)
                    .expect("The item has to exist.")
                    .downcast()
                    .expect("The item has to be a `Feed`.");
                // If the object represents a directory
                if feed_object.is_dir() {
                    // Enter it
                    send!(sender, Msg::EnterDirectory(position.try_into().expect("Couldn't cast `u32` to `usize`")));
                }
            }
        }
    }
}

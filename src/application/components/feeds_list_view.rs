use gtk::glib::{Object, ParamFlags, ParamSpec, ParamSpecBoolean, ParamSpecString, Value};
use gtk::prelude::{Cast, ListModelExt, ObjectExt, StaticType, ToValue};
use gtk::subclass::prelude::{ObjectImpl, ObjectSubclass};
use gtk::{gio, glib};
use once_cell::sync::Lazy;
use relm4::{send, ComponentUpdate, Model, Sender, Widgets};
use std::cell::{Cell, RefCell};
use std::rc::{Rc, Weak};

use super::{AppModel, AppMsg};

// Object holding the state
#[derive(Default)]
pub struct GFeedObject {
    is_dir: Cell<bool>,
    label: RefCell<String>,
}

// The central trait for subclassing a GObject
#[glib::object_subclass]
impl ObjectSubclass for GFeedObject {
    const NAME: &'static str = "TidingsFeedObject";
    type Type = FeedObject;
    type ParentType = glib::Object;
}

// Trait shared by all GObjects
impl ObjectImpl for GFeedObject {
    fn properties() -> &'static [ParamSpec] {
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
        PROPERTIES.as_ref()
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
    pub struct FeedObject(ObjectSubclass<GFeedObject>);
}

impl FeedObject {
    fn new(is_dir: bool, label: &str) -> Self {
        Object::new(&[("is-dir", &is_dir), ("label", &label.to_string())])
            .expect("Could not create `FeedObject`.")
    }

    fn is_dir(&self) -> bool {
        self.property("is-dir")
    }
}

struct FeedsTree {
    root: Rc<FeedsTreeNode>,
    current: RefCell<Rc<FeedsTreeNode>>,
}

impl Default for FeedsTree {
    fn default() -> Self {
        let root = Rc::new(FeedsTreeNode {
            is_dir: true,
            label: "All feeds".to_string(),
            children: RefCell::new(vec![]),
            parent: RefCell::new(Weak::default()),
        });
        let current = RefCell::new(Rc::clone(&root));
        Self { root, current }
    }
}

impl FeedsTree {
    fn is_root(&self) -> bool {
        self.current.borrow().parent.borrow().upgrade().is_none()
    }

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

    fn enter_dir(&self, position: usize) {
        let node = {
            let current = self.current.borrow();
            let children = current.children.borrow();
            Rc::clone(children.get(position).expect("Couldn't get a child node"))
        };
        *self.current.borrow_mut() = node;
    }

    fn list(&self) -> Vec<FeedObject> {
        let mut vec = vec![];
        for child in &*self.current.borrow().children.borrow() {
            let feed_object = FeedObject::new(child.is_dir, &child.label);
            vec.push(feed_object);
        }
        vec
    }

    fn append(parent: &Rc<FeedsTreeNode>, child: &Rc<FeedsTreeNode>) {
        parent.children.borrow_mut().push(Rc::clone(child));
        *child.parent.borrow_mut() = Rc::downgrade(parent);
    }
}

struct FeedsTreeNode {
    is_dir: bool,
    label: String,
    children: RefCell<Vec<Rc<FeedsTreeNode>>>,
    parent: RefCell<Weak<FeedsTreeNode>>,
}

pub struct FeedsModel {
    tree: FeedsTree,
    store: gio::ListStore,
}

pub enum FeedsMsg {
    Back,
    EnterDirectory(usize),
}

impl Model for FeedsModel {
    type Msg = FeedsMsg;
    type Widgets = FeedsWidgets;
    type Components = ();
}

impl ComponentUpdate<AppModel> for FeedsModel {
    fn init_model(_parent_model: &AppModel) -> Self {
        let tree = FeedsTree::default();

        FeedsTree::append(
            &tree.root,
            &Rc::new(FeedsTreeNode {
                is_dir: false,
                label: "Feed".to_string(),
                children: RefCell::new(vec![]),
                parent: RefCell::new(Weak::default()),
            }),
        );

        let feed = Rc::new(FeedsTreeNode {
            is_dir: false,
            label: "Feed inside the directory".to_string(),
            children: RefCell::new(vec![]),
            parent: RefCell::new(Weak::default()),
        });
        let dir = Rc::new(FeedsTreeNode {
            is_dir: true,
            label: "Directory".to_string(),
            children: RefCell::new(vec![]),
            parent: RefCell::new(Weak::default()),
        });

        FeedsTree::append(&tree.root, &dir);
        FeedsTree::append(&dir, &feed);

        let store = gio::ListStore::new(FeedObject::static_type());
        for item in tree.list() {
            store.append(&item);
        }

        FeedsModel { tree, store }
    }

    fn update(
        &mut self,
        msg: FeedsMsg,
        _components: &(),
        _sender: Sender<FeedsMsg>,
        parent_sender: Sender<AppMsg>,
    ) {
        match msg {
            FeedsMsg::Back => {
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
            FeedsMsg::EnterDirectory(position) => {
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

fn list_view(model: &FeedsModel) -> gtk::ListView {
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
            FeedObject::static_type(),
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

#[relm4_macros::widget(pub)]
impl Widgets<FeedsModel, AppModel> for FeedsWidgets {
    view! {
        list_view(model) -> gtk::ListView {
            set_single_click_activate: true,
            connect_activate(sender) => move |list_view, position| {
                // Get `FeedObject` from model
                let list_model = list_view.model().expect("The model has to exist.");
                let feed_object: FeedObject = list_model
                    .item(position)
                    .expect("The item has to exist.")
                    .downcast()
                    .expect("The item has to be an `FeedObject`.");
                // If the object represents a directory
                if feed_object.is_dir() {
                    // Enter it
                    send!(sender, FeedsMsg::EnterDirectory(position as usize));
                }
            }
        }
    }
}

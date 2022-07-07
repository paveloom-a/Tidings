//! Feeds

mod list;
mod tree;

use gtk::prelude::{Cast, ListModelExt, StaticType};
use relm4::{ComponentUpdate, Sender};

use super::{AppModel, AppMsg};
use list::{Item, List, UpdateList};
use tree::{Node, Tree};

/// Model
pub struct Model {
    /// Feeds tree
    tree: Tree,
    /// List of items in current directory
    list: List,
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
        let mut tree = Tree::default();
        // Insert a fake feed
        let feed = Node::Feed {
            label: "Feed".to_owned(),
        };
        tree.insert(tree.current, feed);
        // Insert a fake directory with a fake feed inside
        let dir = Node::Directory {
            label: "Directory".to_owned(),
            children: vec![],
            parent: Some(tree.current),
        };
        let feed = Node::Feed {
            label: "Feed inside the directory".to_owned(),
        };
        if let Some(dir_index) = tree.insert(tree.current, dir) {
            tree.insert(dir_index, feed);
        }
        // Initialize the list
        let mut list = List::new(Item::static_type());
        // Update the list
        list.update(&tree);
        // Return the model
        Self { tree, list }
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
                // Go back in the tree
                self.tree.back();
                // Update the list
                self.list.update(&self.tree);
                // If on the top level
                if self.tree.is_root() {
                    // Hide the back button
                    parent_sender
                        .send(AppMsg::FeedsHideBack)
                        .unwrap_or_else(|e| {
                            log::error!("Couldn't send a message to hide the back button");
                            log::debug!("{e}");
                        });
                }
            }
            Msg::EnterDirectory(position) => {
                // Enter the directory
                self.tree.enter_dir(position);
                // Update the list
                self.list.update(&self.tree);
                // Show the back button
                parent_sender
                    .send(AppMsg::FeedsShowBack)
                    .unwrap_or_else(|e| {
                        log::error!("Couldn't send a message to show the back button");
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
            Item::static_type(),
            Some(&feed_object_expression),
            "label",
        );
        // Bind the labels
        label_expression.bind(&label, "label", Some(&label));
    });
    // Create a filter model
    let filter_model = gtk::FilterListModel::new(Some(&model.list), gtk::Filter::NONE);
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
                    // Get the item at the position
                    if let Some(item) = list_model.item(position) {
                        // Downcast the object
                        if let Ok(item) = item.downcast::<Item>() {
                            // If this item is a directory
                            if item.is_dir() {
                                // If the position can be casted from `u32` to `usize`
                                if let Ok(position) = position.try_into() {
                                    // Enter it
                                    sender.send(Msg::EnterDirectory(position)).unwrap_or_else(|e| {
                                        log::error!("Couldn't send a message to enter the directory");
                                        log::debug!("{e}");
                                    });
                                }
                            }
                        }
                    }
                };
            }
        }
    }
}

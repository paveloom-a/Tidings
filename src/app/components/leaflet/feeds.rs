//! Feeds

mod list;
mod tree;

use gtk::prelude::{BoxExt, ButtonExt, Cast, ListModelExt, OrientableExt, StaticType, WidgetExt};
use relm4::actions::ActionName;
use relm4::{ComponentUpdate, Sender};

use crate::app::actions::{
    ShowAboutDialog, ShowAddDirectoryDialog, ShowAddFeedDialog, ShowHelpOverlay,
};
use list::{Item, List, UpdateList};
use tree::{Node, Tree};

/// Model
pub struct Model {
    /// Feeds tree
    tree: Tree,
    /// List of items in current directory
    list: List,
    /// Is the back button visible?
    back_button_visible: bool,
    /// Are the end buttons visible in the header bar?
    end_buttons_visible: bool,
}

/// Messages
pub enum Msg {
    /// Go one level up in the tree of feeds
    Back,
    /// Enter the directory at the position,
    /// going one level down in the tree of feeds
    EnterDirectory(usize),
    /// Show end buttons in the header bar
    ShowEndButtons,
    /// Hide end buttons in the header bar
    HideEndButtons,
    /// Add a feed
    AddFeed {
        /// Label
        label: String,
    },
    /// Add a directory
    AddDirectory {
        /// Label
        label: String,
    },
}

impl relm4::Model for Model {
    type Msg = Msg;
    type Widgets = Widgets;
    type Components = ();
}

impl ComponentUpdate<super::Model> for Model {
    fn init_model(_parent_model: &super::Model) -> Self {
        // Initialize the feeds tree
        let tree = Tree::default();
        // Initialize the list
        let mut list = List::new(Item::static_type());
        // Update the list
        list.update(&tree);
        // Return the model
        Self {
            tree,
            list,
            back_button_visible: false,
            end_buttons_visible: false,
        }
    }
    fn update(
        &mut self,
        msg: Msg,
        _components: &(),
        _sender: Sender<Msg>,
        _parent_sender: Sender<super::Msg>,
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
                    self.back_button_visible = false;
                }
            }
            Msg::EnterDirectory(position) => {
                // Enter the directory
                self.tree.enter_dir(position);
                // Update the list
                self.list.update(&self.tree);
                // Show the back button
                self.back_button_visible = true;
            }
            Msg::ShowEndButtons => {
                self.end_buttons_visible = true;
            }
            Msg::HideEndButtons => {
                self.end_buttons_visible = false;
            }
            Msg::AddFeed { label } => {
                // Create a new node
                let node = Node::Feed { label };
                // Append the new item to the end of the list
                if let Some(item) = Option::<Item>::from(&node) {
                    self.list.append(&item);
                }
                // Insert the node into the tree
                self.tree.insert(self.tree.current, node);
            }
            Msg::AddDirectory { label } => {
                // Create a new node
                let node = Node::Directory {
                    label,
                    children: vec![],
                    parent: Some(self.tree.current),
                };
                // Append the new item to the end of the list
                if let Some(item) = Option::<Item>::from(&node) {
                    self.list.append(&item);
                }
                // Insert the node into the tree
                self.tree.insert(self.tree.current, node);
            }
        }
    }
}

/// Get a `ListView` from the model
fn list_view(model: &Model) -> gtk::ListView {
    // Create a factory
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

/// Connect the activate event of the List View
fn list_view_connect_activate(sender: &Sender<Msg>, list_view: &gtk::ListView, position: u32) {
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
                        sender.send(Msg::EnterDirectory(position)).ok();
                    }
                }
            }
        }
    }
}

/// Create a Split Button with an action on an activate event
fn split_button() -> adw::SplitButton {
    adw::SplitButton::builder()
        .action_name(&ShowAddFeedDialog::action_name())
        .build()
}

#[allow(clippy::missing_docs_in_private_items)]
#[relm4_macros::widget(pub)]
impl relm4::Widgets<Model, super::Model> for Widgets {
    view! {
        // Box
        gtk::Box {
            set_width_request: 365,
            set_orientation: gtk::Orientation::Vertical,
            set_hexpand: true,
            // Header Bar
            append = &adw::HeaderBar {
                set_show_start_title_buttons: watch!(
                    model.end_buttons_visible
                ),
                set_show_end_title_buttons: watch!(
                    model.end_buttons_visible
                ),
                // Title
                set_title_widget = Some(&adw::WindowTitle) {
                    set_title: "Feeds"
                },
                // Go Back Button
                pack_start = &gtk::Button {
                    set_visible: watch!(model.back_button_visible),
                    set_icon_name: "go-previous-symbolic",
                    set_tooltip_text: Some("Go Back"),
                    connect_clicked(sender) => move |_| {
                        sender.send(Msg::Back).ok();
                    },
                },
                // Add Button
                pack_start = &split_button() -> adw::SplitButton {
                    set_icon_name: "plus-large-symbolic",
                    set_tooltip_text: Some("Add New Feed"),
                    set_menu_model: Some(&add_menu),
                },
                // Menu Button
                pack_end = &gtk::MenuButton {
                    set_visible: watch!(model.end_buttons_visible),
                    set_icon_name: "open-menu-symbolic",
                    set_menu_model: Some(&main_menu),
                },
            },
            // Scrolled Window
            append = &gtk::ScrolledWindow {
                set_hscrollbar_policy: gtk::PolicyType::Never,
                set_hexpand: true,
                set_vexpand: true,
                // List View
                set_child = Some(&list_view(model) -> gtk::ListView) {
                    set_single_click_activate: true,
                    connect_activate(sender) => move |list_view, position| {
                        list_view_connect_activate(&sender, list_view, position);
                    }
                }
            }
        }
    }
    menu! {
        main_menu: {
            "Keyboard Shortcuts" => ShowHelpOverlay,
            "About Tidings" => ShowAboutDialog,
        },
        add_menu: {
            "Feed" => ShowAddFeedDialog,
            "Directory" => ShowAddDirectoryDialog,
        }
    }
}

//! Feeds

mod list;
pub mod tree;

use generational_arena::Index;
use gtk::prelude::{BoxExt, ButtonExt, Cast, ListModelExt, OrientableExt, WidgetExt};
use relm4::{ComponentUpdate, Sender, WidgetPlus};

use super::tidings;
use crate::app::actions::{
    ShowAboutDialog, ShowAddDirectoryDialog, ShowAddFeedDialog, ShowHelpOverlay,
};
use list::{Item, List, ListItemExt};
use tree::{Node, Tree};

/// Model
pub struct Model {
    /// Feeds tree
    tree: Tree,
    /// List of items in the current directory
    list: List,
    /// Is the back button sensitive?
    back_button_sensitive: bool,
    /// Are the end buttons visible in the header bar?
    end_buttons_visible: bool,
}

impl Model {
    /// Insert the feed into the current subtree
    fn insert(&mut self, node: Node) {
        // Create a new item and append it to the end of the list
        if let Some(mut item) = Option::<Item>::from(&node) {
            // Insert the node into the tree
            if let Some(index) = self.tree.insert(self.tree.current, node) {
                // Set the index of the item
                item.set_index(index);
                // Append the item to the list
                self.list.append(&item);
            }
        }
    }
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
    /// Add a node
    Add(Node),
    /// Update all feeds
    UpdateAll,
    /// Update of the particular feed has started
    UpdateStarted(Index),
    /// Update of the particular feed finished
    UpdateFinished(Index),
    /// Show the tidings of this specific
    /// feed in the Tidings component
    ShowTidings(Index),
}

impl relm4::Model for Model {
    type Msg = Msg;
    type Widgets = Widgets;
    type Components = ();
}

impl ComponentUpdate<super::Model> for Model {
    fn init_model(_parent_model: &super::Model) -> Self {
        // Initialize a tree
        let tree = Tree::default();
        // Initialize a list
        let mut list = List::new();
        // Update the list
        list.update(&tree);
        // Return the model
        Self {
            tree,
            list,
            back_button_sensitive: false,
            end_buttons_visible: false,
        }
    }
    fn update(
        &mut self,
        msg: Msg,
        _components: &(),
        _sender: Sender<Msg>,
        parent_sender: Sender<super::Msg>,
    ) {
        match msg {
            Msg::Back => {
                // Go back in the tree
                self.tree.back();
                // Update the list
                self.list.update(&self.tree);
                // If on the top level
                if self.tree.is_root() {
                    // Make the back button insensitive
                    self.back_button_sensitive = false;
                }
            }
            Msg::EnterDirectory(position) => {
                // Enter the directory
                self.tree.enter_dir(position);
                // Update the list
                self.list.update(&self.tree);
                // Make the back button sensitive
                self.back_button_sensitive = true;
            }
            Msg::ShowEndButtons => {
                self.end_buttons_visible = true;
            }
            Msg::HideEndButtons => {
                self.end_buttons_visible = false;
            }
            Msg::Add(node) => {
                // Insert the node into the model
                self.insert(node);
            }
            Msg::UpdateAll => {
                // Get a vector of (index, URL) pairs of the feeds
                let indices_urls = self.tree.indices_urls();
                // Send them to the update message handler
                parent_sender.send(super::Msg::UpdateAll(indices_urls)).ok();
            }
            Msg::UpdateStarted(index) => {
                // Add the updating status of the feed
                self.tree.set_updating(index, true);
            }
            Msg::UpdateFinished(index) => {
                // Remove the updating status of the feed
                self.tree.set_updating(index, false);
            }
            Msg::ShowTidings(index) => {
                // Inform Tidings about which index to show
                parent_sender
                    .send(super::Msg::TransferToTidings(tidings::Msg::Show(index)))
                    .ok();
            }
        }
    }
}

/// Get a `ListView` from the model
fn list_view(model: &Model) -> gtk::ListView {
    // Create a factory
    let factory = gtk::SignalListItemFactory::new();
    // Setup the widget
    factory.connect_setup(move |_, list_item| {
        list_item.setup();
    });
    // Bind it to specific item
    factory.connect_bind(move |_, list_item| {
        list_item.modify(
            // Modify the icon
            |icon, item| {
                // If the item is a directory
                if item.is_dir() {
                    // Set a directory icon
                    icon.set_icon_name(Some("inode-directory-symbolic"));
                } else {
                    // Otherwise, set a favicon
                    icon.set_icon_name(Some("emblem-shared-symbolic"));
                }
            },
            // Modify the title
            |title, item| {
                // Set the title
                title.set_label(&item.title());
            },
        );
    });
    // Create a filter model
    let filter_model = gtk::FilterListModel::new(Some(&model.list.store), gtk::Filter::NONE);
    // Create a sort model
    let sort_model = gtk::SortListModel::new(Some(&filter_model), gtk::Sorter::NONE);
    // Create a selection model
    let selection_model = gtk::NoSelection::new(Some(&sort_model));
    // Create a List View
    gtk::ListView::new(Some(&selection_model), Some(&factory))
}

/// Connect the activate event of the List View
fn list_view_connect_activate(sender: &Sender<Msg>, list_view: &gtk::ListView, position: u32) {
    // Get the model
    if let Some(list_model) = list_view.model() {
        // Get the GObject at the position
        if let Some(object) = list_model.item(position) {
            // Downcast the object
            if let Ok(item) = object.downcast::<Item>() {
                // If this item is a directory
                if item.is_dir() {
                    // If the position can be casted from `u32` to `usize`
                    if let Ok(position) = position.try_into() {
                        // Enter the directory
                        sender.send(Msg::EnterDirectory(position)).ok();
                    }
                // Otherwise, it's a feed, so
                } else {
                    // Get the index of the feed
                    if let Some(index) = item.index() {
                        // Show the tidings of this specific
                        // feed in the Tidings component
                        sender.send(Msg::ShowTidings(index)).ok();
                    }
                }
            }
        }
    }
}

#[allow(clippy::missing_docs_in_private_items)]
#[relm4::widget(pub)]
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
                    set_sensitive: watch!(model.back_button_sensitive),
                    set_icon_name: "go-previous-symbolic",
                    set_tooltip_text: Some("Go Back"),
                    connect_clicked(sender) => move |_| {
                        sender.send(Msg::Back).ok();
                    },
                },
                // Add Split Button
                pack_start = &gtk::MenuButton {
                    set_icon_name: "plus-large-symbolic",
                    set_tooltip_text: Some("Add"),
                    set_menu_model: Some(&add_menu),
                },
                pack_start = &gtk::Button {
                    set_icon_name: "emblem-synchronizing-symbolic",
                    set_tooltip_text: Some("Update All Feeds"),
                    connect_clicked(sender) => move |_| {
                        sender.send(Msg::UpdateAll).ok();
                    }
                },
                // Menu Button
                pack_end = &gtk::MenuButton {
                    set_visible: watch!(model.end_buttons_visible),
                    set_icon_name: "open-menu-symbolic",
                    set_menu_model: Some(&main_menu),
                },
            },
            // Scrolled Window
            append: scrolled_window = &gtk::ScrolledWindow {
                set_hscrollbar_policy: gtk::PolicyType::Never,
                set_hexpand: true,
                set_vexpand: true,
                // List View
                set_child = Some(&list_view(model) -> gtk::ListView) {
                    set_margin_all: 4,
                    set_single_click_activate: true,
                    connect_activate(sender) => move |list_view, position| {
                        list_view_connect_activate(
                            &sender,
                            list_view,
                            position,
                        );
                    }
                }
            }
        }
    }
    fn pre_view() {
        // This is a trick to make the Scrolled Window recalculate
        // the vertical adjustment. This doesn't happen by default
        // after clearing the list
        scrolled_window.set_vadjustment(Option::<&gtk::Adjustment>::None);
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

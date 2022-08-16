//! Feeds

mod list;
pub mod tree;
pub mod update;

use generational_arena::Index;
use gtk::prelude::{BoxExt, ButtonExt, Cast, ListModelExt, OrientableExt, WidgetExt};
use relm4::{
    ComponentParts, ComponentSender, MessageBroker, SimpleComponent, WidgetPlus, WorkerController,
};

use super::tidings;
use crate::app::actions::{
    ShowAboutDialog, ShowAddDirectoryDialog, ShowAddFeedDialog, ShowHelpOverlay,
};
use list::{Item, List, ListItemExt};
use tree::{Node, Tree};

/// Message broker
pub static BROKER: MessageBroker<Model> = MessageBroker::new();

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
    /// Is the update running?
    updating: bool,
    /// Update message handler
    update: Option<WorkerController<update::Model>>,
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
#[derive(Debug)]
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
    /// Start the update of all feeds
    StartUpdateAll,
    /// Stop the update of all feeds
    StopUpdateAll,
    /// Toggle the update of all feeds
    ToggleUpdateAll,
    /// Update of the particular feed has started
    UpdateStarted(Index),
    /// Update of the particular feed finished
    UpdateFinished(Index),
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
fn list_view_connect_activate(
    sender: &ComponentSender<Model>,
    list_view: &gtk::ListView,
    position: u32,
) {
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
                        sender.input(Msg::EnterDirectory(position));
                    }
                // Otherwise, it's a feed, so
                } else {
                    // Get the index of the feed
                    if let Some(index) = item.index() {
                        // Show the tidings of this specific
                        // feed in the Tidings component
                        tidings::BROKER.send(tidings::Msg::Show(index));
                    }
                }
            }
        }
    }
}

#[allow(clippy::clone_on_ref_ptr)]
#[allow(clippy::missing_docs_in_private_items)]
#[allow(unused_variables)]
#[relm4::component(pub)]
impl SimpleComponent for Model {
    type Init = ();
    type Input = Msg;
    type Output = super::Msg;
    type Widgets = Widgets;
    fn init(
        _init: Self::Init,
        root: &Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        // Initialize a tree
        let tree = Tree::default();
        // Initialize a list
        let mut list = List::new();
        // Update the list
        list.update(&tree);
        // Initialize the model
        let model = Self {
            tree,
            list,
            back_button_sensitive: false,
            end_buttons_visible: false,
            updating: false,
            update: None,
        };
        let widgets = view_output!();
        ComponentParts { model, widgets }
    }
    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>) {
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
            Msg::StartUpdateAll => {
                // Create a new update message handler
                let update = update::new(&sender);
                // Get a vector of (index, URL) pairs of the feeds
                let indices_urls = self.tree.indices_urls();
                // Send them to the update message handler
                update.emit(update::Msg::UpdateAll(indices_urls));
                // Notify the UI that the update has started
                self.updating = true;
                // Let the model own the message handler
                self.update = Some(update);
            }
            Msg::StopUpdateAll => {
                // Drop the message handler (thus,
                // cancelling any ongoing update)
                self.update = None;
                // Notify the UI that the update has been canceled
                self.updating = false;
            }
            Msg::ToggleUpdateAll => {
                if self.updating {
                    sender.input(Msg::StopUpdateAll);
                } else {
                    sender.input(Msg::StartUpdateAll);
                }
            }
            Msg::UpdateStarted(index) => {
                // Add the updating status of the feed
                self.tree.set_updating(index, true);
            }
            Msg::UpdateFinished(index) => {
                // Remove the updating status of the feed
                self.tree.set_updating(index, false);
            }
        }
    }
    fn pre_view() {
        // This is a trick to make the Scrolled Window recalculate
        // the vertical adjustment. This doesn't happen by default
        // after clearing the list
        scrolled_window.set_vadjustment(Option::<&gtk::Adjustment>::None);
    }
    view! {
        // Box
        gtk::Box {
            set_width_request: 365,
            set_orientation: gtk::Orientation::Vertical,
            set_hexpand: true,
            // Header Bar
            append = &adw::HeaderBar {
                #[watch]
                set_show_start_title_buttons: model.end_buttons_visible,
                #[watch]
                set_show_end_title_buttons: model.end_buttons_visible ,
                // Title
                #[wrap(Some)]
                set_title_widget = &adw::WindowTitle {
                    set_title: "Feeds"
                },
                // Go Back Button
                pack_start = &gtk::Button {
                    #[watch]
                    set_sensitive: model.back_button_sensitive,
                    set_icon_name: "go-previous-symbolic",
                    set_tooltip_text: Some("Go Back"),
                    connect_clicked[sender] => move |_| {
                        sender.input(Msg::Back);
                    },
                },
                // Add Split Button
                pack_start = &gtk::MenuButton {
                    set_icon_name: "plus-large-symbolic",
                    set_tooltip_text: Some("Add"),
                    set_menu_model: Some(&add_menu),
                },
                pack_start = &gtk::Button {
                    #[watch]
                    set_icon_name: if model.updating {
                        "big-x-symbolic"
                    } else {
                        "emblem-synchronizing-symbolic"
                    },
                    #[watch]
                    set_tooltip_text: if model.updating {
                        Some("Stop Update")
                    } else {
                        Some("Update All Feeds")
                    },
                    connect_clicked[sender] => move |_| {
                        sender.input(Msg::ToggleUpdateAll);
                    }
                },
                // Menu Button
                pack_end = &gtk::MenuButton {
                    #[watch]
                    set_visible: model.end_buttons_visible,
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
                #[wrap(Some)]
                set_child = &list_view(&model) -> gtk::ListView {
                    set_margin_all: 4,
                    set_single_click_activate: true,
                    connect_activate[sender] => move |list_view, position| {
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

//! Tidings

pub(super) mod dictionary;
mod list;

use generational_arena::Index;
use gtk::prelude::{BoxExt, ButtonExt, Cast, ObjectExt, OrientableExt, WidgetExt};
use relm4::{ComponentUpdate, Sender, WidgetPlus};

use crate::app::actions::{ShowAboutDialog, ShowHelpOverlay};
use dictionary::{Dictionary, Tidings};
use list::{Item, List, ListItemExt};

/// Model
pub struct Model {
    /// Dictionary of (index, tidings) key-value pairs
    dictionary: Dictionary,
    /// List of items
    list: List,
    /// Current index displayed
    current: Option<Index>,
    /// Is the parent leaflet in the folded state?
    folded: bool,
}

impl Model {
    /// Refresh the list with the tidings
    /// of the currently selected feed
    pub(super) fn refresh(&mut self) {
        // If there is a currently selected feed
        if let Some(current) = self.current {
            // If there are tidings for this index
            if let Some(tidings) = self.dictionary.get(&current) {
                // Update the list with them
                self.list.update(tidings);
            // Otherwise,
            } else {
                // Render the list as empty
                self.list.update(&[]);
            }
        }
    }
}

/// Messages
pub enum Msg {
    /// Update of the particular feed finished
    UpdateFinished(Index, Tidings),
    /// Show the tidings of the particular feed
    Show(Index),
    /// Set the folded state
    Fold,
    /// Unset the folded state
    Unfold,
    /// Navigate back in the leaflet
    Back,
}

impl relm4::Model for Model {
    type Msg = Msg;
    type Widgets = Widgets;
    type Components = ();
}

impl ComponentUpdate<super::Model> for Model {
    fn init_model(_parent_model: &super::Model) -> Self {
        // Initialize a dictionary
        let dictionary = Dictionary::new();
        // Initialize a list
        let list = List::new();
        Self {
            dictionary,
            list,
            current: None,
            // Whether it's folded is restored
            // on restart by the parent leaflet
            folded: false,
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
            Msg::UpdateFinished(index, tidings) => {
                // Insert the tidings into the dictionary
                // using the index as a key
                self.dictionary.insert(index, tidings);
                // If there is a currently selected feed
                if let Some(current) = self.current {
                    // And its index is the same as this one
                    if index == current {
                        // Refresh the list
                        self.refresh();
                    }
                }
            }
            Msg::Show(index) => {
                // Update the current index
                self.current = Some(index);
                // Refresh the list
                self.refresh();
                // Inform the leaflet that the Tidings page is ready to be shown
                // (this only matters if the leaflet is folded)
                parent_sender.send(super::Msg::ShowTidingsPage).ok();
            }
            Msg::Fold => {
                self.folded = true;
            }
            Msg::Unfold => {
                self.folded = false;
            }
            Msg::Back => {
                // Inform the leaflet that the Tidings page should be hidden
                parent_sender.send(super::Msg::HideTidingsPage).ok();
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
            |icon, _item| {
                // Set the favicon
                icon.set_icon_name(Some("emblem-shared-symbolic"));
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
    // Prepare a sorter
    let sorter = gtk::CustomSorter::new(move |obj_1, obj_2| {
        // Downcast the objects
        if let Some(item_1) = obj_1.downcast_ref::<Item>() {
            if let Some(item_2) = obj_2.downcast_ref::<Item>() {
                // Get the titles
                let title_1: String = item_1.property("title");
                let title_2: String = item_2.property("title");
                // Reverse the sorting order (large strings come first)
                return title_2.cmp(&title_1).into();
            }
        }
        // Default to
        gtk::Ordering::Larger
    });
    // Create a sorter model
    let sort_model = gtk::SortListModel::new(Some(&filter_model), Some(&sorter));
    // Create a selection model
    let selection_model = gtk::NoSelection::new(Some(&sort_model));
    // Create a List View
    gtk::ListView::new(Some(&selection_model), Some(&factory))
}

#[allow(clippy::missing_docs_in_private_items)]
#[relm4::widget(pub)]
impl relm4::Widgets<Model, super::Model> for Widgets {
    view! {
        // Box
        gtk::Box {
            set_width_request: 365,
            set_orientation: gtk::Orientation::Vertical,
            // Header Bar
            append: header_bar = &adw::HeaderBar {
                set_show_start_title_buttons: watch!(
                    !model.folded
                ),
                set_show_end_title_buttons: watch!(
                    !model.folded
                ),
                // Title
                set_title_widget = Some(&adw::WindowTitle) {
                    set_title: "Tidings"
                },
                // Go Back Button
                pack_start = &gtk::Button {
                    set_visible: watch!(model.folded),
                    set_icon_name: "go-previous-symbolic",
                    set_tooltip_text: Some("Go Back"),
                    connect_clicked(sender) => move |_| {
                        sender.send(Msg::Back).ok();
                    },
                },
                // Menu Button
                pack_end = &gtk::MenuButton {
                    set_visible: watch!(!model.folded),
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
        }
    }
}

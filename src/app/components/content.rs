//! Leaflet

mod dictionary;
pub mod source;
pub mod tiding;
mod update;

use generational_arena::{Arena, Index};
use gtk::prelude::{BoxExt, ButtonExt, ListBoxRowExt, OrientableExt, WidgetExt};
use relm4::factory::{DynamicIndex, FactoryVecDeque};
use relm4::{
    ComponentParts, ComponentSender, MessageBroker, SimpleComponent, WidgetPlus, WorkerController,
};
use wyhash::WyHash;

use std::collections::HashMap;
use std::hash::BuildHasherDefault;

use super::AppMsg;
use crate::app::actions::{
    ShowAboutDialog, ShowAddDirectoryDialog, ShowAddFeedDialog, ShowHelpOverlay,
};
use dictionary::Dictionary;
use source::{ArenaSource, ListSource, URLsMap};
use tiding::Model as Tiding;

/// Message broker
pub static BROKER: MessageBroker<Model> = MessageBroker::new();

/// Model
pub struct Model {
    /// Is the leaflet folded?
    folded: bool,
    /// Show tidings in the folded state?
    show_tidings: bool,
    /// Sources List (feeds and directories)
    sources_list: FactoryVecDeque<ListSource>,
    /// Dictionary of (Index, DynamicIndex) key-value pairs
    sources_dictionary: HashMap<Index, DynamicIndex, BuildHasherDefault<WyHash>>,
    /// Sources Arena
    sources_arena: Arena<ArenaSource>,
    /// Sources subtitle
    sources_subtitle: String,
    /// Arena index of the root of all sources
    main_root_index: Index,
    /// Arena index of the currently selected directory
    current_root_index: Index,
    /// Arena index of the currently selected source
    current_source_index: Index,
    /// Tidings list
    tidings_list: FactoryVecDeque<Tiding>,
    /// Dictionary of (URL, Tiding) key-value pairs
    tidings_dictionary: Dictionary,
    /// Is the update running?
    updating: bool,
    /// Number of requests handled (including failed ones)
    update_handled: usize,
    /// Number of nodes requested for update
    update_requested: usize,
    /// Update message handler
    update_worker: Option<WorkerController<update::Model>>,
}

impl Model {
    /// Get a dictionary of the (URL, Vec<Index>) pairs recursively
    fn urls_map(&self) -> Option<URLsMap> {
        // If the root source exists (as it always should!)
        if let Some(root) = self.sources_arena.get(self.main_root_index) {
            // Get a dictionary of the (URL, Vec<Index>) pairs recursively
            let urls_map = root.urls_map(self.main_root_index, &self.sources_arena);
            return Some(urls_map);
        }
        None
    }
    /// Refresh the list of sources with the sources under the current root
    fn refresh_sources(&mut self) {
        // Get the sources list guard
        let mut sources_guard = self.sources_list.guard();
        // If the current root still exists
        if let Some(root) = self.sources_arena.get(self.current_root_index) {
            // Update the subtitle
            self.sources_subtitle = root.title();
            // If the source has children
            if let Some(children) = root.children() {
                // Clear the list of sources
                sources_guard.clear();
                // For each child
                for child_index in children {
                    // If the child source still exists
                    if let Some(child_arena_source) = self.sources_arena.get(*child_index) {
                        // Convert the child arena source to the child list source
                        if let Some(child_list_source) =
                            child_arena_source.clone().into_list_source(*child_index)
                        {
                            // Push the child source to the list
                            sources_guard.push_back(child_list_source);
                        }
                    }
                }
            }
        // Otherwise,
        } else {
            // Render the list as empty
            sources_guard.clear();
        }
    }
    /// Refresh the list of tidings with the tidings of the currently selected source
    fn refresh_tidings(&mut self) {
        // Get the tidings list guard
        let mut tidings_guard = self.tidings_list.guard();
        // If the current source still exists
        if let Some(source) = self.sources_arena.get(self.current_source_index) {
            // Get the URL(s)
            let urls = source.urls(&self.sources_arena);
            // Clear the list of tidings
            tidings_guard.clear();
            // Load the tidings from the URLs
            for url in urls {
                // If there are tidings for this URL
                if let Some(tidings) = self.tidings_dictionary.get(&url) {
                    // Append each tiding to the list
                    for tiding in tidings {
                        tidings_guard.push_back(tiding.clone());
                    }
                }
            }
        // Otherwise,
        } else {
            // Render the list as empty
            tidings_guard.clear();
        }
    }
    /// Insert the source at this index
    fn insert_source(&mut self, new_arena_source: ArenaSource, new_list_index: usize) {
        // Insert the source into the arena
        let new_arena_index = self.sources_arena.insert(new_arena_source.clone());
        // Get the parent source
        if let Some(parent_source) = self.sources_arena.get_mut(self.current_root_index) {
            // Push the new index to the children of the parent
            parent_source.push_to_children(new_arena_index);
        }
        // Convert the arena source to the list source (should always succeed)
        if let Some(new_list_source) = new_arena_source.into_list_source(new_arena_index) {
            // Insert the new source at the specified index
            let new_list_index = self
                .sources_list
                .guard()
                .insert(new_list_index, new_list_source);
            // Connect the two indices
            self.sources_dictionary
                .entry(new_arena_index)
                .and_modify(|dyn_index| *dyn_index = new_list_index.clone())
                .or_insert(new_list_index);
        }
    }
    /// Add a new source to the list
    fn add_source(&mut self, new_arena_source: ArenaSource) {
        // If the current root is selected as the current source
        if self.current_root_index == self.current_source_index {
            // Push the new source to the back of the list
            self.insert_source(new_arena_source, self.sources_list.len());
            // Refresh the tidings list
            self.refresh_tidings();
            return;
        }
        // Otherwise, if the currently selected source still exists
        if let Some(current_arena_source) = self.sources_arena.get(self.current_source_index) {
            // If the currently selected source has a parent
            if let Some(parent_index) = current_arena_source.parent_index() {
                // If the currently selected source is a direct child of the current root
                if self.current_root_index == *parent_index {
                    // If the dynamic index of the currently selected source is
                    // stored in the sources dictionary (as it should be!)
                    if let Some(current_list_dyn_index) =
                        self.sources_dictionary.get(&self.current_source_index)
                    {
                        // Get the current index of the dynamic index
                        let current_list_index = current_list_dyn_index.current_index();
                        // Compute the list index of the new source
                        let new_list_index = current_list_index + 1;
                        // Insert the new source after the current one
                        self.insert_source(new_arena_source, new_list_index);
                    }
                // Otherwise,
                } else {
                    // Push the new source to the back of the list
                    self.insert_source(new_arena_source, self.sources_list.len());
                }
            // Otherwise,
            } else {
                // Push the new source to the back of the list
                self.insert_source(new_arena_source, self.sources_list.len());
            }
        // Otherwise,
        } else {
            // Push the new source to the back of the list
            self.insert_source(new_arena_source, self.sources_list.len());
        }
    }
    /// Show the source by the index
    fn show_source(&mut self, index: Index) {
        // If the source still exists
        if let Some(source) = self.sources_arena.get(index) {
            // If it's a directory
            if source.is_dir() {
                // If the source index is the same as the root index
                if index == self.current_root_index {
                    // If folded
                    if self.folded {
                        // Show the Tidings page
                        self.show_tidings = true;
                    }
                // Otherwise,
                } else {
                    // Update the current root index
                    self.current_root_index = index;
                    // Refresh the sources list
                    self.refresh_sources();
                }
                // If the source index is different from the currently selected one
                if self.current_source_index != index {
                    // Update the current index
                    self.current_source_index = index;
                    // Refresh the tidings list
                    self.refresh_tidings();
                }
            } else {
                // If the source index is different from the currently selected one
                if self.current_source_index != index {
                    // Update the current index
                    self.current_source_index = index;
                    // Refresh the tidings list
                    self.refresh_tidings();
                }
                // If folded
                if self.folded {
                    // Show the Tidings page
                    self.show_tidings = true;
                }
            }
        }
    }
}

/// Messages
#[derive(Debug)]
pub enum Msg {
    //
    // General:
    //
    /// Navigate back through the content
    Back,
    //
    // Leaflet specific:
    //
    /// Set the folding state
    SetFolded(bool),
    //
    // Sources specific:
    //
    /// Add the feed after the current source
    AddFeed(String, String),
    /// Add the directory after the current source
    AddDirectory(String),
    /// Start the update of all feeds
    StartUpdateAll,
    /// Stop the update of all feeds
    StopUpdateAll,
    /// Toggle the update of all feeds
    ToggleUpdateAll,
    /// Update of the particular feed has started
    UpdateStarted(Vec<Index>),
    //
    // Tidings specific:
    //
    /// Insert the tidings at the specified URL
    Insert(Vec<Index>, String, Vec<Tiding>),
    /// Show the tidings from the current root source
    ShowCurrentRoot,
    /// Show the tidings from the particular source in the list
    ShowFromList(i32),
}

/// Get a clone of the Sources List Box
fn sources_list_box(model: &Model) -> gtk::ListBox {
    model.sources_list.widget().clone()
}

/// Get a clone of the Tidings List Box
fn tidings_list_box(model: &Model) -> gtk::ListBox {
    model.tidings_list.widget().clone()
}

#[allow(clippy::as_conversions)]
#[allow(clippy::cast_precision_loss)]
#[allow(clippy::cast_sign_loss)]
#[allow(clippy::clone_on_ref_ptr)]
#[allow(clippy::missing_docs_in_private_items)]
#[relm4::component(pub)]
impl SimpleComponent for Model {
    type Init = ();
    type Input = Msg;
    type Output = AppMsg;
    type Widgets = Widgets;
    fn init(
        _init: Self::Init,
        root: &Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        // Prepare an arena with a root node
        let mut sources_arena = Arena::with_capacity(1);
        let root_source = ArenaSource::new_root();
        let root_index = sources_arena.insert(root_source);
        // Initialize the model
        let model = Self {
            // Whether it's folded is restored on restart
            // by the `connect_folded_notify` function
            folded: false,
            show_tidings: false,
            sources_list: FactoryVecDeque::new(gtk::ListBox::new(), &sender.input),
            sources_dictionary: HashMap::default(),
            sources_arena,
            sources_subtitle: String::from(""),
            main_root_index: root_index,
            current_root_index: root_index,
            current_source_index: root_index,
            tidings_list: FactoryVecDeque::new(gtk::ListBox::new(), &sender.input),
            tidings_dictionary: Dictionary::new(),
            updating: false,
            // Avoiding the nasty division by zero here
            update_handled: 0,
            update_requested: 1,
            update_worker: None,
        };
        let widgets = view_output!();
        // Make sure the separator page isn't navigatable
        let separator_page = widgets.leaflet.page(&widgets.separator);
        separator_page.set_navigatable(false);
        ComponentParts { model, widgets }
    }
    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>) {
        match msg {
            Msg::SetFolded(folded) => {
                self.folded = folded;
            }
            Msg::Back => {
                // If the tidings page is shown
                if self.show_tidings {
                    // Hide the tidings page
                    self.show_tidings = false;
                // Otherwise,
                } else {
                    // If the current root source still exists
                    if let Some(root) = self.sources_arena.get(self.current_root_index) {
                        // If the current root source has a parent
                        if let Some(parent_index) = root.parent_index() {
                            // Change the root
                            self.current_root_index = *parent_index;
                            // Refresh the sources list
                            self.refresh_sources();
                        }
                    }
                }
            }
            Msg::AddFeed(title, url) => {
                // Create a new source
                let new_source = ArenaSource::new_feed(title, url, self.current_root_index);
                // Add it to the list
                self.add_source(new_source);
            }
            Msg::AddDirectory(title) => {
                // Create a new source
                let new_source = ArenaSource::new_directory(title, self.current_root_index);
                // Add it to the list
                self.add_source(new_source);
            }
            Msg::StartUpdateAll => {
                // Get a dictionary of the (URL, Vec<Index>) pairs recursively
                if let Some(urls_map) = self.urls_map() {
                    // If there is something to update
                    if !urls_map.is_empty() {
                        // Create a new update message handler
                        let update = update::new(&sender);
                        // Setup the progress bar
                        self.update_requested = urls_map.len();
                        self.update_handled = 0;
                        // Send the data to the update message handler
                        update.emit(update::Msg::UpdateAll(urls_map));
                        // Notify the UI that the update has started
                        self.updating = true;
                        // Let the model own the message handler
                        self.update_worker = Some(update);
                    }
                }
            }
            Msg::StopUpdateAll => {
                // Drop the message handler (thus,
                // cancelling any ongoing update)
                self.update_worker = None;
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
            Msg::UpdateStarted(indices) => {
                // For each index
                for index in indices {
                    // If there is a source with this index
                    if let Some(source) = self.sources_arena.get_mut(index) {
                        // Update the updating status
                        source.set_updating(true);
                    }
                }
            }
            Msg::Insert(indices, url, tidings) => {
                // Insert the tidings into the dictionary, using the URL as a key
                self.tidings_dictionary.insert(url, tidings);
                // Increment the amount of handled requests
                self.update_handled += 1;
                // For each source
                for index in &indices {
                    // If there is a source with this index
                    if let Some(source) = self.sources_arena.get_mut(*index) {
                        // Remove the updating status of the feed
                        source.set_updating(false);
                    }
                }
                // If the current source still exists
                if let Some(current_source) = self.sources_arena.get(self.current_source_index) {
                    // If it's a directory
                    if current_source.is_dir() {
                        // For each index
                        for index in indices {
                            // If there is a source with this index
                            if let Some(source) = self.sources_arena.get(index) {
                                // If this source is a child (not necessarily a direct one) of the current source
                                if source
                                    .is_child_of(&self.current_source_index, &self.sources_arena)
                                {
                                    // Refresh the tidings list
                                    self.refresh_tidings();
                                    break;
                                }
                            }
                        }
                    // Otherwise, it's a feed, so
                    } else {
                        // For each index
                        for index in indices {
                            // If it matches the current one
                            if index == self.current_source_index {
                                // Refresh the tidings list
                                self.refresh_tidings();
                                break;
                            }
                        }
                    }
                }
            }
            Msg::ShowCurrentRoot => {
                // Show the current root source
                self.show_source(self.current_root_index);
            }
            Msg::ShowFromList(list_index) => {
                // If the source with this index still exists
                if let Some(source) = self.sources_list.get(list_index as usize) {
                    // Get the arena index of the source
                    let index = *source.index();
                    // Show the source
                    self.show_source(index);
                }
            }
        }
    }
    fn pre_view() {
        if model.folded && model.show_tidings {
            // Navigate forward to Tidings
            leaflet.navigate(adw::NavigationDirection::Forward);
        } else {
            // Navigate back to Feeds
            leaflet.navigate(adw::NavigationDirection::Back);
        }
    }
    view! {
        #[wrap(Some)]
        leaflet = &adw::Leaflet {
            connect_folded_notify[sender] => move |leaflet| {
                if leaflet.is_folded() {
                    // Update the folding state
                    sender.input(Msg::SetFolded(true));
                } else {
                    // Update the folding state
                    sender.input(Msg::SetFolded(false));
                    // Hide the tidings page (won't be shown if folded right after)
                    sender.input(Msg::Back);
                }
            },
            // Sources
            prepend = &gtk::Box {
                set_width_request: 365,
                set_orientation: gtk::Orientation::Vertical,
                set_hexpand: true,
                // Header Overlay
                append = &gtk::Overlay {
                    // Header Bar
                    #[wrap(Some)]
                    set_child = &adw::HeaderBar {
                        #[watch]
                        set_show_start_title_buttons: model.folded,
                        #[watch]
                        set_show_end_title_buttons: model.folded,
                        // Title
                        #[wrap(Some)]
                        set_title_widget = &gtk::Overlay {
                            #[wrap(Some)]
                            set_child = &adw::WindowTitle {
                                set_title: "Sources",
                                #[watch]
                                set_subtitle: &model.sources_subtitle,
                            },
                            add_overlay = &gtk::Button {
                                add_css_class: "flat",
                                connect_clicked[sender] => move |_| {
                                    // Show all source under the current root
                                    sender.input(Msg::ShowCurrentRoot);
                                }
                            }
                        },
                        // Go Back Button Revealer
                        pack_start = &gtk::Revealer {
                            #[watch]
                            set_reveal_child: model.current_root_index != model.main_root_index,
                            set_transition_type: gtk::RevealerTransitionType::SlideRight,
                            // Go Back Button
                            #[wrap(Some)]
                            set_child = &gtk::Button {
                                set_icon_name: "go-previous-symbolic",
                                set_tooltip_text: Some("Go Back"),
                                connect_clicked[sender] => move |_| {
                                    sender.input(Msg::Back);
                                },
                            },
                        },
                        // Add Button
                        pack_start = &gtk::MenuButton {
                            set_icon_name: "plus-large-symbolic",
                            set_tooltip_text: Some("Add"),
                            set_menu_model: Some(&add_menu),
                        },
                        // Update All Button
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
                                Some("Update All Sources")
                            },
                            connect_clicked[sender] => move |_| {
                                sender.input(Msg::ToggleUpdateAll);
                            }
                        },
                        // Menu Button Revealer
                        pack_end = &gtk::Revealer {
                            #[watch]
                            set_reveal_child: model.folded,
                            set_transition_type: gtk::RevealerTransitionType::SlideLeft,
                            // Menu Button
                            #[wrap(Some)]
                            set_child = &gtk::MenuButton {
                                set_icon_name: "open-menu-symbolic",
                                set_menu_model: Some(&main_menu),
                            },
                        },
                    },
                    // Update Progress Bar
                    add_overlay = &gtk::Revealer {
                        #[watch]
                        set_reveal_child: model.updating,
                        set_transition_type: gtk::RevealerTransitionType::SlideUp,
                        set_valign: gtk::Align::End,
                        #[wrap(Some)]
                        set_child = &gtk::ProgressBar {
                            add_css_class: "osd",
                            #[watch]
                            set_fraction: model.update_handled as f64 / model.update_requested as f64,
                        }
                    }
                },
                // Sources Scrolled Window
                append: sources_scrolled_window = &gtk::ScrolledWindow {
                    set_hscrollbar_policy: gtk::PolicyType::Never,
                    set_hexpand: true,
                    set_vexpand: true,
                    #[wrap(Some)]
                    set_child = &sources_list_box(&model) -> gtk::ListBox {
                        #[watch]
                        set_visible: !model.sources_list.is_empty(),
                        set_selection_mode: gtk::SelectionMode::None,
                        add_css_class: "boxed-list",
                        set_margin_all: 12,
                        set_valign: gtk::Align::Start,
                        connect_row_activated[sender] => move |_, row| {
                            // Show the tidings from this source
                            sender.input(Msg::ShowFromList(row.index()));
                        }
                    },
                }
            },
            // Separator
            append: separator = &gtk::Separator {
                set_orientation: gtk::Orientation::Horizontal,
            },
            // Tidings
            append = &gtk::Box {
                set_width_request: 365,
                set_orientation: gtk::Orientation::Vertical,
                // Tidings Header Bar
                append = &adw::HeaderBar {
                    #[watch]
                    set_show_start_title_buttons: !model.folded,
                    #[watch]
                    set_show_end_title_buttons: !model.folded,
                    // Title
                    #[wrap(Some)]
                    set_title_widget = &adw::WindowTitle {
                        set_title: "Tidings"
                    },
                    // Go Back Button
                    pack_start = &gtk::Button {
                        #[watch]
                        set_visible: model.folded,
                        set_icon_name: "go-previous-symbolic",
                        set_tooltip_text: Some("Go Back"),
                        connect_clicked[sender] => move |_| {
                            // Hide the tidings page
                            sender.input(Msg::Back);
                        },
                    },
                    // Menu Button Revealer
                    pack_end = &gtk::Revealer {
                        #[watch]
                        set_reveal_child: !model.folded,
                        set_transition_type: gtk::RevealerTransitionType::SlideLeft,
                        // Menu Button
                        #[wrap(Some)]
                        set_child = &gtk::MenuButton {
                            set_icon_name: "open-menu-symbolic",
                            set_menu_model: Some(&main_menu),
                        },
                    },
                },
                // Tidings Scrolled Window
                append: tidings_scrolled_window = &gtk::ScrolledWindow {
                    set_hscrollbar_policy: gtk::PolicyType::Never,
                    set_hexpand: true,
                    set_vexpand: true,
                    #[wrap(Some)]
                    set_child = &tidings_list_box(&model) -> gtk::ListBox {
                        #[watch]
                        set_visible: !model.tidings_list.is_empty(),
                        set_selection_mode: gtk::SelectionMode::None,
                        add_css_class: "boxed-list",
                        set_margin_all: 12,
                        set_valign: gtk::Align::Start,
                    },
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

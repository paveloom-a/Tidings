//! Add Feed Dialog

use adw::prelude::{ActionRowExt, PreferencesRowExt};
use gtk::prelude::{
    BoxExt, ButtonExt, EditableExt, EntryBufferExtManual, EntryExt, GtkWindowExt, OrientableExt,
    WidgetExt,
};
use relm4::{ComponentParts, ComponentSender, MessageBroker, SimpleComponent};

use super::content;
use super::AppMsg;

/// Message broker
pub static BROKER: MessageBroker<Model> = MessageBroker::new();

/// Model
pub struct Model {
    /// Is the window visible?
    visible: bool,
    /// Feed title entry buffer
    title: gtk::EntryBuffer,
    /// Feed URL entry buffer
    url: gtk::EntryBuffer,
    /// Is the feed allowed to be added?
    allowed: bool,
    /// Should we go to the next page?
    next_page: bool,
}

/// Messages
#[derive(Debug)]
pub enum Msg {
    /// Show the dialog
    Show,
    /// Hide the dialog
    Hide,
    /// Check if the feed is allowed to be added
    Check,
    /// Go to the next page
    Next,
    /// Add the feed
    Add,
}

#[allow(clippy::clone_on_ref_ptr)]
#[allow(clippy::missing_docs_in_private_items)]
#[allow(unused_variables)]
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
        // Initialize the model
        let model = Self {
            visible: false,
            title: gtk::EntryBuffer::default(),
            url: gtk::EntryBuffer::default(),
            allowed: false,
            next_page: false,
        };
        let widgets = view_output!();
        // Set the names
        let title_boxed_list_page = widgets.stack.page(&widgets.title_boxed_list);
        let url_boxed_list_page = widgets.stack.page(&widgets.url_boxed_list);
        title_boxed_list_page.set_name("title_boxed_list");
        url_boxed_list_page.set_name("url_boxed_list");
        ComponentParts { model, widgets }
    }
    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>) {
        match msg {
            Msg::Show => self.visible = true,
            Msg::Hide => {
                // Hide the widget
                self.visible = false;
                // Get back to the first page
                self.next_page = false;
                // Empty the buffers
                self.title.delete_text(0, None);
                self.url.delete_text(0, None);
            }
            Msg::Check => {
                // If we're on the next page
                self.allowed = if self.next_page {
                    // Decide from the text of the title
                    !self.title.text().is_empty()
                } else {
                    // Decide from the text of the URL
                    !self.url.text().is_empty()
                }
            }
            Msg::Next => {
                // Proceed with adding data to the feed
                self.next_page = true;
            }
            Msg::Add => {
                // Get the title
                let title = self.title.text();
                // Get the URL
                let url = self.url.text();
                // Add the source
                content::BROKER.send(content::Msg::AddFeed(title, url));
                // Hide the dialog
                sender.input(Msg::Hide);
            }
        }
    }
    fn pre_view() {
        // If we just opened the dialog
        if !add_feed_dialog.is_visible() {
            // Move to the first page
            stack.set_visible_child(url_boxed_list);
            // Update the default widget
            add_feed_dialog.set_default_widget(Some(url_add_button));
            // Focus on the URL entry
            url_entry.grab_focus();
        }
        // If there is a visible child
        if let Some(visible_child_name) = stack.visible_child_name() {
            // If we are to be on the next page and we're not there yet
            if self.next_page && visible_child_name == "url_boxed_list" {
                // Move to the next page
                stack.set_visible_child(title_boxed_list);
                // Update the default widget
                add_feed_dialog.set_default_widget(Some(title_add_button));
                // Let the first entry grab the focus
                title_entry.grab_focus();
            }
        }
    }
    view! {
        add_feed_dialog = gtk::Dialog {
            set_title: Some("Add New Feed"),
            set_width_request: 313,
            set_modal: true,
            set_vexpand: false,
            #[watch]
            set_visible: model.visible,
            set_default_widget: Some(&url_add_button),
            connect_close_request[sender] => move |_| {
                sender.input(Msg::Hide);
                gtk::Inhibit(false)
            },
            // Clamp
            #[wrap(Some)]
            set_child = &adw::Clamp {
                set_maximum_size: 400,
                // Box
                #[wrap(Some)]
                set_child = &gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_margin_top: 24,
                    set_margin_bottom: 24,
                    set_margin_start: 12,
                    set_margin_end: 12,
                    set_spacing: 24,
                    // Stack
                    append: stack = &gtk::Stack {
                        set_transition_type: gtk::StackTransitionType::SlideLeft,
                        // URL Boxed List
                        add_child: url_boxed_list = &gtk::ListBox {
                            set_selection_mode: gtk::SelectionMode::None,
                            add_css_class: "boxed-list",
                            // URL Action Row
                            append = &adw::ActionRow {
                                set_title: "URL",
                                // URL Title Entry
                                add_suffix: url_entry = &gtk::Entry {
                                    set_margin_top: 7,
                                    set_margin_bottom: 7,
                                    set_buffer: &model.url,
                                    set_input_purpose: gtk::InputPurpose::Url,
                                    set_activates_default: true,
                                    // Check if adding the feed is allowed on an entry change
                                    connect_changed[sender] => move |_| {
                                        sender.input(Msg::Check);
                                    }
                                },
                                // URL Add Button
                                add_suffix: url_add_button = &gtk::Button {
                                    set_margin_top: 7,
                                    set_margin_bottom: 7,
                                    set_css_classes: &["suggested-action", "circular"],
                                    set_icon_name: "plus-large-symbolic",
                                    #[watch]
                                    set_sensitive: model.allowed,
                                    // Proceed with adding data to the feed
                                    connect_activate[sender] => move |_| {
                                        sender.input(Msg::Next);
                                    }
                                },
                            }
                        },
                        // Title Boxed List
                        add_child: title_boxed_list = &gtk::ListBox {
                            set_selection_mode: gtk::SelectionMode::None,
                            add_css_class: "boxed-list",
                            // Title Action Row
                            append = &adw::ActionRow {
                                set_title: "Title",
                                // Feed Title Entry
                                add_suffix: title_entry = &gtk::Entry {
                                    set_margin_top: 7,
                                    set_margin_bottom: 7,
                                    set_buffer: &model.title,
                                    set_input_purpose: gtk::InputPurpose::Name,
                                    set_activates_default: true,
                                    // Check if adding the feed is allowed on an entry change
                                    connect_changed[sender] => move |_| {
                                        sender.input(Msg::Check);
                                    }
                                },
                                // Feed Add Button
                                add_suffix: title_add_button = &gtk::Button {
                                    set_margin_top: 7,
                                    set_margin_bottom: 7,
                                    set_css_classes: &["suggested-action", "circular"],
                                    set_icon_name: "plus-large-symbolic",
                                    #[watch]
                                    set_sensitive: model.allowed,
                                    // Add on the press of the button
                                    connect_activate[sender] => move |_| {
                                        sender.input(Msg::Add);
                                    }
                                },
                            }
                        },
                    },
                }
            },
        }
    }
}

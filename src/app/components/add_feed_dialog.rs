//! Add Feed Dialog

use adw::prelude::{ActionRowExt, PreferencesRowExt};
use gtk::prelude::{
    BoxExt, ButtonExt, EditableExt, EntryBufferExtManual, EntryExt, GtkWindowExt, OrientableExt,
    WidgetExt,
};
use relm4::{ComponentParts, ComponentSender, MessageBroker, SimpleComponent};

use super::leaflet::feeds;
use super::AppMsg;

/// Message broker
pub static BROKER: MessageBroker<Model> = MessageBroker::new();

/// Model
pub struct Model {
    /// Is the window visible?
    visible: bool,
    /// Feed title entry buffer
    title: gtk::EntryBuffer,
    /// Is the feed allowed to be added?
    allowed: bool,
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
            allowed: false,
        };
        let widgets = view_output!();
        ComponentParts { model, widgets }
    }
    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>) {
        match msg {
            Msg::Show => self.visible = true,
            Msg::Hide => {
                // Hide the widget
                self.visible = false;
                // Empty the buffer
                self.title.delete_text(0, None);
            }
            Msg::Check => self.allowed = !self.title.text().is_empty(),
            Msg::Add => {
                // Get the title
                let title = self.title.text();
                // Prepare a node
                let node = feeds::tree::Node::new_feed(title, "".to_owned());
                // Prepare a message for the Feeds component
                let msg = feeds::Msg::Add(node);
                // Send the message
                sender.output(AppMsg::TransferToFeeds(msg));
                // Hide the dialog
                sender.input(Msg::Hide);
            }
        }
    }
    fn pre_view() {
        // Focus on the title entry when opening the dialog
        if !add_feed_dialog.is_visible() {
            title_entry.grab_focus();
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
            set_default_widget: Some(&add_button),
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
                    // Boxed List
                    append = &gtk::ListBox {
                        set_selection_mode: gtk::SelectionMode::None,
                        add_css_class: "boxed-list",
                        // Title Action Row
                        append = &adw::ActionRow {
                            set_title: "Title",
                            // Feed title entry
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
                            // Add Button
                            add_suffix: add_button = &gtk::Button {
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
                }
            },
        }
    }
}

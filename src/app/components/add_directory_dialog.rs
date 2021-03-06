//! Add Directory Dialog

use adw::prelude::{ActionRowExt, PreferencesRowExt};
use gtk::prelude::{
    BoxExt, ButtonExt, EditableExt, EntryBufferExtManual, EntryExt, GtkWindowExt, OrientableExt,
    WidgetExt,
};
use relm4::{ComponentUpdate, Sender};

use super::leaflet::feeds;
use super::{AppModel, AppMsg};

/// Model
pub struct Model {
    /// Is the window visible?
    visible: bool,
    /// Directory title entry buffer
    title: gtk::EntryBuffer,
    /// Is the directory allowed to be added?
    allowed: bool,
}

/// Messages
pub enum Msg {
    /// Show the dialog
    Show,
    /// Hide the dialog
    Hide,
    /// Check if the directory is allowed to be added
    Check,
    /// Add the directory
    Add,
}

impl relm4::Model for Model {
    type Msg = Msg;
    type Widgets = Widgets;
    type Components = ();
}

impl ComponentUpdate<AppModel> for Model {
    fn init_model(_parent_model: &AppModel) -> Self {
        Self {
            visible: false,
            title: gtk::EntryBuffer::default(),
            allowed: false,
        }
    }
    fn update(
        &mut self,
        msg: Msg,
        _components: &(),
        sender: Sender<Msg>,
        parent_sender: Sender<AppMsg>,
    ) {
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
                let node = feeds::tree::Node::new_directory(title);
                // Prepare a message for the Feeds component
                let msg = feeds::Msg::Add(node);
                // Send the message
                parent_sender.send(AppMsg::TransferToFeeds(msg)).ok();
                // Hide the dialog
                sender.send(Msg::Hide).ok();
            }
        }
    }
}

#[allow(clippy::missing_docs_in_private_items)]
#[relm4::widget(pub)]
impl relm4::Widgets<Model, AppModel> for Widgets {
    view! {
        add_directory_dialog = gtk::Dialog {
            set_title: Some("Add New Directory"),
            set_width_request: 313,
            set_modal: true,
            set_transient_for: parent!(Some(&parent_widgets.app_window)),
            set_vexpand: false,
            set_visible: watch!(model.visible),
            set_default_widget: Some(&add_button),
            connect_close_request(sender) => move |_| {
                sender.send(Msg::Hide).ok();
                gtk::Inhibit(false)
            },
            // Clamp
            set_child = Some(&adw::Clamp) {
                set_maximum_size: 400,
                // Box
                set_child = Some(&gtk::Box) {
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
                        // Action Row
                        append = &adw::ActionRow {
                            set_title: "Title",
                            // Directory title entry
                            add_suffix: title_entry = &gtk::Entry {
                                set_margin_top: 7,
                                set_margin_bottom: 7,
                                set_buffer: &model.title,
                                set_input_purpose: gtk::InputPurpose::Name,
                                set_activates_default: true,
                            },
                            // Add Button
                            add_suffix: add_button = &gtk::Button {
                                set_margin_top: 7,
                                set_margin_bottom: 7,
                                set_css_classes: &["suggested-action", "circular"],
                                set_icon_name: "plus-large-symbolic",
                                set_sensitive: watch!(model.allowed),
                            },
                        }
                    },
                }
            },
        }
    }
    fn pre_view() {
        // Focus on the title entry when opening the dialog
        if !add_directory_dialog.is_visible() {
            title_entry.grab_focus();
        }
    }
    fn post_init() {
        // Check if adding the directory is allowed on an entry change
        title_entry.connect_changed({
            let sender = sender.clone();
            move |_| {
                sender.send(Msg::Check).ok();
            }
        });
        // Add on the press of the button
        add_button.connect_activate({
            move |_| {
                sender.send(Msg::Add).ok();
            }
        });
    }
}

//! About Dialog

use gettextrs::gettext;
use gtk::prelude::{GtkWindowExt, WidgetExt};
use relm4::{ComponentUpdate, Sender};

use super::{AppModel, AppMsg};
use crate::config::{APP_ID, VERSION};

/// Model
pub struct Model {
    /// Is the window visible?
    visible: bool,
}

/// Messages
pub enum Msg {
    /// Show the window
    Show,
    /// Hide the window
    Hide,
}

impl relm4::Model for Model {
    type Msg = Msg;
    type Widgets = Widgets;
    type Components = ();
}

impl ComponentUpdate<AppModel> for Model {
    fn init_model(_parent_model: &AppModel) -> Self {
        Self { visible: false }
    }
    fn update(
        &mut self,
        msg: Msg,
        _components: &(),
        _sender: Sender<Msg>,
        _parent_sender: Sender<AppMsg>,
    ) {
        match msg {
            Msg::Show => self.visible = true,
            Msg::Hide => self.visible = false,
        }
    }
}

#[allow(clippy::missing_docs_in_private_items)]
#[relm4::widget(pub)]
impl relm4::Widgets<Model, AppModel> for Widgets {
    view! {
        gtk::AboutDialog {
            set_artists: &["Pavel Sobolev"],
            set_authors: &["Pavel Sobolev"],
            set_license_type: gtk::License::Gpl30Only,
            set_logo_icon_name: Some(APP_ID),
            set_modal: true,
            set_transient_for: parent!(Some(&parent_widgets.app_window)),
            set_translator_credits: Some(&gettext("translator-credits")),
            set_version: Some(VERSION),
            set_visible: watch!(model.visible),
            set_website: Some("https://github.com/paveloom-a/Tidings"),
            connect_close_request(sender) => move |_| {
                sender.send(Msg::Hide).ok();
                gtk::Inhibit(false)
            },
        }
    }
}

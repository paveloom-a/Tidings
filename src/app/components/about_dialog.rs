//! About Dialog

use gettextrs::gettext;
use gtk::prelude::{GtkWindowExt, WidgetExt};
use relm4::{ComponentParts, ComponentSender, MessageBroker, SimpleComponent};

use super::AppMsg;
use crate::config::{APP_ID, VERSION};

/// Message broker
pub static BROKER: MessageBroker<Model> = MessageBroker::new();

/// Model
pub struct Model {
    /// Is the window visible?
    visible: bool,
}

/// Messages
#[derive(Debug)]
pub enum Msg {
    /// Show the window
    Show,
    /// Hide the window
    Hide,
}

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
        // Initialize the model
        let model = Self { visible: false };
        let widgets = view_output!();
        ComponentParts { model, widgets }
    }
    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
            Msg::Show => self.visible = true,
            Msg::Hide => self.visible = false,
        }
    }
    view! {
        gtk::AboutDialog {
            set_artists: &["Pavel Sobolev"],
            set_authors: &["Pavel Sobolev"],
            set_license_type: gtk::License::Gpl30Only,
            set_logo_icon_name: Some(APP_ID),
            set_modal: true,
            set_translator_credits: Some(&gettext("translator-credits")),
            set_version: Some(VERSION),
            #[watch]
            set_visible: model.visible,
            set_website: Some("https://github.com/paveloom-a/Tidings"),
            connect_close_request => move |_| {
                sender.input(Msg::Hide);
                gtk::Inhibit(false)
            },
        }
    }
}

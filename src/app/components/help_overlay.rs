//! Help Overlay

use gtk::prelude::{GtkWindowExt, WidgetExt};
use relm4::{ComponentParts, ComponentSender, MessageBroker, SimpleComponent};

use super::AppMsg;

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

/// Get a `ShortcutsWindow`
#[allow(clippy::expect_used)]
fn shortcuts_window() -> gtk::ShortcutsWindow {
    gtk::Builder::from_resource("/paveloom/apps/tidings/gtk/help-overlay.ui")
        .object("help_overlay")
        .expect("Couldn't build the Help Overlay")
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
        shortcuts_window() -> gtk::ShortcutsWindow {
            #[watch]
            set_visible: model.visible,
            connect_close_request[sender] => move |_| {
                sender.input(Msg::Hide);
                gtk::Inhibit(false)
            }
        }
    }
}

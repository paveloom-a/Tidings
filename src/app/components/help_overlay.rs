//! Help Overlay

use gtk::prelude::{GtkWindowExt, WidgetExt};
use relm4::{ComponentUpdate, Sender};

use super::{AppModel, AppMsg};

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

/// Get a `ShortcutsWindow`
#[allow(clippy::expect_used)]
fn shortcuts_window() -> gtk::ShortcutsWindow {
    gtk::Builder::from_resource("/paveloom/apps/tidings/gtk/help-overlay.ui")
        .object("help_overlay")
        .expect("Couldn't build the Help Overlay")
}

#[allow(clippy::missing_docs_in_private_items)]
#[relm4::widget(pub)]
impl relm4::Widgets<Model, AppModel> for Widgets {
    view! {
        shortcuts_window() -> gtk::ShortcutsWindow {
            set_transient_for: parent!(Some(&parent_widgets.app_window)),
            set_visible: watch!(model.visible),
            connect_close_request(sender) => move |_| {
                sender.send(Msg::Hide).ok();
                gtk::Inhibit(false)
            }
        }
    }
}

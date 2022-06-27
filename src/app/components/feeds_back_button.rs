//! Feeds Back Button

use gtk::prelude::{ButtonExt, WidgetExt};
use relm4::{ComponentUpdate, Sender};

use super::{AppModel, AppMsg};

/// Model
pub struct Model {
    /// Is the button visible?
    visible: bool,
}

/// Messages
pub enum Msg {
    /// Show the button
    Show,
    /// Hide the button
    Hide,
    /// Go one level up in the tree of feeds
    Back,
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
        parent_sender: Sender<AppMsg>,
    ) {
        match msg {
            Msg::Show => self.visible = true,
            Msg::Hide => self.visible = false,
            Msg::Back => parent_sender.send(AppMsg::FeedsBack).unwrap_or_else(|e| {
                log::error!("Couldn't send a message to the parent to go back in the Feeds");
                log::debug!("{e}");
            }),
        }
    }
}

#[allow(clippy::missing_docs_in_private_items)]
#[relm4_macros::widget(pub)]
impl relm4::Widgets<Model, AppModel> for Widgets {
    view! {
        gtk::Button {
            set_visible: watch! { model.visible },
            set_icon_name: "go-previous-symbolic",
            connect_clicked(sender) => move |_| {
                sender.send(Msg::Back).unwrap_or_else(|e| {
                    log::error!("Couldn't send a message to go back in the Feeds");
                    log::debug!("{e}");
                });
            },
        }
    }
}

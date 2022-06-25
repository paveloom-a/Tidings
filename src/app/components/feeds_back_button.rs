//! Feeds Back Button

use gtk::prelude::{ButtonExt, WidgetExt};
use relm4::{send, ComponentUpdate, Sender};

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
            Msg::Back => {
                send!(parent_sender, AppMsg::FeedsBack);
            }
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
                send!(sender, Msg::Back);
            },
        }
    }
}

use gtk::prelude::{ButtonExt, WidgetExt};
use relm4::{send, ComponentUpdate, Model, Sender, Widgets};

use super::{AppModel, AppMsg};

pub struct FeedsBackButtonModel {
    visible: bool,
}

pub enum FeedsBackButtonMsg {
    Show,
    Hide,
    Back,
}

impl Model for FeedsBackButtonModel {
    type Msg = FeedsBackButtonMsg;
    type Widgets = FeedsBackButtonWidgets;
    type Components = ();
}

impl ComponentUpdate<AppModel> for FeedsBackButtonModel {
    fn init_model(_parent_model: &AppModel) -> Self {
        FeedsBackButtonModel { visible: false }
    }

    fn update(
        &mut self,
        msg: FeedsBackButtonMsg,
        _components: &(),
        _sender: Sender<FeedsBackButtonMsg>,
        parent_sender: Sender<AppMsg>,
    ) {
        match msg {
            FeedsBackButtonMsg::Show => self.visible = true,
            FeedsBackButtonMsg::Hide => self.visible = false,
            FeedsBackButtonMsg::Back => {
                send!(parent_sender, AppMsg::FeedsBack);
            }
        }
    }
}

#[relm4_macros::widget(pub)]
impl Widgets<FeedsBackButtonModel, AppModel> for FeedsBackButtonWidgets {
    view! {
        gtk::Button {
            set_visible: watch! { model.visible },
            set_icon_name: "go-previous-symbolic",
            connect_clicked(sender) => move |_| {
                send!(sender, FeedsBackButtonMsg::Back);
            },
        }
    }
}

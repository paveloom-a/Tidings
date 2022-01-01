use gettextrs::gettext;
use gtk::prelude::{GtkWindowExt, WidgetExt};
use relm4::{send, ComponentUpdate, Model, Sender, Widgets};

use super::{AppModel, AppMsg};
use crate::config::{APP_ID, VERSION};

pub struct AboutDialogModel {
    visible: bool,
}

pub enum AboutDialogMsg {
    Open,
    Close,
}

impl Model for AboutDialogModel {
    type Msg = AboutDialogMsg;
    type Widgets = AboutDialogWidgets;
    type Components = ();
}

impl ComponentUpdate<AppModel> for AboutDialogModel {
    fn init_model(_parent_model: &AppModel) -> Self {
        AboutDialogModel { visible: false }
    }

    fn update(
        &mut self,
        msg: AboutDialogMsg,
        _components: &(),
        _sender: Sender<AboutDialogMsg>,
        _parent_sender: Sender<AppMsg>,
    ) {
        match msg {
            AboutDialogMsg::Open => self.visible = true,
            AboutDialogMsg::Close => self.visible = false,
        }
    }
}

#[relm4_macros::widget(pub)]
impl Widgets<AboutDialogModel, AppModel> for AboutDialogWidgets {
    view! {
        gtk::AboutDialog {
            set_logo_icon_name: Some(APP_ID),
            set_license_type: gtk::License::Gpl30Only,
            set_website: Some("https://github.com/paveloom-a/Tidings"),
            set_version: Some(VERSION),
            set_transient_for: parent!(Some(&parent_widgets.main_window)),
            set_translator_credits: Some(&gettext("translator-credits")),
            set_modal: true,
            set_authors: &["Pavel Sobolev"],
            set_artists: &["Pavel Sobolev"],
            set_visible: watch!(model.visible),
            connect_close_request(sender) => move |_| {
                send!(sender, AboutDialogMsg::Close);
                gtk::Inhibit(true)
            }
        }
    }
}

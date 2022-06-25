use gtk::prelude::{GtkWindowExt, WidgetExt};
use log::error;
use relm4::{send, ComponentUpdate, Model, Sender, Widgets};

use super::{AppModel, AppMsg};

pub struct HelpOverlayModel {
    visible: bool,
}

pub enum HelpOverlayMsg {
    Open,
    Close,
}

impl Model for HelpOverlayModel {
    type Msg = HelpOverlayMsg;
    type Widgets = HelpOverlayWidgets;
    type Components = ();
}

impl ComponentUpdate<AppModel> for HelpOverlayModel {
    fn init_model(_parent_model: &AppModel) -> Self {
        HelpOverlayModel { visible: false }
    }

    fn update(
        &mut self,
        msg: HelpOverlayMsg,
        _components: &(),
        _sender: Sender<HelpOverlayMsg>,
        _parent_sender: Sender<AppMsg>,
    ) {
        match msg {
            HelpOverlayMsg::Open => self.visible = true,
            HelpOverlayMsg::Close => self.visible = false,
        }
    }
}

fn help_overlay() -> gtk::ShortcutsWindow {
    if let Some(sw) = gtk::Builder::from_resource("/paveloom/apps/tidings/gtk/help-overlay.ui")
        .object::<gtk::ShortcutsWindow>("help_overlay")
    {
        sw
    } else {
        error!("Failed to load Shortcuts UI");
        gtk::builders::ShortcutsWindowBuilder::default().build()
    }
}

#[relm4_macros::widget(pub)]
impl Widgets<HelpOverlayModel, AppModel> for HelpOverlayWidgets {
    view! {
        help_overlay() -> gtk::ShortcutsWindow {
            set_transient_for: parent!(Some(&parent_widgets.main_window)),
            set_visible: watch!(model.visible),
            connect_close_request(sender) => move |_| {
                send!(sender, HelpOverlayMsg::Close);
                gtk::Inhibit(false)
            }
        }
    }
}

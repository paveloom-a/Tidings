//! Leaflet

mod feeds;
mod tidings;

use relm4::{ComponentUpdate, RelmComponent, Sender};

use super::{AppModel, AppMsg};

/// Model
pub struct Model;

/// Messages
pub enum Msg {
    /// Fold the leaflet
    Fold,
    /// Unfold the leaflet
    Unfold,
}

/// Components
#[derive(relm4_macros::Components)]
pub struct Components {
    /// Feeds
    feeds: RelmComponent<feeds::Model, Model>,
    /// Tidings
    tidings: RelmComponent<tidings::Model, Model>,
}

impl relm4::Model for Model {
    type Msg = Msg;
    type Widgets = Widgets;
    type Components = Components;
}

impl ComponentUpdate<AppModel> for Model {
    fn init_model(_parent_model: &AppModel) -> Self {
        Self
    }
    fn update(
        &mut self,
        msg: Msg,
        components: &Components,
        _sender: Sender<Msg>,
        _parent_sender: Sender<AppMsg>,
    ) {
        match msg {
            Msg::Fold => components.feeds.send(feeds::Msg::Fold).ok(),
            Msg::Unfold => components.feeds.send(feeds::Msg::Unfold).ok(),
        };
    }
}

#[allow(clippy::missing_docs_in_private_items)]
#[relm4_macros::widget(pub)]
impl relm4::Widgets<Model, AppModel> for Widgets {
    view! {
        leaflet = Some(&adw::Leaflet) {
            // Notify the components on the folding state
            connect_folded_notify(sender) => move |leaflet| {
                if leaflet.is_folded() {
                    sender.send(Msg::Fold).ok();
                } else {
                    sender.send(Msg::Unfold).ok();
                }
            },
            // Feeds
            prepend: components.feeds.root_widget(),
            // Separator
            append: &gtk::Separator::new(gtk::Orientation::Horizontal),
            // Tidings
            append: components.tidings.root_widget(),
        }
    }
}

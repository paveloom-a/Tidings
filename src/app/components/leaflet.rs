//! Leaflet

pub mod feeds;
mod handlers;
pub mod tidings;

use generational_arena::Index;
use relm4::{ComponentUpdate, RelmComponent, RelmMsgHandler, Sender};

use super::{AppModel, AppMsg};
use crate::app::leaflet::tidings::dictionary::Tidings;
use feeds::tree::IndicesUrls;

/// Model
pub struct Model;

/// Messages
pub enum Msg {
    /// Transfer a message to the Feeds component
    TransferToFeeds(feeds::Msg),
    /// Start update of all feeds
    UpdateAll(IndicesUrls),
    /// Update of the particular feed finished
    UpdateFinished(Index, Tidings),
    /// Show the tidings of the particular feed
    ShowTidings(Index),
}

/// Components
#[derive(relm4_macros::Components)]
pub struct Components {
    /// Feeds
    feeds: RelmComponent<feeds::Model, Model>,
    /// Tidings
    tidings: RelmComponent<tidings::Model, Model>,
    /// Update message handler
    update: RelmMsgHandler<handlers::update::AsyncHandler, Model>,
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
            Msg::TransferToFeeds(message) => {
                // Transfer the message to the feeds
                components.feeds.send(message).ok();
            }
            Msg::UpdateAll(indices_urls) => {
                // Transfer these to the update message handler
                components
                    .update
                    .send(handlers::update::Msg::UpdateAll(indices_urls));
            }
            Msg::UpdateFinished(index, tidings) => {
                // Remove the updating status of the feed
                components
                    .feeds
                    .send(feeds::Msg::UpdateFinished(index))
                    .ok();
                // Send the tidings to the Tidings component,
                // so they're stored in the dictionary
                components
                    .tidings
                    .send(tidings::Msg::UpdateFinished(index, tidings))
                    .ok();
            }
            Msg::ShowTidings(index) => {
                // Inform Tidings to show the tidings of the specified feed
                components.tidings.send(tidings::Msg::Show(index)).ok();
            }
        }
    }
}

#[allow(clippy::missing_docs_in_private_items)]
#[relm4_macros::widget(pub)]
impl relm4::Widgets<Model, AppModel> for Widgets {
    view! {
        leaflet = Some(&adw::Leaflet) {
            // Feeds
            prepend: components.feeds.root_widget(),
            // Separator
            append: &gtk::Separator::new(gtk::Orientation::Horizontal),
            // Tidings
            append: components.tidings.root_widget(),
        }
    }
    fn post_init() {
        // Notify the components on the folding state
        {
            let sender = components.feeds.sender();
            leaflet.connect_folded_notify(move |leaflet| {
                if leaflet.is_folded() {
                    sender.send(feeds::Msg::ShowEndButtons).ok();
                } else {
                    sender.send(feeds::Msg::HideEndButtons).ok();
                }
            });
        }
    }
}

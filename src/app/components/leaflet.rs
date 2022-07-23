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
pub struct Model {
    /// Is the leaflet folded?
    folded: bool,
    /// Show tidings in the folded state?
    show_tidings: bool,
}

/// Messages
pub enum Msg {
    /// Set the folding state
    SetFolded(bool),
    /// Transfer a message to the Feeds component
    TransferToFeeds(feeds::Msg),
    /// Transfer a message to the Tidings component
    TransferToTidings(tidings::Msg),
    /// Start the update of all feeds
    UpdateAll(IndicesUrls),
    /// Update of the particular feed finished
    UpdateFinished(Index, Tidings),
    /// Show the Tidings page
    ShowTidingsPage,
    /// Hide the Tidings page
    HideTidingsPage,
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
        Self {
            // Whether it's folded is restored on restart
            // by the `connect_folded_notify` function
            folded: false,
            show_tidings: false,
        }
    }
    fn update(
        &mut self,
        msg: Msg,
        components: &Components,
        _sender: Sender<Msg>,
        _parent_sender: Sender<AppMsg>,
    ) {
        match msg {
            Msg::SetFolded(folded) => {
                self.folded = folded;
            }
            Msg::TransferToFeeds(message) => {
                components.feeds.send(message).ok();
            }
            Msg::TransferToTidings(message) => {
                components.tidings.send(message).ok();
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
            Msg::ShowTidingsPage => {
                // This is done here and not in the message above
                // just to make sure that the tidings are ready
                if self.folded {
                    // Show the Tidings page
                    self.show_tidings = true;
                    // Hide the buttons in the end of the Tidings' Header Bar
                    components.tidings.send(tidings::Msg::HideEndButtons).ok();
                }
            }
            Msg::HideTidingsPage => {
                // Hide the Tidings page
                self.show_tidings = false;
                // Show the buttons in the end of the Tidings' Header Bar
                components.tidings.send(tidings::Msg::ShowEndButtons).ok();
            }
        }
    }
}

#[allow(clippy::missing_docs_in_private_items)]
#[relm4_macros::widget(pub)]
impl relm4::Widgets<Model, AppModel> for Widgets {
    view! {
        leaflet = Some(&adw::Leaflet) {
            set_transition_type: adw::LeafletTransitionType::Slide,
            connect_folded_notify[
                feeds_sender = components.feeds.sender(),
                tidings_sender = components.tidings.sender(),
            ] => move |leaflet| {
                if leaflet.is_folded() {
                    // Update the folding state
                    sender.send(Msg::SetFolded(true)).ok();
                    // Inform Tidings to show the back button
                    sender.send(Msg::TransferToTidings(
                        tidings::Msg::ShowBackButton
                    )).ok();
                    // Show the buttons in the end of the Tidings' Header Bar
                    feeds_sender.send(feeds::Msg::ShowEndButtons).ok();
                } else {
                    // Update the folding state
                    sender.send(Msg::SetFolded(false)).ok();
                    // Hide the buttons in the end of the Feeds' Header Bar
                    feeds_sender.send(feeds::Msg::HideEndButtons).ok();
                    // Inform Tidings to hide the back button
                    tidings_sender.send(tidings::Msg::HideBackButton).ok();
                    // Hide the Tidings page (won't be shown if folded right after)
                    sender.send(Msg::HideTidingsPage).ok();
                }
            },
        }
    }
    fn post_init() {
        // Doing this manually just to make
        // sure the separator isn't navigatable

        // Feeds
        leaflet.prepend(components.feeds.root_widget());
        // Separator
        let separator_page = leaflet.append(&gtk::Separator::new(gtk::Orientation::Horizontal));
        separator_page.set_navigatable(false);
        // Tidings
        leaflet.append(components.tidings.root_widget());
    }
    fn pre_view() {
        if model.folded && model.show_tidings {
            // Navigate forward to Tidings
            leaflet.navigate(adw::NavigationDirection::Forward);
        } else {
            // Navigate back to Feeds
            leaflet.navigate(adw::NavigationDirection::Back);
        }
    }
}

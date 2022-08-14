//! Leaflet

pub mod feeds;
mod handlers;
pub mod tidings;

use relm4::{
    Component, ComponentController, ComponentParts, ComponentSender, Controller, MessageBroker,
    SimpleComponent,
};

use std::convert::identity;

use super::AppMsg;
use feeds::tree::IndicesUrls;

/// Message broker
pub static BROKER: MessageBroker<Model> = MessageBroker::new();

/// Model
pub struct Model {
    /// Is the leaflet folded?
    folded: bool,
    /// Show tidings in the folded state?
    show_tidings: bool,
    /// Feeds
    feeds: Controller<feeds::Model>,
    /// Tidings
    tidings: Controller<tidings::Model>,
    /// Update message handler
    update: Controller<handlers::update::Model>,
}

/// Messages
#[derive(Debug)]
pub enum Msg {
    /// Set the folding state
    SetFolded(bool),
    /// Transfer a message to the Feeds component
    TransferToFeeds(feeds::Msg),
    /// Transfer a message to the Tidings component
    TransferToTidings(tidings::Msg),
    /// Start the update of all feeds
    UpdateAll(IndicesUrls),
    /// Show the Tidings page
    ShowTidingsPage,
    /// Hide the Tidings page
    HideTidingsPage,
}

#[allow(clippy::missing_docs_in_private_items)]
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
        // Initialize the components
        let feeds = feeds::Model::builder()
            .launch_with_broker((), &feeds::BROKER)
            .forward(sender.input_sender(), identity);
        let tidings = tidings::Model::builder()
            .launch_with_broker((), &tidings::BROKER)
            .forward(sender.input_sender(), identity);
        let update = handlers::update::Model::builder()
            .launch_with_broker((), &handlers::update::BROKER)
            .forward(sender.input_sender(), identity);
        // Initialize the model
        let model = Self {
            // Whether it's folded is restored on restart
            // by the `connect_folded_notify` function
            folded: false,
            show_tidings: false,
            feeds,
            tidings,
            update,
        };
        let widgets = view_output!();
        // Attaching components manually just to make
        // sure the separator isn't navigatable
        //
        // Feeds
        widgets.leaflet.prepend(model.feeds.widget());
        // Separator
        let separator_page = widgets
            .leaflet
            .append(&gtk::Separator::new(gtk::Orientation::Horizontal));
        separator_page.set_navigatable(false);
        // Tidings
        widgets.leaflet.append(model.tidings.widget());
        ComponentParts { model, widgets }
    }
    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
            Msg::SetFolded(folded) => {
                self.folded = folded;
            }
            Msg::TransferToFeeds(message) => {
                self.feeds.sender().send(message);
            }
            Msg::TransferToTidings(message) => {
                self.tidings.sender().send(message);
            }
            Msg::UpdateAll(indices_urls) => {
                // Transfer these to the update message handler
                self.update
                    .sender()
                    .send(handlers::update::Msg::UpdateAll(indices_urls));
            }
            Msg::ShowTidingsPage => {
                // This is done here and not in the message above
                // just to make sure that the tidings are ready
                if self.folded {
                    // Show the Tidings page
                    self.show_tidings = true;
                }
            }
            Msg::HideTidingsPage => {
                // Hide the Tidings page
                self.show_tidings = false;
            }
        }
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
    view! {
        #[wrap(Some)]
        leaflet = &adw::Leaflet {
            connect_folded_notify => move |leaflet| {
                if leaflet.is_folded() {
                    // Update the folding state
                    sender.input(Msg::SetFolded(true));
                    // Inform Tidings to address the folded state
                    tidings::BROKER.send(tidings::Msg::Fold);
                    // Show the buttons in the end of the Tidings' Header Bar
                    feeds::BROKER.send(feeds::Msg::ShowEndButtons);
                } else {
                    // Update the folding state
                    sender.input(Msg::SetFolded(false));
                    // Inform Tidings to address the unfolded state
                    tidings::BROKER.send(tidings::Msg::Unfold);
                    // Hide the buttons in the end of the Feeds' Header Bar
                    feeds::BROKER.send(feeds::Msg::HideEndButtons);
                    // Hide the Tidings page (won't be shown if folded right after)
                    sender.input(Msg::HideTidingsPage);
                }
            },
        }
    }
}

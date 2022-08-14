//! Update message handler

use relm4::{ComponentSender, MessageBroker, Worker};

use crate::app::components::leaflet::feeds::{self, tree::IndicesUrls};
use crate::app::components::leaflet::tidings::{self, dictionary::Tiding};

/// Message broker
pub static BROKER: MessageBroker<Model> = MessageBroker::new();

/// Model
pub struct Model;

/// Messages
#[derive(Debug)]
pub enum Msg {
    /// Update all feeds
    UpdateAll(IndicesUrls),
}

impl Worker for Model {
    type Init = ();
    type Input = Msg;
    type Output = super::Msg;
    fn init(_init: Self::Init, _sender: ComponentSender<Self>) -> Self {
        Self
    }
    fn update(&mut self, msg: Msg, sender: ComponentSender<Self>) {
        match msg {
            Msg::UpdateAll(indices_urls) => {
                // Add a new command future to be executed in the background
                sender.command(|_out, shutdown| {
                    // Cancel the future if the component is shut down in the meantime
                    shutdown
                        .register(async move {
                            // For each pair
                            for (index, _url) in indices_urls {
                                // Add the updating status to the feed
                                feeds::BROKER.send(feeds::Msg::UpdateStarted(index));
                                // Imitate some work
                                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                                // Prepare some fake results
                                let tidings = vec![Tiding {
                                    title: index.into_raw_parts().0.to_string(),
                                }];
                                // Remove the updating status of the feed
                                feeds::BROKER.send(feeds::Msg::UpdateFinished(index));
                                // Send the tidings to Tidings
                                tidings::BROKER.send(tidings::Msg::UpdateFinished(index, tidings));
                            }
                        })
                        .drop_on_shutdown()
                });
            }
        }
    }
}

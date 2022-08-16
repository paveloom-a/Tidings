//! Update message handler

use relm4::{Component, ComponentSender, Worker, WorkerController};

use std::convert::identity;

use super::tree::IndicesUrls;
use crate::app::components::leaflet::tidings::{self, dictionary::Tiding};

/// Model
pub struct Model;

/// Initialize a new worker (this is
/// used to drop the previous worker)
pub(super) fn new(sender: &ComponentSender<super::Model>) -> WorkerController<Model> {
    Model::builder()
        .detach_worker(())
        .forward(&sender.input, identity)
}

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
                                super::BROKER.send(super::Msg::UpdateStarted(index));
                                // Imitate some work
                                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                                // Prepare some fake results
                                let tidings = vec![Tiding {
                                    title: index.into_raw_parts().0.to_string(),
                                }];
                                // Remove the updating status of the feed
                                super::BROKER.send(super::Msg::UpdateFinished(index));
                                // Send the tidings to Tidings
                                tidings::BROKER.send(tidings::Msg::Insert(index, tidings));
                            }
                            // Notify Feeds that the whole update is finished
                            super::BROKER.send(super::Msg::StopUpdateAll);
                        })
                        .drop_on_shutdown()
                });
            }
        }
    }
}

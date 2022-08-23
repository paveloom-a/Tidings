//! Update message handler

use rayon::prelude::*;
use relm4::{Component, ComponentSender, Worker, WorkerController};

use std::convert::identity;

use super::{Tiding, URLsMap};

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
    UpdateAll(URLsMap),
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
                            indices_urls.into_par_iter().for_each(|(url, indices)| {
                                // Add the updating status
                                super::BROKER.send(super::Msg::UpdateStarted(indices.clone()));
                                // Prepare some fake results
                                let tidings = vec![Tiding {
                                    title: format!("URL: {}", url),
                                }];
                                // Imitate some work
                                std::thread::sleep(std::time::Duration::from_secs(1));
                                // Insert the tidings into the dictionary
                                super::BROKER.send(super::Msg::Insert(indices, url, tidings));
                            });
                            // Notify Feeds that the whole update is finished
                            super::BROKER.send(super::Msg::StopUpdateAll);
                        })
                        .drop_on_shutdown()
                });
            }
        }
    }
}

//! Update message handler

use relm4::{MessageHandler, Sender};
use tokio::runtime::{Builder, Runtime};
use tokio::sync::mpsc::{channel, Sender as TokioSender};

use crate::app::components::leaflet::feeds::{self, tree::IndicesUrls};
use crate::app::components::leaflet::tidings::dictionary::Tiding;

/// Async Handler
pub(in super::super) struct AsyncHandler {
    /// Runtime
    _rt: Runtime,
    /// Sender
    sender: TokioSender<Msg>,
}

/// Messages
#[derive(Debug)]
pub(in super::super) enum Msg {
    /// Update all feeds
    UpdateAll(IndicesUrls),
}

impl MessageHandler<super::Model> for AsyncHandler {
    type Msg = Msg;
    type Sender = TokioSender<Msg>;
    fn init(_parent_model: &super::Model, parent_sender: Sender<super::Msg>) -> Self {
        // Create a channel between this component and any calling one
        let (sender, mut rx) = channel::<Msg>(100);
        // Initialize a Tokio Runtime
        #[allow(clippy::unwrap_used)]
        let rt = Builder::new_multi_thread().enable_time().build().unwrap();
        // Spawn a future onto the runtime that handles messages asynchronously
        rt.spawn(async move {
            while let Some(msg) = rx.recv().await {
                let parent_sender = parent_sender.clone();
                tokio::spawn(async move {
                    match msg {
                        Msg::UpdateAll(indices_urls) => {
                            // For each pair
                            for (index, _url) in indices_urls {
                                // Notify Feeds that this particular
                                // feed is in the process of being updated
                                let msg =
                                    super::Msg::TransferToFeeds(feeds::Msg::UpdateStarted(index));
                                parent_sender.send(msg).ok();
                                // Imitate some work
                                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                                // Prepare some fake results
                                let tidings = vec![Tiding {
                                    title: index.into_raw_parts().0.to_string(),
                                }];
                                // Notify Feeds and Tidings that the process of
                                // updating of this particular feed has finished
                                parent_sender
                                    .send(super::Msg::UpdateFinished(index, tidings))
                                    .ok();
                            }
                        }
                    }
                });
            }
        });
        AsyncHandler { _rt: rt, sender }
    }
    fn send(&self, msg: Self::Msg) {
        self.sender.blocking_send(msg).ok();
    }
    fn sender(&self) -> Self::Sender {
        self.sender.clone()
    }
}

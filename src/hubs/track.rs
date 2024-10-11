use crate::hubs::sink::HubSink;
use crate::hubs::unit::HubUnit;
use std::sync::Arc;
use tokio::sync::broadcast::Receiver;
use tokio::sync::{broadcast, RwLock};

pub struct HubTrack {
    sinks: RwLock<Vec<Arc<HubSink>>>,
    tx: broadcast::Sender<HubUnit>,
}

impl HubTrack {
    pub fn new(rx: Receiver<HubUnit>) -> Arc<Self> {
        let (tx, sink_rx) = broadcast::channel(100);
        drop(sink_rx);

        let sinks = RwLock::new(Vec::new());
        let hub_track = Arc::new(HubTrack { sinks, tx });
        hub_track.clone().run(rx);
        hub_track
    }

    pub fn run(self: Arc<Self>, mut rx: Receiver<HubUnit>) {
        // let hub_track = self;
        tokio::spawn(async move {
            loop {
                let hub_unit = rx.recv().await.unwrap();
                // todo transcode?

                if let Err(e) = self.tx.send(hub_unit) {
                    log::warn!("failed to send to sinks: {:?}", e);
                }
            }
        });
    }

    pub async fn add_sink(&self) -> Arc<HubSink> {
        let rx = self.tx.subscribe();
        let hub_sink = HubSink::new(rx);
        self.sinks.write().await.push(hub_sink.clone());
        hub_sink
    }
}

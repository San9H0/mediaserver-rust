use crate::hubs::sink::HubSink;
use crate::hubs::unit::HubUnit;
use std::sync::Arc;
use tokio::sync::broadcast::Receiver;
use tokio::sync::{broadcast, RwLock};
use tokio_util::sync::CancellationToken;

pub struct HubTrack {
    source_token: CancellationToken,
    id: String,
    sinks: RwLock<Vec<Arc<HubSink>>>,
    tx: broadcast::Sender<HubUnit>,
}

impl HubTrack {
    pub fn new(token: CancellationToken) -> Arc<Self> {
        let (tx, _) = broadcast::channel(100);

        let id = uuid::Uuid::new_v4().to_string();
        let sinks = RwLock::new(Vec::new());
        let hub_track = Arc::new(HubTrack {
            source_token: token,
            id,
            sinks,
            tx,
        });

        hub_track
    }

    #[allow(dead_code)]
    pub fn stop(self: &Arc<Self>) {}

    pub async fn run(self: &Arc<Self>, mut rx: Receiver<HubUnit>, token: CancellationToken) {
        loop {
            tokio::select! {
                _ = self.source_token.cancelled() => {
                    break;
                }
                _ = token.cancelled() => {
                    break;
                }
                result = rx.recv() => {
                    let Ok(hub_unit) = result else {
                        break;
                    };
                    // todo transcode?

                    if self.tx.receiver_count() == 0 {
                        continue;
                    }
                    if let Err(_) = self.tx.send(hub_unit) {
                        continue;
                    }
                }
            }
        }
    }

    pub async fn add_sink(self: &Arc<Self>) -> Arc<HubSink> {
        let rx = self.tx.subscribe();
        let hub_sink = HubSink::new(rx);
        let _hub_sink = hub_sink.clone();
        self.sinks.write().await.push(hub_sink.clone());
        hub_sink
    }

    pub async fn remove_sink(self: &Arc<Self>, sink: &Arc<HubSink>) {
        let mut sinks = self.sinks.write().await;
        sinks.retain(|s| !Arc::ptr_eq(s, sink));
    }
}

impl Drop for HubTrack {
    fn drop(&mut self) {
        println!("HubTrack dropped");
    }
}

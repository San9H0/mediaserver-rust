use crate::hubs::unit::HubUnit;
use std::sync::Arc;
use tokio::sync::broadcast::error::RecvError;
use tokio::sync::broadcast::Receiver;
use tokio::sync::RwLock;
use tokio_util::sync::CancellationToken;

pub struct HubSink {
    rx: RwLock<Receiver<HubUnit>>,
    token: CancellationToken,
}

impl HubSink {
    pub fn new(rx: Receiver<HubUnit>) -> Arc<Self> {
        let token = CancellationToken::new();
        Arc::new(HubSink {
            rx: RwLock::new(rx),
            token,
        })
    }
    pub fn stop(self: &Arc<Self>) {
        self.token.cancel()
    }

    pub async fn read_unit(self: &Arc<Self>) -> Result<HubUnit, RecvError> {
        self.rx.write().await.recv().await
    }
}

impl Drop for HubSink {
    fn drop(&mut self) {
        println!("HubSink dropped");
    }
}

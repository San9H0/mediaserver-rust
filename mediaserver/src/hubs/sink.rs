use crate::hubs::unit::HubUnit;
use std::sync::Arc;
use tokio::sync::broadcast::error::RecvError;
use tokio::sync::broadcast::Receiver;
use tokio::sync::RwLock;

pub struct HubSink {
    rx: RwLock<Receiver<HubUnit>>,
}

impl HubSink {
    pub fn new(rx: Receiver<HubUnit>) -> Arc<Self> {
        Arc::new(HubSink {
            rx: RwLock::new(rx),
        })
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

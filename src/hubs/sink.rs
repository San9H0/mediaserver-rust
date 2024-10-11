use crate::hubs::unit::HubUnit;
use std::sync::Arc;
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

    pub async fn read_unit(&self) -> anyhow::Result<HubUnit> {
        let hub_unit = self.rx.write().await.recv().await?;

        Ok(hub_unit)
    }
}

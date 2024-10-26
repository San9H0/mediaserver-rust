use crate::hubs::source::HubSource;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct HubStream {
    sources: RwLock<Vec<Arc<HubSource>>>,
}

impl HubStream {
    pub fn new() -> Arc<Self> {
        Arc::new(HubStream {
            sources: RwLock::new(Vec::new()),
        })
    }

    pub async fn add_source(&self, source: Arc<HubSource>) {
        self.sources.write().await.push(source);
    }

    pub async fn get_sources(&self) -> Vec<Arc<HubSource>> {
        self.sources.read().await.clone()
    }

    pub async fn remove_source(&self, source: Arc<HubSource>) {
        let mut sources = self.sources.write().await;
        sources.retain(|s| !Arc::ptr_eq(s, &source));
    }
}

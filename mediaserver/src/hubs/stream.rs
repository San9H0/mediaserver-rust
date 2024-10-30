use crate::hubs::source::HubSource;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct HubStream {
    uuid: String,
    sources: RwLock<Vec<Arc<HubSource>>>,
}

impl HubStream {
    pub fn new() -> Arc<Self> {
        Arc::new(HubStream {
            uuid: uuid::Uuid::new_v4().to_string(),
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

// HubStream 타입에 PartialEq 구현
impl PartialEq for HubStream {
    fn eq(&self, other: &Self) -> bool {
        self.uuid == other.uuid
    }
}

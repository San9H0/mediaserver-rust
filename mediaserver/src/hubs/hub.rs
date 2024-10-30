use std::collections::HashMap;
use std::sync::Arc;

use crate::hubs::stream;
use crate::hubs::stream::HubStream;
use tokio::sync::RwLock;

pub struct Hub {
    streams: RwLock<HashMap<String, Arc<HubStream>>>,
}

impl Hub {
    pub fn new() -> Arc<Self> {
        Arc::new(Hub {
            streams: RwLock::new(HashMap::new()),
        })
    }

    pub async fn insert_stream(&self, id: &str, stream: &Arc<HubStream>) {
        self.streams
            .write()
            .await
            .insert(id.to_string(), stream.clone());
    }

    pub async fn remove_stream(&self, id: &str, stream: &Arc<HubStream>) {
        let mut streams = self.streams.write().await;
        let Some(get_stream) = streams.get(id) else {
            return;
        };
        if get_stream != stream {
            return;
        }
        streams.remove(id);
    }

    pub async fn get_stream(&self, id: &str) -> Option<Arc<HubStream>> {
        let streams = self.streams.read().await;
        streams.get(id).cloned()
    }
}

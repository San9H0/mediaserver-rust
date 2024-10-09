use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::hubs::stream;

pub struct Hub {
    streams_map: RwLock<HashMap<String,Arc<stream::Stream>>>,
}

impl Hub {
    pub fn new() -> Self {
        Hub {
            streams_map: RwLock::new(HashMap::new()),
        }
    }

    pub fn add_stream(&self, id: String, stream: Arc<stream::Stream>) {
        let mut streams = self.streams_map.write().unwrap();
        streams.insert(id.clone(), stream);
    }

    pub fn remove_stream(&self, id: String) {
        let mut streams = self.streams_map.write().unwrap();
        streams.remove(&id);
    }

    pub fn get_stream(&self, id: String) -> Option<Arc<stream::Stream>> {
        let streams = self.streams_map.read().unwrap();
        streams.get(&id).cloned()
    }
}

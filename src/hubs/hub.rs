use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::hubs::stream;

pub struct Hub {
    streams_map: RwLock<HashMap<String,Arc<stream::HubStream>>>,
}

impl Hub {
    pub fn new() -> Arc<Self> {
        Arc::new(Hub {
            streams_map: RwLock::new(HashMap::new()),
        })
    }

    pub fn insert_stream(
        &self,
        id: String,
        stream: Arc<stream::HubStream>,
    ) {
        self.streams_map
            .write()
            .unwrap()
            .insert(id.clone(), stream);
    }

    pub fn remove_stream(
        &self,
        id: String,
    ) {
        self.streams_map
            .write()
            .unwrap()
            .remove(&id);
    }

    pub fn get_stream(
        &self,
        id: String,
    ) -> Option<Arc<stream::HubStream>> {
        self.streams_map
            .read()
            .unwrap()
            .get(&id)
            .cloned()
    }
}

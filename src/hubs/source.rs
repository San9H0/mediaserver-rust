use crate::codecs::codec::Codec;
use crate::hubs::track::HubTrack;
use crate::hubs::unit::HubUnit;
use rand::Rng;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};

pub struct HubSource {
    tracks: RwLock<HashMap<String, Arc<HubTrack>>>,
    tx: broadcast::Sender<HubUnit>,

    codec: Codec,
}

impl HubSource {
    pub fn new(codec: Codec) -> Arc<Self> {
        let tracks = RwLock::new(HashMap::new());
        let (tx, rx) = broadcast::channel(100);
        drop(rx);
        Arc::new(HubSource { tracks, tx, codec })
    }

    pub fn get_codec(&self) -> Codec {
        self.codec.clone()
    }

    pub async fn get_track(&self) -> Arc<HubTrack> {
        let key: String = rand::thread_rng()
            .sample_iter(&rand::distributions::Alphanumeric)
            .take(10) // 길이 10의 랜덤 문자열 생성
            .map(char::from)
            .collect(); // TODO 임시 코드

        let rx = self.tx.subscribe();

        let hub_track = HubTrack::new(rx);

        self.tracks
            .write()
            .await
            .insert(key.clone(), hub_track.clone());

        hub_track
    }

    pub async fn write_unit(&self, unit: HubUnit) {
        let _ = self.tx.send(unit);
    }
}

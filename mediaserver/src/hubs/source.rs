use crate::codecs::codec::Codec;
use crate::hubs::track::HubTrack;
use crate::hubs::unit::HubUnit;
use rand::Rng;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use tokio_util::sync::CancellationToken;

pub struct HubSource {
    tracks: RwLock<HashMap<String, Arc<HubTrack>>>,
    tx: broadcast::Sender<HubUnit>,
    token: CancellationToken,
    codec: RwLock<Option<Codec>>,
}

impl HubSource {
    pub fn new() -> Arc<Self> {
        let tracks = RwLock::new(HashMap::new());
        let (tx, rx) = broadcast::channel(100);
        let token = CancellationToken::new();
        drop(rx);
        Arc::new(HubSource {
            tracks,
            tx,
            token,
            codec: RwLock::new(None),
        })
    }

    pub fn stop(self: &Arc<Self>) {
        self.token.cancel();
    }

    pub async fn set_codec(self: &Arc<Self>, codec: Codec) {
        self.codec.write().await.replace(codec);
    }

    pub async fn get_codec(self: &Arc<Self>) -> Option<Codec> {
        self.codec.read().await.clone()
    }

    pub async fn get_track(self: &Arc<Self>) -> Arc<HubTrack> {
        let key: String = rand::thread_rng()
            .sample_iter(&rand::distributions::Alphanumeric)
            .take(10) // 길이 10의 랜덤 문자열 생성
            .map(char::from)
            .collect(); // TODO 임시 코드

        let hub_track = HubTrack::new();
        let _track = hub_track.clone();
        let rx = self.tx.subscribe();
        let _self = self.clone();
        let _key = key.clone();
        tokio::spawn(async move {
            _track.run(rx, _self.token.clone()).await;
            _self.tracks.write().await.remove(&_key);
        });

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

impl Drop for HubSource {
    fn drop(&mut self) {
        println!("HubSource dropped");
    }
}

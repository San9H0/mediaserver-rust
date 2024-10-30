use crate::codecs::codec::Codec;
use crate::hubs::track::HubTrack;
use crate::hubs::unit::HubUnit;
use rand::Rng;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use tokio_util::sync::CancellationToken;

pub struct HubSource {
    tracks: RwLock<HashMap<Codec, Arc<HubTrack>>>,
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

    pub async fn get_track(
        self: &Arc<Self>,
        transcoding_codec: &Codec,
    ) -> anyhow::Result<Arc<HubTrack>> {
        let source_codec = self.get_codec().await.ok_or(anyhow::anyhow!("no codec"))?;
        let hub_track = {
            let mut tracks = self.tracks.write().await;
            tracks
                .entry(transcoding_codec.clone())
                .or_insert_with(|| HubTrack::new())
                .clone()
        };

        let result = hub_track.clone();

        let rx = self.tx.subscribe();
        let _self = self.clone();
        let key = transcoding_codec.clone();
        tokio::spawn(async move {
            hub_track.run(rx, _self.token.clone()).await;

            _self.tracks.write().await.remove(&key);
        });

        Ok(result)
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

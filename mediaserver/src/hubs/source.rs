use crate::codecs::codec::Codec;
use crate::hubs::track::HubTrack;
use crate::hubs::unit::HubUnit;
use std::collections::{hash_map::Entry, HashMap};
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
        println!("new hub source");
        let tracks = RwLock::new(HashMap::new());
        let (tx, _) = broadcast::channel(100);
        Arc::new(HubSource {
            tracks,
            tx,
            token: CancellationToken::new(),
            codec: RwLock::new(None),
        })
    }

    pub fn token(self: &Arc<Self>) -> CancellationToken {
        self.token.clone()
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
        // 출력 코덱이 다를경우 transcoding을 고려해주어야 한다. _source_codec 은 그부분을 체크하기 위한 부분임.
        let _source_codec = self.get_codec().await.ok_or(anyhow::anyhow!("no codec"))?;

        let (hub_track, is_new) = {
            let mut tracks = self.tracks.write().await;
            match tracks.entry(transcoding_codec.clone()) {
                Entry::Occupied(entry) => (entry.get().clone(), false),
                Entry::Vacant(entry) => {
                    let hub_track = HubTrack::new(self.token.clone());
                    entry.insert(hub_track.clone());
                    (hub_track, true)
                }
            }
        };

        if is_new {
            let rx = self.tx.subscribe();
            let _self = self.clone();
            let key = transcoding_codec.clone();
            let hub_track = hub_track.clone();
            tokio::spawn(async move {
                hub_track.run(rx, _self.token.clone()).await;

                _self.tracks.write().await.remove(&key);
                println!("source token end? hub_track end");
            });
        }

        Ok(hub_track)
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

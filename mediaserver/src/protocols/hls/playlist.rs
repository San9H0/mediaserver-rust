use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use m3u8_rs::MediaPlaylist;

pub struct Playlist {
    pub playlist: RwLock<MediaPlaylist>,
}

// egress에서 playList를 만들수있도록,
// endpoint 에서 playlist 조회할수 있도록

impl Playlist {
    pub fn new() -> Arc<Self> {
        Arc::new(Self{
            playlist: RwLock::new(MediaPlaylist{
                ..Default::default()
            }),
        })
    }

    pub async fn init(self: &Arc<Self>) {
        let playlist = self.playlist.write().await;
        playlist.version = Some(10);
    }

    pub fn write(self: &Arc<Self>) {

    }
    pub fn get_m3u8 (self: &Arc<Self>) -> String {
        "playlist".to_string()
    }
}
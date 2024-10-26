use std::sync::Arc;
use webrtc::api::media_engine::MediaEngine;
use webrtc::api::setting_engine::SettingEngine;
use webrtc::api::API;
use webrtc::peer_connection::configuration::RTCConfiguration;
use webrtc::peer_connection::RTCPeerConnection;

pub struct WebRtcApi {
    api: API,
}

impl WebRtcApi {
    #[allow(dead_code)]
    pub fn new() -> Arc<Self> {
        let mut setting_engine = SettingEngine::default();
        setting_engine.set_lite(true);

        let mut media_engine = MediaEngine::default();
        media_engine.register_default_codecs().unwrap();

        let api = webrtc::api::APIBuilder::new()
            .with_media_engine(media_engine)
            .with_setting_engine(setting_engine)
            .build();

        Arc::new(Self { api })
    }

    pub fn new_with_media_engine(media_engine: MediaEngine) -> Arc<Self> {
        let mut setting_engine = SettingEngine::default();
        setting_engine.set_lite(true);

        let api = webrtc::api::APIBuilder::new()
            .with_media_engine(media_engine)
            .with_setting_engine(setting_engine)
            .build();

        Arc::new(Self { api })
    }

    pub async fn new_peer_connection(&self) -> RTCPeerConnection {
        let config = RTCConfiguration {
            ..Default::default()
        };

        self.api.new_peer_connection(config).await.unwrap()
    }
}

use webrtc::api::API;
use webrtc::api::media_engine::MediaEngine;
use webrtc::api::setting_engine::SettingEngine;
use webrtc::peer_connection::configuration::RTCConfiguration;
use webrtc::peer_connection::RTCPeerConnection;

pub struct WebRtcApi {
    api: API
}

impl WebRtcApi {
    pub fn new() -> Self {
        let mut setting_engine = SettingEngine::default();
        setting_engine.set_lite(true);

        let mut media_engine = MediaEngine::default();
        media_engine.register_default_codecs().unwrap();

        let api = webrtc::api::APIBuilder::new()
            .with_media_engine(media_engine)
            .with_setting_engine(setting_engine)
            .build();
        Self { api }
    }

    pub async fn new_peer_connection(&self) -> RTCPeerConnection {
        let config = RTCConfiguration { ..Default::default() };
        self.api.new_peer_connection(config).await.unwrap()
    }
}
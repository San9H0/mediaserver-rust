use std::sync::{mpsc, Arc};
use futures::SinkExt;
use webrtc::api::media_engine::MediaEngine;
use webrtc::api::setting_engine::SettingEngine;
use webrtc::ice_transport::ice_server::RTCIceServer;
use webrtc::peer_connection::configuration::RTCConfiguration;
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;
use webrtc::ice_transport::ice_candidate::RTCIceCandidate;
use webrtc::peer_connection::peer_connection_state::RTCPeerConnectionState;
use crate::{hubs, ingress};


pub struct WhipServer{
    hub: Arc<hubs::hub::Hub>
}


impl WhipServer {
    pub fn new(hub :Arc<hubs::hub::Hub>) -> Self {
        WhipServer{hub}
    }
    pub fn init(&mut self) -> anyhow::Result<()>{
        Ok(())
    }
    pub async fn start_session(&self, stream_id: String, offer: String) -> anyhow::Result<String> {
        let stream = hubs::stream::Stream::new(stream_id.clone());
        self.hub.add_stream(stream_id.clone(), Arc::new(stream));

        let mut whip_session = ingress::sessions::whip::WhipSession::new();
        whip_session.init(offer).await
    }
}
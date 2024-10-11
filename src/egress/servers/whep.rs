use crate::egress::sessions::whep::WhepSession;
use crate::hubs::hub::Hub;
use crate::webrtc_wrapper::webrtc_api::WebRtcApi;
use std::sync::Arc;

pub struct WhepServer {
    hub: Arc<Hub>,
    api: Arc<WebRtcApi>,
}

impl WhepServer {
    pub fn new(hub: Arc<Hub>, api: Arc<WebRtcApi>) -> Self {
        WhepServer { hub, api }
    }

    pub async fn start_session(&self, stream_id: String, offer: String) -> anyhow::Result<String> {
        let hub_stream = self
            .hub
            .get_stream(stream_id.to_string())
            .ok_or(anyhow::anyhow!("stream not found"))?;

        let whep_session = WhepSession::new(self.api.clone(), hub_stream.clone()).await;
        let answer = whep_session.init(offer).await?;

        Ok(answer)
    }
}

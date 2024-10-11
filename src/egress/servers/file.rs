use crate::egress::sessions::whep::WhepSession;
use crate::hubs::hub::Hub;
use crate::webrtc_wrapper::webrtc_api::WebRtcApi;
use std::sync::Arc;

pub struct FileServer {
    hub: Arc<Hub>,
}

impl FileServer {
    pub fn new(hub: Arc<Hub>) -> Self {
        FileServer { hub }
    }

    pub async fn start_session(&self, stream_id: String) -> anyhow::Result<()> {
        let hub_stream = self
            .hub
            .get_stream(stream_id.to_string())
            .ok_or(anyhow::anyhow!("stream not found"))?;

        Ok(())
    }
}

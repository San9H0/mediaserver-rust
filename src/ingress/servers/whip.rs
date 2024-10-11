use std::sync::Arc;
use crate::hubs::hub::Hub;
use crate::hubs::stream::HubStream;
use crate::ingress::sessions::whip::WhipSession;
use crate::webrtc_wrapper::webrtc_api::WebRtcApi;

pub struct WhipServer{
    hub: Arc<Hub>,
    api: Arc<WebRtcApi>,
}


impl WhipServer {
    pub fn new(
        hub :Arc<Hub>,
        api: Arc<WebRtcApi>,
    ) -> Self {
        WhipServer{hub, api}
    }
    pub async fn start_session(
        &self,
        stream_id: String,
        offer: String,
    ) -> anyhow::Result<String> {
        let hub_stream = HubStream::new(stream_id.clone());
        self.hub.insert_stream(stream_id.clone(), hub_stream.clone());

        println!("stream_id:{}, offer:{}", stream_id.to_string(), offer.to_string());

        let whip_session2 = WhipSession::new(self.api.clone(), hub_stream.clone()).await;
        let answer = whip_session2.init(offer).await?;

        println!("answer:{}", answer.to_string());
        Ok(answer)
    }
}
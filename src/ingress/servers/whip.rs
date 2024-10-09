use std::sync::{Arc};
use crate::{hubs};
use crate::hubs::hub::Hub;
use crate::ingress::sessions::whip::{WhipSession, WhipSession2};
use crate::webrtc_wrapper::webrtc_api::WebRtcApi;

pub struct WhipServer{
    hub: Arc<Hub>,
    api: Arc<WebRtcApi>,
}


impl WhipServer {
    pub fn new(hub :Arc<Hub>, api: Arc<WebRtcApi>) -> Self {
        WhipServer{hub, api}
    }
    pub fn init(&mut self) -> anyhow::Result<()>{
        Ok(())
    }
    pub async fn start_session(&self, stream_id: String, offer: String) -> anyhow::Result<String> {
        let stream = hubs::stream::Stream::new(stream_id.clone());
        self.hub.add_stream(stream_id.clone(), Arc::new(stream));

        // let mut whip_session = WhipSession::new();
        // let answer = whip_session.init(offer).await?;

        println!("stream_id:{}, offer:{}", stream_id.to_string(), offer.to_string());


        let whip_session2 = WhipSession2::new(self.api.clone()).await;
        let answer = whip_session2.init(offer).await?;

        println!("answer:{}", answer.to_string());
        Ok(answer)
    }
}
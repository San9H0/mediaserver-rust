use crate::hubs::hub::Hub;
use crate::hubs::stream::HubStream;
use crate::ingress::sessions::whip::whip::WhipSession;
use crate::webrtc_wrapper::webrtc_api::WebRtcApi;
use std::sync::Arc;
use tokio::time;

pub struct WhipServer {
    hub: Arc<Hub>,
}

impl WhipServer {
    pub fn new(hub: Arc<Hub>) -> Arc<Self> {
        Arc::new(WhipServer { hub })
    }
    pub async fn start_session(
        self: &Arc<Self>,
        stream_id: String,
        offer: String,
    ) -> anyhow::Result<String> {
        let hub_stream = HubStream::new(stream_id.clone());
        self.hub.insert_stream(&stream_id, hub_stream.clone());

        println!(
            "stream_id:{}, offer:{}",
            stream_id.to_string(),
            offer.to_string()
        );

        let whip_session = WhipSession::new(hub_stream.clone()).await;
        let answer = whip_session.init(offer).await?;
        let server = self.clone();
        let whip_session2 = whip_session.clone();
        tokio::spawn(async move {
            whip_session2.run().await;
            println!("remove_stream called");
            server.hub.remove_stream(&stream_id);
        });

        println!("answer:{}", answer.to_string());
        Ok(answer)
    }
}

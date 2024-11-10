use crate::hubs::hub::Hub;
use crate::ingress::sessions::whip::whip::WhipSession;
use std::sync::Arc;

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
        offer: &str,
    ) -> anyhow::Result<String> {
        let whip_session = WhipSession::new().await?;
        let answer = whip_session.init(offer).await?;

        let server = self.clone();
        tokio::spawn(async move {
            let stream_id = stream_id.to_string();
            let hub_stream = whip_session.hub_stream();
            server.hub.insert_stream(&stream_id, &hub_stream).await;
            whip_session.run().await;
            server.hub.remove_stream(&stream_id, &hub_stream).await;
        });

        Ok(answer)
    }
}

use crate::egress::sessions::whep::whep::WhepSession;
use crate::hubs::hub::Hub;
use std::sync::Arc;

pub struct WhepServer {
    hub: Arc<Hub>,
}

impl WhepServer {
    pub fn new(hub: Arc<Hub>) -> Arc<Self> {
        Arc::new(WhepServer { hub })
    }

    pub async fn start_session(&self, stream_id: String, offer: String) -> anyhow::Result<String> {
        let hub_stream = self
            .hub
            .get_stream(&stream_id)
            .ok_or(anyhow::anyhow!("stream not found"))?;

        let whep_session = WhepSession::new(hub_stream.clone()).await;
        let answer = whep_session.init(offer).await?;

        let session = whep_session.clone();
        tokio::spawn(async move {
            session.run().await;
            println!("whep session closed??");
        });
        Ok(answer)
    }
}

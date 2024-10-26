use std::collections::HashMap;
use crate::egress::sessions::whep::whep::WhepSession;
use crate::hubs::hub::Hub;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use crate::egress::sessions::record::handler::RecordHandler;
use crate::egress::sessions::session::{Session, SessionHandler};
use crate::egress::sessions::whep::handler::WhepHandler;

pub struct WhepServer {
    hub: Arc<Hub>,

    sessions: RwLock<HashMap<String, Arc<Session<WhepHandler>>>>,
}

impl WhepServer {
    pub fn new(hub: Arc<Hub>) -> Arc<Self> {
        Arc::new(WhepServer {
            hub,
            sessions: RwLock::new(HashMap::new()),
        })
    }

    pub async fn start_session(&self, stream_id: String, offer: &str) -> anyhow::Result<String> {
        let hub_stream = self
            .hub
            .get_stream(&stream_id)
            .ok_or(anyhow::anyhow!("stream not found"))?;

        let session_id = Uuid::new_v4().to_string();
        log::info!("record session started: {}", &session_id);
        let whep_handler = WhepHandler::new(&hub_stream, &session_id).await?;
        let answer = whep_handler.init(offer).await?;
        let sess = Session::from_arc(whep_handler.clone());

        self.sessions
            .write()
            .await
            .insert(session_id.to_string(), sess.clone());

        tokio::spawn(async move {
            if let Err(err) = sess.run().await {
                log::warn!("write file failed: {:?}", err);
            }
        });


        Ok(answer)
    }
}

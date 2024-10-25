use crate::hubs::hub::Hub;
use uuid::Uuid;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time;
use crate::egress::sessions::record::handler::RecordHandler;
use crate::egress::sessions::session::{Session, SessionHandler};

pub struct RecordServer {
    hub: Arc<Hub>,

    record_sessions: RwLock<HashMap<String, Arc<Session<RecordHandler>>>>,
}

impl RecordServer {
    pub fn new(hub: Arc<Hub>) -> Arc<Self> {
        let record_sessions = RwLock::new(HashMap::new());
        Arc::new(RecordServer {
            hub,
            record_sessions,
        })
    }

    pub async fn start_session(&self, stream_id: &str) -> anyhow::Result<String> {
        let hub_stream = self
            .hub
            .get_stream(&stream_id)
            .ok_or(anyhow::anyhow!("stream not found"))?;

        let session_id = Uuid::new_v4().to_string();
        log::info!("record session started: {}", &session_id);

        let handler = RecordHandler::new(&hub_stream, &session_id).await?;
        let sess = Session::new(handler);

        self.record_sessions
            .write()
            .await
            .insert(session_id.to_string(), sess.clone());

        tokio::spawn(async move {
            if let Err(err) = sess.run().await {
                log::warn!("write file failed: {:?}", err);
            }
        });


        Ok(session_id)
    }

    pub async fn stop_session(&self, session_id: String) -> anyhow::Result<()> {
        let sessions = self.record_sessions
            .read()
            .await;
        let sess = sessions
            .get(&session_id)
            .ok_or(anyhow::anyhow!("session not found"))?;
        sess.stop();
        Ok(())
    }
}

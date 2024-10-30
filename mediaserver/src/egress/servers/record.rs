use crate::egress::sessions::record::handler::RecordHandler;
use crate::egress::sessions::session::{Session, SessionHandler};
use crate::hubs::hub::Hub;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time;
use uuid::Uuid;

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

    pub async fn start_session(self: &Arc<Self>, stream_id: &str) -> anyhow::Result<String> {
        let hub_stream = self
            .hub
            .get_stream(&stream_id)
            .await
            .ok_or(anyhow::anyhow!("stream not found"))?;

        let session_id = Uuid::new_v4().to_string();
        log::info!("record session started: {}", &session_id);

        let handler = RecordHandler::new(&hub_stream, &session_id).await?;
        let sess = Session::new(handler);

        let server = self.clone();
        self.record_sessions
            .write()
            .await
            .insert(session_id.to_string(), sess.clone());

        tokio::spawn(async move {
            let session_id = sess.session_id();

            server
                .record_sessions
                .write()
                .await
                .insert(session_id.clone(), sess.clone());

            if let Err(err) = sess.run().await {
                log::warn!("write file failed: {:?}", err);
            }

            let _ = server.stop_session(session_id).await;
        });

        Ok(session_id)
    }

    pub async fn stop_session(&self, session_id: String) -> anyhow::Result<()> {
        let mut sessions = self.record_sessions.write().await;
        let session = sessions
            .remove(&session_id)
            .ok_or(anyhow::anyhow!("session not found"))?;
        session.stop();
        log::info!("record session stopped: {}", session_id);
        Ok(())
    }
}

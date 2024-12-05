use crate::egress::services::hls::path::HlsPath;
use crate::egress::services::hls::service::{HlsConfig, HlsService};
use crate::egress::sessions::hls::handler::HlsHandler;
use crate::egress::sessions::session::Session;
use crate::hubs::hub::Hub;
use std::collections::HashMap;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
pub struct HlsServer {
    hub: Arc<Hub>,

    sessions: RwLock<HashMap<String, Arc<HlsSession>>>,
}

pub struct HlsSession {
    pub handler: Arc<Session<HlsHandler>>,
    pub service: Arc<HlsService>,
}

impl HlsServer {
    pub fn new(hub: Arc<Hub>) -> Arc<Self> {
        m3u8_rs::WRITE_OPT_FLOAT_PRECISION.store(5, Ordering::Relaxed);
        Arc::new(Self {
            hub,
            sessions: RwLock::new(HashMap::new()),
        })
    }

    pub async fn start_session(self: &Arc<Self>, stream_id: &str) -> anyhow::Result<String> {
        let hub_stream = self
            .hub
            .get_stream(&stream_id)
            .await
            .ok_or(anyhow::anyhow!("stream not found"))?;

        let session_id = Uuid::new_v4().to_string();
        log::info!("hls session started: {}", &session_id);

        let service = Arc::new(HlsService::new(HlsConfig {
            part_duration: 0.5,
            part_max_count: 2,
            hls_path: HlsPath::new(session_id.to_string()),
        }));
        service.init().await?;

        let handler = HlsHandler::new(&hub_stream, service.clone()).await?;

        let sess = Session::new(&session_id, handler);

        self.sessions.write().await.insert(
            session_id.to_string(),
            Arc::new(HlsSession {
                handler: sess.clone(),
                service: service.clone(),
            }),
        );

        let server = self.clone();
        let session_id2 = session_id.clone();
        tokio::spawn(async move {
            if let Err(err) = sess.run().await {
                log::warn!("write file failed: {:?}", err);
            }

            let _ = server.stop_session(session_id2).await;
        });

        Ok(session_id)
    }

    pub async fn get_session(
        self: &Arc<Self>,
        session_id: &str,
    ) -> anyhow::Result<Arc<HlsSession>> {
        let sessions = self.sessions.read().await;
        let session = sessions
            .get(session_id)
            .ok_or(anyhow::anyhow!("session not found"))?
            .clone();

        Ok(session)
    }

    pub async fn stop_session(&self, session_id: String) -> anyhow::Result<()> {
        let mut sessions = self.sessions.write().await;
        let session = sessions
            .remove(&session_id)
            .ok_or(anyhow::anyhow!("session not found"))?;
        session.handler.stop();
        log::info!("record session stopped: {}", session_id);
        Ok(())
    }
}

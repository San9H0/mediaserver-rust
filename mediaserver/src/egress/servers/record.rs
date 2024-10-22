use crate::egress::sessions::record::record::RecordSession;
use crate::hubs::hub::Hub;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time;
use crate::egress::sessions::session::Session;

pub struct RecordServer {
    hub: Arc<Hub>,

    record_sessions: RwLock<HashMap<String, Arc<RecordSession>>>,
}

impl RecordServer {
    pub fn new(hub: Arc<Hub>) -> Arc<Self> {
        let record_sessions = RwLock::new(HashMap::new());
        Arc::new(RecordServer {
            hub,
            record_sessions,
        })
    }

    pub async fn start_session(&self, stream_id: String) -> anyhow::Result<()> {
        let hub_stream = self
            .hub
            .get_stream(&stream_id)
            .ok_or(anyhow::anyhow!("stream not found"))?;
        let record_session = RecordSession::new(hub_stream).await?;

        let a = record_session.init().await?;

        let sess = record_session.clone();
        tokio::spawn(async move {
            if let Err(err) = sess.run().await {
                log::warn!("write file failed: {:?}", err);
                return;
            }
        });

        let mut sess = record_session.clone();
        tokio::spawn(async move {
            time::sleep(time::Duration::from_secs(10)).await;
            sess.stop();
        });
        // sess.clone().run().await?;

        // let mut sess2 = sess.clone();
        // tokio::spawn(async move {
        //     sleep(std::time::Duration::from_secs(10)).await;
        //     // sess2.close();
        // });

        Ok(())
    }
}

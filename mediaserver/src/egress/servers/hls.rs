use crate::egress::services::hls::config::{ConfigParams, HlsConfig};
use crate::egress::services::hls::service::HlsService;
use crate::egress::sessions::hls::handler::HlsHandler;
use crate::egress::sessions::session::Session;
use crate::hubs::hub::Hub;
use crate::utils::types::types::MediaKind;
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
    pub config: HlsConfig,
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

        let mut width = 0;
        let mut height = 0;
        let mut video_codec_string = "".to_string();
        let mut audio_codec_string = "".to_string();
        for source in hub_stream.get_sources().await {
            let Some(codec) = source.get_codec().await else {
                continue;
            };
            if codec.kind() == MediaKind::Video {
                width = codec.width() as u64;
                height = codec.height() as u64;
                video_codec_string = codec.codec_string();
            } else if codec.kind() == MediaKind::Audio {
                audio_codec_string = codec.codec_string();
            }
        }

        let config: HlsConfig = HlsConfig::new(ConfigParams {
            session_id: session_id.clone(),
            video_base: "video0".to_string(),
            codecs: format!("{},{}", video_codec_string, audio_codec_string),
            // codecs: "avc1.42C020,Opus".to_string(),
            width: width,
            height: height,
            bandwidth: 1000000, // todo
            framerate: 30.0,
            part_duration: 1.0,
            part_max_count: 2,
        });

        let service = Arc::new(HlsService::new(config.clone()));
        service.init().await?;

        let handler = HlsHandler::new(&hub_stream, service.clone()).await?;

        let sess = Session::new(&session_id, handler);

        {
            self.sessions.write().await.insert(
                session_id.to_string(),
                Arc::new(HlsSession {
                    handler: sess.clone(),
                    service: service.clone(),
                    config: config.clone(),
                }),
            );
        }

        let server = self.clone();
        let session_id2 = session_id.clone();
        tokio::spawn(async move {
            println!("run hls session");
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

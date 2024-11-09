use crate::egress::sessions::session::Session;
use crate::hubs::hub::Hub;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::sync::{Mutex, RwLock};
use uuid::Uuid;
use crate::egress::sessions::hls::handler::HlsHandler;
use crate::protocols;
use crate::protocols::hls::playlist::Playlist;

pub(crate) trait HlsPayloader {
    fn get_payload(&mut self) -> anyhow::Result<bytes::Bytes>;
}

pub struct HlsState {
    started: bool,
    index: i32,
    count: i32,
    prev_time: tokio::time::Instant,
    stream_id: String,
}

pub enum HlsServiceEnum {
    Standard,
    LowLatency,
}

pub struct HlsService {
    state: HlsState,
    service: HlsServiceEnum,
    playlist: Arc<Playlist>,
}


impl HlsService {
    pub fn new(state: HlsState, service: HlsServiceEnum, playlist: Arc<Playlist>) -> Self {
        Self {
            state,
            service,
            playlist,
        }
    }

    pub async fn write_segment<T: HlsPayloader>(&mut self, payloader: Arc<Mutex<T>>) -> anyhow::Result<()> {
        if self.state.started {
            self.state.prev_time = tokio::time::Instant::now();
            self.state.started = true;
            return Ok(());
        }
        if self.state.prev_time.elapsed() > tokio::time::Duration::from_secs(5) {
            self.state.prev_time = tokio::time::Instant::now();

            let payload = {
                let mut p = payloader.lock().await;
                p.get_payload()?
            };

            self.playlist.write();

            let segment_index = self.state.index/2;
            let partition_index = self.state.index%2;
            if partition_index == 0 {
                let output_0 = format!("output_{}_{}.m4s", segment_index, 0);
                let output_1 = format!("output_{}_{}.m4s", segment_index, 1);
            }

            self.state.index+=1;

            let path = Path::new("public/test.mp4");
            if let Some(parent) = path.parent() {
                // 부모 디렉토리가 있는지 확인하고 없으면 생성
                tokio::fs::create_dir_all(parent)
                    .await?;
            }
            // save to memory or file
            tokio::fs::File::create(path)
                .await?
                .write_all(&payload)
                .await?;

            println!("write.. payload: {}", payload.len());
        }
        Ok(())
    }
}


pub struct HlsServer {
    hub: Arc<Hub>,

    sessions: RwLock<HashMap<String, Arc<HlsSession>>>,
}

struct HlsSession {
    pub handler: Arc<Session<HlsHandler>>,
    pub playlist: Arc<Playlist>,
}

impl HlsServer {
    pub fn new(hub: Arc<Hub>) -> Arc<Self> {
        let record_sessions = RwLock::new(HashMap::new());
        Arc::new(Self {
            hub,
            sessions: record_sessions,
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

        let use_file = false;

        let service_type: HlsServiceEnum = if use_file {
            HlsServiceEnum::Standard
        } else {
            HlsServiceEnum::LowLatency
        };
        let playlist = Playlist::new();
        let service: HlsService = HlsService::new(HlsState{
            started: false,
            index: 0,
            count: 0,
            prev_time: tokio::time::Instant::now(),
            stream_id: stream_id.to_string(),
        }, service_type, playlist.clone());

        let handler = HlsHandler::new(&hub_stream, service).await?;
        let sess = Session::new(&session_id, handler);

        let server = self.clone();
        self.sessions
            .write()
            .await
            .insert(session_id.to_string(), Arc::new(HlsSession{
                handler: sess.clone(),
                playlist,
            }));

        let session_id2 = session_id.clone();
        tokio::spawn(async move {
            if let Err(err) = sess.run().await {
                log::warn!("write file failed: {:?}", err);
            }

            let _ = server.stop_session(session_id2).await;
        });

        Ok(session_id)
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

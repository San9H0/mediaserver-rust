use crate::egress::sessions::session::{Session, SessionHandler};
use crate::hubs::hub::Hub;
use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;
use tokio::io::{AsyncWrite, AsyncWriteExt};
use tokio::sync::{Mutex, RwLock};
use uuid::Uuid;
use crate::egress::sessions::hls::handler::HlsHandler;

pub(crate) trait HlsPayloader {
    fn get_payload(&mut self) -> anyhow::Result<bytes::Bytes>;
}

pub struct HlsState {
    started: bool,
    count: i32,
    prev_time: tokio::time::Instant,
}

pub enum HlsService {
    Standard(HlsState, tokio::io::BufWriter<tokio::fs::File>),
    LowLatency(HlsState),
}

pub enum HlsServiceEnum {
    Standard(tokio::io::BufWriter<tokio::fs::File>),
    LowLatency,
}

pub struct HlsService2 {
    state: HlsState,
    service: HlsServiceEnum,
}

impl HlsService2 {
    pub fn new(state: HlsState, service: HlsServiceEnum) -> Self {
        Self {
            state,
            service,
        }
    }

    pub async fn write_segment<T: HlsPayloader>(&mut self, payloader: Arc<Mutex<T>>) -> anyhow::Result<()> {
        if self.state.started {
            self.state.prev_time = tokio::time::Instant::now();
            self.state.started = true;
            return Ok(());
        }
        if self.state.prev_time.elapsed() > tokio::time::Duration::from_secs(10) {
            self.state.prev_time = tokio::time::Instant::now();

            let payload = {
                let mut p = payloader.lock().await;
                p.get_payload()?
            };

            tokio::fs::File::create("test.mp4")
                .await?
                .write_all(&payload)
                .await?;

            println!("write.. payload: {}", payload.len());
        }
        Ok(())
    }
}


impl HlsService {
    pub async fn write_segment<T: HlsPayloader>(&mut self, payloader: Arc<Mutex<T>>) -> anyhow::Result<()> {
        match self {
            HlsService::Standard(state, writer) => {
                Ok(())
            },
            HlsService::LowLatency(state) => {
                if state.started {
                    state.prev_time = tokio::time::Instant::now();
                    state.started = true;
                    return Ok(());
                }
                if state.prev_time.elapsed() > tokio::time::Duration::from_secs(10) {
                    state.prev_time = tokio::time::Instant::now();

                    let payload = {
                        let mut p = payloader.lock().await;
                        p.get_payload()?
                    };

                    tokio::fs::File::create("test.mp4")
                        .await?
                        .write_all(&payload)
                        .await?;

                    println!("write.. payload: {}", payload.len());
                }
                Ok(())
            },
        }
    }
}

pub struct HlsServer {
    hub: Arc<Hub>,

    sessions: RwLock<HashMap<String, Arc<Session<HlsHandler>>>>,
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

        let mut service2: HlsService2 = if use_file {
            let file = tokio::fs::File::create("output.txt").await?;
            HlsService2::new(HlsState{
                started: false,
                count: 0,
                prev_time: tokio::time::Instant::now(),
            }, HlsServiceEnum::Standard(tokio::io::BufWriter::new(file)))
        } else {
            HlsService2::new(HlsState{
                started: false,
                count: 0,
                prev_time: tokio::time::Instant::now(),
            }, HlsServiceEnum::LowLatency)
        };

        let handler = HlsHandler::new(&hub_stream, service2).await?;
        let sess = Session::new(&session_id, handler);

        let server = self.clone();
        self.sessions
            .write()
            .await
            .insert(session_id.to_string(), sess.clone());

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
        session.stop();
        log::info!("record session stopped: {}", session_id);
        Ok(())
    }
}

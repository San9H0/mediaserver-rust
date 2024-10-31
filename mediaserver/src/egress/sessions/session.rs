use crate::codecs::bfs::Bfs;
use crate::codecs::codec::Codec;
use crate::hubs::source::HubSource;
use crate::hubs::unit::HubUnit;
use crate::utils::types::types;
use std::sync::Arc;
use tokio_util::sync::CancellationToken;

pub trait SessionHandler {
    type TrackContext: Send + Sync + 'static;
    fn on_initialize(&self) -> impl std::future::Future<Output = anyhow::Result<()>> + Send {
        async { Ok(()) }
    }
    fn on_finalize(&self) -> impl std::future::Future<Output = anyhow::Result<()>> + Send {
        async { Ok(()) }
    }
    fn get_sources(&self) -> Vec<Arc<HubSource>>;
    fn on_track_context(&self, idx: usize, codec: &Codec) -> Self::TrackContext;
    fn on_video(
        &self,
        ctx: &mut Self::TrackContext,
        unit: &HubUnit,
    ) -> impl std::future::Future<Output = ()> + Send;

    fn on_audio(
        &self,
        ctx: &mut Self::TrackContext,
        unit: &HubUnit,
    ) -> impl std::future::Future<Output = ()> + Send;
    async fn read_hub_stream(&self) {}
}

pub struct Session<T>
where
    T: SessionHandler + Send + Sync + 'static,
{
    session_id: String,
    token: CancellationToken,
    handler: Arc<T>,
}

impl<T> Session<T>
where
    T: SessionHandler + Send + Sync + 'static,
{
    pub fn new(session_id: &str, handler: T) -> Arc<Self> {
        Arc::new(Session {
            session_id: session_id.to_string(),
            token: CancellationToken::new(),
            handler: Arc::new(handler),
        })
    }
    pub fn from_arc(session_id: &str, handler: Arc<T>) -> Arc<Self> {
        Arc::new(Session {
            session_id: session_id.to_string(),
            token: CancellationToken::new(),
            handler,
        })
    }

    pub fn stop(self: &Arc<Self>) {
        self.handler.stop();
        self.token.cancel();
    }
    pub async fn run(self: &Arc<Self>) -> anyhow::Result<()> {
        self.handler.on_initialize().await?;

        let mut join_handles = vec![];
        for (idx, source) in self.handler.get_sources().iter().enumerate() {
            let codec = source.get_codec().await.unwrap();
            let track = source.get_track(&codec).await?;
            let sink = track.add_sink().await;
            let cancel_token = self.token.clone();
            let self_ = self.clone();

            let mut ctx = self_.handler.on_track_context(idx, &codec);
            let handler_ = self_.handler.clone();

            let handle = tokio::spawn(async move {
                loop {
                    tokio::select! {
                        _ = cancel_token.cancelled() => {
                            break;
                        }
                        result = sink.read_unit() => {
                             let Ok(unit) = result else {
                                log::warn!("read unit failed");
                                break;
                            };
                            if codec.kind() == types::MediaKind::Audio {
                                self_.handler.on_audio(&mut ctx, &unit).await;
                            } else if codec.kind() == types::MediaKind::Video {
                                self_.handler.on_video(&mut ctx, &unit).await;
                            }
                        }
                    }
                }

                track.remove_sink(&sink).await;
            });
            join_handles.push(handle);
        }

        for handle in join_handles {
            let _ = handle.await;
        }

        self.handler.on_finalize().await?;

        Ok(())
    }
}

impl<T> Drop for Session<T>
where
    T: SessionHandler + Send + Sync + 'static,
{
    fn drop(&mut self) {
        println!("Session.. dropped");
    }
}

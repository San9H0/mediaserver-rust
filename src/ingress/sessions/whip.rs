use std::error::Error;
use std::sync::Arc;
use log::log;
use tokio::sync::mpsc;
use webrtc::api::media_engine::MediaEngine;
use webrtc::api::setting_engine::SettingEngine;
use webrtc::ice_transport::ice_candidate::RTCIceCandidate;
use webrtc::ice_transport::ice_server::RTCIceServer;
use webrtc::peer_connection::configuration::RTCConfiguration;
use webrtc::peer_connection::peer_connection_state::RTCPeerConnectionState;
use webrtc::peer_connection::RTCPeerConnection;
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;
use webrtc::rtp_transceiver::RTCRtpTransceiver;
use webrtc::rtp_transceiver::rtp_receiver::RTCRtpReceiver;
use webrtc::track::track_remote::TrackRemote;

pub struct WhipSession{
    ontrack_ch: (mpsc::Sender<(Arc<TrackRemote>, Arc<RTCRtpReceiver>, Arc<RTCRtpTransceiver>)>, mpsc::Receiver<(Arc<TrackRemote>, Arc<RTCRtpReceiver>, Arc<RTCRtpTransceiver>)>),
    pc: Option<RTCPeerConnection>
}

impl WhipSession {
    pub fn new() -> Self {
        WhipSession{
            ontrack_ch: mpsc::channel(32),
            pc: None
        }
    }
    pub async fn init(&mut self, offer: String) -> anyhow::Result<String> {
        let mut setting_engine = SettingEngine::default();
        setting_engine.set_lite(true);
        let mut media_engine = MediaEngine::default();
        media_engine.register_default_codecs()?;
        let api = webrtc::api::APIBuilder::new()
            .with_media_engine(media_engine)
            .with_setting_engine(setting_engine)
            .build();

        let config = RTCConfiguration {
            ..Default::default()
        };

        let pc = api.new_peer_connection(config).await?;

        let (tx, mut rx) = tokio::sync::mpsc::channel(1);
        pc.on_ice_candidate(Box::new(move |c: Option<RTCIceCandidate>| {
            let tx2 = tx.clone();
            Box::pin(
                async move {
                    if c.is_none() {
                        let result = tx2.send(()).await;
                    }
                })
        }));

        pc.on_peer_connection_state_change(Box::new(|state: RTCPeerConnectionState| {
            Box::pin(async move {
                println!("Peer Connection State has changed {:?}", state);
            })
        }));

        pc.on_track(Box::new(move |remote, receiver, transceiver | {
            Box::pin(async move {
                if let Err(e) = self.ontrack_ch.0.send((remote, receiver, transceiver)).await {
                    log::error!("failed to send ontrack: {}", e);
                }
            })
        }));

        let offer = RTCSessionDescription::offer(offer.clone())?;
        pc.set_remote_description(offer).await?;

        let answer = pc.create_answer(None).await?;
        pc.set_local_description(answer.clone()).await?;

        let _ = rx.recv().await;
        let answer = pc.local_description().await.ok_or(anyhow::anyhow!("no local description"))?;

        println!("set_remote_description called recv:{}", answer.sdp);

        self.pc = Some(pc);

        Ok(answer.sdp.clone())
    }
    pub async fn run(&mut self) -> anyhow::Result<(), Box<dyn Error>>{
        loop {
            let (remote, receiver, transceiver) = self.ontrack_ch.1.recv().await.ok_or("Channel closed")?;
            tokio::spawn(async {

            })
        }
    }
    pub async fn onTrack(remote: TrackRemote, receiver: RTCRtpReceiver, transceiver: RTCRtpTransceiver) {
        log::info!("Track has been received: pt:{}, ssrc:{}, mimeType:{}, streamID:{}, trackID:{}, rid:{}",
            remote.payload_type(),
            remote.ssrc(),
            remote.codec().capability.mime_type,
            remote.stream_id(),
            remote.id(),
            remote.rid(),
        );
    }
}
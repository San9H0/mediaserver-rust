use std::error::Error;
use std::sync::Arc;
use std::time::Duration;
use anyhow::anyhow;
use log::log;
use tokio::sync::mpsc;
use tokio::time;
use webrtc::api::API;
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
use crate::webrtc_wrapper::webrtc_api::WebRtcApi;
use crate::webrtc_wrapper::webrtc_receiver::WebRtcReceiver;

pub struct WhipSession2 {
    pc: RTCPeerConnection,
}

impl WhipSession2 {
    pub async fn new(
        api: Arc<WebRtcApi>
    ) -> Arc<Self> {
        let pc = api.new_peer_connection().await;
        let whip_session = Self { pc };
        Arc::new(whip_session)
    }

    pub async fn init(
        self: Arc<Self>,
        offer: String,
    ) -> anyhow::Result<String> {
        let (end_candidate, mut wait_candidate) = mpsc::channel(1);
        self.pc.on_ice_candidate(Box::new(move |c: Option<RTCIceCandidate>| {
            let end_candidate2 = end_candidate.clone();
            Box::pin(async move {
                if !c.is_none() {
                    return
                }
                if let Err(_) =  time::timeout(Duration::from_secs(1), end_candidate2.send(())).await {
                    println!("send end_candidate failed");
                }
            })
        }));
        self.pc.on_peer_connection_state_change(Box::new(|state: RTCPeerConnectionState| {
            Box::pin(async move {
                println!("Peer Connection State has changed {:?}", state);
            })
        }));

        let self_arc = self.clone();
        self.pc.on_track(Box::new(move |remote, receiver, transceiver | {
            let self_arc2 = self_arc.clone();
            Box::pin(async move {
                self_arc2.on_track(remote, receiver).await;
            })
        }));

        self.pc.set_remote_description(RTCSessionDescription::offer(offer)?)
            .await?;
        let answer = self.pc.create_answer(None)
            .await?;
        self.pc.set_local_description(answer)
            .await?;

        if let Err(_) = time::timeout(Duration::from_secs(2), wait_candidate.recv()).await {
            return Err(anyhow!("wait candidate timeout"));
        }

        let answer_sdp = self.pc.local_description()
            .await
            .ok_or(anyhow!("no local description"))?;

        Ok(answer_sdp.sdp)
    }

    pub async fn on_track(
        &self,
        remote: Arc<TrackRemote>,
        receiver: Arc<RTCRtpReceiver>,
    ) {
        tokio::spawn(async move {
            loop {
                let (rtp_packet, attribute) = remote.read_rtp().await.unwrap();
                println!("RTP Packet has been received");
                println!("RTP Attribute has been received");
            }
        });
    }
}

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

        let tx = self.ontrack_ch.0.clone();
        pc.on_track(Box::new(move |remote, receiver, transceiver | {
            let tx2 = tx.clone();
            Box::pin(async move {
                if let Err(e) = tx2.send((remote, receiver, transceiver)).await {
                    log::error!("failed to send ontrack: {}", e);
                } else {
                    log::info!("ontrack sent");
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

        let mut handles = vec![];
        loop {
            match self.ontrack_ch.1.recv().await {
                Some((remote, receiver, transceiver)) => {
                    let handle = tokio::spawn(async move {
                        Self::on_track(remote, receiver, transceiver).await;
                    });
                    handles.push(handle);
                }
                None => {
                    break;
                }
            }
        }
        for handle in handles {
            match handle.await {
                Ok(_) => {}
                Err(e) => {
                    log::error!("failed to await handle: {}", e);
                }
            }
        }

        Ok(())
    }
    pub async fn on_track(remote: Arc<TrackRemote>, receiver: Arc<RTCRtpReceiver>, transceiver: Arc<RTCRtpTransceiver>) -> anyhow::Result<()> {
        log::info!("Track has been received: pt:{}, ssrc:{}, mimeType:{}, streamID:{}, trackID:{}, rid:{}",
            remote.payload_type(),
            remote.ssrc(),
            remote.codec().capability.mime_type,
            remote.stream_id(),
            remote.id(),
            remote.rid(),
        );
        let (rtp_packet, attribute) = remote.read_rtp().await?;
        log::info!("RTP Packet has been received: {:?}", rtp_packet);
        log::info!("RTP Attribute has been received: {:?}", attribute);

        Ok(())
    }
}
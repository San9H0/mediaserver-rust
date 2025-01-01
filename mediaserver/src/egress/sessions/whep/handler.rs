use crate::codecs::codec::Codec;
use crate::codecs::h264::format::NALUType;
use crate::egress::sessions::session::SessionHandler;
use crate::egress::sessions::whep::local_track::LocalTrack;
use crate::egress::sessions::whep::track_context;
use crate::hubs::source::HubSource;
use crate::hubs::stream::HubStream;
use crate::hubs::unit::HubUnit;
use crate::utils::packet;
use crate::utils::types::types;
use crate::webrtc_wrapper::webrtc_api::WebRtcApi;
use anyhow::anyhow;
use bitstreams::h264::nal_unit::NalUnit;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::time;
use tokio_util::sync::CancellationToken;
use webrtc::api::media_engine::MediaEngine;
use webrtc::ice_transport::ice_candidate::RTCIceCandidate;
use webrtc::ice_transport::ice_gatherer::OnLocalCandidateHdlrFn;
use webrtc::peer_connection::peer_connection_state::RTCPeerConnectionState;
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;
use webrtc::peer_connection::{
    OnPeerConnectionStateChangeHdlrFn, OnTrackHdlrFn, RTCPeerConnection,
};
use webrtc::rtp_transceiver::rtp_codec::{RTCRtpCodecParameters, RTPCodecType};
use webrtc::rtp_transceiver::rtp_receiver::RTCRtpReceiver;
use webrtc::track::track_local::TrackLocalWriter;
use webrtc::track::track_remote::TrackRemote;

pub struct WhepHandler {
    id: String,

    pc: RTCPeerConnection,
    token: CancellationToken,
    local_track: LocalTrack,
    sources: Vec<Arc<HubSource>>,

    started: AtomicBool,
}

impl WhepHandler {
    pub async fn new(id: &str, hub_stream: &Arc<HubStream>) -> anyhow::Result<Arc<Self>> {
        let token = CancellationToken::new();
        let local_track = LocalTrack::new();
        let mut sources = vec![];
        let mut media_engine = MediaEngine::default();
        for source in hub_stream.get_sources().await.iter() {
            let codec = source.get_codec().await.unwrap();
            let mut payload_type = 96;
            let mut kind = RTPCodecType::Video;
            if codec.kind() == types::MediaKind::Audio {
                payload_type = 111;
                kind = RTPCodecType::Audio;
            }
            if let Err(err) = media_engine.register_codec(
                RTCRtpCodecParameters {
                    capability: codec.rtp_codec_capability(),
                    payload_type,
                    stats_id: "".to_string(),
                },
                kind,
            ) {
                log::warn!("register codec failed: {:?}", err);
            };
            sources.push(source.clone());
        }

        let api = WebRtcApi::new_with_media_engine(media_engine);
        let pc = api.new_peer_connection().await;

        Ok(Arc::new(Self {
            id: id.to_string(),
            pc,
            token,
            local_track,
            sources,
            started: AtomicBool::new(false),
        }))
    }

    pub async fn init(self: &Arc<Self>, offer: &str) -> anyhow::Result<String> {
        let (end_candidate, mut wait_candidate) = mpsc::channel(1);
        self.pc
            .on_ice_candidate(self.on_ice_candidate(end_candidate));
        self.pc
            .on_peer_connection_state_change(self.on_peer_connection_state_change());
        self.pc.on_track(self.on_track());

        let _video_transceiver = self
            .pc
            .add_transceiver_from_track(self.local_track.video_local_track.clone(), None)
            .await?;

        let _audio_transceiver = self
            .pc
            .add_transceiver_from_track(self.local_track.audio_local_track.clone(), None)
            .await?;

        self.pc
            .set_remote_description(RTCSessionDescription::offer(offer.to_string())?)
            .await?;
        let answer = self.pc.create_answer(None).await?;
        self.pc.set_local_description(answer).await?;

        if let Err(_) = time::timeout(time::Duration::from_secs(2), wait_candidate.recv()).await {
            return Err(anyhow!("wait candidate timeout"));
        }

        let answer_sdp = self
            .pc
            .local_description()
            .await
            .ok_or(anyhow!("no local description"))?;

        Ok(answer_sdp.sdp)
    }
    fn on_ice_candidate(
        self: &Arc<Self>,
        candidate_tx: mpsc::Sender<()>,
    ) -> OnLocalCandidateHdlrFn {
        Box::new(move |candidate: Option<RTCIceCandidate>| {
            if candidate.is_none() {
                return Box::pin(async move {});
            }
            let candidate_tx = candidate_tx.clone();
            Box::pin(async move {
                if let Err(_) =
                    time::timeout(time::Duration::from_secs(1), candidate_tx.send(())).await
                {
                    log::warn!("send end_candidate failed");
                }
            })
        })
    }

    fn on_peer_connection_state_change(self: &Arc<Self>) -> OnPeerConnectionStateChangeHdlrFn {
        let weak = Arc::downgrade(self);
        Box::new(move |state: RTCPeerConnectionState| {
            let Some(arc) = weak.upgrade() else {
                return Box::pin(async move {});
            };

            match state {
                RTCPeerConnectionState::Connected => {}
                RTCPeerConnectionState::Disconnected
                | RTCPeerConnectionState::Failed
                | RTCPeerConnectionState::Closed => {
                    arc.token.cancel();
                }
                _ => {}
            }
            Box::pin(async move {})
        })
    }
    fn on_track(self: &Arc<Self>) -> OnTrackHdlrFn {
        Box::new(
            move |_remote: Arc<TrackRemote>, _receiver: Arc<RTCRtpReceiver>, _| {
                println!("onTrackCalled");
                Box::pin(async move {})
            },
        )
    }
}

impl SessionHandler for WhepHandler {
    type TrackContext = track_context::TrackContext;

    fn cancel_token(&self) -> CancellationToken {
        self.token.clone()
    }

    fn get_sources(&self) -> Vec<Arc<HubSource>> {
        self.sources.clone()
    }

    fn on_track_context(&self, _: usize, codec: &Codec) -> track_context::TrackContext {
        track_context::TrackContext::new(codec)
    }

    async fn on_video(&self, ctx: &mut track_context::TrackContext, unit: &HubUnit) {
        if !self.started.load(Ordering::Acquire) {
            if unit.frame_info.flag != 1 {
                return;
            }
            self.started.store(true, Ordering::Release);
        }

        let Ok(packets) = ctx.make_packet(unit) else {
            return;
        };
        let local_track = self.local_track.get_local_track(types::MediaKind::Video);
        for packet in packets.iter() {
            if let Err(err) = local_track.write_rtp(packet).await {
                log::warn!("write rtp failed: {:?}", err);
            };
        }
    }

    async fn on_audio(&self, ctx: &mut track_context::TrackContext, unit: &HubUnit) {
        if !self.started.load(Ordering::Acquire) {
            return;
        }

        let Ok(packets) = ctx.make_packet(unit) else {
            return;
        };
        let local_track = self.local_track.get_local_track(types::MediaKind::Audio);
        for packet in packets.iter() {
            // println!("write audio rtp sn:{}, ts:{}", packet.header.sequence_number, packet.header.timestamp);
            if let Err(err) = local_track.write_rtp(packet).await {
                log::warn!("write rtp failed: {:?}", err);
            };
        }
    }
}

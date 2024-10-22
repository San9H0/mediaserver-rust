use crate::codecs;
use crate::codecs::rtp_packetizer::RtpPacketizer;
use crate::codecs::rtp_payloader::RtpPayloader;
use crate::egress::sessions::whep::local_track::LocalTrack;
use crate::hubs::stream::HubStream;
use crate::webrtc_wrapper::webrtc_api::WebRtcApi;
use anyhow::anyhow;
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

pub struct WhepSession {
    pc: RTCPeerConnection,
    hub_stream: Arc<HubStream>,
    token: CancellationToken,
    local_track: LocalTrack,
}

impl WhepSession {
    pub async fn new(hub_stream: Arc<HubStream>) -> Arc<Self> {
        let token = CancellationToken::new();
        let local_track = LocalTrack::new();

        let mut media_engine = MediaEngine::default();
        for source in hub_stream.get_sources().await.iter() {
            let codec = source.get_codec().await.unwrap();
            let mut payload_type = 96;
            let mut kind = RTPCodecType::Video;
            if codec.kind() == "audio" {
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
        }

        let api = WebRtcApi::new_with_media_engine(media_engine);
        let pc = api.new_peer_connection().await;
        Arc::new(WhepSession {
            pc,
            hub_stream,
            token,
            local_track,
        })
    }

    pub fn stop(self: &Arc<Self>) {
        self.token.cancel();
    }
    pub async fn run(self: &Arc<Self>) {
        self.read_source().await;
        self.token.cancelled().await;
        let _ = self.pc.close().await;
    }

    pub async fn init(self: &Arc<Self>, offer: String) -> anyhow::Result<String> {
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
            .set_remote_description(RTCSessionDescription::offer(offer)?)
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
    async fn read_source(self: &Arc<Self>) {
        let (tx, mut rx) = mpsc::channel::<()>(100);
        for source in self.hub_stream.get_sources().await {
            let codec = source.get_codec().await.unwrap();
            let kind = codec.kind();
            let local_track = self.local_track.get_local_track(&codec.kind());
            let track = source.get_track().await;
            let sink = track.add_sink().await;
            let payloader = RtpPayloader::new(codec.mime_type()).unwrap();
            let mut packetizer = RtpPacketizer::new(payloader, codec.clock_rate());

            let _ = kind;
            let local_track_ = local_track;
            let mut sent_key_frame = false;
            tokio::spawn(async move {
                loop {
                    tokio::select! {
                        result = sink.read_unit() => {
                            let Ok(mut unit) = result else {
                                break;
                            };

                            if !sent_key_frame && codec.kind() == "video" {
                                let nalu = unit.payload[0] & 0x1F;
                                if nalu == 1 {
                                    continue
                                }
                                sent_key_frame = true;
                            }

                            let Ok(packets) = packetizer.packetize(&unit.payload, unit.duration) else {
                                continue;
                            };
                            for packet in packets.iter() {
                                if let Err(err) = local_track_.write_rtp(packet).await {
                                    log::warn!("write rtp failed: {:?}", err);
                                };
                            }
                        }
                    }
                }
            });
        }
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
                    arc.stop();
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

impl Drop for WhepSession {
    fn drop(&mut self) {
        println!("whep session drop");
    }
}

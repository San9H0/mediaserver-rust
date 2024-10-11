use crate::codecs;
use crate::codecs::rtp_packetizer::RtpPacketizer;
use crate::codecs::rtp_payloader::RtpPayloader;
use crate::hubs::stream::HubStream;
use crate::webrtc_wrapper::webrtc_api::WebRtcApi;
use anyhow::anyhow;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::time;
use webrtc::ice_transport::ice_candidate::RTCIceCandidate;
use webrtc::peer_connection::peer_connection_state::RTCPeerConnectionState;
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;
use webrtc::peer_connection::RTCPeerConnection;
use webrtc::rtp;
use webrtc::rtp::packetizer::Payloader;
use webrtc::rtp_transceiver::rtp_codec::RTCRtpCodecCapability;
use webrtc::rtp_transceiver::RTCRtpTransceiver;
use webrtc::track::track_local::track_local_static_rtp::TrackLocalStaticRTP;
use webrtc::track::track_local::TrackLocalWriter;

pub struct WhepSession {
    pc: RTCPeerConnection,
    hub_stream: Arc<HubStream>,
}

impl WhepSession {
    pub async fn new(api: Arc<WebRtcApi>, hub_stream: Arc<HubStream>) -> Arc<Self> {
        let pc = api.new_peer_connection().await;
        let whip_session = Self { pc, hub_stream };
        Arc::new(whip_session)
    }

    pub async fn init(self: Arc<Self>, offer: String) -> anyhow::Result<String> {
        let (end_candidate, mut wait_candidate) = mpsc::channel(1);
        self.pc
            .on_ice_candidate(Box::new(move |c: Option<RTCIceCandidate>| {
                let end_candidate2 = end_candidate.clone();
                Box::pin(async move {
                    if !c.is_none() {
                        return;
                    }
                    if let Err(_) =
                        time::timeout(time::Duration::from_secs(1), end_candidate2.send(())).await
                    {
                        println!("send end_candidate failed");
                    }
                })
            }));
        self.pc
            .on_peer_connection_state_change(Box::new(|state: RTCPeerConnectionState| {
                Box::pin(async move {
                    println!("Peer Connection State has changed {:?}", state);
                })
            }));

        self.pc
            .on_track(Box::new(move |_remote, _receiver, _transceiver| {
                Box::pin(async move {
                    println!("on_track");
                })
            }));

        let local_tracks = LocalTrack::new();

        let _video_transceiver = self
            .pc
            .add_transceiver_from_track(local_tracks.video_local_track.clone(), None)
            .await?;

        let _audio_transceiver = self
            .pc
            .add_transceiver_from_track(local_tracks.audio_local_track.clone(), None)
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

        for source in self.hub_stream.get_sources().await {
            let sink = source.get_track().await;
            let track = sink.add_sink().await;
            let codec = source.get_codec();
            let kind = codec.kind();
            let mut parser = codecs::rtp_parser::RtpParser::new(codec.mime_type())
                .map_err(|err| log::warn!("unsupported codec: {:?}", err))
                .unwrap();

            let local_track = local_tracks.get_local_track(&codec.kind());
            tokio::spawn(async move {
                let payloader = match RtpPayloader::new(codec.mime_type()) {
                    Ok(payloader) => payloader,
                    Err(e) => {
                        log::warn!("failed to create payloader: {:?}", e);
                        return;
                    }
                };
                let packetizer = RtpPacketizer::new(kind, payloader, 0, 0, codec.clock_rate());
                let mut packetizer = Box::new(packetizer);

                loop {
                    let hub_unit = match track.read_unit().await {
                        Ok(hub_unit) => hub_unit,
                        Err(e) => {
                            log::warn!("failed to read unit: {:?}", e);
                            return;
                        }
                    };

                    let (payloads, frame_info) = match parser.parse(hub_unit.payload) {
                        Some(data) => data,
                        None => continue,
                    };

                    for (index, payload) in payloads.into_iter().enumerate() {
                        let packets = match packetizer.packetize(&payload, codec.samples()) {
                            Ok(packets) => packets,
                            Err(e) => {
                                log::warn!("failed to packetize: {:?}", e);
                                continue;
                            }
                        };
                        for packet in packets.iter() {
                            if let Err(e) = local_track.write_rtp(packet).await {
                                log::warn!("failed to write rtp: {:?}", e);
                            }
                        }
                    }

                    // if let Err(e) = local_track.write_rtp(&hub_unit.rtp_packet).await {
                    //     log::warn!("failed to write rtp: {:?}", e);
                    // }

                    // let packets = match packetizer.packetize(&hub_unit.payload, codec.samples()) {
                    //     Ok(packets) => packets,
                    //     Err(e) => {
                    //         log::warn!("failed to packetize: {:?}", e);
                    //         continue;
                    //     }
                    // };
                    //
                    // for packet in packets.iter() {
                    //     if let Err(e) = local_track.write_rtp(packet).await {
                    //         log::warn!("failed to write rtp: {:?}", e);
                    //     }
                    // }
                }
            });
        }

        Ok(answer_sdp.sdp)
    }
}

struct LocalTrack {
    audio_local_track: Arc<TrackLocalStaticRTP>,
    video_local_track: Arc<TrackLocalStaticRTP>,
}

impl LocalTrack {
    fn new() -> LocalTrack {
        let stream_id = uuid::Uuid::new_v4().to_string();
        let video_track_id = uuid::Uuid::new_v4().to_string();
        let audio_track_id = uuid::Uuid::new_v4().to_string();

        let video_codec = RTCRtpCodecCapability {
            mime_type: "video/h264".to_string(),
            clock_rate: 90000,
            channels: 0,
            sdp_fmtp_line: "level-asymmetry-allowed=1;packetization-mode=1;profile-level-id=42001f"
                .to_string(),
            rtcp_feedback: vec![],
        };
        let video_local_track = Arc::new(TrackLocalStaticRTP::new(
            video_codec,
            video_track_id,
            stream_id.to_string(),
        ));

        let audio_codec = RTCRtpCodecCapability {
            mime_type: "audio/opus".to_string(),
            clock_rate: 48000,
            channels: 2,
            sdp_fmtp_line: "minptime=10;useinbandfec=1".to_string(),
            rtcp_feedback: vec![],
        };
        let audio_local_track = Arc::new(TrackLocalStaticRTP::new(
            audio_codec,
            audio_track_id,
            stream_id.to_string(),
        ));
        LocalTrack {
            audio_local_track,
            video_local_track,
        }
    }
    fn get_local_track(&self, kind: &str) -> Arc<TrackLocalStaticRTP> {
        match kind {
            "audio" => self.audio_local_track.clone(),
            "video" => self.video_local_track.clone(),
            _ => panic!("unsupported kind"),
        }
    }
}

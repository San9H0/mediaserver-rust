use crate::codecs;
use crate::codecs::codec::Codec;
use crate::hubs::source::HubSource;
use crate::hubs::stream::HubStream;
use crate::hubs::unit::HubUnit;
use crate::webrtc_wrapper::webrtc_api::WebRtcApi;
use anyhow::anyhow;
use std::error::Error;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time;
use webrtc::ice_transport::ice_candidate::RTCIceCandidate;
use webrtc::peer_connection::peer_connection_state::RTCPeerConnectionState;
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;
use webrtc::peer_connection::RTCPeerConnection;
use webrtc::rtcp;
use webrtc::rtcp::payload_feedbacks::picture_loss_indication::PictureLossIndication;
use webrtc::rtp_transceiver::rtp_receiver::RTCRtpReceiver;
use webrtc::track::track_remote::TrackRemote;

pub struct WhipSession {
    pc: RTCPeerConnection,
    hub_stream: Arc<HubStream>,
}

impl WhipSession {
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
                        time::timeout(Duration::from_secs(1), end_candidate2.send(())).await
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

        let self_arc = self.clone();
        self.pc
            .on_track(Box::new(move |remote, receiver, transceiver| {
                let ssrc = remote.ssrc();
                let self_arc2 = self_arc.clone();
                Box::pin(async move {
                    let self_arc3 = self_arc2.clone();
                    let self_arc4 = self_arc2.clone();
                    tokio::spawn(async move {
                        loop {
                            time::sleep(Duration::from_secs(1)).await;
                            let ret = self_arc4
                                .clone()
                                .pc
                                .write_rtcp(&[Box::new(PictureLossIndication {
                                    sender_ssrc: ssrc,
                                    media_ssrc: 0,
                                })])
                                .await;
                        }
                    });
                    self_arc3.on_track(remote, receiver).await;
                })
            }));

        self.pc
            .set_remote_description(RTCSessionDescription::offer(offer)?)
            .await?;
        let answer = self.pc.create_answer(None).await?;
        self.pc.set_local_description(answer).await?;

        if let Err(_) = time::timeout(Duration::from_secs(2), wait_candidate.recv()).await {
            return Err(anyhow!("wait candidate timeout"));
        }

        let answer_sdp = self
            .pc
            .local_description()
            .await
            .ok_or(anyhow!("no local description"))?;

        Ok(answer_sdp.sdp)
    }

    pub async fn on_track(&self, remote: Arc<TrackRemote>, _receiver: Arc<RTCRtpReceiver>) {
        let codec = Codec::new(&remote.codec().capability.mime_type)
            .map_err(|err| log::warn!("unsupported codec: {:?}", err))
            .unwrap();

        let hub_stream = self.hub_stream.clone();
        let timebase = remote.codec().capability.clock_rate;
        tokio::spawn(async move {
            let source = HubSource::new(codec);
            hub_stream.add_source(source.clone()).await;

            let mut parser =
                codecs::rtp_parser::RtpParser::new(&remote.codec().capability.mime_type)
                    .map_err(|err| log::warn!("unsupported codec: {:?}", err))
                    .unwrap();

            let (mut start_ts, mut prev_ts, mut duration) = (0, 0, 0);
            loop {
                let (rtp_packet, _attribute) = remote.read_rtp().await.unwrap();
                if rtp_packet.payload.len() == 0 {
                    continue;
                }

                if start_ts == 0 {
                    start_ts = rtp_packet.header.timestamp;
                }
                let pts = rtp_packet.header.timestamp - start_ts;
                if rtp_packet.header.timestamp != prev_ts {
                    if prev_ts == 0 {
                        duration = 0
                    } else {
                        duration = rtp_packet.header.timestamp - prev_ts;
                    }
                }

                source
                    .write_unit(HubUnit {
                        rtp_packet: rtp_packet.clone(),
                        payload: rtp_packet.payload.clone(),
                        // pts,
                        // dts,
                        // duration,
                        // timebase,
                        // marker,
                    })
                    .await;

                // let (payloads, frame_info) = match parser.parse(rtp_packet.payload) {
                //     Some(data) => data,
                //     None => continue,
                // };
                //
                // for (index, payload) in payloads.into_iter().enumerate() {
                //     let dts = pts;
                //     let marker = index == payload.len() - 1;
                //     let frame_info = frame_info.clone();
                //     source
                //         .write_unit(HubUnit {
                //             payload,
                //             pts,
                //             dts,
                //             duration,
                //             timebase,
                //             marker,
                //             frame_info,
                //         })
                //         .await;
                // }
            }
        });
    }
}

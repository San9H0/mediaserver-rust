use crate::codecs;
use crate::codecs::codec::Codec;
use crate::codecs::h264::rtp_parser::MyParser;
use crate::codecs::rtp_parser::RtpParser;
use crate::hubs::source::HubSource;
use crate::hubs::stream::HubStream;
use crate::hubs::unit::HubUnit;
use crate::ingress::sessions::whip::stats::Stats;
use crate::webrtc_wrapper::webrtc_api::WebRtcApi;
use anyhow::anyhow;
use bytes::{Bytes, BytesMut};
use std::cell::RefCell;
use std::pin::Pin;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;
use futures::lock::Mutex;
use tokio::sync::mpsc;
use tokio::time;
use tokio_util::sync::CancellationToken;
use webrtc::api::media_engine::{MediaEngine, MIME_TYPE_H264, MIME_TYPE_OPUS};
use webrtc::ice_transport::ice_candidate::RTCIceCandidate;
use webrtc::ice_transport::ice_gatherer::OnLocalCandidateHdlrFn;
use webrtc::peer_connection::peer_connection_state::RTCPeerConnectionState;
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;
use webrtc::peer_connection::{
    OnPeerConnectionStateChangeHdlrFn, OnTrackHdlrFn, RTCPeerConnection,
};
use webrtc::rtcp::payload_feedbacks::picture_loss_indication::PictureLossIndication;
use webrtc::rtcp::payload_feedbacks::receiver_estimated_maximum_bitrate::ReceiverEstimatedMaximumBitrate;
use webrtc::rtcp::receiver_report::ReceiverReport;
use webrtc::rtcp::reception_report::ReceptionReport;
use webrtc::rtp_transceiver::rtp_codec::{
    RTCRtpCodecCapability, RTCRtpCodecParameters, RTPCodecType,
};
use webrtc::rtp_transceiver::rtp_receiver::RTCRtpReceiver;
use webrtc::track::track_remote::TrackRemote;
use crate::egress::sessions::record::handler::RecordHandler;
use ffmpeg_next as ffmpeg;
use ffmpeg_next::Rescale;
use crate::codecs::codec::Codec::Opus;
use crate::codecs::opus::codec::OpusCodec;

pub struct WhipSession {
    pc: RTCPeerConnection,
    hub_stream: Arc<HubStream>,
    token: CancellationToken,
}

impl WhipSession {
    pub async fn new(hub_stream: Arc<HubStream>) -> Arc<Self> {
        // let api = WebRtcApi::new();
        let mut media_engine = MediaEngine::default();
        let result = media_engine.register_codec(
            RTCRtpCodecParameters {
                capability: RTCRtpCodecCapability {
                    mime_type: MIME_TYPE_H264.to_string(),
                    clock_rate: 90000,
                    channels: 0,
                    sdp_fmtp_line:
                        "level-asymmetry-allowed=1;packetization-mode=1;profile-level-id=42001f"
                            .to_string(),
                    rtcp_feedback: vec![],
                },
                payload_type: 111,
                stats_id: "".to_string(),
            },
            RTPCodecType::Video,
        );
        let result = media_engine.register_codec(
            RTCRtpCodecParameters {
                capability: RTCRtpCodecCapability {
                    mime_type: MIME_TYPE_OPUS.to_string(),
                    clock_rate: 48000,
                    channels: 2,
                    sdp_fmtp_line: "".to_string(),
                    rtcp_feedback: vec![],
                },
                payload_type: 111,
                stats_id: "".to_string(),
            },
            RTPCodecType::Audio,
        );

        let api = WebRtcApi::new_with_media_engine(media_engine);
        let pc = api.new_peer_connection().await;
        let token = CancellationToken::new();
        Arc::new(WhipSession {
            pc,
            hub_stream,
            token,
        })
    }

    pub fn stop(&self) {
        self.token.cancel();
    }
    pub async fn run(&self) {
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
    fn on_ice_candidate(
        self: &Arc<Self>,
        candidate_tx: mpsc::Sender<()>,
    ) -> OnLocalCandidateHdlrFn {
        Box::new(move |candidate: Option<RTCIceCandidate>| {
            if candidate.is_none() {
                return Box::pin(async move {});
            }
            println!("recv ice candidate...");
            let candidate_tx = candidate_tx.clone();
            Box::pin(async move {
                if let Err(_) =
                    time::timeout(time::Duration::from_secs(1), candidate_tx.send(())).await
                {
                    println!("send end_candidate failed");
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
        let weak = Arc::downgrade(self);
        Box::new(
            move |remote: Arc<TrackRemote>, receiver: Arc<RTCRtpReceiver>, _| {
                let Some(arc) = weak.upgrade() else {
                    return Box::pin(async move {});
                };
                let self_ = &arc;
                let stats = Stats::new(remote.codec().capability.clock_rate);

                if remote.kind() == RTPCodecType::Video {
                    self_.send_pli(&remote);
                }
                self_.send_rtcp(&remote, &stats);
                if remote.kind() == RTPCodecType::Audio {
                    self_.read_rtp_audio(&remote, &receiver, &stats);
                } else {
                    self_.read_rtp_video(&remote, &receiver, &stats);
                }
                Box::pin(async move {})
            },
        )
    }
    fn send_pli(self: &Arc<Self>, remote: &Arc<TrackRemote>) {
        let remote_ = remote.clone();
        let self_ = self.clone();
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = self_.token.cancelled() => {
                        break;
                    }
                    _ = tokio::time::sleep(Duration::from_secs(1)) => {
                        if let Err(e) = self_.clone().pc.write_rtcp(&[Box::new(PictureLossIndication {
                            sender_ssrc: 0,
                            media_ssrc: remote_.ssrc(),
                        })]).await {
                            log::warn!("failed to send PLI: {}", e);
                        };
                    }
                }
            }
        });
    }
    fn send_rtcp(self: &Arc<Self>, remote: &Arc<TrackRemote>, stats: &Arc<Stats>) {
        let remote_ = remote.clone();
        let self_ = self.clone();
        let stats_ = stats.clone();
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = self_.token.cancelled() => {
                        break;
                    }
                    _ = tokio::time::sleep(Duration::from_secs(1)) => {
                        let remb = ReceiverEstimatedMaximumBitrate{
                            sender_ssrc: 0,
                            bitrate: 3_000_000f32 ,
                            ssrcs: vec![remote_.ssrc()],
                        };
                        let rr = stats_.make_receiver_report(remote_.ssrc()).await;
                        if let Err(e) = self_.clone().pc.write_rtcp(&[Box::new(remb), Box::new(rr)]).await {
                            log::warn!("failed to send rtcp: {}", e);
                        };
                    }
                }
            }
        });
    }
    fn read_rtp_audio(
        self: &Arc<Self>,
        remote: &Arc<TrackRemote>,
        receiver: &Arc<RTCRtpReceiver>,
        stats: &Arc<Stats>,
    ) -> anyhow::Result<()>{

        // let codec_info = Opus(OpusCodec::new());
        // let name_filename = "output2.mp4";
        // let mut output_ctx = ffmpeg::format::output(&name_filename)?;
        // let codec = ffmpeg::codec::encoder::find(codec_info.av_codec_id())
        //     .ok_or(ffmpeg::Error::EncoderNotFound)?;
        // let encoder_ctx = ffmpeg::codec::context::Context::new_with_codec(codec);
        // let mut audio = encoder_ctx.encoder().audio()?;
        // let a = codec_info.set_av_audio(&mut audio);
        // let encoder = audio.open_as(codec)?;
        // output_ctx.add_stream_with(&encoder)?;
        // output_ctx.write_header()?;



        let self_ = self.clone();
        let remote_ = remote.clone();
        let receiver_ = receiver.clone();
        let mut stats_ = stats.clone();
        tokio::spawn(async move {
            let source = HubSource::new();
            self_.hub_stream.add_source(source.clone()).await;

            let mut source_1 = source.clone();
            let mut parser = RtpParser::new(&remote_.codec().capability.mime_type,
                Box::new(move |codec: Codec| {
                    let source = source_1.clone();
                    Box::pin(async move {
                        log::info!("whip set codec: {:?}", codec.mime_type());
                        source.set_codec(codec).await;
                    })
                }),
            )
            .map_err(|err| log::warn!("unsupported codec: {:?}", err))
            .unwrap();
            let mut start_ts = 0;
            let mut last_ts = 0;
            let mut duration = 0;
            let mut timebase = remote_.codec().capability.clock_rate;
            let mut ts_offset = 0;
            let mut prev_pts = 0;
            let mut test_prev_ts = 0;

            //
            // let mut record_pts = 0;
            // let mut record_dts = 0;
            // let mut record_base_ts = 0;
            // let mut record_set_pts = false;
            loop {
                tokio::select! {
                    _ = self_.token.cancelled() => {
                        break;
                    }
                    result = remote_.read_rtp() => {
                        let Ok((rtp_packet, _)) = result else { break };
                        stats_.calc_rtp_stats(&rtp_packet).await;
                        if rtp_packet.payload.len() == 0 {
                            continue;
                        }
                        if start_ts == 0 {
                            start_ts = rtp_packet.header.timestamp;
                            last_ts = rtp_packet.header.timestamp;
                        }
                        let pts = rtp_packet.header.timestamp.wrapping_sub(start_ts);
                        let dts = pts;
                        if rtp_packet.header.timestamp != last_ts {
                            duration = rtp_packet.header.timestamp.wrapping_sub(last_ts);
                            last_ts = rtp_packet.header.timestamp;
                        }

                        let Some((payloads, frame_info)) = parser.parse(rtp_packet.payload).await else {
                            continue;
                        };

                        let len = payloads.len();
                        for (index, payload) in payloads.into_iter().enumerate() {
                            let marker = index == len - 1;
                            let frame_info = frame_info.clone();

                            source
                            .write_unit(HubUnit {
                                payload,
                                pts,
                                dts,
                                duration,
                                timebase,
                                marker,
                                frame_info,
                            }).await;

                            // let unit = HubUnit {
                            //     payload,
                            //     pts,
                            //     dts,
                            //     duration,
                            //     timebase,
                            //     marker,
                            //     frame_info,
                            // };
                            //
                            //
                            // if !record_set_pts {
                            //     record_set_pts = true;
                            //     record_base_ts = unit.pts;
                            // }
                            // record_pts = unit.pts - record_base_ts;
                            // record_dts = unit.dts - record_base_ts;
                            //
                            // let data_len = unit.payload.len() as u32;
                            // let mut buffer = BytesMut::with_capacity(unit.payload.len());
                            // buffer.extend_from_slice(&unit.payload);
                            // let data = buffer.freeze();
                            //
                            // let input_time_base = ffmpeg::Rational::new(1, unit.timebase as i32);
                            // let output_time_base = ffmpeg::Rational::new(1, codec_info.clock_rate() as i32);
                            //
                            // let mut pkt = ffmpeg::packet::Packet::copy(data.iter().as_slice());
                            // pkt.set_stream(0);
                            // pkt.set_pts(Some((pts as i64).rescale(input_time_base, output_time_base)));
                            // pkt.set_dts(Some((dts as i64).rescale(input_time_base, output_time_base)));
                            // pkt.set_duration((unit.duration as i64).rescale_with(
                            //     input_time_base,
                            //     output_time_base,
                            //     ffmpeg::mathematics::Rounding::NearInfinity,
                            // ));
                            // if unit.frame_info.flag == 1 {
                            //     pkt.set_flags(ffmpeg::packet::Flags::KEY);
                            // }
                            // {
                            //     println!("write pkt pts:{:?}, dts:{:?}, duration:{:?}, flag:{}", pkt.pts(), pkt.dts(), unit.duration, unit.frame_info.flag);
                            //     if let Err(err) = pkt.write_interleaved(&mut output_ctx) {
                            //         log::warn!("failed to write packet: {}", err);
                            //         continue;
                            //     };
                            //     println!("end write pkt pts:{:?}, dts:{:?}, duration:{:?}, flag:{}", pkt.pts(), pkt.dts(), unit.duration, unit.frame_info.flag);
                            // }
                        }
                    }
                }
            }
            //
            // output_ctx.write_trailer();

            self_.hub_stream.remove_source(source.clone()).await;
            source.stop();
        });

        Ok(())
    }


    fn read_rtp_video(
        self: &Arc<Self>,
        remote: &Arc<TrackRemote>,
        receiver: &Arc<RTCRtpReceiver>,
        stats: &Arc<Stats>,
    ) {

        let self_ = self.clone();
        let remote_ = remote.clone();
        let receiver_ = receiver.clone();
        let mut stats_ = stats.clone();
        tokio::spawn(async move {
            let source = HubSource::new();
            self_.hub_stream.add_source(source.clone()).await;

            let mut source_1 = source.clone();
            let mut parser = RtpParser::new(&remote_.codec().capability.mime_type,
                                            Box::new(move |codec: Codec| {
                                                let source = source_1.clone();
                                                Box::pin(async move {
                                                    log::info!("whip set codec: {:?}", codec.mime_type());
                                                    source.set_codec(codec).await;
                                                })
                                            }),
            )
                .map_err(|err| log::warn!("unsupported codec: {:?}", err))
                .unwrap();
            let mut start_ts = 0;
            let mut last_ts = 0;
            let mut duration = 0;
            let mut timebase = remote_.codec().capability.clock_rate;
            let mut ts_offset = 0;
            let mut prev_pts = 0;
            let mut test_prev_ts = 0;
            loop {
                tokio::select! {
                    _ = self_.token.cancelled() => {
                        break;
                    }
                    result = remote_.read_rtp() => {
                        let Ok((rtp_packet, _)) = result else { break };
                        stats_.calc_rtp_stats(&rtp_packet).await;
                        if rtp_packet.payload.len() == 0 {
                            continue;
                        }
                        if start_ts == 0 {
                            start_ts = rtp_packet.header.timestamp;
                            last_ts = rtp_packet.header.timestamp;
                        }
                        let pts = rtp_packet.header.timestamp.wrapping_sub(start_ts);
                        let dts = pts;
                        if rtp_packet.header.timestamp != last_ts {
                            duration = rtp_packet.header.timestamp.wrapping_sub(last_ts);
                            last_ts = rtp_packet.header.timestamp;
                        }

                        let Some((payloads, frame_info)) = parser.parse(rtp_packet.payload).await else {
                            continue;
                        };

                        let len = payloads.len();
                        for (index, payload) in payloads.into_iter().enumerate() {
                            let marker = index == len - 1;
                            let frame_info = frame_info.clone();

                            source
                            .write_unit(HubUnit {
                                payload,
                                pts,
                                dts,
                                duration,
                                timebase,
                                marker,
                                frame_info,
                            })
                            .await;
                        }
                    }
                }
            }

            self_.hub_stream.remove_source(source.clone()).await;
            source.stop();
        });
    }
}

impl Drop for WhipSession {
    fn drop(&mut self) {
        println!("WhipSession drop called");
        self.stop();
    }
}

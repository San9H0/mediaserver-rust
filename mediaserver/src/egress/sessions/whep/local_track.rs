use std::sync::Arc;
use webrtc::rtp_transceiver::rtp_codec::RTCRtpCodecCapability;
use webrtc::track::track_local::track_local_static_rtp::TrackLocalStaticRTP;
use crate::utils::types::types;

pub struct LocalTrack {
    pub audio_local_track: Arc<TrackLocalStaticRTP>,
    pub video_local_track: Arc<TrackLocalStaticRTP>,
}

impl LocalTrack {
    pub fn new() -> LocalTrack {
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
    pub fn get_local_track(&self, kind: types::MediaKind) -> Arc<TrackLocalStaticRTP> {
        match kind {
            types::MediaKind::Audio => self.audio_local_track.clone(),
            types::MediaKind::Video => self.video_local_track.clone(),
        }
    }
}

use std::sync::Arc;
use webrtc::rtp_transceiver::RTCRtpTransceiver;
use webrtc::rtp_transceiver::rtp_receiver::RTCRtpReceiver;
use webrtc::track::track_remote::TrackRemote;

pub struct WebRtcReceiver {
    remote: Arc<TrackRemote>,
    receiver: Arc<RTCRtpReceiver>,
    transceiver: Arc<RTCRtpTransceiver>,
}

impl WebRtcReceiver {
    pub fn new(
        remote: Arc<TrackRemote>,
        receiver: Arc<RTCRtpReceiver>,
        transceiver: Arc<RTCRtpTransceiver>,
    ) -> Self {
        WebRtcReceiver {
            remote,
            receiver,
            transceiver,
        }
    }

}
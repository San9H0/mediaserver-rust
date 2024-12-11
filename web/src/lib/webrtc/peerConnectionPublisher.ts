export interface PeerConnectionPublisherInit {
    useMaintainResolution: boolean;
    videoCodecName: string;
    audioCodecName: string;
}

export const createPeerConnectionPublisher = async (mediaStream: MediaStream, config: PeerConnectionPublisherInit): Promise<RTCPeerConnection> => {
    const rtpConfig: RTCConfiguration = {
        iceServers: [
            { urls: 'stun:stun.l.google.com:19302' }
        ]
    };
    const audioEncodings = [{ maxBitrate: 1024 * 1024 }];
    const videoEncodings = [{ maxBitrate: 1024 * 1024 * 1024 }];
    const pc = new RTCPeerConnection(rtpConfig);

    const audioInit: RTCRtpTransceiverInit = {
        direction: 'sendonly',
        sendEncodings: audioEncodings,
        streams: [mediaStream],
    };
    const videoInit: RTCRtpTransceiverInit = {
        direction: 'sendonly',
        sendEncodings: videoEncodings,
        streams: [mediaStream],
    };
    const audioTransceiver: RTCRtpTransceiver = pc.addTransceiver('audio', audioInit);
    const videoTransceiver: RTCRtpTransceiver = pc.addTransceiver('video', videoInit);

    pc.addTrack(mediaStream.getAudioTracks()[0], mediaStream);
    pc.addTrack(mediaStream.getVideoTracks()[0], mediaStream);

    if (config.useMaintainResolution) {
        const params = videoTransceiver.sender.getParameters();
        params.degradationPreference = "maintain-resolution";
        await videoTransceiver.sender.setParameters(params);
    }
    
    if (config.videoCodecName) {
        const videoProfile = config.videoCodecName === "h264" ? "42001f" : config.videoCodecName === "vp9" ? "profile-id=0" : "";
        console.log('videoProfile:', videoProfile);

        // TODO: RTCRtpReceiver가 쓰였는데 확인해봐야함.
        let selectedCodecs = RTCRtpSender.getCapabilities("video")?.codecs.filter(codec => 
            codec.mimeType.toLowerCase() === `video/${config.videoCodecName}`.toLowerCase()
        )
        console.log("selectedCodecs:", selectedCodecs);

        if (videoProfile !== "") {
            selectedCodecs = selectedCodecs?.filter(codec => 
                codec.sdpFmtpLine?.includes(`profile-level-id=${videoProfile}`)
            );
        }
        console.log("selectedCodecs2:", selectedCodecs);
        if (selectedCodecs && selectedCodecs.length > 0) {
            videoTransceiver.setCodecPreferences(selectedCodecs);
        }
    }

    if (config.audioCodecName) {
        // TODO: RTCRtpSender가 쓰였는데 확인해봐야함. 비디오는 RTCRtpTransceiver가 쓰였음.
        const audioCodecs = RTCRtpSender.getCapabilities('audio')?.codecs
        const opusFilteredCodec = audioCodecs?.filter(codec => 
            codec.mimeType.toLowerCase() === `audio/${config.audioCodecName}`
        )
        if (opusFilteredCodec && opusFilteredCodec.length > 0) {
            audioTransceiver.setCodecPreferences(opusFilteredCodec)
        }
    }

    pc.onicecandidate = event => {
        if (event.candidate) {
            console.log('ICE Candidate:', event.candidate);
        }
    };
    pc.onconnectionstatechange = () => {
        console.log('Connection State:', pc.connectionState);
    }

    return pc;
};
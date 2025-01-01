export interface PeerConnectionPublisherInit {
    useMaintainResolution: boolean;
    videoCodecName: string;
    audioCodecName: string;
}

export const createPeerConnectionSubscriber = async (): Promise<RTCPeerConnection> => {
    const rtcConfig: RTCConfiguration = {};
    const pc = new RTCPeerConnection(rtcConfig);
    pc.addTransceiver('audio', { direction: 'recvonly' });
    pc.addTransceiver('video', { direction: 'recvonly' });

    return pc;
};
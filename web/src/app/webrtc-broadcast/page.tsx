"use client";
import React from "react";

import { createPeerConnectionPublisher } from "@/lib/webrtc/peerConnectionPublisher";
interface HTMLVideoElementWithCaptureStream extends HTMLVideoElement {
  captureStream?: () => MediaStream;
}

export default function WebRTCBroadcast() {
  const [fileVideoSource, setFileVideoSource] = React.useState<string | null>(
    null
  );
  const [webcamVideoSource, setWebcamVideoSource] =
    React.useState<MediaStream | null>(null);
  const fileInputRef = React.useRef<HTMLInputElement>(null);
  const videoElementRef = React.useRef<HTMLVideoElementWithCaptureStream>(null);
  const streamKeyRef = React.useRef<HTMLInputElement>(null);

  const handleFileChange = (event: React.ChangeEvent<HTMLInputElement>) => {
    try {
      const file = event.target.files?.[0];
      if (!file) {
        return;
      }
      const url = URL.createObjectURL(file);
      setFileVideoSource(url);
    } catch (err: unknown) {
      console.error("Error accessing media devices.", err);
    } finally {
      setWebcamVideoSource(null);
    }
  };

  const handleWebcamStart = async () => {
    try {
      const stream: MediaStream = await navigator.mediaDevices.getUserMedia({
        video: true,
      });
      setWebcamVideoSource(stream);
    } catch (err: unknown) {
      console.error("Error accessing media devices.", err);
    } finally {
      setFileVideoSource(null);
    }
  };

  const handleStart = async () => {
    const mediaStream =
      webcamVideoSource ||
      (fileVideoSource && videoElementRef.current?.captureStream?.());
    if (!mediaStream) {
      throw new Error("No media stream available.");
    }
    const streamKey = streamKeyRef.current?.value;
    if (!streamKey) {
      throw new Error("No stream key available.");
    }

    try {
      const pc = await createPeerConnectionPublisher(mediaStream, {
        useMaintainResolution: true,
        videoCodecName: "h264",
        audioCodecName: "opus",
      });
      const offer = await pc.createOffer();
      await pc.setLocalDescription(offer);
      const response = await fetch("/v1/whip", {
        method: "POST",
        headers: {
          "Content-Type": "application/sdp",
          Authorization: `Bearer ${streamKey}`,
        },
        body: offer.sdp,
      });
      if (!response.ok) {
        console.error("Error sending offer to server.", response);
        return;
      }
      await pc.setRemoteDescription(
        new RTCSessionDescription({
          type: "answer",
          sdp: await response.text(),
        })
      );
    } catch (err: unknown) {
      console.error("Error creating peer connection.", err);
    }
  };

  return (
    <div className="container">
      <h1>WebRTC 방송</h1>
      <div>
        <input type="text" placeholder="Enter Stream Key" ref={streamKeyRef} />
      </div>
      <div>
        <input type="file" ref={fileInputRef} onChange={handleFileChange} />
        <button onClick={handleStart}>Start!</button>
        <button onClick={handleWebcamStart}>Start Webcam!</button>
      </div>
      {fileVideoSource && (
        <div>
          fileVideoSource
          <video
            ref={videoElementRef}
            src={fileVideoSource}
            controls
            autoPlay
            loop
          />
        </div>
      )}
      {webcamVideoSource && (
        <div>
          webcamVideoSource
          <video
            ref={(video) => {
              if (video) {
                video.srcObject = webcamVideoSource;
              }
            }}
            controls
            autoPlay
          />
        </div>
      )}
    </div>
  );
}

"use client";
import React from "react";

import { createPeerConnectionPublisher } from "@/lib/webrtc/peerConnectionPublisher";
import { asyncFileClick } from "@/lib/elements/input";
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

  const onFileStartClick = async () => {
    try {
      const file = await asyncFileClick(fileInputRef);
      if (file) {
        const url = URL.createObjectURL(file);
        setFileVideoSource(url); // 비디오 소스 설정
        setWebcamVideoSource(null); // 웹캠 소스 초기화
      }
    } catch (err) {
      console.error("file error ", err);
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
    <div className="container ">
      <input type="file" ref={fileInputRef} hidden />{" "}
      <h1 className="text-4xl font-bold text-center text-primary">
        WebRTC Publisher
      </h1>
      {/* space-y-4 */}
      <div className="flex flex-col">
        <div className="flex-1">
          <div className="flex flex-row justify-end">
            <button className=" btn btn-outline" onClick={onFileStartClick}>
              File
            </button>
            <button className="btn btn-outline" onClick={handleWebcamStart}>
              WebCam
            </button>
          </div>
        </div>
        <input
          type="text"
          placeholder="Enter Stream Key"
          ref={streamKeyRef}
          className="input input-bordered w-full"
        />
        <button
          className="flex-1 btn btn-outline btn-ghost"
          onClick={handleStart}
        >
          Start!
        </button>
      </div>
      {fileVideoSource && (
        <div className="mt-8">
          <h2 className="text-lg font-semibold text-primary">
            File Video Source
          </h2>
          <video
            ref={videoElementRef}
            src={fileVideoSource}
            controls
            autoPlay
            loop
            className="mt-4 w-full rounded-lg shadow-lg"
          />
        </div>
      )}
      {webcamVideoSource && (
        <div className="mt-8">
          <h2 className="text-lg font-semibold text-primary">
            Webcam Video Source
          </h2>
          <video
            ref={(video) => {
              if (video) {
                video.srcObject = webcamVideoSource;
              }
            }}
            controls
            autoPlay
            className="mt-4 w-full rounded-lg shadow-lg"
          />
        </div>
      )}
    </div>
  );
}

"use client";
import React from "react";
import "../../styles/globals.css";

import { createPeerConnectionPublisher } from "@/lib/webrtc/peerConnectionPublisher";
import { generateStreamKey } from "@/lib/id/streamkey";
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

  const getFile = async (): Promise<File | undefined> => {
    return await new Promise<File | undefined>((resolve, reject) => {
      if (!fileInputRef.current) {
        reject(new Error("File input reference is not defined."));
        return;
      }
      const input = fileInputRef.current;

      // Change 이벤트 핸들러 정의
      const handleChange = (event: Event) => {
        const target = event.target as HTMLInputElement;
        const file = target.files?.[0];

        // 이벤트 리스너 정리
        input.removeEventListener("change", handleChange);

        resolve(file); // 파일 반환
      };

      // Change 이벤트 리스너 추가
      input.addEventListener("change", handleChange);
    });
  };

  const onStartFileClick = async () => {
    await fileInputRef.current?.click();
    const file = await getFile();

    console.log("file:", file);
    if (file) {
      const url = URL.createObjectURL(file);
      setFileVideoSource(url); // 비디오 소스 설정
      setWebcamVideoSource(null); // 웹캠 소스 초기화
    }

    await handleStart();
  };

  const onStartWebCamClick = async () => {
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

    await handleStart();
  };

  const handleStart = async () => {
    const mediaStream =
      webcamVideoSource ||
      (fileVideoSource && videoElementRef.current?.captureStream?.());
    console.log("mediaStream:", mediaStream);
    if (!mediaStream) {
      throw new Error("No media stream available.");
    }
    const streamKey = generateStreamKey();
    if (streamKeyRef.current) {
      streamKeyRef.current.value = streamKey;
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
      <div>WebRTC 방송</div>
      파일 선택 및 WebCam 을 선택하세요
      <div>
        <input
          type="file"
          ref={fileInputRef}
          onChange={getFile}
          className="hidden"
        />
        <button className="btn" onClick={onStartFileClick}>
          Start File!
        </button>
        <button className="btn" onClick={onStartWebCamClick}>
          Start Webcam!
        </button>
      </div>
      <div>
        <input
          type="text"
          placeholder="Stream Key"
          ref={streamKeyRef}
          disabled
        />
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

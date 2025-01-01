"use client";
import { createPeerConnectionSubscriber } from "@/lib/webrtc/peerConnectionSubscriber";
import React from "react";

export default function WebRTCWatch() {
  const videoElementRef = React.useRef<HTMLVideoElement>(null);
  const streamKeyRef = React.useRef<HTMLInputElement>(null);
  const [videoStats, setVideoStats] = React.useState({
    bitrate: 0,
    fps: 0,
    delay: 0,
    resolution: { width: 0, height: 0 },
  });

  const statsRef = React.useRef({
    timestampPrev: 0,
    bytesPrev: 0,
    bufferPrev: 0,
    emittedPrev: 0,
  });

  const handleStart = async () => {
    const streamKey = streamKeyRef.current?.value;
    if (!streamKey) {
      throw new Error("No stream key available.");
    }

    const pc = await createPeerConnectionSubscriber();
    pc.ontrack = async (event) => {
      if (!videoElementRef.current) {
        return;
      }
      console.log("track:", event);
      if (videoElementRef.current.srcObject !== event.streams[0]) {
        videoElementRef.current.srcObject = event.streams[0];
        console.log("play?");
        await videoElementRef.current.play();
      }
    };
    const offer = await pc.createOffer();
    await pc.setLocalDescription(offer);
    const response = await fetch("/v1/whep", {
      method: "POST",
      headers: {
        "Content-Type": "application/sdp",
        Authorization: `Bearer ${streamKey}`,
      },
      body: offer.sdp,
    });
    console.log("response:", response);
    if (!response.ok) {
      console.error("Error sending offer to server.", response);
      return;
    }
    const answerSDP = await response.text();
    console.log("answerSDP:", answerSDP);
    await pc.setRemoteDescription(
      new RTCSessionDescription({ type: "answer", sdp: answerSDP })
    );

    setInterval(() => collectStats(pc), 1000);
  };

  const collectStats = async (pc: RTCPeerConnection) => {
    if (pc.connectionState !== "connected") {
      return;
    }
    const stats = await pc.getStats();

    stats.forEach((report) => {
      if (report.type !== "inbound-rtp" || report.mediaType !== "video") {
        return;
      }
      let bitrate = 0;
      let delay = 0;
      const now = report.timestamp;
      const bytes = report.bytesReceived;

      if (statsRef.current.timestampPrev) {
        bitrate = Math.floor(
          (8 * (bytes - statsRef.current.bytesPrev)) /
            (now - statsRef.current.timestampPrev)
        );
        delay = Math.floor(
          ((report.jitterBufferDelay - statsRef.current.bufferPrev) /
            (report.jitterBufferEmittedCount - statsRef.current.emittedPrev)) *
            1000
        );
      }

      statsRef.current = {
        timestampPrev: now,
        bytesPrev: bytes,
        bufferPrev: report.jitterBufferDelay,
        emittedPrev: report.jitterBufferEmittedCount,
      };

      const fps = report.framesPerSecond || 0;
      const width = videoElementRef.current?.videoWidth || 0;
      const height = videoElementRef.current?.videoHeight || 0;

      setVideoStats({
        bitrate,
        fps,
        delay,
        resolution: { width, height },
      });
    });
  };

  return (
    <div className="container">
      <div className="flex flex-col gap-4">
        <h1 className="text-2xl font-bold">WebRTC Watch</h1>
        <div className="flex flex-col gap-2">
          <label htmlFor="stream-key" className="text-sm font-medium">
            Stream Key
          </label>
          <input
            type="text"
            id="stream-key"
            ref={streamKeyRef}
            className="border border-gray-300 rounded-md p-2"
          />
        </div>
        <button
          className="bg-blue-500 text-white rounded-md p-2"
          onClick={handleStart}
        >
          Start
        </button>
        <video ref={videoElementRef} className="w-full h-full" />

        {/* 통계 정보 표시 */}
        <div className="bg-gray-100 p-4 rounded-md">
          <h2 className="font-bold mb-2">Video Statistics</h2>
          <div className="space-y-1">
            <p>
              {videoStats.bitrate > 0
                ? `${videoStats.bitrate} kbps @ ${videoStats.fps} fps`
                : "NO MEDIA"}
            </p>
            <p>
              {videoStats.delay > 0
                ? `Video Delay: ${videoStats.delay} ms`
                : "No Video Delay"}
            </p>
            <p>
              {videoStats.resolution.width > 0
                ? `Resolution: ${videoStats.resolution.width}x${videoStats.resolution.height}`
                : "NO VIDEO"}
            </p>
          </div>
        </div>
      </div>
    </div>
  );
}

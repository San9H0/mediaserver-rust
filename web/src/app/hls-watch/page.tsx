"use client";
import React from "react";
import Hls from "hls.js";

export default function HLSWatch() {
  const videoRef = React.useRef<HTMLVideoElement>(null);
  const streamKeyRef = React.useRef<HTMLInputElement>(null);
  const sessionIdRef = React.useRef<HTMLInputElement>(null);

  const handleStart = () => {
    const sessionId = sessionIdRef.current?.value;
    if (sessionId && videoRef.current) {
      if (Hls.isSupported()) {
        const hls = new Hls({
          lowLatencyMode: true, // LL-HLS 모드 활성화
          liveSyncDuration: 0.5, // 라이브 동기화 지점.
          liveMaxLatencyDuration: 1, // 최대 지연 허용 시간. 최대 n 초까지 떨어질 수 있음.
          enableWorker: true, // 웹 워커 사용
          // debug: true, // 디버그 모드 활성화
          backBufferLength: 30, // 백 버퍼 길이 설정
          maxLiveSyncPlaybackRate: 1.5, // 최대 1.5배속으로 재생
          // progressive: true,
        });
        hls.loadSource(`v1/public/hls/${sessionId}/index.m3u8`);
        hls.attachMedia(videoRef.current);
        hls.on(Hls.Events.MANIFEST_PARSED, () => {
          videoRef.current?.play();
        });
        hls.on(Hls.Events.ERROR, (event, data) => {
          console.log("err:", event, data);
        });
        hls.on(Hls.Events.FRAG_CHANGED, () => {
          // console.log(
          //   "videoElement.buffered:",
          //   videoRef.current?.buffered,
          //   "videoElement.currentTime:",
          //   videoRef.current?.currentTime
          // );
        });
      } else if (
        videoRef.current.canPlayType("application/vnd.apple.mpegurl")
      ) {
        videoRef.current.src = `v1/public/hls/${sessionId}/index.m3u8`;
        videoRef.current.addEventListener("loadedmetadata", () => {
          videoRef.current?.play();
        });
      }

      setInterval(() => {
        if (!videoRef.current) {
          return;
        }
        const buffered = videoRef.current.buffered;
        const currentTime = videoRef.current.currentTime;
        // console.log("buffered:", buffered, "currentTime:", currentTime);
        for (let i = 0; i < buffered.length; i++) {
          if (
            buffered.start(i) <= currentTime &&
            buffered.end(i) >= currentTime
          ) {
            const bufferDelay = (buffered.end(i) - currentTime).toFixed(2);
            console.log("bufferDelay:", bufferDelay);
            // console.log("buffered:", buffered.start(i), buffered.end(i));
          }
        }
      }, 1000);
    }
  };

  const handleFetch = async () => {
    const streamKey = streamKeyRef.current?.value;
    if (!streamKey) {
      console.error("Stream key is not provided.");
      return;
    }

    try {
      const response = await fetch("v1/hls", {
        method: "POST",
        headers: {
          Authorization: `Bearer ${streamKey}`,
        },
      });

      if (!response.ok) {
        console.error("Error sending request to server.", response);
        return;
      }

      const data = await response.json();
      if (sessionIdRef.current) {
        sessionIdRef.current.value = data.sessionId;
      }
      console.log("Request successful.");
    } catch (err) {
      console.error("Error sending request to server.", err);
    }
  };

  return (
    <div className="container">
      <h1 className="text-4xl font-bold text-center text-primary">
        HLS Subscriber
      </h1>

      <div className="flex flex-col space-y-4">
        <div>
          <input
            type="text"
            placeholder="Enter Sesssion ID"
            ref={sessionIdRef}
            className="input input-bordered w-full"
          />
        </div>

        <div className="flex space-x-4">
          <button className="btn btn-outline btn-error" onClick={handleStart}>
            Start!
          </button>
        </div>
      </div>
      <div>
        <input type="text" placeholder="Enter Session ID" ref={sessionIdRef} />
        <button onClick={handleStart}>Start</button>
      </div>
      <div>
        <input type="text" placeholder="Enter Stream Key" ref={streamKeyRef} />
        <button onClick={handleFetch}>Send Fetch Request</button>
      </div>
      <div>
        <video ref={videoRef} controls className="video-js vjs-default-skin" />
      </div>
    </div>
  );
}

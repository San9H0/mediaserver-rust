# MediaServer-Rust

MediaServer-Rust is a media server project developed in pure rust.

This project is a pure rust project (no dependencies)

## Key Features
- **WebRTC Ingress**: Easily ingest media streams using WebRTC, enabling low-latency and real-time communication. Implements WHIP as the WebRTC signaling protocol
- **Multi-Protocol Viewing**:
- **WebRTC Viewing**: Provides real-time streaming with latency under 50ms. Implements WHEP as the WebRTC signaling protocol
- **HLS Viewing**: Supports Low-Latency HLS with streaming latency of 1-2 seconds.

### Supported Codecs
- Video: H.264
- Audio: Opus

### projects
- **m3u8-rs**: Forked from the m3u8-rs repository with the added Low-Latency HLS(LLHLS) feature
- **mp4-rust**: Forked from the mp4-rust repository with the added fragmented MP4 (fmp4) for llhls
- **bitstreams**: is the crate that bitstream functionalites for H.264
- **mediaserver**: is the main media server project
- **web**: A web application designed to test for streaming of media server 

## Installation and Execution

### Clone this repository:
```bash
https://github.com/San9H0/mediaserver-rust.git
cd mediaServer-rust
git submodule init
git submodule update
cargo run -p mediaserver
```

## Usage
### Web
You can run an example demo on the web page.
```
https://github.com/San9H0/mediaserver-rust.git
cd web
npm install && npm dev run
```

## Checklist
- [x] webrtc ingress
- [x] webrtc egress
- [x] ll-hls egress
- [x] remove ffmpeg dependencies

## TODO
- **AV1 Codec**
- **File Output**
- **RTMP Input**
- **Adaptive Bitrate**
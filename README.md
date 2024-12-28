# MediaServer-Rust

MediaServer-Rust is a media server project developed in Rust. 
WebRTC live streaming allows viewing through both WebRTC and HLS protocols.

## Key Features

- **WebRTC Broadcasting**: Start broadcasting using the WebRTC protocol.
- **WebRTC Viewing**: Provides real-time streaming with latency under 50ms.
- **HLS Viewing**: Supports Low-Latency HLS with streaming latency of 1-2 seconds.

## Structure
```
 ingress      hub      egress
            +-----+
            |     |
            |     +--- WebRTC
+-------+   |     |
| WebRTC|---+ Hub +--- HLS
+-------+   |     |
            |     |
            +-----+
```

## Supported Codecs

- Video: VP8, H.264
- Audio: Opus

## Third-Party Tools

- **FFmpeg**: Used as a third-party tool for codec processing and streaming.

## Installation and Execution

1. Clone this repository:
   ```bash
   https://github.com/San9H0/mediaserver-rust.git
   cd MediaServer-Rust
   ```

2. Install dependencies:
   ```bash
   cargo build --release
   ```

3. Run the server:
   ```bash
   ./target/release/mediaserver
   ```

## Usage

### Broadcastring
- **WebRTC**: Provide live streaming using WebRTC and signal protocol uses WHIP.

### Viewing
- **WebRTC**: Provides live viewing using WebRTC, where the signaling protocol is WHEP.
- **HLS**: Provides live viewing using HLS. provides 1~2 seconds of latency using LL-HLS.


## Example

You can run an example demo on the web page.

## TODO
- **Adaptive Bitrate**
- **AV1 Codec**
- **File Output**
- **RTMP Input**
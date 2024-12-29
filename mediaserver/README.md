
# MediaServer

Main Project for MediaServer of MediaServer-Rust 

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
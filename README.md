# webrtc-media

extensible customizable system that broadcasts various media sources (v4l2, rtcp, video) into webrtc streams with ICE trickling

## Running in debug mode

```bash
RUST_LOG=webrtc_loopback=debug cargo run -- --video videos/horse_drawn.264
```
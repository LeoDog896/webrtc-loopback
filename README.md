# webrtc-media

-  stream various media sources into webrtc streams.
- a ground-up rust alternative to [webrtc-streamer](github.com/mpromonet/webrtc-streamer) or [uv4l](https://www.linux-projects.org/uv4l/)'s webrtc streaming.
- all with a single binary, and web-ready out of the box with a served NPM package or even a pre-built UI.

```
webrtc-media v4l2:/dev/video0
```

Targeted (& tested) supports:
- [x] h264 video
- [ ] v4l2 video
- [ ] rtcp video
- [ ] general video -> (ffmpeg) -> h264

## Supported schemas

> note All schemas will be tagged as `video` or `audio` and will be passed to the API. To explicitly specify the type, add `video:` or `audio:` before the schema, e.g. `video:v4l2:/dev/video0`

- `v4l2:` - v4l2 video device, e.g. `v4l2:/dev/video0`
- `file:` - generic file, e.g. `file:videos/horse_drawn.264`
- `rtcp:` - rtcp stream (format to be determined)
- `screen:` - screen capture (format to be determined)
- `window:` - window capture (format to be determined)

## Default UI

The default UI is a simple web page that displays all the streams. It can be accessed at `http://localhost:8080`.

## Glob & Watching

In order to apply `webrtc-media` to multiple devices, you can use glob patterns. For example, to apply `webrtc-media` to all video devices, you can use `v4l2:/dev/video*`.

## Running in debug mode

```bash
RUST_LOG=webrtc_loopback=debug cargo run -- --video videos/horse_drawn.264
```
# ELK Recorder v2

This is a reasonably simple tool built to run on a raspberry pi 2 and record
live music at parties. It's intended to be plugged on the main stage console.

The goal is to have a self-service tool for artists to record their performance.
Before they start their sets, they can plug an USB key into the raspberry pi and
it'll start pushing WebM/Opus encoded data onto the key.

This is a new, simpler design of the elk-recorder. There's a rust backend with
records audio, encode it and stream it on files. It's controlled via a
minimalist GRPC API.

On the other side, there's a small python daemon which uses DBus to receive
events when new USB drives are mounted and requests recording.

## Requirements

- rust/cargo
- python/uv
- udisk
- `apt install protobuf-compiler libopus0 libopusenc0 libcairo2-dev libasound2`
  - I'm probably missing a few because python-dbus has a million deps
  - You'll also need either `libgirepository1.0-dev` or `libgirepository-2.0-dev`

## Usage

- First, you need to run the `engine`
  - `cd engine && cargo run --bin serve --audio-input pulse`
  - You can find the known audio inputs using `cargo run --bin list`
- Then you need to start the manager
  - ```bash
  cd manager
  uv run python -m grpc_tools.protoc -I ../engine/proto/ --python_out=proto --grpc_python_out=proto elkr.proto
  uv run main.py watch
  ```
- Plug an USB key and mount somewhere (your system might be doing so automatically)

### Post processing

Since the stuff is operated in a very brutal manner by pluging USB sticks in and
out, it's likely that the last segment of the file will be corrupted. Also, we
limit buffering and concatenate a lot of webm segments into a single file.

I think this is supported by the spec, but a lot of players won't like it, so it's advised to post-process the file using `ffmpeg` or something similar.

`ffmpeg -i /media/stick/elkr/elkr_0001.weba /tmp/elkr_0001.flac`

You might receive a lot of complaints from the demuxer, but you *should*
hopefully be fine

## Cross compile for raspi 2b

The engine is cross compiled for the raspbi 2b using

`cd engine`
`cross build --target armv7-unknown-linux-gnueabihf --release`

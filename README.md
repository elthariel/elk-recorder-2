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

## Cross compile for raspi 2b

The engine is cross compiled for the raspbi 2b using

`cd engine`
`cross build --target armv7-unknown-linux-gnueabihf --release`

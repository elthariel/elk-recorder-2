// use tonic::transport::Server;

pub use engine::service::{ElkrServiceServer, MyService};
pub use engine::{audio, config, encoder, sink};

// #[tokio::main]
// async fn main() -> Result<(), Box<dyn std::error::Error>> {
//     let addr = "[::1]:50051".parse()?;
//     let audio = Audio::new();
//     let svc = MyService::default();

//     println!("Starting server, listening to '{}'", addr);
//     Server::builder()
//         .add_service(ElkrServiceServer::new(svc))
//         .serve(addr)
//         .await?;

//     Ok(())
// }

use anyhow::Result;
use std::{path::Path, thread, time};
use tokio::sync::oneshot;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let test_path = Path::new("/tmp/elkr.weba");

    let (enc_cmd_sender, enc_cmd_receiver) = encoder::Command::channel();
    let (sink_exit_tx, sink_exit_rx) = oneshot::channel::<()>();
    let (audio_thread, audio_receiver) =
        audio::Audio::new(String::from("pipewire"), config::SAMPLE_RATE)?;
    let enc = encoder::Encoder::new(enc_cmd_receiver, audio_receiver);
    let (sink, sink_sender) = sink::Sink::new(sink_exit_rx, test_path)?;

    enc_cmd_sender.send(encoder::Command::Add(test_path.to_path_buf(), sink_sender));

    let _stream = audio_thread.start();
    enc.start();
    sink.start();

    let sleep_duration = time::Duration::from_millis(2_000);
    let wait_duration = time::Duration::from_millis(100);
    thread::sleep(sleep_duration);
    let _ = enc_cmd_sender.send(encoder::Command::Exit);
    thread::sleep(wait_duration);

    Ok(())
}

use anyhow::Result;
use clap::Parser;
use tonic::transport::Server;

use engine::{audio, encoder, proto::elkr_service_server::ElkrServiceServer, service};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value_t = String::from("[::1]:50051"))]
    bind: String,

    #[arg(short, long, default_value_t = String::from("pipewire"))]
    audio_input: String,

    #[arg(short, long, default_value_t = 48_000)]
    sample_rate: u32,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let addr = args.bind.parse()?;
    let (audio_engine, audio_data_receiver) =
        audio::Audio::new(args.audio_input, args.sample_rate)?;
    let (encoder, encoder_cmd_sender) = encoder::Encoder::new(audio_data_receiver);
    let svc = service::Controller::new(encoder_cmd_sender);

    let _stream = audio_engine.start();
    encoder.start();

    println!("Starting control server, listening to '{}'", args.bind);
    Server::builder()
        .add_service(ElkrServiceServer::new(svc))
        .serve(addr)
        .await?;

    Ok(())
}

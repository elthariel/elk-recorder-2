use std::{collections::HashMap, path::PathBuf, sync::mpsc, thread, time, vec::Vec};

use anyhow::{anyhow, Result};
use lockfree;
use opus;

use crate::{audio::AudioReceiver, config, sink::SinkSender};

pub enum Command {
    Exit,
    Add(PathBuf, SinkSender),
    Remove(PathBuf),
}

pub type CommandSender = mpsc::Sender<Command>;
type CommandReceiver = mpsc::Receiver<Command>;

pub struct Encoder {
    cmd: CommandReceiver,
    audio_in: AudioReceiver,

    encoder: opus::Encoder,
    // Opus can only encode fixed-length audio chunks, so we need a buffer to store data
    // that's not been encoded yet
    input_buffer: Vec<f32>,
    packets_count: u64,

    // A list of sink thread we're pushing data to
    sinks: HashMap<PathBuf, SinkSender>,
}

impl Command {
    pub fn channel() -> (CommandSender, CommandReceiver) {
        mpsc::channel::<Command>()
    }
}

impl Encoder {
    pub fn new(audio_in: lockfree::channel::spsc::Receiver<Vec<f32>>) -> (Encoder, CommandSender) {
        let (cmd_tx, cmd_rx) = Command::channel();

        println!("Creating Opus encoder (frame = {})", config::FRAME_SAMPLES);
        let mut opus_encoder =
            opus::Encoder::new(48000_u32, opus::Channels::Stereo, opus::Application::Audio)
                .expect("Unable to create encoder Opus");
        let _ = opus_encoder.set_bitrate(opus::Bitrate::Max);

        let encoder = Encoder {
            cmd: cmd_rx,
            audio_in,
            encoder: opus_encoder,
            input_buffer: Vec::<f32>::with_capacity(config::FRAME_SAMPLES),
            packets_count: 0,
            sinks: HashMap::new(),
        };

        (encoder, cmd_tx)
    }

    pub fn start(self) {
        thread::spawn(move || self.main_loop());
    }

    fn main_loop(mut self) {
        let ms = time::Duration::from_millis(1);

        loop {
            match self.cmd.try_recv() {
                Ok(command) => {
                    if let Err(_) = self.handle_command(command) {
                        println!("Exiting encoder thread... Goodbye !");
                        return;
                    }
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    println!("Control disconnected, exiting encoder thread... Goodbye !");
                    return;
                }
                Err(mpsc::TryRecvError::Empty) => {}
            };

            if let Ok(data) = self.audio_in.recv() {
                match self.handle_audio(data) {
                    Ok(_) => {}
                    Err(e) => {
                        println!("Error in handle_audio: {:?}", e);
                    }
                };
            } else {
                thread::sleep(ms);
            }
        }
    }

    fn handle_command(&mut self, cmd: Command) -> Result<()> {
        match cmd {
            Command::Exit => {
                return Err(anyhow!("Exit requested"));
            }
            Command::Add(path, sender) => {
                println!("Adding sink for {}", path.as_path().to_string_lossy());
                if let Some(_old) = self.sinks.insert(path.clone(), sender) {
                    println!(
                        "WARN: Overwriting sink for {}",
                        path.as_path().to_string_lossy()
                    );
                }
            }
            Command::Remove(path) => {
                println!("Removing sink for {}", path.as_path().to_string_lossy());
                if self.sinks.remove(&path).is_none() {
                    println!("WARN: No sink for {}", path.as_path().to_string_lossy());
                }
            }
        }

        Ok(())
    }

    fn handle_audio(&mut self, data: Vec<f32>) -> Result<()> {
        // println!("Received data from lockfree channel. {}", data.len());

        self.input_buffer.extend_from_slice(data.as_slice());

        while self.input_buffer.len() >= config::FRAME_SAMPLES {
            let frame: Vec<f32> = self.input_buffer.drain(..config::FRAME_SAMPLES).collect();
            let result = self
                .encoder
                .encode_vec_float(frame.as_slice(), config::OPUS_MAX_PACKET_SIZE);

            match result {
                Ok(encoded) => {
                    // println!("Encoded frame ({} bytes)", encoded.len());
                    let _ = self.handle_packet(encoded)?;
                }
                Err(err) => {
                    println!("Encoding error {:?}", err);
                }
            }
        }

        Ok(())
    }

    // A chunk of audio was encoded into a packet, let's push it to the sinks
    fn handle_packet(&mut self, packet: Vec<u8>) -> Result<()> {
        self.packets_count += 1;

        for (_path, sink) in self.sinks.iter() {
            // println!("Sending packet !");
            sink.send(packet.clone())?;
        }

        if self.packets_count % 1000 == 0 {
            println!("Encoded {}k opus packets", self.packets_count / 1000);
        }

        Ok(())
    }
}

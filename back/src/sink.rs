use std::{
    borrow::Cow,
    fs,
    path::{Path, PathBuf},
    sync::mpsc,
    thread, time,
    io::Write,
};

use anyhow::{anyhow, Result};
use tokio::sync::oneshot;
use webm::mux;

use crate::config;

pub type SinkSender = mpsc::Sender<Vec<u8>>;
type SinkReceiver = mpsc::Receiver<Vec<u8>>;

pub struct Sink {
    exit: oneshot::Receiver<()>,
    receiver: SinkReceiver,
    path: PathBuf,
    packets_count: u64,
    fd: Option<fs::File>,
    segment: Option<mux::Segment<fs::File>>,
    track_id: Option<mux::AudioTrack>,
}

impl Sink {
    pub fn new(exit: oneshot::Receiver<()>, path: &Path) -> Result<(Sink, SinkSender)> {
        let (sender, receiver) = mpsc::channel::<Vec<u8>>();
        let fd = fs::File::create(path)?;
        // XXX: Use this one to avoid overwriting
        // let fd = fs::File::create_new(path)?;

        let sink = Sink {
            path: path.to_path_buf(),
            receiver,
            exit,
            fd: Some(fd),
            segment: None,
            track_id: None,
            packets_count: 0,
        };

        Ok((sink, sender))
    }

    pub fn path_str(&self) -> Cow<'_, str> {
        self.path.to_string_lossy()
    }
    pub fn start(mut self) {
        thread::spawn(move || self.thread());
    }

    fn thread(&mut self) {
        println!("Started sink thread for {}", self.path_str());
        self.main_loop();
        println!("Exiting sink thread for {}", self.path_str());

        if let Some(segment) = self.segment.take() {
            let duration = (self.packets_count + 1) * config::FRAME_TIME_MS * 1_000_000;
            let writer = match segment.finalize(Some(duration)) {
                Ok(writer) => writer,
                Err(writer) => {
                    println!("Error finalizing {}", self.path_str());
                    writer
                }
            };

            let mut fd = writer.into_inner();
            if let Err(err) = fd.flush() {
                println!("Error flushing file for {}: {:?}", self.path_str(), err);
            }
        }
    }

    fn build_segment(&mut self) -> Result<()> {
        let writer = if let Some(segment) = self.segment.take() {
            match segment.finalize(None) {
                Ok(writer) => writer,
                Err(writer) => {
                    println!("Error finalizing webm for {}", self.path_str());
                    writer
                }
            }
        } else {
            mux::Writer::new(self.fd.take().unwrap())
        };

        println!("new builder");
        let builder = mux::SegmentBuilder::new(writer)?;
        println!("set app");
        let builder = builder.set_writing_app("elk_recorder")?;
        println!("add track");
        let (builder, track_id) = builder.add_audio_track(
            config::SAMPLE_RATE,
            config::CHANNELS as u32,
            mux::AudioCodecId::Opus,
            Some(23),
        )?;

        self.segment = Some(builder.build());
        self.track_id = Some(track_id);

        Ok(())
    }

    fn main_loop(&mut self) {
        let timeout = time::Duration::from_millis(500);

        loop {
            if let Ok(_) = self.exit.try_recv() {
                return;
            }

            if self.packets_count % config::FRAMES_PER_SEGMENT == 0 {
                if let Err(e) = self.build_segment() {
                    println!("Unable to build segment: {:?}", e);
                }
            }

            let result = self.receiver.recv_timeout(timeout);
            println!("Received data in sink");
            match result {
                Ok(data) => {
                    if let Err(e) = self.handle_data(data) {
                        println!("Error writing on {} : {:?}", self.path_str(), e);
                    }
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    return;
                }
                Err(_) => (),
            }
        }
    }

    fn handle_data(&mut self, data: Vec<u8>) -> Result<()> {
        if let Some(mut segment) = self.segment.take() {
            let timestamp = (self.packets_count as u64) * config::FRAME_TIME_MS * 1_000_000_u64;
            let result = segment.add_frame(self.track_id.unwrap(), data.as_slice(), timestamp, true);
            self.packets_count += 1;
            self.segment = Some(segment);

            match result {
                Ok(_) => Ok(()),
                Err(e) => Err(anyhow!(e)),
            }
        } else {
            Err(anyhow!("Uninitialized segment for {}", self.path_str()))
        }
    }
}

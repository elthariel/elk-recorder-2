use std::{
    borrow::Cow,
    fs,
    io::Write,
    path::{Path, PathBuf},
    sync::mpsc,
    thread, time,
};

use anyhow::{anyhow, Result};
use tokio::sync::oneshot;
use webm::mux;

use crate::config;

pub type SinkSender = mpsc::Sender<Vec<u8>>;
type SinkReceiver = mpsc::Receiver<Vec<u8>>;
pub type SinkExitSender = oneshot::Sender<()>;
type SinkExitReceiver = oneshot::Receiver<()>;

pub struct Sink {
    exit: SinkExitReceiver,
    receiver: SinkReceiver,
    path: PathBuf,
    packets_count: u64,
    fd: Option<fs::File>,
    segment: Option<mux::Segment<fs::File>>,
    track_id: Option<mux::AudioTrack>,
    segments_count: u64,
}

impl Sink {
    pub fn new(path: &Path) -> Result<(Sink, SinkSender, SinkExitSender)> {
        let (sender, receiver) = mpsc::channel::<Vec<u8>>();
        let (exit_sender, exit_receiver) = oneshot::channel::<()>();

        let fd = fs::File::create(path)?;
        // XXX: Use this one to avoid overwriting
        // let fd = fs::File::create_new(path)?;

        let sink = Sink {
            path: path.to_path_buf(),
            receiver,
            exit: exit_receiver,
            fd: Some(fd),
            segment: None,
            track_id: None,
            packets_count: 0,
            segments_count: 0,
        };

        Ok((sink, sender, exit_sender))
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
            println!("Initializing writer for {}", self.path_str());
            mux::Writer::new(self.fd.take().unwrap())
        };

        let builder = mux::SegmentBuilder::new(writer)?;
        let builder = builder.set_writing_app("elk_recorder")?;
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
                self.segments_count += 1;

                if let Err(e) = self.build_segment() {
                    println!("Unable to build segment: {:?}", e);
                } else {
                    println!(
                        "Built segment {} for {}",
                        self.segments_count,
                        self.path_str()
                    );
                }
            }

            let result = self.receiver.recv_timeout(timeout);
            match result {
                Ok(data) => {
                    if let Err(e) = self.handle_data(data) {
                        println!("Error writing on {} : {:?}", self.path_str(), e);
                    }
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    return;
                }
                Err(e) => {
                    println!("Error for sink {} : {:?}", self.path_str(), e);
                },
            }
        }
    }

    fn handle_data(&mut self, data: Vec<u8>) -> Result<()> {
        if let Some(mut segment) = self.segment.take() {
            let timestamp = (self.packets_count as u64) * config::FRAME_TIME_MS * 1_000_000_u64;
            let result =
                segment.add_frame(self.track_id.unwrap(), data.as_slice(), timestamp, true);
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

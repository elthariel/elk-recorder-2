pub const CHANNELS: u16 = 2;
pub const SAMPLE_RATE: u32 = 48_000;
pub const FRAMES_PER_SEGMENT: u64 = 256;

pub const FRAME_TIME_MS: u64 = 100;
pub const FRAME_TIME_S: f32 = (100 as f32) / 1000_f32;
pub const FRAME_SAMPLES_PER_CHANNEL: usize = ((SAMPLE_RATE as f32) * FRAME_TIME_S) as usize;
pub const FRAME_SAMPLES: usize = FRAME_SAMPLES_PER_CHANNEL * (CHANNELS as usize);
pub const OPUS_MAX_PACKET_SIZE: usize = 16 * 1024;

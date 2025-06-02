use anyhow::{anyhow, Result};
use cpal::{
    traits::{DeviceTrait, HostTrait},
    Device, Host, SampleFormat, SampleRate, Stream, SupportedStreamConfig,
};
use lockfree;

use crate::config;

type AudioSender = lockfree::channel::spsc::Sender<Vec<f32>>;
pub type AudioReceiver = lockfree::channel::spsc::Receiver<Vec<f32>>;

pub struct Audio {
    audio_out: AudioSender,
    device: Device,
    config: SupportedStreamConfig,
}

fn get_input_device(host: Host, name: String) -> Result<Device> {
    for device in host.input_devices()? {
        if let Ok(current_name) = device.name() {
            if current_name == name {
                return Ok(device);
            }
        }
    }
    return Err(anyhow!("Unable to find input device: {}", name));
}

fn get_input_device_config(
    device: &Device,
    rate: SampleRate,
    fmt: SampleFormat,
) -> Result<SupportedStreamConfig> {
    let supported_configs_range = device
        .supported_output_configs()
        .expect("error while querying configs");

    for config in supported_configs_range {
        if config.channels() == config::CHANNELS && config.sample_format() == fmt {
            return Ok(config.with_sample_rate(rate));
        }
    }

    return Err(anyhow!("No suitable supported config found!"));
}

impl Audio {
    pub fn new(device_name: String, sample_rate: u32) -> Result<(Audio, AudioReceiver)> {
        let host = cpal::default_host();
        println!("Cpal host: {}", host.id().name());

        let device = get_input_device(host, device_name)?;
        println!("Device {} found !", device.name().unwrap());

        let config = get_input_device_config(&device, SampleRate(sample_rate), SampleFormat::F32)
            .expect("Unable to find suitable supported audio config");
        println!(
            "Audio configuration: channels: {}, sample rate: {}, format: {}",
            config.channels(),
            config.sample_rate().0,
            config.sample_format()
        );

        let (audio_out, audio_in) = lockfree::channel::spsc::create::<Vec<f32>>();

        let audio = Audio {
            audio_out,
            device,
            config,
        };

        Ok((audio, audio_in))
    }

    pub fn start(mut self) -> Result<Stream> {
        let stream = self.device.build_input_stream(
            &self.config.config(),
            move |data: &[f32], _info: &cpal::InputCallbackInfo| {
                // react to stream events and read or write stream data here.
                // println!("Audio thread data {:?}", _info);
                if let Err(_err) = self.audio_out.send(data.to_vec()) {
                    println!("Unable to send audio data to encoder input chan");
                }
            },
            move |err| {
                println!("Audio thread received error {:?}", err);
            },
            None, // None=blocking, Some(Duration)=timeout
        )?;

        Ok(stream)
    }
}

use anyhow::Result;

use cpal::{
    traits::{DeviceTrait, HostTrait},
    Host,
};

fn list_devices(host: &Host) -> Result<Vec<String>> {
    let mut devices = Vec::<String>::new();

    for device in host.input_devices()? {
        if let Ok(name) = device.name() {
            devices.push(name);
        }
    }

    Ok(devices)
}

fn main() -> Result<()> {
    let host = cpal::default_host();

    let devices = list_devices(&host)?;
    for name in devices {
        println!("{}", name);
    }

    Ok(())
}

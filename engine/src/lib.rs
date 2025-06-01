pub mod audio;
pub mod config;
pub mod encoder;
pub mod sink;

pub mod proto {
    tonic::include_proto!("elkr");
}
pub mod service;

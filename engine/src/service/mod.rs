use std::{collections::HashMap, path::PathBuf, sync::Mutex};

use anyhow::Result;
use tonic::{Request, Response, Status};

use crate::{
    encoder,
    proto::{self, elkr_service_server::ElkrService},
    sink,
};

type SinkMap = HashMap<PathBuf, sink::SinkExitSender>;

#[derive(Debug)]
pub struct Controller {
    encoder_cmd: encoder::CommandSender,
    sinks: Mutex<SinkMap>,
}

impl Controller {
    pub fn new(encoder_cmd: encoder::CommandSender) -> Controller {
        Controller {
            encoder_cmd,
            sinks: Mutex::new(SinkMap::new()),
        }
    }
}

#[tonic::async_trait]
impl ElkrService for Controller {
    async fn add_sink(
        &self,
        request: Request<proto::AddSinkRequest>,
    ) -> Result<Response<proto::AddSinkResponse>, Status> {
        let response = proto::AddSinkResponse {
            code: proto::Code::Ok as i32,
        };
        println!("add_sink: {:?}", request);
        let path = PathBuf::from(request.into_inner().path);

        match path.try_exists() {
            Err(e) => {
                println!(
                    "Unable to check for file {} existence: {e:?}",
                    path.display()
                );
                return Ok(Response::new(proto::AddSinkResponse {
                    code: proto::Code::ServerError as i32,
                }));
            }
            Ok(true) => {
                println!("File already exists: {}", path.display());
                return Ok(Response::new(proto::AddSinkResponse {
                    code: proto::Code::ClientError as i32,
                }));
            }
            Ok(false) => {}
        }

        match sink::Sink::new(path.clone().into_boxed_path().as_ref()) {
            Ok((sink, sink_sender, sink_exit_sender)) => {
                let mut sinks = self.sinks.lock().unwrap();
                let cmd = encoder::Command::Add(path.clone(), sink_sender);

                self.encoder_cmd.send(cmd).unwrap();
                sinks.insert(path, sink_exit_sender);
                sink.start();

                Ok(Response::new(response))
            }
            Err(e) => {
                println!("Error starting sink for {}, {e:?}", path.display());
                Ok(Response::new(proto::AddSinkResponse {
                    code: proto::Code::ClientError as i32,
                }))
            }
        }
    }

    async fn remove_sink(
        &self,
        request: Request<proto::RemoveSinkRequest>,
    ) -> Result<Response<proto::RemoveSinkResponse>, Status> {
        println!("remove_sink: {:?}", request);
        let path = PathBuf::from(request.into_inner().path);
        let mut sinks = self.sinks.lock().unwrap();

        match sinks.remove(&path) {
            Some(exit_sender) => {
                let cmd = encoder::Command::Remove(path.clone());

                self.encoder_cmd.send(cmd).unwrap();
                exit_sender.send(()).unwrap();
                Ok(Response::new(proto::RemoveSinkResponse {
                    code: proto::Code::Ok as i32,
                }))
            }
            None => {
                println!("No sink for {}", path.display());
                Ok(Response::new(proto::RemoveSinkResponse {
                    code: proto::Code::ClientError as i32,
                }))
            }
        }
    }

    async fn list_sinks(
        &self,
        _request: Request<proto::ListSinksRequest>,
    ) -> Result<Response<proto::ListSinksResponse>, Status> {
        let sinks = self.sinks.lock().unwrap();
        let mut result = Vec::<String>::new();

        for (path, _val) in sinks.iter() {
            result.push(String::from(path.as_path().to_string_lossy()));
        }

        let response = proto::ListSinksResponse {
            code: proto::Code::Ok as i32,
            sinks: result,
        };

        Ok(Response::new(response))
    }
}

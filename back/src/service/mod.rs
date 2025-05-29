use tonic::{transport::Server, Request, Response, Status};

pub mod elkr_proto {
    tonic::include_proto!("elkr");
}

pub use elkr_proto::elkr_service_server::{ElkrService, ElkrServiceServer};
use elkr_proto::{HelloReply, HelloRequest};

#[derive(Debug, Default)]
pub struct MyService {}

#[tonic::async_trait]
impl ElkrService for MyService {
    async fn say_hello(
        &self,
        request: Request<HelloRequest>, // Accept request of type HelloRequest
    ) -> Result<Response<HelloReply>, Status> {
        // Return an instance of type HelloReply
        println!("Got a request: {:?}", request);

        let reply = HelloReply {
            // We must use .into_inner() as the fields of gRPC requests and responses are private
            message: format!("Hello {}!", request.into_inner().name),
        };

        Ok(Response::new(reply)) // Send back our formatted greeting
    }
}

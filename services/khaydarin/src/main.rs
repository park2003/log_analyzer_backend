use std::net::SocketAddr;
use tracing::info;

// Module declarations
mod domain;
mod application;
mod infrastructure;
mod presentation;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // TODO: Initialize dependencies from infrastructure layer
    // let db_pool = infrastructure::database::connect().await?;
    // let job_repository = infrastructure::database::PostgresJobRepository::new(db_pool);
    // let use_cases = application::use_cases::UseCases::new(job_repository);
    // let grpc_service = presentation::grpc_services::KhaydarinGrpcServer::new(use_cases);

    let addr = "[::1]:50051".parse::<SocketAddr>()?;
    info!("gRPC server listening on {}", addr);

    // Placeholder for tonic server
    // tonic::transport::Server::builder()
    //     .add_service(grpc_service)
    //     .serve(addr)
    //     .await?;

    Ok(())
}
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
    // let curation_repository = infrastructure::database::PostgresCurationRepository::new(db_pool);
    // let use_cases = application::use_cases::CurationUseCases::new(curation_repository);
    // let grpc_service = presentation::grpc_services::DataCuratorGrpcServer::new(use_cases);

    let addr = "[::1]:50052".parse::<SocketAddr>()?;
    info!("Data Curator gRPC server listening on {}", addr);

    // Placeholder for tonic server
    // tonic::transport::Server::builder()
    //     .add_service(grpc_service)
    //     .serve(addr)
    //     .await?;

    Ok(())
}
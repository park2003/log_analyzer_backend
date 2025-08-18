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
    // let workflow_repository = infrastructure::database::PostgresWorkflowRepository::new(db_pool);
    // let flyte_client = infrastructure::flyte_client::FlyteClient::new();
    // let use_cases = application::use_cases::WorkflowUseCases::new(workflow_repository, flyte_client);
    // let grpc_service = presentation::grpc_services::FlyteIntegrationGrpcServer::new(use_cases);

    let addr = "[::1]:50053".parse::<SocketAddr>()?;
    info!("Flyte Integration gRPC server listening on {}", addr);

    // Placeholder for tonic server
    // tonic::transport::Server::builder()
    //     .add_service(grpc_service)
    //     .serve(addr)
    //     .await?;

    Ok(())
}
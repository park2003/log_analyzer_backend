use std::net::SocketAddr;
use std::sync::Arc;
use tonic::transport::Server;
use tracing::info;

// Module declarations
mod application;
mod domain;
mod infrastructure;
mod presentation;

use application::use_cases::WorkflowUseCases;
use infrastructure::{
    database::PostgresExecutionRepository,
    flyte_client::{FlyteConfig, HttpFlyteClient},
};
use presentation::grpc_services::{
    ExecutionServiceImpl, savassan::v1::execution_service_server::ExecutionServiceServer,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .with_thread_ids(true)
        .with_thread_names(true)
        .init();

    info!("Starting Flyte Integration Service");

    // Load configuration from environment
    dotenvy::dotenv().ok(); // Load .env file if present

    // Database configuration
    let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgresql://postgres:password@localhost/flyte_integration".to_string()
    });

    info!("Connecting to database...");
    let db_pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    // Initialize repository and run migrations
    let repository = Arc::new(PostgresExecutionRepository::new(db_pool));
    repository.migrate().await?;
    info!("Database migrations completed");

    // Initialize Flyte client
    let flyte_config = FlyteConfig::from_env();
    info!("Flyte Admin URL: {}", flyte_config.admin_url);
    info!("Flyte Project: {}", flyte_config.project);
    info!("Flyte Domain: {}", flyte_config.domain);

    let flyte_client = Arc::new(HttpFlyteClient::new(
        flyte_config.admin_url,
        flyte_config.project,
        flyte_config.domain,
    ));

    // Initialize use cases
    let use_cases = Arc::new(WorkflowUseCases::new(
        repository.clone(),
        flyte_client.clone(),
    ));

    // Initialize gRPC service
    let grpc_service = ExecutionServiceImpl::new(use_cases.clone());
    let grpc_server = ExecutionServiceServer::new(grpc_service);

    // Start background task for monitoring active executions
    let monitor_use_cases = use_cases.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(30));
        loop {
            interval.tick().await;
            if let Err(e) = monitor_use_cases.monitor_active_executions().await {
                tracing::error!("Failed to monitor active executions: {}", e);
            } else {
                tracing::debug!("Monitored active executions");
            }
        }
    });

    // Configure and start gRPC server
    let addr = std::env::var("GRPC_ADDR")
        .unwrap_or_else(|_| "[::1]:50051".to_string())
        .parse::<SocketAddr>()?;

    info!("gRPC server listening on {}", addr);

    Server::builder()
        .add_service(grpc_server)
        .serve(addr)
        .await?;

    Ok(())
}

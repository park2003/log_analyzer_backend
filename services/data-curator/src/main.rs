use std::net::SocketAddr;
use std::sync::Arc;
use tracing::{info, error};
use tonic::transport::Server;
use opentelemetry::global;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{propagation::TraceContextPropagator, trace as sdktrace, Resource};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

// Module declarations
mod domain;
mod application;
mod infrastructure;
mod presentation;

use crate::{
    infrastructure::{
        database::{PostgresCurationJobRepository, PgVectorEmbeddingRepository},
        storage::{S3StorageService, LocalStorageService},
        embeddings::{ClipEmbeddingService, MockEmbeddingService},
    },
    presentation::grpc_services::{DataCuratorGrpcService, data_curator_proto::data_curator_service_server::DataCuratorServiceServer},
};

fn init_tracing() -> Result<sdktrace::Tracer, opentelemetry::trace::TraceError> {
    let otlp_endpoint = std::env::var("OTLP_ENDPOINT")
        .unwrap_or_else(|_| "http://localhost:4317".to_string());
    
    opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint(otlp_endpoint),
        )
        .with_trace_config(sdktrace::config().with_resource(Resource::new(vec![
            opentelemetry::KeyValue::new(
                "service.name",
                "data-curator",
            ),
        ])))
        .install_batch(opentelemetry_sdk::runtime::Tokio)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables
    dotenvy::dotenv().ok();
    
    // Initialize OpenTelemetry
    global::set_text_map_propagator(TraceContextPropagator::new());
    
    let tracer = init_tracing().expect("Failed to initialize tracing");
    
    // Initialize tracing subscriber
    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);
    tracing_subscriber::registry()
        .with(telemetry)
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer())
        .init();
    
    info!("Starting Data Curator service");
    
    // Initialize database connection
    let db_pool = infrastructure::database::connect().await?;
    
    // Setup database tables (in production, use migrations)
    if std::env::var("RUN_MIGRATIONS").unwrap_or_else(|_| "false".to_string()) == "true" {
        infrastructure::database::setup_database(&db_pool).await?;
        info!("Database setup completed");
    }
    
    // Initialize repositories
    let job_repository = Arc::new(PostgresCurationJobRepository::new(db_pool.clone()));
    let embedding_repository = Arc::new(PgVectorEmbeddingRepository::new(db_pool.clone()));
    
    // Initialize storage service
    let storage_service: Arc<dyn application::StorageService> = if let Ok(bucket) = std::env::var("S3_BUCKET") {
        info!("Using S3 storage with bucket: {}", bucket);
        Arc::new(S3StorageService::new(bucket).await?)
    } else {
        info!("Using local storage for development");
        let base_path = std::env::var("LOCAL_STORAGE_PATH")
            .unwrap_or_else(|_| "/tmp/data-curator".to_string());
        Arc::new(LocalStorageService::new(base_path.into()))
    };
    
    // Initialize embedding service
    let embedding_service: Arc<dyn application::EmbeddingService> = if let Ok(model_path) = std::env::var("CLIP_MODEL_PATH") {
        info!("Using CLIP model from: {}", model_path);
        Arc::new(ClipEmbeddingService::new(&model_path).await?)
    } else {
        info!("Using mock embedding service for development");
        Arc::new(MockEmbeddingService::new())
    };
    
    // Create gRPC service
    let grpc_service = DataCuratorGrpcService::new(
        job_repository,
        embedding_repository,
        storage_service,
        embedding_service,
    );
    
    let grpc_server = grpc_service.into_server();
    
    // Configure server address
    let addr = std::env::var("GRPC_ADDR")
        .unwrap_or_else(|_| "[::1]:50052".to_string())
        .parse::<SocketAddr>()?;
    
    info!("Data Curator gRPC server listening on {}", addr);
    
    // Configure health check service
    let (mut health_reporter, health_service) = tonic_health::server::health_reporter();
    health_reporter
        .set_serving::<DataCuratorServiceServer<_>>()
        .await;
    
    // Start gRPC server with graceful shutdown
    let server = Server::builder()
        .add_service(health_service)
        .add_service(grpc_server)
        .serve_with_shutdown(addr, shutdown_signal());
    
    // Handle server result
    match server.await {
        Ok(_) => {
            info!("Server shut down gracefully");
        }
        Err(e) => {
            error!("Server error: {}", e);
            return Err(e.into());
        }
    }
    
    // Shutdown OpenTelemetry
    global::shutdown_tracer_provider();
    
    Ok(())
}

async fn shutdown_signal() {
    // Wait for Ctrl+C signal
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to install CTRL+C signal handler");
    
    info!("Received shutdown signal");
}
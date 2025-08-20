use std::net::SocketAddr;
use std::sync::Arc;
use tracing::{info, error};
use tonic::transport::Server;
use opentelemetry::{global, KeyValue, trace::TracerProvider as _};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{
    propagation::TraceContextPropagator, 
    trace::{Config, Tracer, SdkTracerProvider},
    runtime,
    Resource
};
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

// Wrapper types to bridge trait objects with generic parameters
struct StorageServiceWrapper(Arc<dyn application::StorageService>);
struct EmbeddingServiceWrapper(Arc<dyn application::EmbeddingService>);

#[async_trait::async_trait]
impl application::StorageService for StorageServiceWrapper {
    async fn list_images(&self, uri: &str) -> anyhow::Result<Vec<String>> {
        self.0.list_images(uri).await
    }
    
    async fn download_image(&self, uri: &str) -> anyhow::Result<Vec<u8>> {
        self.0.download_image(uri).await
    }
    
    async fn upload_dataset(&self, data: Vec<String>, uri: &str) -> anyhow::Result<()> {
        self.0.upload_dataset(data, uri).await
    }
}

#[async_trait::async_trait]
impl application::EmbeddingService for EmbeddingServiceWrapper {
    async fn generate_embedding(&self, image_data: &[u8]) -> anyhow::Result<Vec<f32>> {
        self.0.generate_embedding(image_data).await
    }
}

fn init_tracing() -> Result<Tracer, Box<dyn std::error::Error>> {
    use opentelemetry_otlp::SpanExporter;
    
    let otlp_endpoint = std::env::var("OTLP_ENDPOINT")
        .unwrap_or_else(|_| "http://localhost:4317".to_string());
    
    let exporter = SpanExporter::builder()
        .with_tonic()
        .with_endpoint(otlp_endpoint)
        .build()?;
    
    let provider = SdkTracerProvider::builder()
        .with_simple_exporter(exporter)
        .build();
    
    let tracer = provider.tracer("data-curator");
    global::set_tracer_provider(provider);
    
    Ok(tracer)
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
    
    // Create gRPC service with wrapper types
    let storage_wrapper = Arc::new(StorageServiceWrapper(storage_service));
    let embedding_wrapper = Arc::new(EmbeddingServiceWrapper(embedding_service));
    
    let grpc_service = DataCuratorGrpcService::new(
        job_repository,
        embedding_repository,
        storage_wrapper,
        embedding_wrapper,
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
        .set_serving::<DataCuratorServiceServer<DataCuratorGrpcService<
            PostgresCurationJobRepository,
            PgVectorEmbeddingRepository,
            StorageServiceWrapper,
            EmbeddingServiceWrapper
        >>>()
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
    
    // Shutdown OpenTelemetry - function no longer exists in newer versions
    // Tracer provider will be dropped automatically
    
    Ok(())
}

async fn shutdown_signal() {
    // Wait for Ctrl+C signal
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to install CTRL+C signal handler");
    
    info!("Received shutdown signal");
}
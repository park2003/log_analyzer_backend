use axum::{Router, routing::get};
use opentelemetry::global;
use opentelemetry::trace::TracerProvider;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{Resource, propagation::TraceContextPropagator, trace::SdkTracerProvider};
use std::net::SocketAddr;
use tonic::transport::Server;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

// Module declarations
mod application;
mod domain;
mod infrastructure;
mod presentation;

// Use statements for our modules
use crate::application::llm_chain::JsonOutputParser;
use crate::application::use_cases::ProcessPromptUseCase;
use crate::infrastructure::database::PostgresKhaydarinRepository;
use crate::infrastructure::external_apis::LlmApiClient;
use crate::presentation::grpc_services::{
    KhaydarinGrpcServer, khaydarin_proto::khaydarin_service_server::KhaydarinServiceServer,
};

// OTLP tracing pipeline initialization function
fn init_tracing() -> Result<SdkTracerProvider, Box<dyn std::error::Error>> {
    use opentelemetry::KeyValue;

    // Create OTLP exporter
    let exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_endpoint("http://localhost:4317")
        .build()?;

    // Create resource with service name using ResourceBuilder
    let resource = Resource::builder()
        .with_service_name("khaydarin-service")
        .build();

    // Create TracerProvider with the exporter
    let provider = SdkTracerProvider::builder()
        .with_batch_exporter(exporter)
        .with_resource(resource)
        .build();

    Ok(provider)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Set global Propagator (propagate context via HTTP headers)
    global::set_text_map_propagator(TraceContextPropagator::new());

    // Initialize tracer provider
    let provider = init_tracing().expect("Failed to initialize tracing pipeline.");
    let tracer = provider.tracer("khaydarin-service");

    // Setup Tracing Subscriber
    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);
    tracing_subscriber::registry()
        .with(telemetry)
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Set the global tracer provider
    let _ = global::set_tracer_provider(provider);

    // Initialize dependencies from infrastructure layer
    tracing::info!("Connecting to database...");
    let db_pool = infrastructure::database::connect().await?;

    // Initialize the repository
    let khaydarin_repository = PostgresKhaydarinRepository::new(db_pool.clone());

    // Create database schema if it doesn't exist
    khaydarin_repository.init_schema().await?;

    // Initialize LLM client (using mock for development)
    let llm_client = LlmApiClient::new_mock();

    // Initialize output parser
    let output_parser = JsonOutputParser;

    // Create use case
    let use_case = ProcessPromptUseCase::new(khaydarin_repository, llm_client, output_parser);

    // Create gRPC service
    let grpc_service = KhaydarinGrpcServer::new(use_case);

    // Setup Axum HTTP server for health checks and metrics
    let app = Router::new()
        .route("/health", get(health_handler))
        .route("/ready", get(ready_handler))
        // This layer automatically creates spans for each request
        .layer(TraceLayer::new_for_http());

    // HTTP server address for health checks
    let http_addr = "0.0.0.0:8080".parse::<SocketAddr>()?;

    // gRPC server address
    let grpc_addr = "[::1]:50051".parse::<SocketAddr>()?;

    tracing::info!("Starting Khaydarin service");
    tracing::info!("HTTP server (health checks) listening on {}", http_addr);
    tracing::info!("gRPC server listening on {}", grpc_addr);

    // Spawn HTTP server for health checks
    let http_listener = tokio::net::TcpListener::bind(http_addr).await?;
    tokio::spawn(async move {
        axum::serve(http_listener, app).await.unwrap();
    });

    // Start gRPC server
    let grpc_server = Server::builder()
        .add_service(KhaydarinServiceServer::new(grpc_service))
        .serve(grpc_addr);

    tracing::info!("gRPC server started on {}", grpc_addr);

    // Run gRPC server until interrupted
    tokio::select! {
        res = grpc_server => {
            if let Err(e) = res {
                tracing::error!("gRPC server error: {}", e);
            }
        }
        _ = tokio::signal::ctrl_c() => {
            tracing::info!("Received shutdown signal");
        }
    }

    tracing::info!("Shutting down Khaydarin service");
    Ok(())
}

#[tracing::instrument]
async fn health_handler() -> &'static str {
    "OK"
}

#[tracing::instrument]
async fn ready_handler() -> &'static str {
    // In a production system, this would check database connectivity
    // For now, we'll return READY
    "READY"
}

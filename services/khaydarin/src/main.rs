use std::net::SocketAddr;
use axum::{routing::get, Router};
use opentelemetry::global;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{propagation::TraceContextPropagator, trace as sdktrace, Resource};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

// Module declarations
mod domain;
mod application;
mod infrastructure;
mod presentation;

// OTLP tracing pipeline initialization function
fn init_tracing() -> Result<sdktrace::Tracer, opentelemetry::trace::TraceError> {
    opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic() // Use gRPC/Tonic to send to OTLP endpoint
                .with_endpoint("http://localhost:4317"), // Jaeger/Collector address
        )
        .with_config(sdktrace::config().with_resource(Resource::new(vec![
            opentelemetry::KeyValue::new(
                "service.name",
                "khaydarin-service",
            ),
        ])))
        .install_batch(opentelemetry_sdk::runtime::Tokio)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Set global Propagator (propagate context via HTTP headers)
    global::set_text_map_propagator(TraceContextPropagator::new());
    
    // Initialize tracer
    let tracer = init_tracing().expect("Failed to initialize tracing pipeline.");
    
    // Setup Tracing Subscriber
    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);
    tracing_subscriber::registry()
        .with(telemetry)
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer())
        .init();

    // TODO: Initialize dependencies from infrastructure layer
    // let db_pool = infrastructure::database::connect().await?;
    // let khaydarin_repository = infrastructure::database::PostgresKhaydarinRepository::new(db_pool);
    // let use_cases = application::use_cases::UseCases::new(khaydarin_repository);
    // let grpc_service = presentation::grpc_services::KhaydarinGrpcServer::new(use_cases);

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

    // TODO: Start gRPC server
    // tonic::transport::Server::builder()
    //     .add_service(grpc_service)
    //     .serve(grpc_addr)
    //     .await?;
    
    // Keep the main task running
    tokio::signal::ctrl_c().await?;
    tracing::info!("Shutting down Khaydarin service");

    Ok(())
}

#[tracing::instrument]
async fn health_handler() -> &'static str {
    "OK"
}

#[tracing::instrument]
async fn ready_handler() -> &'static str {
    // TODO: Check database connectivity and other dependencies
    "READY"
}